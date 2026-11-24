pub mod macos_native;
pub mod spotube;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub artwork_data_url: Option<String>,
    pub duration_secs: f64,
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
    pub provider_name: String,
    pub app_name: String,
    pub bundle_id: String,
    pub is_detected: bool,
    pub status: PlaybackStatus,
    pub track: Option<TrackMetadata>,
    pub position_secs: f64,
    pub timestamp_ms: u64,
}

pub trait PlaybackProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn get_state(&self) -> Result<PlaybackState, String>;
}
