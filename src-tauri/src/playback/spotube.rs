use super::macos_native::{get_native_now_playing, send_native_media_command};
use super::{PlaybackCommand, PlaybackProvider, PlaybackState, PlaybackStatus, TrackMetadata};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SpotubeProvider;

impl SpotubeProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpotubeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackProvider for SpotubeProvider {
    fn id(&self) -> &'static str {
        "spotube"
    }

    fn display_name(&self) -> &'static str {
        "Spotube"
    }

    fn is_available(&self) -> bool {
        if let Some(snapshot) = get_native_now_playing() {
            if snapshot.is_spotube {
                return true;
            }
        }
        false
    }

    fn get_state(&self) -> Result<PlaybackState, String> {
        let now_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        let now_epoch_ms = (now_epoch_secs * 1000.0) as u64;

        let snapshot = match get_native_now_playing() {
            Some(s) => s,
            None => {
                return Ok(PlaybackState {
                    provider_id: self.id().to_string(),
                    provider_name: self.display_name().to_string(),
                    app_name: "Spotube".to_string(),
                    bundle_id: "org.spotube.spotube".to_string(),
                    is_detected: false,
                    status: PlaybackStatus::Inactive,
                    track: None,
                    position_secs: 0.0,
                    timestamp_ms: now_epoch_ms,
                });
            }
        };

        // If Spotube is not the current Now Playing client
        if !snapshot.is_spotube {
            return Ok(PlaybackState {
                provider_id: self.id().to_string(),
                provider_name: self.display_name().to_string(),
                app_name: snapshot.app_name,
                bundle_id: snapshot.bundle_id,
                is_detected: false,
                status: PlaybackStatus::Inactive,
                track: None,
                position_secs: 0.0,
                timestamp_ms: now_epoch_ms,
            });
        }

        // Active client is Spotube
        let has_track = !snapshot.title.trim().is_empty();
        let track = if has_track {
            Some(TrackMetadata {
                title: snapshot.title,
                artist: snapshot.artist,
                album: snapshot.album,
                artwork_data_url: snapshot.artwork_data_url,
                duration_secs: snapshot.duration_secs,
            })
        } else {
            None
        };

        let status = if !has_track {
            PlaybackStatus::Stopped
        } else if snapshot.is_playing {
            PlaybackStatus::Playing
        } else {
            PlaybackStatus::Paused
        };

        let position_secs = if has_track {
            if snapshot.is_playing && snapshot.playback_rate > 0.0 {
                let diff = (now_epoch_secs - snapshot.timestamp_epoch).max(0.0);
                (snapshot.elapsed_secs + diff * snapshot.playback_rate).clamp(0.0, snapshot.duration_secs)
            } else {
                snapshot.elapsed_secs.clamp(0.0, snapshot.duration_secs)
            }
        } else {
            0.0
        };

        Ok(PlaybackState {
            provider_id: self.id().to_string(),
            provider_name: self.display_name().to_string(),
            app_name: if snapshot.app_name.is_empty() { "Spotube".to_string() } else { snapshot.app_name },
            bundle_id: if snapshot.bundle_id.is_empty() { "org.spotube.spotube".to_string() } else { snapshot.bundle_id },
            is_detected: true,
            status,
            track,
            position_secs,
            timestamp_ms: now_epoch_ms,
        })
    }

    fn send_command(&self, cmd: PlaybackCommand) -> Result<bool, String> {
        let cmd_str = match cmd {
            PlaybackCommand::Play => "play",
            PlaybackCommand::Pause => "pause",
            PlaybackCommand::Toggle => "toggle",
            PlaybackCommand::Stop => "stop",
            PlaybackCommand::Next => "next",
            PlaybackCommand::Previous => "prev",
        };
        Ok(send_native_media_command(cmd_str))
    }
}
