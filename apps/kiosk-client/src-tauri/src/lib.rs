mod config;
mod metrics;
mod room;

use tauri::Manager;

#[tauri::command]
fn is_kiosk_mode() -> bool {
    kiosk_mode_enabled()
}

fn kiosk_mode_enabled() -> bool {
    match std::env::var("PISTATION_KIOSK").ok().as_deref() {
        Some("1") | Some("true") => true,
        Some("0") | Some("false") => false,
        _ => !cfg!(debug_assertions),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(room::RoomHandle::default())
        .invoke_handler(tauri::generate_handler![
            config::load_config,
            config::save_config,
            config::hardware_id,
            metrics::collect_metrics,
            is_kiosk_mode,
            room::room_connect,
            room::room_disconnect,
            room::video_subscribe,
            room::video_unsubscribe
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if kiosk_mode_enabled() {
                    let _ = window.set_fullscreen(true);
                    let _ = window.set_always_on_top(true);
                    let _ = window.set_cursor_visible(false);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
