pub mod playback;

use playback::macos_native::send_native_media_command;
use playback::spotube::SpotubeProvider;
use playback::{PlaybackProvider, PlaybackState};
use std::time::Duration;
use tauri::Emitter;

#[tauri::command]
fn get_spotube_playback_state() -> Result<PlaybackState, String> {
    let provider = SpotubeProvider::new();
    provider.get_state()
}

#[tauri::command]
fn send_media_control(command: String) -> Result<bool, String> {
    let success = send_native_media_command(&command);
    Ok(success)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let provider = SpotubeProvider::new();
                loop {
                    if let Ok(state) = provider.get_state() {
                        let _ = app_handle.emit("playback-state", &state);
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_spotube_playback_state,
            send_media_control
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
