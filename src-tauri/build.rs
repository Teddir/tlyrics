fn main() {
    #[cfg(target_os = "macos")]
    {
        // 1. Compile Objective-C Now Playing bridge
        cc::Build::new()
            .file("src/native/now_playing_macos.m")
            .flag("-fobjc-arc")
            .compile("now_playing_macos");

        // 2. Compile Swift Touch Bar integration
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let swift_obj = format!("{}/touchbar_macos.o", out_dir);
        let swift_lib = format!("{}/libtouchbar.a", out_dir);

        let status = std::process::Command::new("swiftc")
            .args(&[
                "-c",
                "src/native/touchbar_macos.swift",
                "-o",
                &swift_obj,
                "-parse-as-library",
                "-O",
            ])
            .status()
            .expect("Failed to run swiftc");
        assert!(status.success(), "swiftc compilation failed");

        let ar_status = std::process::Command::new("ar")
            .args(&["rcs", &swift_lib, &swift_obj])
            .status()
            .expect("Failed to run ar");
        assert!(ar_status.success(), "ar failed to create static library");

        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=touchbar");
        println!("cargo:rerun-if-changed=src/native/now_playing_macos.m");
        println!("cargo:rerun-if-changed=src/native/now_playing_macos.h");
        println!("cargo:rerun-if-changed=src/native/touchbar_macos.swift");
    }

    tauri_build::build();
}
