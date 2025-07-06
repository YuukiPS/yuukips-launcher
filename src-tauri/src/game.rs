//! Game management module
//! Handles game launching, monitoring, and process management

use scopeguard;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;


use tauri::command;

use crate::patch::{
    check_and_apply_patches, cleanup_remaining_patches, restore_from_backups,
    restore_original_files,
};
use crate::proxy;
use crate::hoyoplay::{get_game_executable_names, remove_all_hoyo_pass};

use crate::utils::create_hidden_command;
use crate::system::start_task_manager_monitor_internal;

// Global game monitoring state
static GAME_MONITOR_STATE: once_cell::sync::Lazy<Arc<Mutex<Option<GameMonitorHandle>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

struct GameMonitorHandle {
    should_stop: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
    game_folder_path: String,
    patched_files: Arc<Mutex<Vec<String>>>,
    patch_response: Arc<Mutex<Option<crate::patch::PatchResponse>>>,
}

#[derive(Serialize, Deserialize)]
pub struct LaunchResult {
    pub message: String,
    #[serde(rename = "processId")]
    pub process_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct PatchCheckResult {
    pub has_message: bool,
    pub message: String,
    pub can_proceed: bool,
}

/// Check for patch messages before launching
#[command]
pub fn check_patch_message(
    _game_id: Number,
    _version: String,
    _channel: Number,
    game_folder_path: String,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Validate game folder path
        if game_folder_path.is_empty() {
            return Err(format!(
                "Game folder path not set for {} version {}. Please configure it in game settings.",
                _game_id, _version
            ));
        }

        // Check if game folder exists
        if !Path::new(&game_folder_path).exists() {
            return Err(format!(
                "Game folder not found: {}. Please verify the path in game settings.",
                game_folder_path
            ));
        }

        // Get game executable name
        let game_exe_name = get_game_executable_names(_game_id.clone(), _channel.clone())?;

        // Construct full path to game executable
        let game_exe_path = Path::new(&game_folder_path).join(game_exe_name);

        // Check if game executable exists
        if !game_exe_path.exists() {
            return Err(format!(
                "Game executable not found: {} = {} > channel_id {}. Please verify the game installation.",
                game_exe_path.display(),
                _game_id,
                _channel
            ));
        }

        // Calculate MD5 for game executable
        let file_contents = std::fs::read(&game_exe_path)
            .map_err(|e| format!("Failed to read game executable for MD5 calculation: {}", e))?;
        let md5 = md5::compute(&file_contents);
        let md5_str = format!("{:x}", md5);

        // Check patches to get message without applying them
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create async runtime: {}", e))?;
        
        let result = rt.block_on(async {
            match crate::patch::fetch_patch_info(_game_id.clone(), _version.clone(), _channel.clone(), md5_str.clone()).await {
                Ok(patch_response) => {
                    if !patch_response.message.is_empty() {
                        PatchCheckResult {
                            has_message: true,
                            message: patch_response.message.clone(),
                            can_proceed: true,
                        }
                    } else {
                        PatchCheckResult {
                            has_message: false,
                            message: String::new(),
                            can_proceed: true,
                        }
                    }
                }
                Err(_) => {
                    PatchCheckResult {
                        has_message: false,
                        message: String::new(),
                        can_proceed: true,
                    }
                }
            }
        });
        
        match serde_json::to_string(&result) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Failed to serialize patch check result: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Patch checking is only supported on Windows".to_string())
    }
}

/// Get the configured game folder path
#[command]
pub fn get_game_folder_path(game_id: Number, version: String) -> Result<String, String> {
    // This would typically read from a config file or database
    // For now, we'll return an error indicating the path should be set via frontend
    Err(format!(
        "Game folder path not configured for game {} version {}. Please set it in game settings.",
        game_id, version
    ))
}

/// Launch a game with the specified parameters
#[command]
pub fn launch_game(
    app_handle: tauri::AppHandle,
    _game_id: Number,
    _version: String,
    _channel: Number,
    game_folder_path: String,
    delete_hoyo_pass: Option<bool>,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Validate game folder path
        if game_folder_path.is_empty() {
            return Err(format!(
                "Game folder path not set for {} version {}. Please configure it in game settings.",
                _game_id, _version
            ));
        }

        // Check if game folder exists
        if !Path::new(&game_folder_path).exists() {
            return Err(format!(
                "Game folder not found: {}. Please verify the path in game settings.",
                game_folder_path
            ));
        }

        // Get game executable name
        let game_exe_name = get_game_executable_names(_game_id.clone(), _channel.clone())?;

        // Construct full path to game executable
        let game_exe_path = Path::new(&game_folder_path).join(game_exe_name);

        // Check if game executable exists
        if !game_exe_path.exists() {
            return Err(format!(
                "Game executable not found: {} = {} > channel_id {}. Please verify the game installation.",
                game_exe_path.display(),
                _game_id,
                _channel
            ));
        }

        // Calculate MD5 for game executable
        let file_contents = std::fs::read(&game_exe_path)
            .map_err(|e| format!("Failed to read game executable for MD5 calculation: {}", e))?;
        let md5 = md5::compute(&file_contents);
        let md5_str = format!("{:x}", md5);

        // Apply patches if needed
        let (patched_files, patch_response_data) = match check_and_apply_patches(
            _game_id.clone(),
            _version.clone(),
            _channel.clone(),
            md5_str.clone(),
            game_folder_path.clone(),
        ) {
            Ok((patch_message, response, files)) => {
                if !patch_message.is_empty() {
                    println!("🔧 Patch status: {}", patch_message);
                }
                let patch_response_data = response.clone();

                // Check if we need to show a message to user before proceeding
                if let Some(ref resp) = response {
                    if !resp.message.is_empty() {
                        //println!("📢 Important message: {}", resp.message);
                    }

                    // Check if proxy should be skipped
                    if !resp.proxy {
                        println!("⚠️ Proxy disabled by patch response");
                    }
                }
                (files, patch_response_data)
            }
            Err(e) => {
                let _ = stop_game_monitor();
                return Err(format!("Cannot launch game: Patching failed. Error: {}", e));
            }
        };

        // Start game monitoring AFTER patching is complete
        if let Err(e) = start_game_monitor(app_handle, _game_id.clone(), _channel.clone()) {
            return Err(format!("Failed to start game monitoring: {}", e));
        }

        // Update the game monitor with patching information
        if let Ok(mut monitor_state) = GAME_MONITOR_STATE.lock() {
            if let Some(handle) = monitor_state.as_mut() {
                handle.version = _version.clone();
                handle.channel = _channel.clone();
                handle.md5 = md5_str.clone();
                handle.game_folder_path = game_folder_path.clone();

                // Update patched files list
                if let Ok(mut files) = handle.patched_files.lock() {
                    *files = patched_files;
                }

                // Update patch response
                if let Ok(mut response) = handle.patch_response.lock() {
                    *response = patch_response_data;
                }
            }
        }

        // Remove HoyoPass entries if requested (default: true)
        let should_delete_hoyo_pass = delete_hoyo_pass.unwrap_or(true);
        if should_delete_hoyo_pass {
            match remove_all_hoyo_pass() {
                Ok(deleted_entries) => {
                    if !deleted_entries.is_empty() {
                        println!("🗑️ Removed {} HoyoPass registry entries: {:?}", deleted_entries.len(), deleted_entries);
                    }
                }
                Err(e) => {
                    println!("⚠️ Warning: Failed to remove HoyoPass entries: {}", e);
                    // Continue with game launch even if HoyoPass removal fails
                }
            }
        }

        // Launch the game executable
        match Command::new(&game_exe_path)
            .current_dir(&game_folder_path)
            .spawn()
        {
            Ok(child) => {
                let process_id = child.id();
                let result = LaunchResult {
                    message: format!(
                        "Successfully launched: {} > {}",
                        game_exe_path.display(),
                        md5_str
                    ),
                    process_id,
                };
                match serde_json::to_string(&result) {
                    Ok(json) => Ok(json),
                    Err(e) => Err(format!("Failed to serialize launch result: {}", e)),
                }
            }
            Err(e) => {
                // If game launch fails, stop monitoring and clean up proxy
                let _ = stop_game_monitor();
                let _ = proxy::stop_proxy();
                Err(format!("Failed to launch game: {}", e))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Game launching is only supported on Windows".to_string())
    }
}

/// Validate a game directory path without launching the game
#[command]
pub fn validate_game_directory(
    _game_id: Number,
    _channel: Number,
    game_folder_path: String,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Validate game folder path
        if game_folder_path.is_empty() {
            return Err("Game folder path cannot be empty".to_string());
        }

        // Check if game folder exists
        if !Path::new(&game_folder_path).exists() {
            return Err(format!(
                "Game folder not found: {}. Please verify the path.",
                game_folder_path
            ));
        }

        // Get game executable name
        let game_exe_name = get_game_executable_names(_game_id.clone(), _channel.clone())?;

        // Construct full path to game executable
        let game_exe_path = Path::new(&game_folder_path).join(&game_exe_name);

        // Check if game executable exists
        if !game_exe_path.exists() {
            return Err(format!(
                "Game executable not found: {}. Please verify the game installation.",
                game_exe_path.display()
            ));
        }

        // Calculate MD5 for game executable (same as launch_game)
        let file_contents = std::fs::read(&game_exe_path)
            .map_err(|e| format!("Failed to read game executable for MD5 calculation: {}", e))?;
        
        if file_contents.is_empty() {
            return Err(format!(
                "Game executable appears to be corrupted (0 bytes): {}",
                game_exe_path.display()
            ));
        }
        
        let md5 = md5::compute(&file_contents);
        let md5_str = format!("{:x}", md5);

        Ok(format!(
            "Valid game directory: {} (executable: {}, MD5: {})",
            game_folder_path,
            game_exe_name,
            md5_str
        ))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Game validation is only supported on Windows".to_string())
    }
}

/// Check if a game is installed
#[command]
pub fn check_game_installed(_game_id: Number, _version: String, _game_folder_path: String) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if game is installed by verifying the configured folder path exists
        if _game_folder_path.is_empty() {
            return false; // No path configured means not installed
        }

        // Check if the configured game folder exists
        Path::new(&_game_folder_path).exists()
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Internal function to check if a specific game is running
pub fn check_game_running_internal(_game_id: &Number, _channel_id: &Number) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let game_exe_name = get_game_executable_names(_game_id.clone(), _channel_id.clone())?;
        
        let output = create_hidden_command("tasklist")
            .args([
                "/FI",
                &format!("IMAGENAME eq {}", &game_exe_name),
                "/FO",
                "CSV",
            ])
            .output()
            .map_err(|e| format!("Failed to execute tasklist: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Check if the game executable is listed in the output
            let lines: Vec<&str> = output_str.lines().collect();
            for line in lines.iter().skip(1) {
                // Skip header line
                if line.contains(&game_exe_name) && !line.trim().is_empty() {
                    // Additional validation: check if the line contains actual process info
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 && parts[0].trim_matches('"') == &game_exe_name {
                        return Ok(true);
                    }
                }
            }
        } else {
            // If tasklist fails, return the error
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tasklist failed for {}: {}", game_exe_name, error_msg));
        }
        
        Ok(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(false)
    }
}

/// Check if a game is currently running
#[command]
pub fn check_game_running(game_id: Number, channel_id: Number) -> Result<bool, String> {
    check_game_running_internal(&game_id, &channel_id)
}

/// Kill game processes if running
pub fn kill_game_processes(_game_id: &Number, _channel_id: &Number) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let game_exe_name = get_game_executable_names(_game_id.clone(), _channel_id.clone())?;
        
        // First check if the process is running
        let check_output = create_hidden_command("tasklist")
            .args([
                "/FI",
                &format!("IMAGENAME eq {}", &game_exe_name),
                "/FO",
                "CSV",
            ])
            .output()
            .map_err(|e| format!("Failed to execute tasklist: {}", e))?;

        if check_output.status.success() {
            let output_str = String::from_utf8_lossy(&check_output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            let mut process_found = false;

            for line in lines.iter().skip(1) {
                // Skip header line
                if line.contains(&game_exe_name) && !line.trim().is_empty() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 && parts[0].trim_matches('"') == &game_exe_name {
                        process_found = true;
                        break;
                    }
                }
            }

            if process_found {
                println!("🔪 Killing running game process: {}", game_exe_name);

                // Try to kill the process gracefully first
                let kill_output = create_hidden_command("taskkill")
                    .args(["/IM", &game_exe_name, "/T"])
                    .output();

                match kill_output {
                    Ok(output) if output.status.success() => {
                        println!("✅ Successfully killed: {}", game_exe_name);
                        // Wait a moment for the process to fully terminate
                        std::thread::sleep(Duration::from_millis(1000));
                        Ok(format!("Killed game process: {}", game_exe_name))
                    }
                    Ok(output) => {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        println!(
                            "⚠️ Failed to kill {} gracefully: {}",
                            game_exe_name, error_msg
                        );

                        // Try force kill as fallback
                        let force_kill_output = create_hidden_command("taskkill")
                            .args(["/IM", &game_exe_name, "/T", "/F"])
                            .output();

                        match force_kill_output {
                            Ok(force_output) if force_output.status.success() => {
                                println!("✅ Force killed: {}", game_exe_name);
                                std::thread::sleep(Duration::from_millis(1000));
                                Ok(format!("Force killed game process: {}", game_exe_name))
                            }
                            _ => {
                                Err(format!(
                                    "Failed to kill {}: {}",
                                    game_exe_name, error_msg
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        Err(format!(
                            "Failed to execute taskkill for {}: {}",
                            game_exe_name, e
                        ))
                    }
                }
            } else {
                Ok("No game processes were running".to_string())
            }
        } else {
            let error_msg = String::from_utf8_lossy(&check_output.stderr);
            Err(format!("Failed to check process status: {}", error_msg))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Process killing not supported on this platform".to_string())
    }
}

/// Kill a specific game
#[command]
pub fn kill_game(game_id: Number, channel_id: Number) -> Result<String, String> {
    // Stop the monitor first - this will handle cleanup automatically
    if let Ok(monitor_state) = GAME_MONITOR_STATE.lock() {
        if let Some(handle) = monitor_state.as_ref() {
            if handle.game_id == game_id {
                drop(monitor_state); // Release lock before calling stop_game_monitor
                let _ = stop_game_monitor();
            }
        }
    }
    
    let result = kill_game_processes(&game_id, &channel_id);
    
    result
}

/// Start monitoring a specific game - Single source of truth for game monitoring
#[command]
pub fn start_game_monitor(app_handle: tauri::AppHandle, game_id: Number, channel_id: Number) -> Result<String, String> {
    let mut monitor_state = GAME_MONITOR_STATE
        .lock()
        .map_err(|e| format!("Failed to lock monitor state: {}", e))?;

    // Stop existing monitor if running
    if let Some(mut handle) = monitor_state.take() {
        *handle.should_stop.lock().unwrap() = true;
        if let Some(thread_handle) = handle.thread_handle.take() {
            let _ = thread_handle.join();
        }
    }

    let should_stop = Arc::new(Mutex::new(false));
    let should_stop_clone = Arc::clone(&should_stop);
    let game_id_clone = game_id.clone();
    let channel_id_clone = channel_id.clone();
    let app_handle_clone = app_handle.clone();

    let thread_handle = thread::spawn(move || {
        let mut last_game_state = false;
        let mut proxy_started_by_us = false;
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 5;
        
        // Wait for game to actually start before beginning monitoring
        let mut game_started = false;
        let mut startup_checks = 0;
        const MAX_STARTUP_CHECKS: u32 = 30; // 30 seconds max wait for game to start

        println!(
            "🔍 Started monitoring game {} - waiting for game to start",
            game_id_clone
        );

        // Ensure we clean up the monitor state when thread exits
        let _cleanup_guard = scopeguard::guard((), |_| {
            if let Ok(mut monitor_state) = GAME_MONITOR_STATE.lock() {
                *monitor_state = None;
                println!("🔧 Game monitor state cleared for game {}", game_id_clone);
            }
        });

        loop {
            // Check if we should stop monitoring
            if *should_stop_clone.lock().unwrap() {
                break;
            }

            // Initial startup phase - wait for game to start
            if !game_started {
                startup_checks += 1;
                if startup_checks > MAX_STARTUP_CHECKS {
                    println!("⚠️ Game {} did not start within 30 seconds, stopping monitor", game_id_clone);
                    break;
                }
                
                match check_game_running_internal(&game_id_clone, &channel_id_clone) {
                    Ok(is_running) => {
                        if is_running {
                            game_started = true;
                            last_game_state = true;
                            println!("🎮 Game {} detected as running - starting active monitoring", game_id_clone);
                            
                            // Start proxy if needed
                            let should_start_proxy = if let Ok(monitor_state) = GAME_MONITOR_STATE.lock() {
                                if let Some(handle) = monitor_state.as_ref() {
                                    if let Ok(response) = handle.patch_response.lock() {
                                        if let Some(ref resp) = *response {
                                            resp.proxy
                                        } else {
                                            true
                                        }
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                }
                            } else {
                                true
                            };
                            
                            if should_start_proxy {
                                if !proxy::is_proxy_running() {
                                    match proxy::start_proxy() {
                                        Ok(_) => {
                                            proxy_started_by_us = true;
                                            println!("🎮 Game {} started - Proxy activated automatically", game_id_clone);
                                        }
                                        Err(e) => {
                                            eprintln!("⚠️ Failed to start proxy when game started: {}", e);
                                        }
                                    }
                                }
                            }
                            
                            // Start Task Manager monitoring when game starts
                            match start_task_manager_monitor_internal(app_handle_clone.clone()) {
                                Ok(_) => {
                                    println!("🔍 Task Manager monitoring started for game {}", game_id_clone);
                                }
                                Err(e) => {
                                    eprintln!("⚠️ Failed to start Task Manager monitoring: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ Error checking game startup status: {}", e);
                    }
                }
                
                // Wait 1 second before next startup check
                thread::sleep(Duration::from_secs(1));
                continue;
            }

            // Active monitoring phase - game is running, monitor for stop
            match check_game_running_internal(&game_id_clone, &channel_id_clone) {
                Ok(is_running) => {
                    consecutive_errors = 0; // Reset error counter on success

                    if !is_running {
                        // Game has stopped - handle cleanup in separate thread to avoid blocking
                        if let Ok(monitor_state) = GAME_MONITOR_STATE.lock() {
                            if let Some(handle) = monitor_state.as_ref() {
                                // Clone the handle data for cleanup
                                let handle_clone = GameMonitorHandle {
                                    should_stop: Arc::clone(&handle.should_stop),
                                    thread_handle: None,
                                    game_id: handle.game_id.clone(),
                                    version: handle.version.clone(),
                                    channel: handle.channel.clone(),
                                    md5: handle.md5.clone(),
                                    game_folder_path: handle.game_folder_path.clone(),
                                    patched_files: Arc::clone(&handle.patched_files),
                                    patch_response: Arc::clone(&handle.patch_response),
                                };
                                
                                thread::spawn(move || {
                                    handle_game_stopped_cleanup_with_handle(&handle_clone);
                                });
                            }
                        }

                        // Stop proxy without blocking
                        if proxy::is_proxy_running() && proxy_started_by_us {
                            // Spawn detached thread for proxy cleanup - don't wait for it
                            thread::spawn(|| {
                                match proxy::force_stop_proxy() {
                                    Ok(_) => {
                                        println!("🎮 Proxy force stopped automatically");
                                    }
                                    Err(e) => {
                                        eprintln!("⚠️ Failed to force stop proxy: {}", e);
                                        // Try regular stop as fallback
                                        match proxy::stop_proxy() {
                                            Ok(_) => {
                                                println!("🎮 Proxy stopped with fallback method");
                                            }
                                            Err(e2) => {
                                                eprintln!("⚠️ Failed to stop proxy with fallback: {}", e2);
                                            }
                                        }
                                    }
                                }
                            });
                            proxy_started_by_us = false;
                        }

                        // Stop monitoring after game stops
                        println!("🔧 Monitor stopped");
                        break;
                    }
                    // Game is still running, continue monitoring
                }
                Err(e) => {
                    consecutive_errors += 1;
                    eprintln!(
                        "⚠️ Error checking game status (attempt {}): {}",
                        consecutive_errors, e
                    );

                    // If we have too many consecutive errors, assume game stopped
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS && last_game_state {
                        eprintln!(
                            "⚠️ Too many consecutive errors, assuming game {} has stopped",
                            game_id_clone
                        );
                        if proxy::is_proxy_running() && proxy_started_by_us {
                            // Spawn detached thread for proxy cleanup - don't wait for it
                            thread::spawn(|| {
                                match proxy::force_stop_proxy() {
                                    Ok(_) => {
                                        println!("🎮 Proxy force stopped due to errors");
                                    }
                                    Err(e) => {
                                        eprintln!("⚠️ Failed to force stop proxy after error detection: {}", e);
                                        match proxy::stop_proxy() {
                                            Ok(_) => {
                                                println!("🎮 Proxy stopped with fallback due to errors");
                                            }
                                            Err(e2) => {
                                                eprintln!("⚠️ Failed to stop proxy with fallback after error detection: {}", e2);
                                            }
                                        }
                                    }
                                }
                            });
                            proxy_started_by_us = false;
                        }

                        // Stop monitoring after assuming game stopped
                        println!("🔧 Monitor stopped");
                        break;
                    }
                }
            }

            // Wait 3 seconds before next check (reduced frequency since we only monitor for stop)
            thread::sleep(Duration::from_secs(3));
        }

        // Clean up proxy if we started it when monitor stops (detached)
        if proxy_started_by_us && proxy::is_proxy_running() {
            // Spawn detached cleanup thread - don't wait for it
            thread::spawn(|| {
                match proxy::stop_proxy() {
                    Ok(_) => {
                        println!("🔧 Monitor stopped - Proxy deactivated");
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to stop proxy during cleanup: {}", e);
                    }
                }
            });
        }
    });

    *monitor_state = Some(GameMonitorHandle {
        should_stop,
        thread_handle: Some(thread_handle),
        game_id: game_id.clone(),
        version: String::new(),
        channel: Number::from(0),
        md5: String::new(),
        game_folder_path: String::new(),
        patched_files: Arc::new(Mutex::new(Vec::new())),
        patch_response: Arc::new(Mutex::new(None)),
    });

    Ok(format!(
        "Started monitoring game {} - proxy will auto-start/stop with game",
        game_id
    ))
}

/// Handle cleanup when game stops
fn handle_game_stopped_cleanup_with_handle(handle: &GameMonitorHandle) {
    // Get patched files list and patch response
    let patched_files = if let Ok(files) = handle.patched_files.lock() {
        files.clone()
    } else {
        Vec::new()
    };

    let patch_response = if let Ok(response) = handle.patch_response.lock() {
        response.clone()
    } else {
        None
    };

    // Restore files if we have patch information
    if !patched_files.is_empty() {
        println!("🔄 Starting cleanup for {} patched files...", patched_files.len());
        
        if let Some(response) = patch_response {
            // Try API-based restoration first
            match restore_original_files(&response, &handle.game_folder_path) {
                Ok(message) => {
                    println!("🔄 {}", message);
                }
                Err(e) => {
                    println!("⚠️ API restoration failed: {}", e);
                    // Fallback to backup restoration
                    match restore_from_backups(&handle.game_folder_path, &patched_files) {
                        Ok(message) => {
                            println!("🔄 {}", message);
                        }
                        Err(e) => {
                            println!("⚠️ Backup restoration also failed: {}", e);
                        }
                    }
                }
            }

            // Additional cleanup: rename any remaining patched files to .patch
            match cleanup_remaining_patches(&handle.game_folder_path, &patched_files) {
                Ok(message) => {
                    if !message.is_empty() {
                        println!("🧹 {}", message);
                    }
                }
                Err(e) => {
                    println!("⚠️ Patch cleanup warning: {}", e);
                }
            }
        } else {
            // No patch response, try backup restoration
            match restore_from_backups(&handle.game_folder_path, &patched_files) {
                Ok(message) => {
                    println!("🔄 {}", message);
                }
                Err(e) => {
                    println!("⚠️ Backup restoration failed: {}", e);
                }
            }
        }
        
        println!("✅ Cleanup completed");
    } else {
        println!("ℹ️ No patched files to clean up");
    }
}

/// Stop game monitoring
#[command]
pub fn stop_game_monitor() -> Result<String, String> {
    let mut monitor_state = GAME_MONITOR_STATE
        .lock()
        .map_err(|e| format!("Failed to lock monitor state: {}", e))?;

    if let Some(mut handle) = monitor_state.take() {
        // Signal the thread to stop first
        *handle.should_stop.lock().unwrap() = true;
        
        // Perform cleanup in a separate thread to avoid blocking the UI
        // We need to clone the handle data before moving it
        let handle_clone = GameMonitorHandle {
            should_stop: Arc::clone(&handle.should_stop),
            thread_handle: None, // We don't need the thread handle for cleanup
            game_id: handle.game_id.clone(),
            version: handle.version.clone(),
            channel: handle.channel.clone(),
            md5: handle.md5.clone(),
            game_folder_path: handle.game_folder_path.clone(),
            patched_files: Arc::clone(&handle.patched_files),
            patch_response: Arc::clone(&handle.patch_response),
        };
        
        thread::spawn(move || {
            handle_game_stopped_cleanup_with_handle(&handle_clone);
        });
        
        // Don't wait for thread to join - just detach it
        // The thread will clean itself up when it detects the stop signal
        if let Some(_thread_handle) = handle.thread_handle.take() {
            // Thread will stop on its own when it checks should_stop flag
            println!("🔧 Game monitor stop signal sent");
        }
        
        Ok("Game monitoring stopped".to_string())
    } else {
        Err("Game monitoring is not active".to_string())
    }
}

/// Check if game monitoring is active
#[command]
pub fn is_game_monitor_active() -> Result<bool, String> {
    let monitor_state = GAME_MONITOR_STATE
        .lock()
        .map_err(|e| format!("Failed to lock monitor state: {}", e))?;
    Ok(monitor_state.is_some())
}

/// Check if any game is currently running (used for launcher close prevention)
#[command]
pub fn is_any_game_running() -> Result<bool, String> {
    let monitor_state = GAME_MONITOR_STATE
        .lock()
        .map_err(|e| format!("Failed to lock monitor state: {}", e))?;
    
    if let Some(handle) = monitor_state.as_ref() {
        // If monitor is active, check if the game is actually running
        match check_game_running_internal(&handle.game_id, &handle.channel) {
            Ok(is_running) => Ok(is_running),
            Err(_) => Ok(false), // If we can't check, assume not running
        }
    } else {
        Ok(false) // No monitor active means no game running
    }
}

/// Force stop game monitor - ensures clean shutdown
#[command]
pub fn force_stop_game_monitor() -> Result<String, String> {
    let mut monitor_state = GAME_MONITOR_STATE
        .lock()
        .map_err(|e| format!("Failed to lock monitor state: {}", e))?;
    
    if let Some(mut handle) = monitor_state.take() {
        // Signal the thread to stop first
        *handle.should_stop.lock().unwrap() = true;
        
        // Perform cleanup in a separate thread to avoid blocking the UI
        // We need to clone the handle data before moving it
        let handle_clone = GameMonitorHandle {
            should_stop: Arc::clone(&handle.should_stop),
            thread_handle: None, // We don't need the thread handle for cleanup
            game_id: handle.game_id.clone(),
            version: handle.version.clone(),
            channel: handle.channel.clone(),
            md5: handle.md5.clone(),
            game_folder_path: handle.game_folder_path.clone(),
            patched_files: Arc::clone(&handle.patched_files),
            patch_response: Arc::clone(&handle.patch_response),
        };
        
        thread::spawn(move || {
            handle_game_stopped_cleanup_with_handle(&handle_clone);
        });
        
        // Don't wait for thread to join - just detach it
        if let Some(_thread_handle) = handle.thread_handle.take() {
            println!("🔧 Game monitor force stop signal sent");
        }
        
        Ok("Game monitor stopped".to_string())
    } else {
        Ok("No monitor was running".to_string())
    }
}

/// Stop a game process by PID
#[command]
pub fn stop_game_process(_process_id: u32) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // First check if the process exists
        let check_output = create_hidden_command("tasklist")
            .args(["/FI", &format!("PID eq {}", _process_id)])
            .output()
            .map_err(|e| format!("Failed to check process existence: {}", e))?;

        if check_output.status.success() {
            let check_output_str = String::from_utf8_lossy(&check_output.stdout);
            if !check_output_str.contains(&_process_id.to_string()) {
                return Ok(format!(
                    "Process with PID {} is not running (already terminated)",
                    _process_id
                ));
            }
        }

        // Use taskkill to terminate the process by PID
        let output = create_hidden_command("taskkill")
            .args(["/PID", &_process_id.to_string(), "/F"])
            .output()
            .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

        if output.status.success() {
            Ok(format!(
                "Successfully terminated process with PID: {}",
                _process_id
            ))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            // Handle common error cases more gracefully
            if error_msg.contains("not found") || error_msg.contains("not running") {
                Ok(format!(
                    "Process with PID {} was not running (already terminated)",
                    _process_id
                ))
            } else {
                Err(format!("Failed to terminate process: {}", error_msg))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Process termination is only supported on Windows".to_string())
    }
}

/// Stop a game by game ID
#[command]
pub fn stop_game(_game_id: Number, _channel_id: Number) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Stop the monitor first - this will handle cleanup automatically
        if let Ok(monitor_state) = GAME_MONITOR_STATE.lock() {
            if let Some(handle) = monitor_state.as_ref() {
                if handle.game_id == _game_id {
                    drop(monitor_state); // Release lock before calling stop_game_monitor
                    let _ = stop_game_monitor();
                }
            }
        }
        
        let game_exe_name = get_game_executable_names(_game_id.clone(), _channel_id.clone())?;
        
        let output = create_hidden_command("taskkill")
            .args(["/IM", &game_exe_name, "/F"])
            .output()
            .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

        if output.status.success() {
            // Stop the monitor after terminating the game
            let _ = stop_game_monitor();
            Ok(format!("Successfully terminated: {}", game_exe_name))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            // Handle common error cases more gracefully
            if error_msg.contains("not found") || error_msg.contains("not running") {
                Ok("No game processes were running".to_string())
            } else {
                Err(format!("Failed to terminate {}: {}", game_exe_name, error_msg))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Game termination is only supported on Windows".to_string())
    }
}
