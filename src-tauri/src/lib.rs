pub mod lyrics;
pub mod playback;
pub mod touchbar;

use lyrics::engine::LyricsEngine;
use lyrics::SyncedLyricsState;
use playback::manager::PlaybackManager;
use playback::{PlaybackCommand, PlaybackState, PlaybackStatus, ProviderDescriptor};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, State};
use touchbar::TouchBarManager;

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

#[tauri::command]
fn get_synced_lyrics(
    engine: State<'_, Arc<LyricsEngine>>,
    manager: State<'_, Arc<PlaybackManager>>,
) -> Result<SyncedLyricsState, String> {
    let state = manager.get_state()?;
    Ok(engine.update_playback(&state))
}

#[tauri::command]
fn get_touchbar_integration_info(
    touchbar: State<'_, Arc<TouchBarManager>>,
) -> Result<serde_json::Value, String> {
    Ok(touchbar.get_integration_info())
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
    let lyrics_engine = Arc::new(LyricsEngine::new());
    let touchbar_manager = Arc::new(TouchBarManager::new());

    // Initialize native Touch Bar on macOS
    touchbar_manager.init_native();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(playback_manager.clone())
        .manage(lyrics_engine.clone())
        .manage(touchbar_manager.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let manager = playback_manager;
            let engine = lyrics_engine;
            let touchbar = touchbar_manager;

            std::thread::spawn(move || {
                loop {
                    if let Ok(state) = manager.get_state() {
                        let lyrics_state = engine.update_playback(&state);

                        // Update Touch Bar (both native foreground and BTT/MTMR background files)
                        let is_playing = state.status == PlaybackStatus::Playing;
                        let curr_text = lyrics_state.current_line.as_ref().map(|l| l.text.as_str());
                        let next_text = lyrics_state.next_line.as_ref().map(|l| l.text.as_str());
                        let title = state.track.as_ref().map(|t| t.title.as_str());
                        let artist = state.track.as_ref().map(|t| t.artist.as_str());

                        touchbar.update(curr_text, next_text, is_playing, title, artist);

                        let _ = app_handle.emit("playback-state", &state);
                        let _ = app_handle.emit("lyrics-state", &lyrics_state);
                    }
                    std::thread::sleep(Duration::from_millis(300));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_playback_state,
            send_playback_command,
            get_playback_providers,
            set_active_provider,
            get_synced_lyrics,
            get_touchbar_integration_info,
            get_spotube_playback_state,
            send_media_control
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
