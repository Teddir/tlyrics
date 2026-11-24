use super::cache::LyricsCache;
use super::lrclib::LrclibClient;
use super::parser::find_current_index;
use super::{LyricsData, LyricsType, SyncedLyricsState};
use crate::playback::PlaybackState;
use std::sync::{Arc, RwLock};

pub struct LyricsEngine {
    client: LrclibClient,
    cache: LyricsCache,
    current_fingerprint: RwLock<Option<String>>,
    current_lyrics: RwLock<Option<LyricsData>>,
    is_fetching: RwLock<bool>,
    offset_ms: RwLock<i64>,
}

impl LyricsEngine {
    pub fn new() -> Self {
        Self {
            client: LrclibClient::new(),
            cache: LyricsCache::new(),
            current_fingerprint: RwLock::new(None),
            current_lyrics: RwLock::new(None),
            is_fetching: RwLock::new(false),
            offset_ms: RwLock::new(0),
        }
    }

    /// Sets the lyrics timing offset in milliseconds
    pub fn set_offset_ms(&self, ms: i64) {
        let mut lock = self.offset_ms.write().unwrap();
        *lock = ms;
    }

    /// Returns the lyrics timing offset in milliseconds
    pub fn get_offset_ms(&self) -> i64 {
        *self.offset_ms.read().unwrap()
    }

    /// Clears cached lyrics from memory and disk and resets state
    pub fn clear_cache(&self) {
        self.cache.clear();
        let mut lyr = self.current_lyrics.write().unwrap();
        *lyr = None;
        let mut fp = self.current_fingerprint.write().unwrap();
        *fp = None;
    }

    /// Explicitly fetch or retrieve cached lyrics for given track metadata
    pub async fn fetch_for_track(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration: f64,
        fingerprint: &str,
    ) -> Result<LyricsData, String> {
        // Check cache
        if let Some(cached) = self.cache.get(fingerprint) {
            return Ok(cached);
        }

        // Fetch from LRCLIB
        let data = self.client.fetch_lyrics(title, artist, album, duration).await?;

        // Save to cache
        self.cache.set(fingerprint, &data);
        Ok(data)
    }

    /// Synchronizes playback state: detects track changes, updates current lyrics,
    /// and calculates active and upcoming lyric lines.
    pub fn update_playback(
        self: &Arc<Self>,
        playback_state: &PlaybackState,
    ) -> SyncedLyricsState {
        let track = match &playback_state.track {
            Some(t) => t,
            None => {
                let mut fp_lock = self.current_fingerprint.write().unwrap();
                *fp_lock = None;
                let mut lyr_lock = self.current_lyrics.write().unwrap();
                *lyr_lock = None;
                return SyncedLyricsState {
                    lyrics_type: LyricsType::None,
                    current_line_index: None,
                    current_line: None,
                    next_line: None,
                    lines: Vec::new(),
                    has_lyrics: false,
                    track_fingerprint: String::new(),
                };
            }
        };

        let fingerprint = track.fingerprint();
        let last_fingerprint = self.current_fingerprint.read().unwrap().clone();

        // Track Change Detection!
        if last_fingerprint.as_deref() != Some(&fingerprint) {
            {
                let mut fp_lock = self.current_fingerprint.write().unwrap();
                *fp_lock = Some(fingerprint.clone());
            }

            // Check cache synchronously
            if let Some(cached) = self.cache.get(&fingerprint) {
                let mut lyr_lock = self.current_lyrics.write().unwrap();
                *lyr_lock = Some(cached);
            } else {
                // Trigger background fetch if not already in progress
                let mut fetching_lock = self.is_fetching.write().unwrap();
                if !*fetching_lock {
                    *fetching_lock = true;
                    let engine = Arc::clone(self);
                    let title = track.title.clone();
                    let artist = track.artist.clone();
                    let album = track.album.clone();
                    let duration = track.duration_secs;
                    let fp = fingerprint.clone();

                    tauri::async_runtime::spawn(async move {
                        let res = engine
                            .fetch_for_track(
                                &title,
                                &artist,
                                album.as_deref(),
                                duration,
                                &fp,
                            )
                            .await;

                        if let Ok(data) = res {
                            let mut lyr_lock = engine.current_lyrics.write().unwrap();
                            *lyr_lock = Some(data);
                        }
                        let mut fetching = engine.is_fetching.write().unwrap();
                        *fetching = false;
                    });
                }
            }
        }

        // Compute current synchronized position with user offset
        let offset_secs = (*self.offset_ms.read().unwrap() as f64) / 1000.0;
        let adjusted_pos = (playback_state.position_secs + offset_secs).max(0.0);
        self.compute_synced_state(adjusted_pos, &fingerprint)
    }

    /// Computes the synchronized state based on active position in seconds
    pub fn compute_synced_state(&self, position_secs: f64, fingerprint: &str) -> SyncedLyricsState {
        let lyrics_guard = self.current_lyrics.read().unwrap();
        let data = match lyrics_guard.as_ref() {
            Some(d) => d,
            None => {
                return SyncedLyricsState {
                    lyrics_type: LyricsType::None,
                    current_line_index: None,
                    current_line: None,
                    next_line: None,
                    lines: Vec::new(),
                    has_lyrics: false,
                    track_fingerprint: fingerprint.to_string(),
                };
            }
        };

        let lines = data.lines.clone();
        let current_line_index = find_current_index(&lines, position_secs);
        let current_line = current_line_index.and_then(|idx| lines.get(idx).cloned());
        let next_line = current_line_index
            .map(|idx| idx + 1)
            .and_then(|idx| lines.get(idx).cloned());

        SyncedLyricsState {
            lyrics_type: data.lyrics_type,
            current_line_index,
            current_line,
            next_line,
            lines,
            has_lyrics: data.lyrics_type != LyricsType::None,
            track_fingerprint: fingerprint.to_string(),
        }
    }
}

impl Default for LyricsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lrclib_fetch_and_parse() {
        tauri::async_runtime::block_on(async {
            let engine = LyricsEngine::new();
            // Fetch known track: "Sisa Rasa" by "Mahalini"
            let res = engine
                .fetch_for_track(
                    "Sisa Rasa",
                    "Mahalini",
                    None,
                    255.0,
                    "sisa rasa::mahalini::255.0",
                )
                .await;

            assert!(res.is_ok(), "Failed to fetch lyrics: {:?}", res.err());
            let lyrics = res.unwrap();
            println!("\n=== LRCLIB FETCH VERIFICATION ===");
            println!("Track: {}", lyrics.track_title);
            println!("Artist: {}", lyrics.artist_name);
            println!("Type: {:?}", lyrics.lyrics_type);
            println!("Lines count: {}", lyrics.lines.len());
            if !lyrics.lines.is_empty() {
                println!("First line: [{:.2}s] {}", lyrics.lines[0].time_secs, lyrics.lines[0].text);
                println!("Sample line 5: [{:.2}s] {}", lyrics.lines[4].time_secs, lyrics.lines[4].text);
            }
            println!("=================================\n");

            assert_eq!(lyrics.lyrics_type, LyricsType::Synced);
            assert!(!lyrics.lines.is_empty());
            assert!(lyrics.lines[0].time_secs > 0.0);
        });
    }

    #[test]
    fn test_lyrics_cache_roundtrip() {
        let engine = LyricsEngine::new();
        let fp = "cache_test_track::cache_test_artist::180.0";

        let sample_data = LyricsData {
            track_title: "Test Track".into(),
            artist_name: "Test Artist".into(),
            lyrics_type: LyricsType::Synced,
            lines: vec![
                super::super::LyricLine { time_secs: 5.0, text: "Line 1".into() },
                super::super::LyricLine { time_secs: 10.0, text: "Line 2".into() },
            ],
            raw_lrc: Some("[00:05.00]Line 1\n[00:10.00]Line 2".into()),
            raw_plain: None,
            source: "test".into(),
        };

        engine.cache.set(fp, &sample_data);

        let retrieved = engine.cache.get(fp);
        assert!(retrieved.is_some());
        let r = retrieved.unwrap();
        assert_eq!(r.track_title, "Test Track");
        assert_eq!(r.lines.len(), 2);
    }

    #[test]
    fn test_live_spotube_lyrics_sync() {
        use crate::playback::manager::PlaybackManager;

        tauri::async_runtime::block_on(async {
            let manager = PlaybackManager::new();
            let engine = Arc::new(LyricsEngine::new());

            let playback_state = manager.get_state().expect("Failed to get playback state");
            println!("\n=== LIVE SPOTUBE LYRICS SYNC TEST ===");
            println!("Active app: {}", playback_state.app_name);
            println!("Status: {:?}", playback_state.status);

            if let Some(track) = &playback_state.track {
                println!("Live Track: {} - {}", track.title, track.artist);
                println!("Live Position: {:.2}s / {:.2}s", playback_state.position_secs, track.duration_secs);

                // Fetch lyrics
                let fp = track.fingerprint();
                let lyrics_res = engine.fetch_for_track(
                    &track.title,
                    &track.artist,
                    track.album.as_deref(),
                    track.duration_secs,
                    &fp,
                ).await;

                match lyrics_res {
                    Ok(data) => {
                        println!("Lyrics Type: {:?}", data.lyrics_type);
                        println!("Total Lines: {}", data.lines.len());
                        println!("Source: {}", data.source);

                        // Populate and compute synced state
                        {
                            let mut lyr_lock = engine.current_lyrics.write().unwrap();
                            *lyr_lock = Some(data);
                        }

                        let synced = engine.compute_synced_state(playback_state.position_secs, &fp);
                        println!("Current Line Index: {:?}", synced.current_line_index);
                        if let Some(curr) = synced.current_line {
                            println!("ACTIVE LYRIC LINE: [{:.2}s] {}", curr.time_secs, curr.text);
                        }
                        if let Some(next) = synced.next_line {
                            println!("NEXT LYRIC LINE:   [{:.2}s] {}", next.time_secs, next.text);
                        }
                    }
                    Err(e) => {
                        println!("Lyrics not found or error: {}", e);
                    }
                }
            } else {
                println!("No track currently playing in Spotube");
            }
            println!("=====================================\n");
        });
    }
}
