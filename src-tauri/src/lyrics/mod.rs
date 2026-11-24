pub mod cache;
pub mod engine;
pub mod lrclib;
pub mod parser;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub time_secs: f64,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LyricsType {
    Synced,
    Plain,
    Instrumental,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsData {
    pub track_title: String,
    pub artist_name: String,
    pub lyrics_type: LyricsType,
    pub lines: Vec<LyricLine>,
    pub raw_lrc: Option<String>,
    pub raw_plain: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncedLyricsState {
    pub lyrics_type: LyricsType,
    pub current_line_index: Option<usize>,
    pub current_line: Option<LyricLine>,
    pub next_line: Option<LyricLine>,
    pub lines: Vec<LyricLine>,
    pub has_lyrics: bool,
    pub track_fingerprint: String,
}
