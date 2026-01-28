use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, AsyncSeekExt, SeekFrom};
use futures_util::StreamExt;
use super::http::HttpHelper;
use serde::Serialize;
use crate::download::{Downloader, DownloadContext, DownloadError, DownloadMeta};
use crate::download::gdrive::GDriveDownloader;
use crate::storage::DownloadType;

#[derive(Clone, Serialize)]
pub struct ProgressEvent {
    pub id: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed: u64,
}

pub struct DownloadManager {
    app: Option<AppHandle>,
    tasks: HashMap<String, tokio::task::AbortHandle>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            app: None,
            tasks: HashMap::new(),
        }
    }
    
    pub fn init(&mut self, app: AppHandle) {
        self.app = Some(app);
    }
    
    pub async fn download(&mut self, url: String, path: String) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let app = self.app.clone().ok_or("App not initialized")?;
        let http = HttpHelper::new();

        let meta: DownloadMeta = if GDriveDownloader::detect(&url) {
            match GDriveDownloader::analyze(&url, &http).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    DownloadMeta {
                        download_type: DownloadType::Http,
                        direct_url: url.clone(),
                        original_url: None,
                        suggested_filename: None,
                    }
                }
                Err(e) => {
                    let _ = app.emit("download://error", serde_json::json!({
                        "id": id,
                        "error": e.to_string(),
                    }));
                    return Err(e.to_string());
                }
            }
        } else {
            DownloadMeta {
                download_type: DownloadType::Http,
                direct_url: url.clone(),
                original_url: None,
                suggested_filename: None,
            }
        };

        let task_id = id.clone();
        let task_url = meta.direct_url.clone();
        let task_path = path.clone();
        let task_app = app.clone();
        let task_original_url = meta.original_url.clone();
        let download_type = meta.download_type.clone();

        let handle = tokio::spawn(async move {
            let ctx = DownloadContext {
                id: task_id.clone(),
                url: task_url,
                save_path: task_path,
                app: task_app.clone(),
                http: HttpHelper::new(),
                original_url: task_original_url,
                downloaded_bytes: 0,
            };

            let result = match download_type {
                DownloadType::GoogleDrive => GDriveDownloader::run(ctx).await,
                _ => FileDownloader::run_legacy(ctx).await,
            };

            if let Err(e) = result {
                eprintln!("Download error for {}: {}", task_id, e);
                let _ = task_app.emit("download://error", (task_id.clone(), e.to_string()));
            }
        });

        self.tasks.insert(id.clone(), handle.abort_handle());

        Ok(serde_json::json!({
            "id": id,
            "download_type": meta.download_type.as_str(),
            "original_url": meta.original_url,
            "direct_url": meta.direct_url,
        }).to_string())
    }

    pub fn pause(&mut self, id: String) -> Result<(), String> {
        if let Some(handle) = self.tasks.remove(&id) {
            handle.abort();
            if let Some(app) = &self.app {
                let _ = app.emit("download://paused", id);
            }
            Ok(())
        } else {
            Err("Task not found".to_string())
        }
    }
}

pub struct FileDownloader;
impl FileDownloader {
    pub async fn run_legacy(ctx: DownloadContext) -> Result<(), DownloadError> {
        let DownloadContext { id, url, save_path: path, app, http, original_url: _, downloaded_bytes: _ } = ctx;
        // 1. Get metadata
        let meta = http.get_metadata(&url).await.map_err(|e| DownloadError::NetworkError(e))?;
        
        // 2. Check file - use .fdm extension for incomplete downloads
        let file_path = PathBuf::from(&path);
        let temp_path = PathBuf::from(format!("{}.fdm", path));
        let mut downloaded = 0;
        if temp_path.exists() {
             downloaded = tokio::fs::metadata(&temp_path).await.map(|m| m.len()).unwrap_or(0);
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&temp_path)
            .await.map_err(|e| DownloadError::IoError(e.to_string()))?;
            
        // 3. Start stream logic
        // If file exists and server supports range, resume.
        // If file exists but no range support, restart (truncate).
        
        let response = if downloaded > 0 && meta.accept_ranges {
            if let Some(total) = meta.size {
                http.download_range_request(&url, downloaded, total).await.map_err(|e| DownloadError::NetworkError(e))?
            } else {
                http.download_stream_request(&url).await.map_err(|e| DownloadError::NetworkError(e))?
            }
        } else {
            if downloaded > 0 {
                // Truncate
                file.set_len(0).await.map_err(|e| DownloadError::IoError(e.to_string()))?;
                file.seek(SeekFrom::Start(0)).await.map_err(|e| DownloadError::IoError(e.to_string()))?;
                downloaded = 0;
            }
            http.download_stream_request(&url).await.map_err(|e| DownloadError::NetworkError(e))?
        };
        
        let mut stream = response.bytes_stream();
        let total = meta.size;
        
        use std::time::Instant;
        let mut last_emit = Instant::now();
        let mut bytes_since_emit = 0;

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| DownloadError::NetworkError(e.to_string()))?;
            file.write_all(&chunk).await.map_err(|e| DownloadError::IoError(e.to_string()))?;
            let len = chunk.len() as u64;
            downloaded += len;
            bytes_since_emit += len;
            
            // Limit event emission to every 100ms or so
            if last_emit.elapsed().as_millis() > 100 {
                let speed = (bytes_since_emit as f64 / last_emit.elapsed().as_secs_f64()) as u64;
                let _ = app.emit("download://progress", ProgressEvent {
                    id: id.clone(),
                    downloaded,
                    total,
                    speed,
                });
                last_emit = Instant::now();
                bytes_since_emit = 0;
            }
        }
        
        // Final event
        let _ = app.emit("download://progress", ProgressEvent {
            id: id.clone(),
            downloaded,
            total,
            speed: 0,
        });
        
        // Close file and rename from .fdm to final name
        drop(file);
        tokio::fs::rename(&temp_path, &file_path)
            .await
            .map_err(|e| DownloadError::IoError(format!("Failed to rename file: {}", e)))?;
        
        // Emit completion event
        let _ = app.emit("download://complete", id);
        
        Ok(())
    }
}
