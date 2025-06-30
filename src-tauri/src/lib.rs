#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::Number;
use serde::{Deserialize, Serialize};
use tauri::command;
use std::fs;
use std::path::Path;
// Import the proxy module
mod proxy;

// Import scopeguard for cleanup
use scopeguard;

// Global game monitoring state
static GAME_MONITOR_STATE: once_cell::sync::Lazy<Arc<Mutex<Option<GameMonitorHandle>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

// Global download progress state
static DOWNLOAD_PROGRESS: once_cell::sync::Lazy<Arc<Mutex<Vec<DownloadProgress>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DownloadProgress {
    file_name: String,
    downloaded: u64,
    total: u64,
    percentage: f64,
    status: String, // "downloading", "completed", "failed"
}

struct GameMonitorHandle {
    should_stop: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
    game_folder_path: String,
    patched_files: Arc<Mutex<Vec<String>>>,
    patch_response: Arc<Mutex<Option<PatchResponse>>>,
}

#[derive(Serialize, Deserialize)]
struct LaunchResult {
    message: String,
    #[serde(rename = "processId")]
    process_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PatchResponse {
    patch: bool,
    proxy: bool,
    message: String,
    metode: u32,
    #[serde(default)]
    patched: Vec<PatchFile>,
    #[serde(default)]
    original: Vec<PatchFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PatchFile {
    location: String, // location file in game folder, so use path_folder+location
    md5: String, // for check match file 
    file: String, // file url to download
}

// Function to check if running as administrator on Windows
#[cfg(target_os = "windows")]
fn is_running_as_admin() -> bool {
    use std::ptr;
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use winapi::um::handleapi::CloseHandle;
    
    unsafe {
        let mut token_handle = ptr::null_mut();
        let process_handle = GetCurrentProcess();
        
        if winapi::um::processthreadsapi::OpenProcessToken(
            process_handle,
            TOKEN_QUERY,
            &mut token_handle,
        ) == 0 {
            return false;
        }
        
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0;
        
        let result = GetTokenInformation(
            token_handle,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );
        
        CloseHandle(token_handle);
        
        result != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(target_os = "windows"))]
fn is_running_as_admin() -> bool {
    false // Not applicable on non-Windows systems
}

#[command]
fn check_admin_privileges() -> Result<bool, String> {
    Ok(is_running_as_admin())
}

#[command]
fn start_proxy() -> Result<String, String> {
    proxy::start_proxy()
}

#[command]
fn stop_proxy() -> Result<String, String> {
    proxy::stop_proxy()
}

#[command]
fn check_proxy_status() -> Result<bool, String> {
    Ok(proxy::is_proxy_running())
}

#[command]
fn force_stop_proxy() -> Result<String, String> {
    proxy::force_stop_proxy()
}

#[command]
fn check_and_disable_windows_proxy() -> Result<String, String> {
    proxy::check_and_disable_windows_proxy()
}

#[command]
fn install_ssl_certificate() -> Result<String, String> {
    // Use the new install_ca_certificate function from proxy module
    proxy::install_ca_certificate()
}

#[command]
fn check_certificate_status() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        let output = Command::new("certutil")
            .args(["-store", "Root"])
            .output()
            .map_err(|e| format!("Failed to check certificate store: {}", e))?;
        
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("DO_NOT_TRUST_YuukiPS_Root") {
                Ok("installed".to_string())
            } else {
                Ok("not_installed".to_string())
            }
        } else {
            Ok("not_installed".to_string())
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok("manual_check_required".to_string())
    }
}

// Function to make HTTP requests bypassing proxy settings
// Test command to verify proxy bypass functionality
#[command]
fn test_proxy_bypass() -> Result<String, String> {
    println!("[DEBUG] Testing proxy bypass functionality...");
    
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
    
    rt.block_on(async {
        // Test with a simple HTTP endpoint
        let test_url = "https://httpbin.org/ip";
        println!("[DEBUG] Testing with URL: {}", test_url);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .no_proxy()
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        let response = client
            .get(test_url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP request failed with status: {}", response.status()));
        }
        
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;
        
        println!("[DEBUG] Test successful, response: {}", response_text);
        Ok(format!("Proxy bypass test successful: {}", response_text))
    })
}

#[command]
fn fetch_api_data(url: String) -> Result<String, String> {
    println!("[DEBUG] fetch_api_data called with URL: {}", url);
    
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| {
            let error_msg = format!("Failed to create tokio runtime: {}", e);
            println!("[ERROR] {}", error_msg);
            error_msg
        })?;
    
    rt.block_on(async {
        println!("[DEBUG] Creating HTTP client with no_proxy()...");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .no_proxy()
            .build()
            .map_err(|e| {
                let error_msg = format!("Failed to create HTTP client: {}", e);
                println!("[ERROR] {}", error_msg);
                error_msg
            })?;
        
        println!("[DEBUG] Sending GET request to: {}", url);
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                let error_msg = format!("HTTP request failed: {}", e);
                println!("[ERROR] {}", error_msg);
                error_msg
            })?;
        
        println!("[DEBUG] Response status: {}", response.status());
        
        if !response.status().is_success() {
            let error_msg = format!("HTTP request failed with status: {}", response.status());
            println!("[ERROR] {}", error_msg);
            return Err(error_msg);
        }
        
        let response_text = response
            .text()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to read response body: {}", e);
                println!("[ERROR] {}", error_msg);
                error_msg
            })?;
        
        println!("[DEBUG] Response received, length: {} bytes", response_text.len());
        Ok(response_text)
    })
}

#[command]
fn check_ssl_certificate_installed() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Check if our certificate is installed in the Root store
        let output = Command::new("certutil")
            .args(["-store", "Root"])
            .output()
            .map_err(|e| format!("Failed to check certificate store: {}", e))?;
        
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Look for our certificate subject (YuukiPS Proxy)
            Ok(output_str.contains("YuukiPS"))
        } else {
            Ok(false)
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For non-Windows systems, we can't easily check automatically
        // Return false to prompt manual installation
        Ok(false)
    }
}

#[command]
fn get_game_folder_path(game_id: Number, version: String) -> Result<String, String> {
    // This would typically read from a config file or database
    // For now, we'll return an error indicating the path should be set via frontend
    Err(format!("Game folder path not configured for game {} version {}. Please set it in game settings.", game_id, version))
}

#[command]
fn launch_game(
    game_id: Number,    
    version: String,
    channel: Number,
    game_folder_path: String,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Use the provided game folder path from frontend settings
        if game_folder_path.is_empty() {
            return Err(format!("Game folder path not set for {} version {}. Please configure it in game settings.", game_id, version));
        }
        
        // Check if game folder exists
        if !std::path::Path::new(&game_folder_path).exists() {
            return Err(format!("Game folder not found: {}. Please verify the path in game settings.", game_folder_path));
        }
        
        // Determine game executable name based on game ID
        let game_exe_name = match game_id.as_u64() {
            Some(1) => "GenshinImpact.exe",
            Some(2) => "StarRail.exe", // Common names: StarRail.exe, HonkaiStarRail.exe, or StarRail_Data.exe
            //Some(3) => "BlueArchive.exe",
            _ => return Err(format!("Unsupported game ID: {}", game_id)),
        };

        // Construct full path to game executable
        let game_exe_path = std::path::Path::new(&game_folder_path).join(game_exe_name);        
        // Check if game executable exists
        if !game_exe_path.exists() {
            return Err(format!("Game executable not found: {} = {}. Please verify the game installation.", game_exe_path.display(),game_id));
        }

        // Check MD5 for game_exe_path
        let file_contents = std::fs::read(&game_exe_path)
            .map_err(|e| format!("Failed to read game executable for MD5 calculation: {}", e))?;
        let md5 = md5::compute(&file_contents);
        let md5_str = format!("{:x}", md5);

        // Get patch info from API and apply patches if needed
        let (patched_files, patch_response_data) = match check_and_apply_patches(game_id.clone(), version.clone(), channel.clone(), md5_str.clone(), game_folder_path.clone()) {
            Ok((patch_message, response, files)) => {
                if !patch_message.is_empty() {
                    println!("🔧 Patch status: {}", patch_message);
                }
                let patch_response_data = response.clone();
                
                // Check if we need to show a message to user before proceeding
                if let Some(ref resp) = response {
                    if !resp.message.is_empty() {
                        // For now, just log the message. In a full implementation,
                        // this would show a popup in the UI and wait for user confirmation
                        println!("📢 Important message: {}", resp.message);
                    }
                    
                    // Check if proxy should be skipped
                    if !resp.proxy {
                        println!("⚠️ Proxy disabled by patch response");
                    }
                }
                (files, patch_response_data)
            },
            Err(e) => {
                // Any patching failure should abort game launch
                let _ = stop_game_monitor();
                return Err(format!("Cannot launch game: Patching failed. Error: {}", e));
            }
        };
        
        // Start game monitoring AFTER patching is complete
        // This ensures proxy is not started until patches are applied
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
                    message: format!("Successfully launched: {} > {}", game_exe_path.display(), md5_str),
                    process_id,
                };
                match serde_json::to_string(&result) {
                    Ok(json) => Ok(json),
                    Err(e) => Err(format!("Failed to serialize launch result: {}", e)),
                }
            },
            Err(e) => {
                // If game launch fails, stop monitoring and clean up proxy
                let _ = stop_game_monitor();
                let _ = proxy::stop_proxy();
                Err(format!("Failed to launch game: {}", e))
            },
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Game launching is only supported on Windows".to_string())
    }
}

#[command]
fn show_game_folder(game_id: Number) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Open the game folder in Windows Explorer
        let game_path = format!("C:\\Games\\{}", game_id); // Example path
        match Command::new("explorer")
            .arg(&game_path)
            .spawn()
        {
            Ok(_) => Ok(format!("Opened folder for {}", game_id)),
            Err(e) => Err(format!("Failed to open folder: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Folder opening is only supported on Windows".to_string())
    }
}

#[command]
fn open_directory(path: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        match Command::new("explorer")
            .arg(&path)
            .spawn()
        {
            Ok(_) => Ok(format!("Opened directory: {}", path)),
            Err(e) => Err(format!("Failed to open directory: {}", e)),
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        match Command::new("open")
            .arg(&path)
            .spawn()
        {
            Ok(_) => Ok(format!("Opened directory: {}", path)),
            Err(e) => Err(format!("Failed to open directory: {}", e)),
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        match Command::new("xdg-open")
            .arg(&path)
            .spawn()
        {
            Ok(_) => Ok(format!("Opened directory: {}", path)),
            Err(e) => Err(format!("Failed to open directory: {}", e)),
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Directory opening is not supported on this platform".to_string())
    }
}

#[command]
fn check_game_installed(_game_id: Number, _version: String, game_folder_path: String) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if game is installed by verifying the configured folder path exists
        if game_folder_path.is_empty() {
            return false; // No path configured means not installed
        }
        
        // Check if the configured game folder exists
        std::path::Path::new(&game_folder_path).exists()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

// Internal function to check if a specific game is running
fn check_game_running_internal(game_id: &Number) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Determine game executable names based on game ID
        let game_exe_names = match game_id.as_u64() {
            Some(1) => vec!["GenshinImpact.exe"],
            Some(2) => vec!["StarRail.exe"],
            //Some(3) => vec!["BlueArchive.exe"],
            _ => return Err(format!("Unsupported game ID: {}", game_id)),
        };
        
        // Check each possible executable name
        for game_exe_name in game_exe_names {
            let output = Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}", game_exe_name), "/FO", "CSV"])
                .output()
                .map_err(|e| format!("Failed to execute tasklist: {}", e))?;
            
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Check if the game executable is listed in the output
                // Use CSV format for more reliable parsing
                let lines: Vec<&str> = output_str.lines().collect();
                for line in lines.iter().skip(1) { // Skip header line
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
                eprintln!("⚠️ Warning: tasklist failed for {}: {}", game_exe_name, error_msg);
            }
        }
        Ok(false)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For non-Windows systems, we can't easily check process status
        Ok(false)
    }
}

// Function to kill game processes if running
fn kill_game_processes(game_id: &Number) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Determine game executable names based on game ID
        let game_exe_names = match game_id.as_u64() {
            Some(1) => vec!["GenshinImpact.exe"],
            Some(2) => vec!["StarRail.exe"],
            //Some(3) => vec!["BlueArchive.exe"],
            _ => return Err(format!("Unsupported game ID: {}", game_id)),
        };
        
        let mut killed_processes = Vec::new();
        
        // Kill each possible executable
        for game_exe_name in game_exe_names {
            // First check if the process is running
            let check_output = Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}", game_exe_name), "/FO", "CSV"])
                .output()
                .map_err(|e| format!("Failed to execute tasklist: {}", e))?;
            
            if check_output.status.success() {
                let output_str = String::from_utf8_lossy(&check_output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();
                let mut process_found = false;
                
                for line in lines.iter().skip(1) { // Skip header line
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
                        },
                        Ok(output) => {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            println!("⚠️ Failed to kill {} gracefully: {}", game_exe_name, error_msg);
                            
                            // Try force kill as fallback
                            let force_kill_output = Command::new("taskkill")
                                .args(["/IM", game_exe_name, "/T", "/F"])
                                .output();
                            
                            match force_kill_output {
                                Ok(force_output) if force_output.status.success() => {
                                    killed_processes.push(format!("{} (force killed)", game_exe_name));
                                    println!("✅ Force killed: {}", game_exe_name);
                                    std::thread::sleep(Duration::from_millis(1000));
                                },
                                _ => {
                                    return Err(format!("Failed to kill {}: {}", game_exe_name, error_msg));
                                }
                            }
                        },
                        Err(e) => {
                            return Err(format!("Failed to execute taskkill for {}: {}", game_exe_name, e));
                        }
                    }
                }
            }
        }
        
        if killed_processes.is_empty() {
            Ok("No game processes were running".to_string())
        } else {
            Ok(format!("Killed game processes: {}", killed_processes.join(", ")))
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok("Process killing not supported on this platform".to_string())
    }
}

#[command]
fn check_game_running(game_id: Number) -> Result<bool, String> {
    check_game_running_internal(&game_id)
}

#[command]
fn kill_game(game_id: Number) -> Result<String, String> {
    kill_game_processes(&game_id)
}

#[command]
fn check_patch_status(
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
) -> Result<String, String> {
    let api_url = format!(
        "https://ps.yuuki.me/game/patch/{}/{}/{}/{}.json",
        game_id, version, channel, md5
    );
    
    match fetch_patch_info(&api_url) {
        Ok(response) => {
            if response.patch {
                Ok(format!("Patches available (Method {}): {} files to patch", response.metode, response.patched.len()))
            } else {
                Ok("No patches needed".to_string())
            }
        },
        Err(e) => Err(format!("Failed to check patch status: {}", e))
    }
}

#[command]
fn restore_game_files(
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
    game_folder_path: String,
) -> Result<String, String> {
    let api_url = format!(
        "https://ps.yuuki.me/game/patch/{}/{}/{}/{}.json",
        game_id, version, channel, md5
    );
    
    match fetch_patch_info(&api_url) {
        Ok(response) => {
            if response.patch {
                restore_original_files(&response, &game_folder_path)
            } else {
                Ok("No patches to restore".to_string())
            }
        },
        Err(e) => Err(format!("Failed to restore files: {}", e))
    }
}

#[command]
fn get_download_progress() -> Result<Vec<DownloadProgress>, String> {
    let progress = DOWNLOAD_PROGRESS.lock()
        .map_err(|e| format!("Failed to lock download progress: {}", e))?;
    Ok(progress.clone())
}

#[command]
fn clear_download_progress() -> Result<String, String> {
    let mut progress = DOWNLOAD_PROGRESS.lock()
        .map_err(|e| format!("Failed to lock download progress: {}", e))?;
    progress.clear();
    Ok("Download progress cleared".to_string())
}

// Function to check and apply patches
fn check_and_apply_patches(
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
    game_folder_path: String,
) -> Result<(String, Option<PatchResponse>, Vec<String>), String> {
    
    // Ensure proxy is not running during patching to prevent conflicts
    let proxy_was_running = proxy::is_proxy_running();
    if proxy_was_running {
        println!("⚠️ Stopping proxy for patching...");
        if let Err(e) = proxy::stop_proxy() {
            return Err(format!("Failed to stop proxy before patching: {}", e));
        }
    }

    // Construct API URL
    let api_url = format!(
        "https://ps.yuuki.me/game/patch/{}/{}/{}/{}.json",
        game_id, version, channel, md5
    );
    
    println!("🔍 Checking for patches: {}", api_url);
    
    // Make HTTP request to get patch info
    let patch_response = fetch_patch_info(&api_url)?;
    
    if !patch_response.patch {
        return Ok(("No patches needed".to_string(), Some(patch_response), Vec::new()));
    }
    
    println!("🔧 Patches required (Method {})", patch_response.metode);
    
    // Check if game is running outside the launcher and kill it if necessary
    println!("🔍 Checking if game is running outside launcher...");
    match check_game_running_internal(&game_id) {
        Ok(true) => {
            println!("⚠️ Game is running outside launcher, attempting to close it...");
            match kill_game_processes(&game_id) {
                Ok(kill_message) => {
                    println!("✅ {}", kill_message);
                },
                Err(e) => {
                    return Err(format!("Cannot apply patches: Failed to close running game. {}", e));
                }
            }
        },
        Ok(false) => {
            println!("✅ No game processes detected, proceeding with patching...");
        },
        Err(e) => {
            println!("⚠️ Warning: Could not check if game is running: {}", e);
            println!("🔄 Proceeding with patching anyway...");
        }
    }    
    
    // Apply patches based on method
    let result = match patch_response.metode {
        0 => {
            // Method 0: Skip patch
            Ok(("Patch skipped (Method 0)".to_string(), Some(patch_response), Vec::new()))
        },
        1 => {
            match apply_file_patches(&patch_response, &game_folder_path) {
                Ok((message, patched_files)) => Ok((message, Some(patch_response), patched_files)),
                Err(e) => Err(e),
            }
        },
        _ => Err(format!("Unsupported patch method: {}", patch_response.metode)),
    };
    
    // Note: We don't restart the proxy here because the game monitor will handle it
    // based on the patch response proxy flag when the game actually starts
    
    result
}

// Function to fetch patch info from API
fn fetch_patch_info(url: &str) -> Result<PatchResponse, String> {
    // Use tokio runtime for async HTTP request
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
    
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .no_proxy()
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP request failed with status: {}", response.status()));
        }
        
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;
        
        serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse API response: {}", e))
    })
}

// Function to apply file patches (method 1)
fn apply_file_patches(patch_response: &PatchResponse, game_folder_path: &str) -> Result<(String, Vec<String>), String> {
    let mut patched_files = Vec::new();
    
    // Create backups and apply patches
    for patch_file in &patch_response.patched {
        let file_path = Path::new(game_folder_path).join(&patch_file.location);
        let patch_cache_path = format!("{}.patch", file_path.display());
        
        // Check if we already have a cached patch file
        if Path::new(&patch_cache_path).exists() {
            // Verify the cached patch file
            if let Ok(cached_contents) = fs::read(&patch_cache_path) {
                let cached_md5 = format!("{:x}", md5::compute(&cached_contents));
                if cached_md5.to_uppercase() == patch_file.md5.to_uppercase() {
                    println!("📦 Using cached patch: {}", patch_file.location);
                    
                    // Create backup of original file if it exists
                    if file_path.exists() {
                        let backup_path = format!("{}.backup", file_path.display());
                        fs::copy(&file_path, &backup_path)
                            .map_err(|e| format!("Failed to backup {}: {}", patch_file.location, e))?;
                        println!("📦 Backed up: {}", patch_file.location);
                    }
                    
                    // Copy cached patch to target location
                    fs::copy(&patch_cache_path, &file_path)
                        .map_err(|e| format!("Failed to apply cached patch {}: {}", patch_file.location, e))?;
                    
                    patched_files.push(patch_file.location.clone());
                    println!("✅ Applied cached patch: {}", patch_file.location);
                    continue;
                }
            }
            // If cached file is invalid, remove it
            let _ = fs::remove_file(&patch_cache_path);
        }
        
        // Create backup of original file if it exists
        if file_path.exists() {
            let backup_path = format!("{}.backup", file_path.display());
            fs::copy(&file_path, &backup_path)
                .map_err(|e| format!("Failed to backup {}: {}", patch_file.location, e))?;
            println!("📦 Backed up: {}", patch_file.location);
        }
        
        // Download and apply patch
        download_and_verify_file(&patch_file.file, &file_path, &patch_file.md5)?;
        
        // Save patch file for future use
        if let Err(e) = fs::copy(&file_path, &patch_cache_path) {
            println!("⚠️ Warning: Failed to cache patch file {}: {}", patch_file.location, e);
        } else {
            println!("💾 Cached patch: {}", patch_file.location);
        }
        
        patched_files.push(patch_file.location.clone());
        println!("✅ Patched: {}", patch_file.location);
    }
    
    Ok((format!("Successfully patched {} files", patched_files.len()), patched_files))
}

// Function to download and verify a file
fn download_and_verify_file(url: &str, file_path: &Path, expected_md5: &str) -> Result<(), String> {
    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }
    
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Use tokio runtime for async HTTP request
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
    
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 minutes for file downloads
            .no_proxy()
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        println!("📥 Downloading: {}", url);
        
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Download request failed: {}", e))?;
        
        if !response.status().is_success() {
            // Update progress with failed status
            if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
                progress.retain(|p| p.file_name != file_name);
                progress.push(DownloadProgress {
                    file_name: file_name.clone(),
                    downloaded: 0,
                    total: 0,
                    percentage: 0.0,
                    status: "failed".to_string(),
                });
            }
            return Err(format!("Download failed with status: {}", response.status()));
        }
        
        let total_size = response.content_length().unwrap_or(0);
        
        // Initialize progress
        if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
            progress.retain(|p| p.file_name != file_name);
            progress.push(DownloadProgress {
                file_name: file_name.clone(),
                downloaded: 0,
                total: total_size,
                percentage: 0.0,
                status: "downloading".to_string(),
            });
        }
        
        let file_contents = response
            .bytes()
            .await
            .map_err(|e| {
                // Update progress with failed status
                if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
                    progress.retain(|p| p.file_name != file_name);
                    progress.push(DownloadProgress {
                        file_name: file_name.clone(),
                        downloaded: 0,
                        total: total_size,
                        percentage: 0.0,
                        status: "failed".to_string(),
                    });
                }
                format!("Failed to read download response: {}", e)
            })?;
        
        // Update progress to verifying
        if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
            progress.retain(|p| p.file_name != file_name);
            progress.push(DownloadProgress {
                file_name: file_name.clone(),
                downloaded: file_contents.len() as u64,
                total: total_size,
                percentage: 100.0,
                status: "verifying".to_string(),
            });
        }
        
        // Verify MD5 before writing to disk
        let actual_md5 = format!("{:x}", md5::compute(&file_contents));
        
        if actual_md5.to_uppercase() != expected_md5.to_uppercase() {
            // Update progress with failed status
            if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
                progress.retain(|p| p.file_name != file_name);
                progress.push(DownloadProgress {
                    file_name: file_name.clone(),
                    downloaded: file_contents.len() as u64,
                    total: total_size,
                    percentage: 100.0,
                    status: "failed".to_string(),
                });
            }
            return Err(format!(
                "MD5 mismatch for {}: expected {}, got {}",
                file_path.display(),
                expected_md5,
                actual_md5
            ));
        }
        
        // Write file to disk
        fs::write(file_path, &file_contents)
            .map_err(|e| {
                // Update progress with failed status
                if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
                    progress.retain(|p| p.file_name != file_name);
                    progress.push(DownloadProgress {
                        file_name: file_name.clone(),
                        downloaded: file_contents.len() as u64,
                        total: total_size,
                        percentage: 100.0,
                        status: "failed".to_string(),
                    });
                }
                format!("Failed to write file {}: {}", file_path.display(), e)
            })?;
        
        // Update progress to completed
        if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
            progress.retain(|p| p.file_name != file_name);
            progress.push(DownloadProgress {
                file_name: file_name.clone(),
                downloaded: file_contents.len() as u64,
                total: total_size,
                percentage: 100.0,
                status: "completed".to_string(),
            });
        }
        
        println!("✅ Downloaded and verified: {}", file_path.display());
        Ok(())
    })
}

// Function to restore original files (unpatch)
fn restore_original_files(patch_response: &PatchResponse, game_folder_path: &str) -> Result<String, String> {
    let mut restored_files = Vec::new();
    
    for original_file in &patch_response.original {
        let file_path = Path::new(game_folder_path).join(&original_file.location);
        let patch_cache_path = format!("{}.patch", file_path.display());
        
        // If there's a patched file, rename it to .patch for future use
        if file_path.exists() {
            if let Err(e) = fs::rename(&file_path, &patch_cache_path) {
                println!("⚠️ Warning: Failed to save patch file {}: {}", original_file.location, e);
            } else {
                println!("💾 Saved patch file: {}", original_file.location);
            }
        }
        
        // Download and restore original file
        download_and_verify_file(&original_file.file, &file_path, &original_file.md5)?;
        restored_files.push(original_file.location.clone());
        println!("🔄 Restored: {}", original_file.location);
    }
    
    Ok(format!("Successfully restored {} files", restored_files.len()))
}

// Function to restore from backup files
fn restore_from_backups(game_folder_path: &str, file_locations: &[String]) -> Result<String, String> {
    let mut restored_files = Vec::new();
    
    for location in file_locations {
        let file_path = Path::new(game_folder_path).join(location);
        let backup_path = format!("{}.backup", file_path.display());
        let patch_cache_path = format!("{}.patch", file_path.display());
        
        if Path::new(&backup_path).exists() {
            // If there's a patched file, rename it to .patch for future use
            if file_path.exists() {
                if let Err(e) = fs::rename(&file_path, &patch_cache_path) {
                    println!("⚠️ Warning: Failed to save patch file {}: {}", location, e);
                } else {
                    println!("💾 Saved patch file: {}", location);
                }
            }
            
            fs::copy(&backup_path, &file_path)
                .map_err(|e| format!("Failed to restore {} from backup: {}", location, e))?;
            
            // Remove backup file after successful restore
            let _ = fs::remove_file(&backup_path);
            restored_files.push(location.clone());
            println!("🔄 Restored from backup: {}", location);
        }
    }
    
    Ok(format!("Successfully restored {} files from backup", restored_files.len()))
}

// Function to cleanup remaining patch files by renaming them to .patch extension
fn cleanup_remaining_patches(game_folder_path: &str, patched_files: &[String]) -> Result<String, String> {
    let mut cleaned_files = Vec::new();
    
    for location in patched_files {
        let file_path = Path::new(game_folder_path).join(location);
        let patch_cache_path = format!("{}.patch", file_path.display());
        
        // Check if the file exists and doesn't already have .patch extension
        if file_path.exists() && !location.ends_with(".patch") {
            // Only rename if the .patch file doesn't already exist
            if !Path::new(&patch_cache_path).exists() {
                match fs::rename(&file_path, &patch_cache_path) {
                    Ok(_) => {
                        cleaned_files.push(location.clone());
                        println!("🧹 Cleaned up patch file: {} -> {}.patch", location, location);
                    }
                    Err(e) => {
                        println!("⚠️ Warning: Failed to cleanup patch file {}: {}", location, e);
                    }
                }
            } else {
                // If .patch already exists, remove the current file
                match fs::remove_file(&file_path) {
                    Ok(_) => {
                        cleaned_files.push(location.clone());
                        println!("🧹 Removed duplicate patch file: {}", location);
                    }
                    Err(e) => {
                        println!("⚠️ Warning: Failed to remove duplicate patch file {}: {}", location, e);
                    }
                }
            }
        }
    }
    
    if cleaned_files.is_empty() {
        Ok(String::new()) // Return empty string if no cleanup was needed
    } else {
        Ok(format!("Cleaned up {} patch files", cleaned_files.len()))
    }
}

// Function to start monitoring a specific game
#[command]
fn start_game_monitor(game_id: Number) -> Result<String, String> {
    let mut monitor_state = GAME_MONITOR_STATE.lock().map_err(|e| format!("Failed to lock monitor state: {}", e))?;
    
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
        
        println!("🔍 Started monitoring game {} for automatic proxy management", game_id_clone);
        
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
                                 let should_start_proxy = if let Ok(monitor_state) = GAME_MONITOR_STATE.lock() {
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
                                                 eprintln!("⚠️ Failed to start proxy when game started: {}", e);
                                             }
                                         }
                                     } else {
                                         println!("🎮 Game {} started - Proxy was already running", game_id_clone);
                                     }
                                 } else {
                                     println!("🎮 Game {} started - Proxy disabled by patch response", game_id_clone);
                                 }
                         } else {
                            // Game just stopped - handle cleanup
                            
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
                            
                            // Then stop proxy
                            if proxy::is_proxy_running() {
                                match proxy::force_stop_proxy() {
                                    Ok(_) => {
                                        proxy_started_by_us = false;
                                        println!("🎮 Game {} stopped - Proxy force stopped automatically", game_id_clone);
                                    }
                                    Err(e) => {
                                        eprintln!("⚠️ Failed to force stop proxy when game stopped: {}", e);
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
                                println!("🎮 Game {} stopped - Proxy was not running", game_id_clone);
                            }
                             
                             // Stop monitoring after game stops to allow frontend to reset
                             println!("🔧 Game {} stopped - Stopping automatic monitoring", game_id_clone);
                             break;
                         }
                         last_game_state = is_running;
                     }
                 }
                Err(e) => {
                    consecutive_errors += 1;
                    eprintln!("⚠️ Error checking game status (attempt {}): {}", consecutive_errors, e);
                    
                    // If we have too many consecutive errors, assume game stopped
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS && last_game_state {
                        eprintln!("⚠️ Too many consecutive errors, assuming game {} has stopped", game_id_clone);
                        if proxy::is_proxy_running() {
                            match proxy::force_stop_proxy() {
                                Ok(_) => {
                                    proxy_started_by_us = false;
                                    println!("🎮 Game {} assumed stopped due to errors - Proxy force stopped", game_id_clone);
                                }
                                Err(e) => {
                                    eprintln!("⚠️ Failed to force stop proxy after error detection: {}", e);
                                    // Try regular stop as fallback
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
                        //last_game_state = false;
                        //consecutive_errors = 0; // Reset counter
                        
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
        game_id: game_id.clone(),
        version: String::new(),
        channel: Number::from(0),
        md5: String::new(),
        game_folder_path: String::new(),
        patched_files: Arc::new(Mutex::new(Vec::new())),
        patch_response: Arc::new(Mutex::new(None)),
    });
    
    Ok(format!("Started monitoring game {} - proxy will auto-start/stop with game", game_id))
}

// Function to stop game monitoring
#[command]
fn stop_game_monitor() -> Result<String, String> {
    let mut monitor_state = GAME_MONITOR_STATE.lock().map_err(|e| format!("Failed to lock monitor state: {}", e))?;
    
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

// Function to check if game monitoring is active
#[command]
fn is_game_monitor_active() -> Result<bool, String> {
    let monitor_state = GAME_MONITOR_STATE.lock().map_err(|e| format!("Failed to lock monitor state: {}", e))?;
    Ok(monitor_state.is_some())
}

#[command]
fn stop_game_process(process_id: u32) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // First check if the process exists
        let check_output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", process_id)])
            .output()
            .map_err(|e| format!("Failed to check process existence: {}", e))?;
        
        if check_output.status.success() {
            let check_output_str = String::from_utf8_lossy(&check_output.stdout);
            if !check_output_str.contains(&process_id.to_string()) {
                return Ok(format!("Process with PID {} is not running (already terminated)", process_id));
            }
        }
        
        // Use taskkill to terminate the process by PID
        let output = Command::new("taskkill")
            .args(["/PID", &process_id.to_string(), "/F"])
            .output()
            .map_err(|e| format!("Failed to execute taskkill: {}", e))?;
        
        if output.status.success() {
            Ok(format!("Successfully terminated process with PID: {}", process_id))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            // Handle common error cases more gracefully
            if error_msg.contains("not found") || error_msg.contains("not running") {
                Ok(format!("Process with PID {} was not running (already terminated)", process_id))
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

#[command]
fn stop_game(game_id: Number) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Determine game executable names based on game ID
        let game_exe_names = match game_id.as_u64() {
            Some(1) => vec!["GenshinImpact.exe"],
            Some(2) => vec!["StarRail.exe", "HonkaiStarRail.exe", "StarRail_Data.exe", "Game.exe"],
            Some(3) => vec!["BlueArchive.exe"],
            _ => return Err(format!("Unsupported game ID: {}", game_id)),
        };
        
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
                    last_error = Some(format!("Failed to terminate {}: {}", game_exe_name, error_msg));
                }
            }
        }
        
        if !terminated_processes.is_empty() {
            Ok(format!("Successfully terminated: {}", terminated_processes.join(", ")))
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

pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![launch_game, get_game_folder_path, show_game_folder, check_game_installed, check_game_running, kill_game, start_game_monitor, stop_game_monitor, is_game_monitor_active, stop_game_process, stop_game, open_directory, start_proxy, stop_proxy, check_proxy_status, force_stop_proxy, check_and_disable_windows_proxy, install_ssl_certificate, check_certificate_status, check_ssl_certificate_installed, check_admin_privileges, check_patch_status, restore_game_files, get_download_progress, clear_download_progress, fetch_api_data, test_proxy_bypass, proxy::set_proxy_addr, proxy::get_proxy_addr, proxy::get_proxy_logs, proxy::clear_proxy_logs, proxy::get_proxy_domains, proxy::add_proxy_domain, proxy::remove_proxy_domain, proxy::set_proxy_port, proxy::get_proxy_port, proxy::find_available_port, proxy::start_proxy_with_port])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      
      // Check and disable any Windows proxy settings on application startup
      println!("🔍 Checking Windows proxy settings on startup...");
      match proxy::check_and_disable_windows_proxy() {
        Ok(message) => {
          println!("✅ {}", message);
        }
        Err(e) => {
          eprintln!("⚠️ Failed to check/disable Windows proxy on startup: {}", e);
        }
      }
      
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
