use crate::download::{DownloadContext, DownloadError, DownloadMeta, DownloadResult, Downloader};
use crate::download::http::HttpHelper;
use crate::storage::DownloadType;
use async_trait::async_trait;
use reqwest::StatusCode;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;

pub struct GDriveDownloader;

impl GDriveDownloader {
    fn is_direct_link(url: &str) -> bool {
        url.contains("drive.usercontent.google.com/download") ||
        url.contains("takeout-download-drive.usercontent.google.com")
    }

    fn extract_filename_from_header(header_value: &str) -> Option<String> {
        if let Some(start) = header_value.find("filename=\"") {
            let start = start + 10;
            if let Some(end) = header_value[start..].find('"') {
                return Some(header_value[start..start + end].to_string());
            }
        }
        if let Some(start) = header_value.find("filename=") {
            let start = start + 9;
            let end = header_value[start..].find(';').unwrap_or(header_value.len() - start);
            let filename = header_value[start..start + end].trim();
            if !filename.is_empty() {
                return Some(filename.to_string());
            }
        }
        None
    }
}

#[async_trait]
impl Downloader for GDriveDownloader {
    fn detect(url: &str) -> bool {
        // Only support direct download URLs and Takeout URLs
        url.contains("drive.usercontent.google.com/download") ||
        url.contains("takeout-download-drive.usercontent.google.com")
    }

    async fn analyze(url: &str, http: &HttpHelper) -> DownloadResult<Option<DownloadMeta>> {
        eprintln!("[GDrive] Analyzing URL: {}", url);
        if !Self::detect(url) {
            return Ok(None);
        }

        // Only accept direct download URLs
        if !Self::is_direct_link(url) && !url.contains("takeout.google.com") {
            return Err(DownloadError::InvalidUrl(
                "Only direct Google Drive download URLs are supported.\n\nSupported formats:\n• https://drive.usercontent.google.com/download?id=...\n• Google Takeout URLs\n\nShare links (drive.google.com/file/d/.../view) are not supported.\nTo download from browser, copy the URL from the browser's download manager, not the share link.".to_string()
            ));
        }

        Ok(Some(DownloadMeta {
            download_type: DownloadType::GoogleDrive,
            direct_url: url.to_string(),
            original_url: Some(url.to_string()), // Store the URL for copying later
            suggested_filename: None,
        }))
    }

    async fn run(ctx: DownloadContext) -> DownloadResult<()> {
        let DownloadContext { id, url, save_path, app, http, original_url: _, downloaded_bytes } = ctx;
        eprintln!("[GDrive] Starting download: id={}, url={}, path={}", id, url, save_path);

        let response = if downloaded_bytes > 0 {
            eprintln!("[GDrive] Resuming download from byte {}", downloaded_bytes);
            http.client()
                .get(&url)
                .header("Range", format!("bytes={}-", downloaded_bytes))
                .send()
                .await
                .map_err(|e| DownloadError::NetworkError(e.to_string()))?
        } else {
            eprintln!("[GDrive] Starting fresh download");
            http.client()
                .get(&url)
                .send()
                .await
                .map_err(|e| DownloadError::NetworkError(e.to_string()))?
        };
        eprintln!("[GDrive] Response status: {}", response.status());

        let response = if response.status() == StatusCode::OK {
            let content_type = response.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            eprintln!("[GDrive] Content-Type: {}", content_type);
            
            if content_type.contains("text/html") {
                let body = response.text().await.map_err(|e| DownloadError::NetworkError(e.to_string()))?;
                
                // Save HTML for debugging
                if let Some(temp_dir) = std::env::temp_dir().to_str() {
                    let debug_path = format!("{}\\gdrive_response.html", temp_dir);
                    if let Err(e) = std::fs::write(&debug_path, &body) {
                        eprintln!("[GDrive] Failed to save debug HTML: {}", e);
                    } else {
                        eprintln!("[GDrive] Saved HTML response to: {}", debug_path);
                    }
                }
                
                // Check if it's an authentication issue
                if body.contains("signin") || body.contains("ServiceLogin") || body.contains("accounts.google.com") {
                    return Err(DownloadError::AccessDenied(
                        "This file requires Google account authentication.\n\nThe download URL may have expired or requires login.\nPlease copy a fresh download URL from your browser's download manager.".to_string()
                    ));
                }
                
                return Err(DownloadError::AccessDenied(
                    "Received HTML instead of file content.\n\nPossible causes:\n• The download URL has expired\n• The file requires authentication\n• The file is not publicly accessible\n\nPlease copy a fresh download URL from your browser's download manager.".to_string()
                ));
            } else {
                response
            }
        } else {
            response
        };
        eprintln!("[GDrive] Final response status after processing: {}", response.status());

        match response.status() {
            StatusCode::OK | StatusCode::PARTIAL_CONTENT => {}
            StatusCode::FORBIDDEN => {
                return Err(DownloadError::AccessDenied(
                    "Access denied. This may be a private file.".to_string()
                ));
            }
            StatusCode::RANGE_NOT_SATISFIABLE => {
                return Err(DownloadError::ResumeNotPossible(
                    "File has changed on server. Cannot resume download.".to_string()
                ));
            }
            status => {
                return Err(DownloadError::NetworkError(
                    format!("Server returned: {}", status)
                ));
            }
        }

        let total_size = response.headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(|len| len + downloaded_bytes);
        eprintln!("[GDrive] Total size: {:?}", total_size);

        let filename = response.headers()
            .get("content-disposition")
            .and_then(|v| v.to_str().ok())
            .and_then(Self::extract_filename_from_header);
        eprintln!("[GDrive] Extracted filename: {:?}", filename);
        
        // Use extracted filename if available and save_path is a directory or generic
        let final_path = if let Some(ref fname) = filename {
            let path = std::path::Path::new(&save_path);
            if path.is_dir() || save_path.ends_with("download") || save_path.ends_with("view") {
                // save_path is a directory or generic name, append the real filename
                let parent = if path.is_dir() { 
                    path.to_path_buf() 
                } else { 
                    path.parent().unwrap_or(path).to_path_buf() 
                };
                let final_path = parent.join(fname);
                eprintln!("[GDrive] Using extracted filename, final path: {}", final_path.display());
                final_path.to_string_lossy().to_string()
            } else {
                save_path.clone()
            }
        } else {
            save_path.clone()
        };
        
        // Add .fdm extension for incomplete downloads
        let temp_path = format!("{}.fdm", final_path);
        eprintln!("[GDrive] Downloading to temporary path: {}", temp_path);

        let mut file = if downloaded_bytes > 0 {
            OpenOptions::new()
                .append(true)
                .open(&temp_path)
                .await
                .map_err(|e| DownloadError::IoError(e.to_string()))?
        } else {
            File::create(&temp_path)
                .await
                .map_err(|e| DownloadError::IoError(e.to_string()))?
        };

        let mut stream = response.bytes_stream();
        let mut downloaded = downloaded_bytes;
        let mut last_emit = Instant::now();
        let mut bytes_since_last_emit = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| DownloadError::NetworkError(e.to_string()))?;
            
            file.write_all(&chunk)
                .await
                .map_err(|e| DownloadError::IoError(e.to_string()))?;
            
            let chunk_size = chunk.len() as u64;
            downloaded += chunk_size;
            bytes_since_last_emit += chunk_size;

            if last_emit.elapsed() >= Duration::from_millis(100) {
                let elapsed_secs = last_emit.elapsed().as_secs_f64();
                let speed = if elapsed_secs > 0.0 {
                    (bytes_since_last_emit as f64 / elapsed_secs) as u64
                } else {
                    0
                };
                
                let _ = app.emit("download://progress", serde_json::json!({
                    "id": id,
                    "downloaded": downloaded,
                    "total": total_size,
                    "speed": speed,
                    "filename": filename,
                }));
                last_emit = Instant::now();
                bytes_since_last_emit = 0;
            }
        }

        file.flush().await.map_err(|e| DownloadError::IoError(e.to_string()))?;
        drop(file);
        
        // Remove .fdm extension by renaming to final path
        eprintln!("[GDrive] Renaming {} to {}", temp_path, final_path);
        tokio::fs::rename(&temp_path, &final_path)
            .await
            .map_err(|e| DownloadError::IoError(format!("Failed to rename file: {}", e)))?;
        
        eprintln!("[GDrive] Download complete: {} bytes", downloaded);

        let _ = app.emit("download://complete", serde_json::json!({
            "id": id,
            "path": final_path,
            "filename": filename,
        }));

        Ok(())
    }

    async fn refresh_url(_original_url: &str, _http: &HttpHelper) -> DownloadResult<Option<String>> {
        // Direct download URLs don't support refresh - they are time-limited URLs
        Ok(None)
    }
}
