pub mod lyrics;
pub mod playback;
pub mod settings;
pub mod touchbar;

use lyrics::engine::LyricsEngine;
use lyrics::SyncedLyricsState;
use playback::manager::PlaybackManager;
use playback::{PlaybackCommand, PlaybackState, PlaybackStatus, ProviderDescriptor};
use settings::{AppSettings, SettingsManager};
use std::sync::Arc;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, State};
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

#[tauri::command]
fn present_global_touchbar(
    touchbar: State<'_, Arc<TouchBarManager>>,
) -> Result<(), String> {
    touchbar.present_global();
    Ok(())
}

#[tauri::command]
fn dismiss_global_touchbar(
    touchbar: State<'_, Arc<TouchBarManager>>,
) -> Result<(), String> {
    touchbar.dismiss_global();
    Ok(())
}

#[tauri::command]
fn set_lyrics_offset(
    offset_ms: i64,
    engine: State<'_, Arc<LyricsEngine>>,
) -> Result<(), String> {
    engine.set_offset_ms(offset_ms);
    Ok(())
}

#[tauri::command]
fn get_lyrics_offset(
    engine: State<'_, Arc<LyricsEngine>>,
) -> Result<i64, String> {
    Ok(engine.get_offset_ms())
}

#[tauri::command]
fn clear_lyrics_cache(
    engine: State<'_, Arc<LyricsEngine>>,
) -> Result<(), String> {
    engine.clear_cache();
    Ok(())
}

#[tauri::command]
fn restart_lyrics_engine(
    engine: State<'_, Arc<LyricsEngine>>,
) -> Result<(), String> {
    engine.clear_cache();
    Ok(())
}

#[tauri::command]
fn get_app_settings(
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<AppSettings, String> {
    Ok(settings.get())
}

#[tauri::command]
fn save_app_settings(
    new_settings: AppSettings,
    settings: State<'_, Arc<SettingsManager>>,
    touchbar: State<'_, Arc<TouchBarManager>>,
) -> Result<(), String> {
    if new_settings.touchbar_enabled {
        touchbar.present_global();
    } else {
        touchbar.dismiss_global();
    }
    settings.save(new_settings)
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
    let settings_manager = Arc::new(SettingsManager::new());

    // Initialize native global Touch Bar on macOS
    touchbar_manager.init_native();

    let sm_for_setup = settings_manager.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(playback_manager.clone())
        .manage(lyrics_engine.clone())
        .manage(touchbar_manager.clone())
        .manage(settings_manager.clone())
        .setup(move |app| {
            // 1. Hide Dock Icon: Configure macOS Accessory Activation Policy
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // 2. Setup macOS Menu Bar Tray Icon with required items
            let open_lyrics_item = MenuItem::with_id(app, "open_lyrics", "Open Lyrics", true, None::<&str>)?;
            let toggle_window_item = MenuItem::with_id(app, "toggle_window", "Show/Hide Window", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let track_info_item = MenuItem::with_id(app, "track_info", "♪ No Track Playing", true, None::<&str>)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let toggle_tb_item = MenuItem::with_id(app, "toggle_tb", "Enable/Disable Touch Bar", true, None::<&str>)?;
            let restart_engine_item = MenuItem::with_id(app, "restart_engine", "Restart Lyrics Engine", true, None::<&str>)?;
            let sep3 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit TLyrics", true, None::<&str>)?;

            let tray_menu = Menu::with_items(app, &[
                &open_lyrics_item,
                &toggle_window_item,
                &sep1,
                &track_info_item,
                &sep2,
                &settings_item,
                &toggle_tb_item,
                &restart_engine_item,
                &sep3,
                &quit_item,
            ])?;

            let default_icon = app.default_window_icon().unwrap().clone();
            let _tray = TrayIconBuilder::new()
                .icon(default_icon)
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .tooltip("TLyrics - Universal Lyrics Engine")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "open_lyrics" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                                let sm = app.state::<Arc<SettingsManager>>();
                                sm.set_last_window_visible(true);
                            }
                        }
                        "toggle_window" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let visible = w.is_visible().unwrap_or(false);
                                let sm = app.state::<Arc<SettingsManager>>();
                                if visible {
                                    let _ = w.hide();
                                    sm.set_last_window_visible(false);
                                } else {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                    sm.set_last_window_visible(true);
                                }
                            }
                        }
                        "track_info" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                                let sm = app.state::<Arc<SettingsManager>>();
                                sm.set_last_window_visible(true);
                            }
                        }
                        "settings" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                                let _ = w.emit("open-settings", ());
                                let sm = app.state::<Arc<SettingsManager>>();
                                sm.set_last_window_visible(true);
                            }
                        }
                        "toggle_tb" => {
                            let tb = app.state::<Arc<TouchBarManager>>();
                            let sm = app.state::<Arc<SettingsManager>>();
                            let mut current = sm.get();
                            current.touchbar_enabled = !current.touchbar_enabled;
                            if current.touchbar_enabled {
                                tb.present_global();
                            } else {
                                tb.dismiss_global();
                            }
                            let _ = sm.save(current);
                        }
                        "restart_engine" => {
                            let engine = app.state::<Arc<LyricsEngine>>();
                            engine.clear_cache();
                        }
                        "quit" => {
                            let tb = app.state::<Arc<TouchBarManager>>();
                            tb.dismiss_global();
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if let Ok(visible) = w.is_visible() {
                                let sm = app.state::<Arc<SettingsManager>>();
                                if visible {
                                    let _ = w.hide();
                                    sm.set_last_window_visible(false);
                                } else {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                    sm.set_last_window_visible(true);
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            // 3. Startup window visibility: check --autostart / --hidden / settings
            let is_autostart = std::env::args().any(|arg| arg == "--autostart" || arg == "--hidden");
            let initial_settings = sm_for_setup.get();
            let should_show = !is_autostart && !initial_settings.start_hidden && initial_settings.last_window_visible;

            if should_show {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }

            // 4. Background continuous monitoring and lyrics synchronization loop
            let app_handle = app.handle().clone();
            let manager = playback_manager;
            let engine = lyrics_engine;
            let touchbar = touchbar_manager;
            let track_info_ref = track_info_item.clone();

            std::thread::spawn(move || {
                loop {
                    if let Ok(state) = manager.get_state() {
                        let lyrics_state = engine.update_playback(&state);

                        // Update dynamic track information in native menu bar
                        let track_label = if let Some(t) = &state.track {
                            format!("♪ {} - {}", t.title, t.artist)
                        } else {
                            "♪ No Track Playing".to_string()
                        };
                        let _ = track_info_ref.set_text(&track_label);

                        // Update global Touch Bar & background IPC files
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
        .on_window_event(|window, event| {
            // Keep app running in background when main window is closed
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                if let Some(sm) = window.app_handle().try_state::<Arc<SettingsManager>>() {
                    sm.set_last_window_visible(false);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_playback_state,
            send_playback_command,
            get_playback_providers,
            set_active_provider,
            get_synced_lyrics,
            get_touchbar_integration_info,
            present_global_touchbar,
            dismiss_global_touchbar,
            set_lyrics_offset,
            get_lyrics_offset,
            clear_lyrics_cache,
            restart_lyrics_engine,
            get_app_settings,
            save_app_settings,
            get_spotube_playback_state,
            send_media_control
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
