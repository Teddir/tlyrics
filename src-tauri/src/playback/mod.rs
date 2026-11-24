pub mod generic_now_playing;
pub mod macos_native;
pub mod manager;
pub mod spotube;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackCommand {
    Play,
    Pause,
    Toggle,
    Stop,
    Next,
    Previous,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub artwork_data_url: Option<String>,
    pub duration_secs: f64,
}

impl TrackMetadata {
    /// Generates a normalized identity fingerprint for reliable track-change detection
    pub fn fingerprint(&self) -> String {
        format!(
            "{}::{}::{:.1}",
            self.title.trim().to_lowercase(),
            self.artist.trim().to_lowercase(),
            self.duration_secs
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackState {
    pub provider_id: String,
    pub provider_name: String,
    pub app_name: String,
    pub bundle_id: String,
    pub is_detected: bool,
    pub status: PlaybackStatus,
    pub track: Option<TrackMetadata>,
    pub position_secs: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDescriptor {
    pub id: String,
    pub display_name: String,
    pub is_available: bool,
    pub is_active: bool,
}

/// Core interface that all playback sources must implement.
/// The lyrics engine interacts solely with this abstraction.
pub trait PlaybackProvider: Send + Sync {
    /// Unique machine identifier (e.g. "spotube", "macos_now_playing")
    fn id(&self) -> &'static str;

    /// Human-friendly display label (e.g. "Spotube", "macOS System Media")
    fn display_name(&self) -> &'static str;

    /// Returns true if this provider's target application or service is reachable
    fn is_available(&self) -> bool;

    /// Queries the current playback state and track metadata
    fn get_state(&self) -> Result<PlaybackState, String>;

    /// Dispatches transport controls (play, pause, next, prev)
    fn send_command(&self, cmd: PlaybackCommand) -> Result<bool, String>;
}
