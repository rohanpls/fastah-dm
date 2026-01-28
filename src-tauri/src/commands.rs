use crate::storage;
use crate::download::manager::DownloadManager;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use std::path::Path;
use serde::Serialize;

#[derive(Serialize)]
pub struct DownloadResponse {
    pub id: String,
    pub download_type: String,
    pub original_url: Option<String>,
}

#[tauri::command]
pub fn get_system_storage(path: String) -> Result<storage::StorageInfo, String> {
    match storage::get_disk_info(&path) {
        Some(info) => Ok(info),
        None => Err("Could not determine disk for path".to_string()),
    }
}

#[tauri::command]
pub fn file_exists(path: String) -> bool {
    Path::new(&path).exists()
}

#[tauri::command]
pub async fn download_file(
    state: State<'_, Mutex<DownloadManager>>,
    url: String,
    save_path: String
) -> Result<DownloadResponse, String> {
    let mut manager = state.lock().await;
    let result_json = manager.download(url, save_path).await?;
    
    let result: serde_json::Value = serde_json::from_str(&result_json)
        .map_err(|e| e.to_string())?;
    
    Ok(DownloadResponse {
        id: result["id"].as_str().unwrap_or_default().to_string(),
        download_type: result["download_type"].as_str().unwrap_or("http").to_string(),
        original_url: result["original_url"].as_str().map(|s| s.to_string()),
    })
}

#[tauri::command]
pub async fn pause_download(
    state: State<'_, Mutex<DownloadManager>>,
    id: String
) -> Result<(), String> {
    let mut manager = state.lock().await;
    manager.pause(id)
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Result<storage::AppSettings, String> {
    storage::load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: storage::AppSettings) -> Result<(), String> {
    storage::save_settings(&app, &settings)
}

#[tauri::command]
pub fn load_download_history(app: AppHandle) -> Result<storage::DownloadHistory, String> {
    storage::load_download_history(&app)
}

#[tauri::command]
pub fn save_download_history(app: AppHandle, history: storage::DownloadHistory) -> Result<(), String> {
    storage::save_download_history(&app, &history)
}

#[tauri::command]
pub fn clear_download_history(app: AppHandle) -> Result<(), String> {
    storage::clear_download_history(&app)
}
