mod commands;
mod download;
mod storage;

use tauri::Manager;
use tauri::AppHandle;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_positioner::{Position, WindowExt};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, GlobalShortcutExt};

fn setup_window_size_and_position(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // Center the window (size is set in tauri.conf.json)
        let _ = window.move_window(Position::Center);
    }
}

fn toggle_window_visibility(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let mut manager = download::manager::DownloadManager::new();
            manager.init(app.handle().clone());
            app.manage(tokio::sync::Mutex::new(manager));
            
            // Setup window size and position
            setup_window_size_and_position(&app.handle());
            
            // Register default global shortcut (Ctrl+Shift+D)
            let app_handle = app.handle().clone();
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);
            let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                toggle_window_visibility(&app_handle);
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_storage,
            commands::file_exists,
            commands::download_file,
            commands::pause_download,
            commands::load_settings,
            commands::save_settings,
            commands::load_download_history,
            commands::save_download_history,
            commands::clear_download_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

