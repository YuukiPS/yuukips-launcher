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
use crate::utils::get_game_executable_names;

// Global game monitoring state
static GAME_MONITOR_STATE: once_cell::sync::Lazy<Arc<Mutex<Option<GameMonitorHandle>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

struct GameMonitorHandle {
    should_stop: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
   //game_id: Number,
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
    game_id: Number,
    version: String,
    channel: Number,
    game_folder_path: String,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Validate game folder path
        if game_folder_path.is_empty() {
            return Err(format!(
                "Game folder path not set for {} version {}. Please configure it in game settings.",
                game_id, version
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
        let game_exe_names = get_game_executable_names(&game_id)?;
        let game_exe_name = game_exe_names[0]; // Use first executable name

        // Construct full path to game executable
        let game_exe_path = Path::new(&game_folder_path).join(game_exe_name);

        // Check if game executable exists
        if !game_exe_path.exists() {
            return Err(format!(
                "Game executable not found: {} = {}. Please verify the game installation.",
                game_exe_path.display(),
                game_id
            ));
        }

        // Calculate MD5 for game executable
        let file_contents = std::fs::read(&game_exe_path)
            .map_err(|e| format!("Failed to read game executable for MD5 calculation: {}", e))?;
        let md5 = md5::compute(&file_contents);
        let md5_str = format!("{:x}", md5);

        // Apply patches if needed
        let (patched_files, patch_response_data) = match check_and_apply_patches(
            game_id.clone(),
            version.clone(),
            channel.clone(),
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
                        println!("📢 Important message: {}", resp.message);
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
        if let Err(e) = start_game_monitor(game_id.clone()) {
            return Err(format!("Failed to start game monitoring: {}", e));
        }

        // Update the game monitor with patching information
        if let Ok(mut monitor_state) = GAME_MONITOR_STATE.lock() {
            if let Some(handle) = monitor_state.as_mut() {
                handle.version = version.clone();
                handle.channel = channel.clone();
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

/// Check if a game is installed
#[command]
pub fn check_game_installed(_game_id: Number, _version: String, game_folder_path: String) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if game is installed by verifying the configured folder path exists
        if game_folder_path.is_empty() {
            return false; // No path configured means not installed
        }

        // Check if the configured game folder exists
        Path::new(&game_folder_path).exists()
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Internal function to check if a specific game is running
pub fn check_game_running_internal(game_id: &Number) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let game_exe_names = get_game_executable_names(game_id)?;

        // Check each possible executable name
        for game_exe_name in game_exe_names {
            let output = Command::new("tasklist")
                .args([
                    "/FI",
                    &format!("IMAGENAME eq {}", game_exe_name),
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
                    if line.contains(game_exe_name) && !line.trim().is_empty() {
                        // Additional validation: check if the line contains actual process info
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 2 && parts[0].trim_matches('"') == game_exe_name {
                            return Ok(true);
                        }
                    }
                }
            } else {
                // If tasklist fails, log the error but continue checking other executables
                let error_msg = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "⚠️ Warning: tasklist failed for {}: {}",
                    game_exe_name, error_msg
                );
            }
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
pub fn check_game_running(game_id: Number) -> Result<bool, String> {
    check_game_running_internal(&game_id)
}

/// Kill game processes if running
pub fn kill_game_processes(game_id: &Number) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let game_exe_names = get_game_executable_names(game_id)?;
        let mut killed_processes = Vec::new();

        // Kill each possible executable
        for game_exe_name in game_exe_names {
            // First check if the process is running
            let check_output = Command::new("tasklist")
                .args([
                    "/FI",
                    &format!("IMAGENAME eq {}", game_exe_name),
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
                    if line.contains(game_exe_name) && !line.trim().is_empty() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 2 && parts[0].trim_matches('"') == game_exe_name {
                            process_found = true;
                            break;
                        }
                    }
                }

                if process_found {
                    println!("🔪 Killing running game process: {}", game_exe_name);

                    // Try to kill the process gracefully first
                    let kill_output = Command::new("taskkill")
                        .args(["/IM", game_exe_name, "/T"])
                        .output();

                    match kill_output {
                        Ok(output) if output.status.success() => {
                            killed_processes.push(game_exe_name.to_string());
                            println!("✅ Successfully killed: {}", game_exe_name);

                            // Wait a moment for the process to fully terminate
                            std::thread::sleep(Duration::from_millis(1000));
                        }
                        Ok(output) => {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            println!(
                                "⚠️ Failed to kill {} gracefully: {}",
                                game_exe_name, error_msg
                            );

                            // Try force kill as fallback
                            let force_kill_output = Command::new("taskkill")
                                .args(["/IM", game_exe_name, "/T", "/F"])
                                .output();

                            match force_kill_output {
                                Ok(force_output) if force_output.status.success() => {
                                    killed_processes
                                        .push(format!("{} (force killed)", game_exe_name));
                                    println!("✅ Force killed: {}", game_exe_name);
                                    std::thread::sleep(Duration::from_millis(1000));
                                }
                                _ => {
                                    return Err(format!(
                                        "Failed to kill {}: {}",
                                        game_exe_name, error_msg
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(format!(
                                "Failed to execute taskkill for {}: {}",
                                game_exe_name, e
                            ));
                        }
                    }
                }
            }
        }

        if killed_processes.is_empty() {
            Ok("No game processes were running".to_string())
        } else {
            Ok(format!(
                "Killed game processes: {}",
                killed_processes.join(", ")
            ))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Process killing not supported on this platform".to_string())
    }
}

/// Kill a specific game
#[command]
pub fn kill_game(game_id: Number) -> Result<String, String> {
    kill_game_processes(&game_id)
}

/// Start monitoring a specific game
#[command]
pub fn start_game_monitor(game_id: Number) -> Result<String, String> {
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

    let thread_handle = thread::spawn(move || {
        let mut last_game_state = false;
        let mut proxy_started_by_us = false;
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 5;

        println!(
            "🔍 Started monitoring game {} for automatic proxy management",
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

            // Check if game is currently running
            match check_game_running_internal(&game_id_clone) {
                Ok(is_running) => {
                    consecutive_errors = 0; // Reset error counter on success

                    if is_running != last_game_state {
                        if is_running {
                            // Game just started - check if proxy should be started
                            let should_start_proxy =
                                if let Ok(monitor_state) = GAME_MONITOR_STATE.lock() {
                                    if let Some(handle) = monitor_state.as_ref() {
                                        if let Ok(response) = handle.patch_response.lock() {
                                            if let Some(ref resp) = *response {
                                                resp.proxy // Only start proxy if patch response allows it
                                            } else {
                                                true // Default to true if no patch response
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
                                            eprintln!(
                                                "⚠️ Failed to start proxy when game started: {}",
                                                e
                                            );
                                        }
                                    }
                                } else {
                                    println!(
                                        "🎮 Game {} started - Proxy was already running",
                                        game_id_clone
                                    );
                                }
                            } else {
                                println!(
                                    "🎮 Game {} started - Proxy disabled by patch response",
                                    game_id_clone
                                );
                            }
                        } else {
                            // Game just stopped - handle cleanup
                            handle_game_stopped_cleanup(&game_id_clone);

                            // Stop proxy
                            if proxy::is_proxy_running() {
                                match proxy::force_stop_proxy() {
                                    Ok(_) => {
                                        proxy_started_by_us = false;
                                        println!("🎮 Game {} stopped - Proxy force stopped automatically", game_id_clone);
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "⚠️ Failed to force stop proxy when game stopped: {}",
                                            e
                                        );
                                        // Try regular stop as fallback
                                        match proxy::stop_proxy() {
                                            Ok(_) => {
                                                proxy_started_by_us = false;
                                                println!("🎮 Game {} stopped - Proxy stopped with fallback method", game_id_clone);
                                            }
                                            Err(e2) => {
                                                eprintln!("⚠️ Failed to stop proxy with fallback method: {}", e2);
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!(
                                    "🎮 Game {} stopped - Proxy was not running",
                                    game_id_clone
                                );
                            }

                            // Stop monitoring after game stops
                            println!(
                                "🔧 Game {} stopped - Stopping automatic monitoring",
                                game_id_clone
                            );
                            break;
                        }
                        last_game_state = is_running;
                    }
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
                        if proxy::is_proxy_running() {
                            match proxy::force_stop_proxy() {
                                Ok(_) => {
                                    proxy_started_by_us = false;
                                    println!("🎮 Game {} assumed stopped due to errors - Proxy force stopped", game_id_clone);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "⚠️ Failed to force stop proxy after error detection: {}",
                                        e
                                    );
                                    match proxy::stop_proxy() {
                                        Ok(_) => {
                                            proxy_started_by_us = false;
                                            println!("🎮 Game {} assumed stopped due to errors - Proxy stopped with fallback", game_id_clone);
                                        }
                                        Err(e2) => {
                                            eprintln!("⚠️ Failed to stop proxy with fallback after error detection: {}", e2);
                                        }
                                    }
                                }
                            }
                        }

                        // Stop monitoring after assuming game stopped
                        println!("🔧 Game {} assumed stopped due to errors - Stopping automatic monitoring", game_id_clone);
                        break;
                    }
                }
            }

            // Wait 1 second before next check
            thread::sleep(Duration::from_secs(1));
        }

        // Clean up proxy if we started it when monitor stops
        if proxy_started_by_us && proxy::is_proxy_running() {
            let _ = proxy::stop_proxy();
            println!("🔧 Monitor stopped - Proxy deactivated");
        }
    });

    *monitor_state = Some(GameMonitorHandle {
        should_stop,
        thread_handle: Some(thread_handle),
        //game_id: game_id.clone(),
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
fn handle_game_stopped_cleanup(_game_id: &Number) {
    // First, restore patched files
    if let Ok(monitor_state) = GAME_MONITOR_STATE.lock() {
        if let Some(handle) = monitor_state.as_ref() {
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
            }
        }
    }
}

/// Stop game monitoring
#[command]
pub fn stop_game_monitor() -> Result<String, String> {
    let mut monitor_state = GAME_MONITOR_STATE
        .lock()
        .map_err(|e| format!("Failed to lock monitor state: {}", e))?;

    if let Some(mut handle) = monitor_state.take() {
        *handle.should_stop.lock().unwrap() = true;
        if let Some(thread_handle) = handle.thread_handle.take() {
            let _ = thread_handle.join();
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

/// Stop a game process by PID
#[command]
pub fn stop_game_process(process_id: u32) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // First check if the process exists
        let check_output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", process_id)])
            .output()
            .map_err(|e| format!("Failed to check process existence: {}", e))?;

        if check_output.status.success() {
            let check_output_str = String::from_utf8_lossy(&check_output.stdout);
            if !check_output_str.contains(&process_id.to_string()) {
                return Ok(format!(
                    "Process with PID {} is not running (already terminated)",
                    process_id
                ));
            }
        }

        // Use taskkill to terminate the process by PID
        let output = Command::new("taskkill")
            .args(["/PID", &process_id.to_string(), "/F"])
            .output()
            .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

        if output.status.success() {
            Ok(format!(
                "Successfully terminated process with PID: {}",
                process_id
            ))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            // Handle common error cases more gracefully
            if error_msg.contains("not found") || error_msg.contains("not running") {
                Ok(format!(
                    "Process with PID {} was not running (already terminated)",
                    process_id
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
pub fn stop_game(game_id: Number) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let game_exe_names = get_game_executable_names(&game_id)?;
        let mut terminated_processes = Vec::new();
        let mut last_error = None;

        // Try to terminate each possible executable
        for game_exe_name in game_exe_names {
            let output = Command::new("taskkill")
                .args(["/IM", game_exe_name, "/F"])
                .output()
                .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

            if output.status.success() {
                terminated_processes.push(game_exe_name.to_string());
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                // Only store error if it's not a "process not found" error
                if !error_msg.contains("not found") && !error_msg.contains("not running") {
                    last_error = Some(format!(
                        "Failed to terminate {}: {}",
                        game_exe_name, error_msg
                    ));
                }
            }
        }

        if !terminated_processes.is_empty() {
            Ok(format!(
                "Successfully terminated: {}",
                terminated_processes.join(", ")
            ))
        } else if let Some(error) = last_error {
            Err(error)
        } else {
            Ok("No game processes were running".to_string())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Game termination is only supported on Windows".to_string())
    }
}
