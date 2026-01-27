use crate::storage;
use crate::download::manager::DownloadManager;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use std::path::Path;

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
) -> Result<String, String> {
    let mut manager = state.lock().await;
    manager.download(url, save_path).await
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
