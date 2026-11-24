use super::LyricsData;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

pub struct LyricsCache {
    memory_cache: RwLock<HashMap<String, LyricsData>>,
    cache_dir: Option<PathBuf>,
}

impl LyricsCache {
    pub fn new() -> Self {
        let cache_dir = dirs_fallback_cache();
        if let Some(ref dir) = cache_dir {
            let _ = fs::create_dir_all(dir);
        }

        Self {
            memory_cache: RwLock::new(HashMap::new()),
            cache_dir,
        }
    }

    /// Tries to get cached lyrics from memory first, then disk
    pub fn get(&self, fingerprint: &str) -> Option<LyricsData> {
        // 1. Memory cache
        if let Ok(mem) = self.memory_cache.read() {
            if let Some(data) = mem.get(fingerprint) {
                return Some(data.clone());
            }
        }

        // 2. Disk cache
        if let Some(ref dir) = self.cache_dir {
            let file_path = dir.join(format!("{}.json", safe_key(fingerprint)));
            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if let Ok(mut data) = serde_json::from_str::<LyricsData>(&content) {
                        data.source = "cache".to_string();
                        // Populate memory cache
                        if let Ok(mut mem) = self.memory_cache.write() {
                            mem.insert(fingerprint.to_string(), data.clone());
                        }
                        return Some(data);
                    }
                }
            }
        }

        None
    }

    /// Stores lyrics in both memory and disk cache
    pub fn set(&self, fingerprint: &str, data: &LyricsData) {
        // 1. Memory cache
        if let Ok(mut mem) = self.memory_cache.write() {
            mem.insert(fingerprint.to_string(), data.clone());
        }

        // 2. Disk cache
        if let Some(ref dir) = self.cache_dir {
            let file_path = dir.join(format!("{}.json", safe_key(fingerprint)));
            if let Ok(json) = serde_json::to_string(data) {
                let _ = fs::write(file_path, json);
            }
        }
    }
}

impl Default for LyricsCache {
    fn default() -> Self {
        Self::new()
    }
}

fn safe_key(input: &str) -> String {
    // Generate simple deterministic alphanumeric key
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash_val = hasher.finish();

    let alphanumeric: String = input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(24)
        .collect();

    format!("{}_{:x}", alphanumeric, hash_val)
}

fn dirs_fallback_cache() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        Some(PathBuf::from(home).join(".cache").join("tlyrics").join("lyrics"))
    } else {
        None
    }
}
