//! YuukiPS Launcher - Main Library
//! This is the main entry point for the Tauri application backend.

use tauri::Manager;

// Import all modules
mod download;
mod game;
mod http;
mod patch;
mod proxy;
mod system;
mod utils;

// Re-export commonly used functions for easier access
pub use download::*;
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
            is_admin,
            check_and_disable_windows_proxy,
            install_ssl_certificate,
            check_ssl_certificate_installed,
            open_directory,
            clear_launcher_data,
            get_yuukips_data_path,
            get_app_data_path,
            get_temp_files_path,
            // Download functions
            download::start_download,
            download::pause_download,
            download::resume_download,
            download::cancel_download,
            download::restart_download,
            download::get_active_downloads,
            download::get_download_status,
            download::get_download_history,
            download::clear_completed_downloads,
            download::clear_download_history,
            download::get_download_stats,
            download::open_download_location,
            download::set_download_directory,
            download::get_download_directory,
            download::bulk_pause_downloads,
            download::bulk_resume_downloads,
            download::bulk_cancel_downloads,
            download::validate_download_url,
            download::validate_download_url_with_options,
            download::get_file_size_from_url,
            download::check_file_exists,
            download::get_available_disk_space,
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
            proxy::get_active_proxy_domains,
            proxy::get_proxy_status_with_domains,
            proxy::initialize_user_domains_if_empty,
            proxy::generate_ca_files,
            // HTTP functions
            test_proxy_bypass,
            fetch_api_data,
            test_game_api_call,
            test_network_connectivity,
            get_current_version,
            fetch_latest_release,
            download_and_install_update,
            restart_application,
            terminate_for_update,
            // Game functions
            get_game_folder_path,
            launch_game,
            check_patch_message,
            check_game_installed,
            check_game_running,
            kill_game,
            start_game_monitor,
            stop_game_monitor,
            is_game_monitor_active,
            force_stop_game_monitor,
            stop_game_process,
            stop_game,
            // Patch functions
            get_download_progress,
            clear_download_progress,
            check_patch_status,
            restore_game_files,
        ])
        .setup(|app| {
            // Check admin privileges at startup - required for patch operations and proxy functionality
            if !is_running_as_admin() {
                eprintln!("❌ Administrator privileges required!");
                eprintln!("This launcher requires administrator access to:");
                eprintln!("  • Copy and apply game patches");
                eprintln!("  • Run the proxy server");
                eprintln!("  • Modify system proxy settings");
                eprintln!("Please restart the launcher as administrator.");
                
                // Show error dialog to user
                use tauri_plugin_dialog::DialogExt;
                let _ = app.dialog()
                    .message("This launcher requires administrator privileges to perform patch operations and run the proxy server.\n\nPlease restart the application as administrator.")
                    .title("Administrator Required")
                    .blocking_show();
                
                std::process::exit(1);
            }
            
            println!("✅ Running with administrator privileges");
            
            // Check and disable Windows proxy on startup
            match check_and_disable_windows_proxy() {
                Ok(message) => println!("🔧 Startup proxy check: {}", message),
                Err(e) => eprintln!("⚠️ Startup proxy check failed: {}", e),
            }
            
            // Show the main window after initialization
            let main_window = app.get_webview_window("main").unwrap();
            main_window.show().unwrap();
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
