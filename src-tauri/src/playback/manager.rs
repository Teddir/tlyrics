use std::sync::{Arc, RwLock};

use super::generic_now_playing::GenericNowPlayingProvider;
use super::spotube::SpotubeProvider;
use super::{PlaybackCommand, PlaybackProvider, PlaybackState, PlaybackStatus, ProviderDescriptor};

pub struct PlaybackManager {
    providers: Vec<Arc<dyn PlaybackProvider>>,
    selected_provider_id: RwLock<Option<String>>,
}

impl PlaybackManager {
    pub fn new() -> Self {
        let providers: Vec<Arc<dyn PlaybackProvider>> = vec![
            Arc::new(SpotubeProvider::new()),
            Arc::new(GenericNowPlayingProvider::new()),
        ];

        Self {
            providers,
            selected_provider_id: RwLock::new(None),
        }
    }

    /// Registers a custom provider at runtime
    pub fn register_provider(&mut self, provider: Arc<dyn PlaybackProvider>) {
        self.providers.push(provider);
    }

    /// Gets current list of providers and their status
    pub fn get_providers(&self) -> Vec<ProviderDescriptor> {
        let active_id = self.selected_provider_id.read().unwrap().clone();
        self.providers
            .iter()
            .map(|p| {
                let id = p.id().to_string();
                let is_active = match &active_id {
                    Some(pinned) => pinned == &id,
                    None => id == "spotube", // Spotube is default prioritized
                };
                ProviderDescriptor {
                    id,
                    display_name: p.display_name().to_string(),
                    is_available: p.is_available(),
                    is_active,
                }
            })
            .collect()
    }

    /// Sets or clears a pinned provider (None = auto Spotube-first)
    pub fn set_active_provider(&self, provider_id: Option<String>) -> Result<(), String> {
        if let Some(ref id) = provider_id {
            if !self.providers.iter().any(|p| p.id() == id.as_str()) {
                return Err(format!("Unknown provider ID: {}", id));
            }
        }
        let mut lock = self.selected_provider_id.write().unwrap();
        *lock = provider_id;
        Ok(())
    }

    /// Unified playback state retrieval following the Spotube-first routing policy
    pub fn get_state(&self) -> Result<PlaybackState, String> {
        let pinned = self.selected_provider_id.read().unwrap().clone();

        // 1. If user explicitly pinned a provider, query only that provider
        if let Some(pinned_id) = pinned {
            if let Some(provider) = self.providers.iter().find(|p| p.id() == pinned_id.as_str()) {
                return provider.get_state();
            }
        }

        // 2. Spotube-first auto routing policy
        // Step A: Query Spotube first
        let mut spotube_state: Option<PlaybackState> = None;
        if let Some(spotube) = self.providers.iter().find(|p| p.id() == "spotube") {
            if let Ok(state) = spotube.get_state() {
                if state.is_detected && (state.status == PlaybackStatus::Playing || state.status == PlaybackStatus::Paused) {
                    return Ok(state);
                }
                spotube_state = Some(state);
            }
        }

        // Step B: If Spotube is not playing, check other fallback providers for active audio
        for provider in &self.providers {
            if provider.id() == "spotube" {
                continue;
            }
            if let Ok(state) = provider.get_state() {
                if state.is_detected && (state.status == PlaybackStatus::Playing || state.status == PlaybackStatus::Paused) {
                    return Ok(state);
                }
            }
        }

        // Step C: If nothing is playing anywhere, return Spotube state as the default anchor
        if let Some(st) = spotube_state {
            return Ok(st);
        }

        // Default inactive fallback
        if let Some(first) = self.providers.first() {
            first.get_state()
        } else {
            Err("No playback providers registered".to_string())
        }
    }

    /// Dispatches command to the active provider
    pub fn send_command(&self, cmd: PlaybackCommand) -> Result<bool, String> {
        let pinned = self.selected_provider_id.read().unwrap().clone();

        if let Some(pinned_id) = pinned {
            if let Some(provider) = self.providers.iter().find(|p| p.id() == pinned_id.as_str()) {
                return provider.send_command(cmd);
            }
        }

        // Default to Spotube first
        if let Some(spotube) = self.providers.iter().find(|p| p.id() == "spotube") {
            if spotube.is_available() {
                return spotube.send_command(cmd);
            }
        }

        // Or first available
        for provider in &self.providers {
            if provider.is_available() {
                return provider.send_command(cmd);
            }
        }

        Err("No active playback provider available for command".to_string())
    }
}

impl Default for PlaybackManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::TrackMetadata;

    #[test]
    fn test_track_fingerprint() {
        let t1 = TrackMetadata {
            title: "Sisa Rasa  ".to_string(),
            artist: "Mahalini".to_string(),
            album: None,
            artwork_data_url: None,
            duration_secs: 254.86,
        };
        let t2 = TrackMetadata {
            title: "sisa rasa".to_string(),
            artist: "mahalini".to_string(),
            album: Some("Some Album".to_string()),
            artwork_data_url: None,
            duration_secs: 254.89,
        };
        // Normalized title and artist match and rounded duration matches
        assert_eq!(t1.fingerprint(), t2.fingerprint());
    }

    #[test]
    fn test_manager_providers_list() {
        let manager = PlaybackManager::new();
        let providers = manager.get_providers();
        assert!(providers.len() >= 2);
        assert_eq!(providers[0].id, "spotube");
        assert_eq!(providers[1].id, "macos_now_playing");
    }

    #[test]
    fn test_spotube_first_priority() {
        let manager = PlaybackManager::new();
        let state = manager.get_state().expect("Failed to get state");
        println!("\n=== PLAYBACK MANAGER SPOTUBE-FIRST ROUTING ===");
        println!("Resolved Provider ID: {}", state.provider_id);
        println!("Resolved Provider Name: {}", state.provider_name);
        println!("Detected App: {}", state.app_name);
        println!("Status: {:?}", state.status);
        if let Some(track) = &state.track {
            println!("Track: {} by {}", track.title, track.artist);
            println!("Fingerprint: {}", track.fingerprint());
        }
        println!("==============================================\n");

        // When Spotube is running, it must be the resolved provider
        if state.is_detected && state.app_name.to_lowercase().contains("spotube") {
            assert_eq!(state.provider_id, "spotube");
        }
    }

    #[test]
    fn test_pin_provider() {
        let manager = PlaybackManager::new();
        // Pin to macos_now_playing
        manager.set_active_provider(Some("macos_now_playing".to_string())).unwrap();
        let state = manager.get_state().expect("Failed to get pinned state");
        assert_eq!(state.provider_id, "macos_now_playing");

        // Revert to auto (Spotube-first)
        manager.set_active_provider(None).unwrap();
        let state_auto = manager.get_state().expect("Failed to get auto state");
        println!("Reverted to auto: Provider ID = {}", state_auto.provider_id);
    }
}
