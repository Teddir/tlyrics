use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub launch_at_login: bool,
    pub start_hidden: bool,
    pub background_mode: bool,
    pub touchbar_enabled: bool,
    pub last_window_visible: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            start_hidden: false,
            background_mode: true,
            touchbar_enabled: true,
            last_window_visible: true,
        }
    }
}

pub struct SettingsManager {
    settings: RwLock<AppSettings>,
    config_path: Option<PathBuf>,
}

impl SettingsManager {
    pub fn new() -> Self {
        let path = get_settings_file_path();
        let loaded = if let Some(ref p) = path {
            if p.exists() {
                fs::read_to_string(p)
                    .ok()
                    .and_then(|data| serde_json::from_str::<AppSettings>(&data).ok())
                    .unwrap_or_default()
            } else {
                AppSettings::default()
            }
        } else {
            AppSettings::default()
        };

        Self {
            settings: RwLock::new(loaded),
            config_path: path,
        }
    }

    pub fn get(&self) -> AppSettings {
        self.settings.read().unwrap().clone()
    }

    pub fn save(&self, new_settings: AppSettings) -> Result<(), String> {
        // Apply launch at login configuration
        let _ = configure_macos_launch_agent(new_settings.launch_at_login);

        {
            let mut lock = self.settings.write().unwrap();
            *lock = new_settings.clone();
        }

        if let Some(ref p) = self.config_path {
            if let Some(parent) = p.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&new_settings) {
                let _ = fs::write(p, json);
            }
        }

        Ok(())
    }

    pub fn set_last_window_visible(&self, visible: bool) {
        let mut updated = self.get();
        if updated.last_window_visible != visible {
            updated.last_window_visible = visible;
            let _ = self.save(updated);
        }
    }
}

fn get_settings_file_path() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        Some(PathBuf::from(home).join(".cache").join("tlyrics").join("settings.json"))
    } else {
        None
    }
}

/// Configures macOS user LaunchAgent at ~/Library/LaunchAgents/com.mac.tlyrics.plist
pub fn configure_macos_launch_agent(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").map_err(|e| e.to_string())?;
        let launch_agents_dir = PathBuf::from(home).join("Library").join("LaunchAgents");
        let plist_path = launch_agents_dir.join("com.mac.tlyrics.plist");

        if !enable {
            if plist_path.exists() {
                let _ = fs::remove_file(&plist_path);
            }
            return Ok(());
        }

        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe_path.to_string_lossy().to_string();

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.mac.tlyrics</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>--autostart</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>ProcessType</key>
    <string>Interactive</string>
</dict>
</plist>
"#,
            exe_str
        );

        let _ = fs::create_dir_all(&launch_agents_dir);
        let _ = fs::write(&plist_path, plist_content);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default_values() {
        let defaults = AppSettings::default();
        assert!(defaults.background_mode, "Background mode should be true by default");
        assert!(defaults.touchbar_enabled, "Touch Bar should be true by default");
        assert!(!defaults.start_hidden, "Start hidden should be false by default");
        assert!(!defaults.launch_at_login, "Launch at login should be false by default");
    }

    #[test]
    fn test_settings_save_and_load() {
        let manager = SettingsManager::new();
        let mut s = manager.get();
        s.touchbar_enabled = false;
        let _ = manager.save(s);
        assert!(!manager.get().touchbar_enabled);

        // Revert
        let mut s2 = manager.get();
        s2.touchbar_enabled = true;
        let _ = manager.save(s2);
        assert!(manager.get().touchbar_enabled);
    }
}
