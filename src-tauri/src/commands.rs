use crate::storage;
use crate::download::manager::DownloadManager;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub fn get_system_storage(path: String) -> Result<storage::StorageInfo, String> {
    match storage::get_disk_info(&path) {
        Some(info) => Ok(info),
        None => Err("Could not determine disk for path".to_string()),
    }
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
