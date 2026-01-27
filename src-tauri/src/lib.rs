mod commands;
mod download;
mod storage;

use tauri::Manager; // Import Manager trait

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let mut manager = download::manager::DownloadManager::new();
            manager.init(app.handle().clone());
            app.manage(tokio::sync::Mutex::new(manager));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_storage,
            commands::download_file,
            commands::pause_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

