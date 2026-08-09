pub mod client;
mod events;
mod frames;
mod video;

pub use client::RoomHandle;

use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, State};

/// The webview opens one channel for decoded video and keeps it open. Frames are JPEG
/// encoded in Rust and drawn onto a canvas, which keeps the annotation overlay working
/// exactly as it does in the browser.
#[tauri::command]
pub fn video_subscribe(channel: Channel<InvokeResponseBody>) {
    frames::set_sink(Some(channel));
}

#[tauri::command]
pub fn video_unsubscribe() {
    frames::set_sink(None);
}

#[tauri::command]
pub async fn room_connect(
    app: AppHandle,
    handle: State<'_, RoomHandle>,
    url: String,
    token: String,
) -> Result<(), String> {
    handle.connect(app, url, token).await
}

#[tauri::command]
pub async fn room_disconnect(handle: State<'_, RoomHandle>) -> Result<(), String> {
    handle.disconnect().await;
    Ok(())
}
