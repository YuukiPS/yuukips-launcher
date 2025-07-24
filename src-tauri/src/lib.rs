//! YuukiPS Launcher - Main Library
//! This is the main entry point for the Tauri application backend.

use tauri::Manager;

// Import all modules
mod download;
mod game;
mod hoyoplay;
mod http;
mod patch;
mod proxy;
mod settings;
mod system;
mod utils;

// Re-export commonly used functions for easier access
pub use download::*;
pub use game::*;
pub use hoyoplay::*;
pub use http::*;
pub use patch::*;
pub use settings::*;
pub use system::*;
pub use utils::*;

// Task manager monitoring functions are already available through pub use system::*

/// Initialize and run the Tauri application
pub fn run() {
    // YuukiPS Launcher initialization
    // Determine log level based on environment variable
    let log_level = match std::env::var("TAURI_LOG_LEVEL").as_deref() {
        Ok(level) => {
            let level_lower = level.to_lowercase();
            match level_lower.as_str() {
                "error" => {
                    log::info!("🚨 Error-only logging enabled via TAURI_LOG_LEVEL={}", level);
                    log::LevelFilter::Error
                },
                "warn" => {
                    log::info!("⚠️ Warning+ logging enabled via TAURI_LOG_LEVEL={}", level);
                    log::LevelFilter::Warn
                },
                "info" => {
                    log::info!("ℹ️ Info+ logging enabled via TAURI_LOG_LEVEL={}", level);
                    log::LevelFilter::Info
                },
                "debug" => {
                    log::info!("🔍 Debug logging enabled via TAURI_LOG_LEVEL={}", level);
                    log::LevelFilter::Debug
                },
                "trace" => {
                    log::info!("🔬 Trace logging enabled via TAURI_LOG_LEVEL={}", level);
                    log::LevelFilter::Trace
                },
                "off" => {
                    log::info!("🔇 Logging disabled via TAURI_LOG_LEVEL={}", level);
                    log::LevelFilter::Off
                },
                _ => {
                    log::info!("⚠️ Unknown log level '{}', defaulting to Info", level);
                    log::info!("   Valid levels: error, warn, info, debug, trace, off");
                    log::LevelFilter::Info
                }
            }
        },
        Err(_) => {
            log::info!("ℹ️ Using default Info log level (set TAURI_LOG_LEVEL to change)");
            log::info!("   Available npm scripts: tauri:dev:error, tauri:dev:warn, tauri:dev:info, tauri:dev:debug, tauri:dev:trace, tauri:dev:off");
            log::LevelFilter::Info
        }
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .targets([
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stderr),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: Some("yuukips-launcher".to_string()) }),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
            ])
            .level(log_level)
            .build())
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
            start_task_manager_monitor,
            stop_task_manager_monitor,
            is_task_manager_monitor_active,
            open_devtools,
            minimize_launcher_window,
            restore_launcher_window,
            // Download functions
            download::start_download,
            download::pause_download,
            download::resume_download,
            download::cancel_download,
            download::cancel_and_delete_download,
            download::restart_download,
            download::get_active_downloads,
            download::get_download_status,
            download::clear_completed_downloads,
            download::get_download_stats,
            download::open_download_location,
            download::set_download_directory,
            download::get_download_directory,
            download::bulk_pause_downloads,
            download::bulk_resume_downloads,
            download::bulk_cancel_downloads,
            download::bulk_cancel_and_delete_downloads,
            // URL validation functions removed
            download::get_file_size_from_url,
            download::check_file_exists,
            download::delete_file,
            download::check_file_existence_info,
            download::get_available_disk_space,
            download::get_activities,
            download::clear_activities,
            download::add_user_interaction_activity,
            download::save_download_state,
            download::load_download_state,
            download::resume_interrupted_downloads,
            download::get_state_version,
            download::set_auto_save_enabled,
            download::get_partial_downloads,
            download::get_speed_limit,
            download::set_speed_limit,
            download::get_divide_speed_enabled,
            download::set_divide_speed_enabled,
            download::get_max_simultaneous_downloads,
            download::set_max_simultaneous_downloads,
            download::check_and_fix_stalled_downloads,
            download::get_download_resume_support,
            // Settings functions (new simple JSON persistence)
            settings::get_app_speed_limit,
            settings::set_app_speed_limit,
            settings::get_app_divide_speed_enabled,
            settings::set_app_divide_speed_enabled,
            settings::get_app_max_simultaneous_downloads,
            settings::set_app_max_simultaneous_downloads,
            settings::get_all_app_settings,
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
            get_current_version,
            fetch_latest_release,
            download_and_install_update,
            restart_application,
            terminate_for_update,
            // Game functions
            get_game_folder_path,
            launch_game,
            validate_game_directory,
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
            is_any_game_running,
            get_available_drives,
            scan_drive_for_games,
            get_all_game_name_codes,
            get_game_md5,
            get_file_md5,
            get_file_md5_chunked,
            get_file_md5_chunked_with_progress,
            cancel_md5_calculation,
            // Patch functions
            get_download_progress,
            clear_download_progress,
            check_patch_status,
            fetch_patch_info_command,
            restore_game_files,
            // HoyoPlay functions (includes moved functions from utils.rs)
            get_game_executable_names,
            get_game_folder,
            get_hoyoplay_list_game,
            get_hoyoplay_game_folder,
            remove_all_hoyo_pass,
        ])
        .setup(|app| {
            // Check admin privileges at startup - required for patch operations and proxy functionality
            if !is_running_as_admin() {
                log::error!("❌ Administrator privileges required!");
                log::error!("This launcher requires administrator access to:");
                log::error!("  • Copy and apply game patches");
                log::error!("  • Run the proxy server");
                log::error!("  • Modify system proxy settings");
                log::error!("Please restart the launcher as administrator.");
                
                // Show error dialog to user
                use tauri_plugin_dialog::DialogExt;
                let _ = app.dialog()
                    .message("This launcher requires administrator privileges to perform patch operations and run the proxy server.\n\nPlease restart the application as administrator.")
                    .title("Administrator Required")
                    .blocking_show();
                
                std::process::exit(1);
            }
            
            log::info!("✅ Running with administrator privileges");
            
            // Check and disable Windows proxy on startup
            match check_and_disable_windows_proxy() {
                Ok(message) => log::info!("🔧 Startup proxy check: {}", message),
                Err(e) => log::error!("⚠️ Startup proxy check failed: {}", e),
            }
            
            // Show the main window after initialization
            let main_window = app.get_webview_window("main").unwrap();
            main_window.show().unwrap();
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Check if any game is running before allowing close
                match is_any_game_running() {
                    Ok(true) => {
                        // Game is running, prevent close and show warning
                        api.prevent_close();
                        
                        // Show warning dialog
                        use tauri_plugin_dialog::DialogExt;
                        let dialog = window.app_handle().dialog()
                            .message("Cannot close launcher while a game is running.\n\nClosing the launcher while playing will cause:\n• Proxy settings not to be turned off automatically\n• Remaining patch files not to be deleted\n• Game may not run normally on official servers\n\nPlease close the game first, then close the launcher.")
                            .title("Game Running - Cannot Close Launcher")
                            .buttons(tauri_plugin_dialog::MessageDialogButtons::Ok);
                        
                        // Show dialog in a separate thread to avoid blocking
                          tauri::async_runtime::spawn(async move {
                              dialog.show(|_| {});
                          });
                    }
                    Ok(false) => {
                        // No game running, allow close
                        // Cleanup will be handled by the normal shutdown process
                    }
                    Err(e) => {
                        // Error checking game status, log it but allow close
                        log::error!("⚠️ Error checking game status during close: {}", e);
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
