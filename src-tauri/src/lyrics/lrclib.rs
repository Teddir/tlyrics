use super::parser::{parse_lrc, parse_plain_lyrics};
use super::{LyricsData, LyricsType};
use serde::Deserialize;
use std::time::Duration;

const LRCLIB_BASE_URL: &str = "https://lrclib.net/api";
const USER_AGENT: &str = "TLyrics/0.1.0 (macOS; Universal Lyrics Application)";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibResponse {
    pub id: Option<u64>,
    pub track_name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    pub instrumental: Option<bool>,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

pub struct LrclibClient {
    client: reqwest::Client,
}

impl LrclibClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// Fetches lyrics from LRCLIB for a given track, using exact get first then search fallback.
    pub async fn fetch_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration_secs: f64,
    ) -> Result<LyricsData, String> {
        // Step 1: Clean/normalize title (strip subtitles or "(From: ...)" if helpful)
        let clean_title = title.trim();
        let clean_artist = artist.trim();

        if clean_title.is_empty() {
            return Err("Track title is empty".to_string());
        }

        // Try exact match first
        if let Ok(Some(data)) = self.try_exact_get(clean_title, clean_artist, album, duration_secs).await {
            return Ok(data);
        }

        // If title contains " - (From: ...)" or " (feat. ...)", try stripped title
        if let Some(stripped_title) = simplify_track_title(clean_title) {
            if let Ok(Some(data)) = self.try_exact_get(&stripped_title, clean_artist, album, duration_secs).await {
                return Ok(data);
            }
        }

        // Step 2: Try search API fallback
        if let Ok(Some(data)) = self.try_search(clean_title, clean_artist, duration_secs).await {
            return Ok(data);
        }

        // Also try search with primary artist if multiple artists (comma separated)
        if clean_artist.contains(',') {
            let primary_artist = clean_artist.split(',').next().unwrap_or(clean_artist).trim();
            if let Ok(Some(data)) = self.try_search(clean_title, primary_artist, duration_secs).await {
                return Ok(data);
            }
        }

        Ok(LyricsData {
            track_title: clean_title.to_string(),
            artist_name: clean_artist.to_string(),
            lyrics_type: LyricsType::None,
            lines: Vec::new(),
            raw_lrc: None,
            raw_plain: None,
            source: "lrclib".to_string(),
        })
    }

    async fn try_exact_get(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration_secs: f64,
    ) -> Result<Option<LyricsData>, String> {
        let mut query = vec![
            ("track_name", title.to_string()),
            ("artist_name", artist.to_string()),
        ];

        if let Some(alb) = album {
            if !alb.trim().is_empty() {
                query.push(("album_name", alb.trim().to_string()));
            }
        }

        if duration_secs > 0.0 {
            query.push(("duration", format!("{}", duration_secs.round() as u64)));
        }

        let url = format!("{}/get", LRCLIB_BASE_URL);
        let resp = self.client.get(&url).query(&query).send().await;

        let response = match resp {
            Ok(r) => r,
            Err(e) => return Err(format!("Network request failed: {}", e)),
        };

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Ok(None);
        }

        let lrclib: LrclibResponse = match response.json().await {
            Ok(j) => j,
            Err(_) => return Ok(None),
        };

        Ok(Some(convert_lrclib_to_data(title, artist, lrclib, duration_secs)))
    }

    async fn try_search(
        &self,
        title: &str,
        artist: &str,
        duration_secs: f64,
    ) -> Result<Option<LyricsData>, String> {
        let query = vec![
            ("track_name", title),
            ("artist_name", artist),
        ];

        let url = format!("{}/search", LRCLIB_BASE_URL);
        let resp = self.client.get(&url).query(&query).send().await;

        let response = match resp {
            Ok(r) => r,
            Err(e) => return Err(format!("Search request failed: {}", e)),
        };

        if !response.status().is_success() {
            return Ok(None);
        }

        let items: Vec<LrclibResponse> = match response.json().await {
            Ok(j) => j,
            Err(_) => return Ok(None),
        };

        // First find item with syncedLyrics
        if let Some(best) = items.iter().find(|i| i.synced_lyrics.is_some() && !i.synced_lyrics.as_ref().unwrap().trim().is_empty()) {
            return Ok(Some(convert_lrclib_to_data(title, artist, LrclibResponse {
                id: best.id,
                track_name: best.track_name.clone(),
                artist_name: best.artist_name.clone(),
                album_name: best.album_name.clone(),
                duration: best.duration,
                instrumental: best.instrumental,
                plain_lyrics: best.plain_lyrics.clone(),
                synced_lyrics: best.synced_lyrics.clone(),
            }, duration_secs)));
        }

        // Otherwise first item with plainLyrics
        if let Some(first) = items.into_iter().next() {
            return Ok(Some(convert_lrclib_to_data(title, artist, first, duration_secs)));
        }

        Ok(None)
    }
}

impl Default for LrclibClient {
    fn default() -> Self {
        Self::new()
    }
}

fn simplify_track_title(title: &str) -> Option<String> {
    // E.g. "Mencintaimu - (From: 2nd Miracle in Cell No.7)" -> "Mencintaimu"
    // E.g. "Song Name (feat. Artist)" -> "Song Name"
    if let Some(pos) = title.find(" - (") {
        return Some(title[..pos].trim().to_string());
    }
    if let Some(pos) = title.find(" (feat.") {
        return Some(title[..pos].trim().to_string());
    }
    if let Some(pos) = title.find(" (ft.") {
        return Some(title[..pos].trim().to_string());
    }
    None
}

fn convert_lrclib_to_data(
    fallback_title: &str,
    fallback_artist: &str,
    resp: LrclibResponse,
    target_duration: f64,
) -> LyricsData {
    let title = resp.track_name.unwrap_or_else(|| fallback_title.to_string());
    let artist = resp.artist_name.unwrap_or_else(|| fallback_artist.to_string());

    if resp.instrumental.unwrap_or(false) {
        return LyricsData {
            track_title: title,
            artist_name: artist,
            lyrics_type: LyricsType::Instrumental,
            lines: Vec::new(),
            raw_lrc: None,
            raw_plain: None,
            source: "lrclib".to_string(),
        };
    }

    if let Some(synced) = resp.synced_lyrics {
        if !synced.trim().is_empty() {
            let lines = parse_lrc(&synced);
            if !lines.is_empty() {
                return LyricsData {
                    track_title: title,
                    artist_name: artist,
                    lyrics_type: LyricsType::Synced,
                    lines,
                    raw_lrc: Some(synced),
                    raw_plain: resp.plain_lyrics,
                    source: "lrclib".to_string(),
                };
            }
        }
    }

    if let Some(plain) = resp.plain_lyrics {
        if !plain.trim().is_empty() {
            let lines = parse_plain_lyrics(&plain, target_duration);
            return LyricsData {
                track_title: title,
                artist_name: artist,
                lyrics_type: LyricsType::Plain,
                lines,
                raw_lrc: None,
                raw_plain: Some(plain),
                source: "lrclib".to_string(),
            };
        }
    }

    LyricsData {
        track_title: title,
        artist_name: artist,
        lyrics_type: LyricsType::None,
        lines: Vec::new(),
        raw_lrc: None,
        raw_plain: None,
        source: "lrclib".to_string(),
    }
}
