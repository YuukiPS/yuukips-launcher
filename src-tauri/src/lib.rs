//! YuukiPS Launcher - Main Library
//! This is the main entry point for the Tauri application backend.

// Import all modules
mod game;
mod http;
mod patch;
mod proxy;
mod system;
mod utils;

// Re-export commonly used functions for easier access
pub use game::*;
pub use http::*;
pub use patch::*;
pub use system::*;
pub use utils::*;

/// Initialize and run the Tauri application
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // System functions
            check_admin_privileges,
            check_and_disable_windows_proxy,
            install_ssl_certificate,
            check_certificate_status,
            check_ssl_certificate_installed,
            get_system_info,
            check_windows_defender_status,
            get_dotnet_versions,
            check_service_status,
            // Proxy functions
            proxy::get_proxy_addr,
            proxy::set_proxy_addr,
            proxy::get_proxy_port,
            proxy::set_proxy_port,
            proxy::find_available_port,
            proxy::start_proxy_with_port,
            proxy::add_proxy_domain,
            proxy::remove_proxy_domain,
            proxy::get_proxy_logs,
            proxy::clear_proxy_logs,
            proxy::start_proxy,
            proxy::stop_proxy,
            proxy::check_proxy_status,
            proxy::force_stop_proxy,
            proxy::get_proxy_domains,
        proxy::get_user_proxy_domains,
        proxy::get_all_proxy_domains,
        proxy::initialize_user_domains_if_empty,
            // HTTP functions
            test_proxy_bypass,
            fetch_api_data,
            test_network_connectivity,
            // Game functions
            get_game_folder_path,
            launch_game,
            check_game_installed,
            check_game_running,
            kill_game,
            start_game_monitor,
            stop_game_monitor,
            is_game_monitor_active,
            stop_game_process,
            stop_game,
            // Patch functions
            get_download_progress,
            clear_download_progress,
            check_patch_status,
            restore_game_files,
        ])
        .setup(|_app| {
            // Check and disable Windows proxy on startup
            match check_and_disable_windows_proxy() {
                Ok(message) => println!("🔧 Startup proxy check: {}", message),
                Err(e) => eprintln!("⚠️ Startup proxy check failed: {}", e),
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
