use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, AsyncSeekExt, SeekFrom};
use futures_util::StreamExt;
use super::http::HttpHelper;
use serde::Serialize;

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
        let task_id = id.clone();
        
        let handle = tokio::spawn(async move {
            match FileDownloader::run(task_id.clone(), url, path, app.clone(), http).await {
                Ok(_) => {
                    let _ = app.emit("download://complete", task_id);
                },
                Err(e) => {
                     let _ = app.emit("download://error", (task_id, e));
                }
            }
        });
        
        self.tasks.insert(id.clone(), handle.abort_handle());
        
        Ok(id)
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

struct FileDownloader;
impl FileDownloader {
    async fn run(id: String, url: String, path: String, app: AppHandle, http: HttpHelper) -> Result<(), String> {
        // 1. Get metadata
        let meta = http.get_metadata(&url).await.map_err(|e| e.to_string())?;
        
        // 2. Check file
        let file_path = PathBuf::from(&path);
        let mut downloaded = 0;
        if file_path.exists() {
             downloaded = tokio::fs::metadata(&file_path).await.map(|m| m.len()).unwrap_or(0);
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&file_path)
            .await.map_err(|e| e.to_string())?;
            
        // 3. Start stream logic
        // If file exists and server supports range, resume.
        // If file exists but no range support, restart (truncate).
        
        let response = if downloaded > 0 && meta.accept_ranges {
            // Assume we want the rest
             // range request from downloaded to specific end if we knew it, or just open ended
             // Since http helper takes start/end, let's look at http helper sig.
             // download_range_request(url, start, end). 
             // Ideally we want "byte=start-".
             // Let's modify http helper to handle end=0 as "end of file" or add a new method?
             // Or just pass a very large number? "bytes=start-" is standard.
             // I'll adjust the call to be specific if I know size, or "start-" if supported.
             
             // For now, let's assume we want until the end if size is known.
             // Note: My http helper takes u64 for end. It needs to format string.
             // I'll hack it for now: if end < start, format as "start-"? No, helper implementation:
             // format!("bytes={}-{}", start, end); -> this is closed range.
             // I need to update HttpHelper to support open range.
             
             if let Some(total) = meta.size {
                 http.download_range_request(&url, downloaded, total).await?
             } else {
                 // Fallback if size unknown but range supported? Rare.
                 // Ideally "bytes=start-".
                 // I will skip resume if size unknown for this iteration to accept simplicity.
                 http.download_stream_request(&url).await?
             }
        } else {
            if downloaded > 0 {
                // Truncate
                file.set_len(0).await.map_err(|e| e.to_string())?;
                file.seek(SeekFrom::Start(0)).await.map_err(|e| e.to_string())?;
                downloaded = 0;
            }
            http.download_stream_request(&url).await?
        };
        
        let mut stream = response.bytes_stream();
        let total = meta.size;
        
        use std::time::Instant;
        let mut last_emit = Instant::now();
        let mut bytes_since_emit = 0;

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| e.to_string())?;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
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
        
        Ok(())
    }
}
