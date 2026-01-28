use sysinfo::Disks;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DownloadType {
    #[default]
    Http,
    #[serde(rename = "gdrive")]
    GoogleDrive,
    Torrent,
    Magnet,
}

impl DownloadType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadType::Http => "http",
            DownloadType::GoogleDrive => "gdrive",
            DownloadType::Torrent => "torrent",
            DownloadType::Magnet => "magnet",
        }
    }
}
use tauri::Manager;

#[derive(Serialize)]
pub struct StorageInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub mount_point: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub wallpaper_url: Option<String>,
    pub theme: Option<String>,
    pub default_download_path: Option<String>,
    pub author: String,
    pub launch_on_startup: bool,
    pub toggle_keybind: Option<String>,
    pub use_new_ui: bool,
    pub auto_update_enabled: bool,
    pub silent_updates: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            wallpaper_url: None,
            theme: Some("dark".to_string()),
            default_download_path: None,
            author: "@rohanpls".to_string(),
            launch_on_startup: false,
            toggle_keybind: Some("Ctrl+Shift+D".to_string()),
            use_new_ui: true,
            auto_update_enabled: true,
            silent_updates: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadHistoryItem {
    pub id: String,
    pub url: String,
    pub path: String,
    pub filename: String,
    pub total: Option<u64>,
    pub downloaded: u64,
    pub status: String,
    pub etag: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub download_type: DownloadType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_url: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DownloadHistory {
    pub items: Vec<DownloadHistoryItem>,
}

pub fn get_disk_info(path_str: &str) -> Option<StorageInfo> {
    let disks = Disks::new_with_refreshed_list();
    let path = Path::new(path_str);

    // Find the disk that contains the path
    // We look for the longest matching mount point
    let best_match = disks.list().iter()
        .filter(|disk| path.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len());

    if let Some(disk) = best_match {
        Some(StorageInfo {
            total: disk.total_space(),
            used: disk.total_space() - disk.available_space(),
            free: disk.available_space(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
        })
    } else {
        None
    }
}

fn get_data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| e.to_string())
}

fn ensure_data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = get_data_dir(app)?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

pub fn load_settings(app: &tauri::AppHandle) -> Result<AppSettings, String> {
    let dir = get_data_dir(app)?;
    let path = dir.join("settings.json");
    
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let dir = ensure_data_dir(app)?;
    let path = dir.join("settings.json");
    
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn load_download_history(app: &tauri::AppHandle) -> Result<DownloadHistory, String> {
    let dir = get_data_dir(app)?;
    let path = dir.join("history.json");
    
    if !path.exists() {
        return Ok(DownloadHistory::default());
    }
    
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_download_history(app: &tauri::AppHandle, history: &DownloadHistory) -> Result<(), String> {
    let dir = ensure_data_dir(app)?;
    let path = dir.join("history.json");
    
    let content = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn clear_download_history(app: &tauri::AppHandle) -> Result<(), String> {
    let dir = get_data_dir(app)?;
    let path = dir.join("history.json");
    
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
