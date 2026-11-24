#[repr(C)]
pub struct MacNowPlayingInfo {
    pub success: bool,
    pub is_spotube: bool,
    pub is_playing: bool,
    pub app_name: *mut std::os::raw::c_char,
    pub bundle_id: *mut std::os::raw::c_char,
    pub title: *mut std::os::raw::c_char,
    pub artist: *mut std::os::raw::c_char,
    pub album: *mut std::os::raw::c_char,
    pub artwork_data_url: *mut std::os::raw::c_char,
    pub duration_secs: f64,
    pub elapsed_secs: f64,
    pub timestamp_epoch: f64,
    pub playback_rate: f64,
}

unsafe extern "C" {
    pub fn mac_now_playing_copy_info() -> *mut MacNowPlayingInfo;
    pub fn mac_now_playing_free_info(info: *mut MacNowPlayingInfo);
    pub fn mac_now_playing_send_command(command: i32) -> bool;
}

#[derive(Debug, Clone)]
pub struct NativeNowPlayingSnapshot {
    pub is_spotube: bool,
    pub is_playing: bool,
    pub app_name: String,
    pub bundle_id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub artwork_data_url: Option<String>,
    pub duration_secs: f64,
    pub elapsed_secs: f64,
    pub timestamp_epoch: f64,
    pub playback_rate: f64,
}

fn c_str_to_string(ptr: *const std::os::raw::c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
    c_str.to_str().ok().map(|s| s.to_string())
}

pub fn get_native_now_playing() -> Option<NativeNowPlayingSnapshot> {
    unsafe {
        let raw = mac_now_playing_copy_info();
        if raw.is_null() {
            return None;
        }

        let info = &*raw;
        if !info.success {
            mac_now_playing_free_info(raw);
            return None;
        }

        let app_name = c_str_to_string(info.app_name).unwrap_or_default();
        let bundle_id = c_str_to_string(info.bundle_id).unwrap_or_default();
        let title = c_str_to_string(info.title).unwrap_or_default();
        let artist = c_str_to_string(info.artist).unwrap_or_default();
        let album_raw = c_str_to_string(info.album);
        let album = album_raw.and_then(|s| if s.trim().is_empty() { None } else { Some(s) });
        let artwork_data_url = c_str_to_string(info.artwork_data_url);

        let snapshot = NativeNowPlayingSnapshot {
            is_spotube: info.is_spotube,
            is_playing: info.is_playing,
            app_name,
            bundle_id,
            title,
            artist,
            album,
            artwork_data_url,
            duration_secs: info.duration_secs,
            elapsed_secs: info.elapsed_secs,
            timestamp_epoch: info.timestamp_epoch,
            playback_rate: info.playback_rate,
        };

        mac_now_playing_free_info(raw);
        Some(snapshot)
    }
}

pub fn send_native_media_command(cmd: &str) -> bool {
    let code = match cmd.to_lowercase().as_str() {
        "play" => 0,
        "pause" => 1,
        "toggle" | "playpause" | "toggleplaypause" => 2,
        "stop" => 3,
        "next" | "nexttrack" => 4,
        "prev" | "previous" | "previoustrack" => 5,
        _ => return false,
    };
    unsafe { mac_now_playing_send_command(code) }
}
