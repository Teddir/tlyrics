pub mod playback;

use playback::manager::PlaybackManager;
use playback::{PlaybackCommand, PlaybackState, ProviderDescriptor};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, State};

#[tauri::command]
fn get_playback_state(manager: State<'_, Arc<PlaybackManager>>) -> Result<PlaybackState, String> {
    manager.get_state()
}

#[tauri::command]
fn send_playback_command(
    command: String,
    manager: State<'_, Arc<PlaybackManager>>,
) -> Result<bool, String> {
    let cmd = match command.to_lowercase().as_str() {
        "play" => PlaybackCommand::Play,
        "pause" => PlaybackCommand::Pause,
        "toggle" | "playpause" => PlaybackCommand::Toggle,
        "stop" => PlaybackCommand::Stop,
        "next" => PlaybackCommand::Next,
        "prev" | "previous" => PlaybackCommand::Previous,
        _ => return Err(format!("Unknown command: {}", command)),
    };
    manager.send_command(cmd)
}

#[tauri::command]
fn get_playback_providers(
    manager: State<'_, Arc<PlaybackManager>>,
) -> Result<Vec<ProviderDescriptor>, String> {
    Ok(manager.get_providers())
}

#[tauri::command]
fn set_active_provider(
    provider_id: Option<String>,
    manager: State<'_, Arc<PlaybackManager>>,
) -> Result<(), String> {
    manager.set_active_provider(provider_id)
}

// Backward-compatible alias for Phase 1
#[tauri::command]
fn get_spotube_playback_state(
    manager: State<'_, Arc<PlaybackManager>>,
) -> Result<PlaybackState, String> {
    manager.get_state()
}

#[tauri::command]
fn send_media_control(
    command: String,
    manager: State<'_, Arc<PlaybackManager>>,
) -> Result<bool, String> {
    send_playback_command(command, manager)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let playback_manager = Arc::new(PlaybackManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(playback_manager.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let manager = playback_manager;

            std::thread::spawn(move || {
                loop {
                    if let Ok(state) = manager.get_state() {
                        let _ = app_handle.emit("playback-state", &state);
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_playback_state,
            send_playback_command,
            get_playback_providers,
            set_active_provider,
            get_spotube_playback_state,
            send_media_control
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
