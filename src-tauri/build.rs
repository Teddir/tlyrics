fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("src/native/now_playing_macos.m")
            .flag("-fobjc-arc")
            .compile("now_playing_macos");
    }

    tauri_build::build();
}
