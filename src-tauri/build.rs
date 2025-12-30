fn main() {
    #[cfg(target_os = "windows")]
    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("yuukips-launcher.exe.manifest"));

    #[cfg(not(target_os = "windows"))]
    let windows = tauri_build::WindowsAttributes::new();

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run build script");
}