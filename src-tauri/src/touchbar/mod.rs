use std::ffi::CString;
use std::fs;
use std::path::PathBuf;

unsafe extern "C" {
    fn tlyrics_touchbar_init();
    fn tlyrics_touchbar_update(
        current: *const std::os::raw::c_char,
        next: *const std::os::raw::c_char,
        is_playing: bool,
    );
}

pub struct TouchBarManager {
    cache_dir: Option<PathBuf>,
}

impl TouchBarManager {
    pub fn new() -> Self {
        let cache_dir = dirs_touchbar_cache();
        if let Some(ref dir) = cache_dir {
            let _ = fs::create_dir_all(dir);
            // Write sample BTT & MTMR integration presets on initialization
            let _ = fs::write(dir.join("btt_widget_script.sh"), generate_btt_script());
            let _ = fs::write(dir.join("mtmr_item.json"), generate_mtmr_preset());
        }

        Self { cache_dir }
    }

    /// Initializes native AppKit Touch Bar attached to application and windows
    pub fn init_native(&self) {
        #[cfg(target_os = "macos")]
        unsafe {
            tlyrics_touchbar_init();
        }
    }

    /// Updates both foreground native Touch Bar and background integrations (BTT / MTMR / Pock)
    pub fn update(
        &self,
        current_line: Option<&str>,
        next_line: Option<&str>,
        is_playing: bool,
        title: Option<&str>,
        artist: Option<&str>,
    ) {
        let curr = current_line.unwrap_or("♪ TLyrics").trim();
        let next = next_line.unwrap_or("").trim();

        // 1. Update native macOS AppKit Touch Bar
        #[cfg(target_os = "macos")]
        {
            let c_curr = CString::new(curr).unwrap_or_default();
            let c_next = CString::new(next).unwrap_or_default();
            unsafe {
                tlyrics_touchbar_update(c_curr.as_ptr(), c_next.as_ptr(), is_playing);
            }
        }

        // 2. Update background integrations (BetterTouchTool, MTMR, Pock) via IPC files
        if let Some(ref dir) = self.cache_dir {
            // Write single-line text for simple shell widgets:
            let display_text = if !next.is_empty() {
                format!("{} ‣ {}", curr, next)
            } else {
                curr.to_string()
            };
            let _ = fs::write(dir.join("current_lyric.txt"), &display_text);

            // Write full JSON for rich BTT / MTMR script widgets:
            let json_payload = serde_json::json!({
                "currentLine": curr,
                "nextLine": next,
                "title": title.unwrap_or(""),
                "artist": artist.unwrap_or(""),
                "isPlaying": is_playing,
                "text": display_text,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
            });
            let _ = fs::write(dir.join("touchbar.json"), json_payload.to_string());
        }
    }

    pub fn get_integration_info(&self) -> serde_json::Value {
        let base_path = self
            .cache_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "~/.cache/tlyrics".to_string());

        serde_json::json!({
            "cacheDir": base_path,
            "txtPath": format!("{}/current_lyric.txt", base_path),
            "jsonPath": format!("{}/touchbar.json", base_path),
            "bttScript": generate_btt_script(),
            "mtmrJson": generate_mtmr_preset(),
        })
    }
}

impl Default for TouchBarManager {
    fn default() -> Self {
        Self::new()
    }
}

fn dirs_touchbar_cache() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        Some(PathBuf::from(home).join(".cache").join("tlyrics"))
    } else {
        None
    }
}

fn generate_btt_script() -> String {
    r#"#!/bin/bash
# BetterTouchTool Touch Bar Shell Script Widget (Refresh every 1.0s)
CACHE_FILE="$HOME/.cache/tlyrics/current_lyric.txt"
if [ -f "$CACHE_FILE" ]; then
    cat "$CACHE_FILE"
else
    echo "♪ TLyrics"
fi
"#.to_string()
}

fn generate_mtmr_preset() -> String {
    r#"{
  "type": "shell",
  "refreshInterval": 1,
  "source": {
    "filePath": "/bin/sh",
    "args": ["-c", "cat $HOME/.cache/tlyrics/current_lyric.txt 2>/dev/null || echo '♪ TLyrics'"]
  },
  "bordered": false,
  "width": 500
}"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touchbar_manager_update_and_ipc() {
        let manager = TouchBarManager::new();
        manager.update(
            Some("Current Lyric Line Test"),
            Some("Next Lyric Line Test"),
            true,
            Some("Test Song"),
            Some("Test Artist"),
        );

        let info = manager.get_integration_info();
        println!("\n=== TOUCH BAR INTEGRATION INFO ===");
        println!("Cache Directory: {}", info["cacheDir"]);
        println!("TXT Path: {}", info["txtPath"]);
        println!("JSON Path: {}", info["jsonPath"]);
        println!("==================================\n");

        if let Some(dir) = manager.cache_dir {
            let txt = fs::read_to_string(dir.join("current_lyric.txt")).unwrap();
            assert_eq!(txt, "Current Lyric Line Test ‣ Next Lyric Line Test");

            let json_str = fs::read_to_string(dir.join("touchbar.json")).unwrap();
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            assert_eq!(val["currentLine"], "Current Lyric Line Test");
            assert_eq!(val["nextLine"], "Next Lyric Line Test");
            assert_eq!(val["isPlaying"], true);
        }
    }
}
