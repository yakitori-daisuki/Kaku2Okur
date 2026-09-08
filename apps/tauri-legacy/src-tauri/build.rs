fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("native/TrackpadCapture.m")
            .flag("-fobjc-arc")
            .compile("kaku2okur_trackpad");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
    tauri_build::build()
}
