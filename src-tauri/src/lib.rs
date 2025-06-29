#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::Number;
use serde::{Deserialize, Serialize};
use tauri::command;
// Import the proxy module
mod proxy;

// Global game monitoring state
static GAME_MONITOR_STATE: once_cell::sync::Lazy<Arc<Mutex<Option<GameMonitorHandle>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

struct GameMonitorHandle {
    should_stop: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Serialize, Deserialize)]
struct LaunchResult {
    message: String,
    #[serde(rename = "processId")]
    process_id: u32,
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

// Function to automatically install SSL certificate on Windows with multiple methods
#[cfg(target_os = "windows")]
fn auto_install_certificate(cert_path: &std::path::Path) -> Result<(), String> {
    use std::process::Command;
    
    println!("🔧 Installing CA certificate automatically for all domains...");
    
    // First, try to remove any existing certificate with the same name
    let _ = Command::new("certutil")
        .args(["-delstore", "Root", "YuukiPS MITM Proxy CA"])
        .output();
    
    // Method 1: Try certutil first
    let output = Command::new("certutil")
        .args(["-addstore", "-f", "Root", &cert_path.to_string_lossy()])
        .output()
        .map_err(|e| format!("Failed to execute certutil: {}", e))?;
    
    if output.status.success() {
        println!("✅ CA certificate installed successfully via certutil!");
        return Ok(());
    }
    
    println!("⚠️ Certutil failed, trying PowerShell method...");
    
    // Method 2: Try PowerShell as fallback
    let ps_script = format!(
        "Import-Certificate -FilePath '{}' -CertStoreLocation Cert:\\LocalMachine\\Root",
        cert_path.to_string_lossy()
    );
    
    let ps_output = Command::new("powershell")
        .args(["-Command", &ps_script])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
    
    if ps_output.status.success() {
        println!("✅ CA certificate installed successfully via PowerShell!");
        return Ok(());
    }
    
    // Method 3: Try elevated PowerShell
    println!("⚠️ Standard PowerShell failed, trying elevated PowerShell...");
    let elevated_ps_script = format!(
        "Start-Process powershell -ArgumentList '-Command Import-Certificate -FilePath \\'{}\\' -CertStoreLocation Cert:\\\\LocalMachine\\\\Root' -Verb RunAs -Wait",
        cert_path.to_string_lossy()
    );
    
    let elevated_output = Command::new("powershell")
        .args(["-Command", &elevated_ps_script])
        .output()
        .map_err(|e| format!("Failed to execute elevated PowerShell: {}", e))?;
    
    if elevated_output.status.success() {
        println!("✅ CA certificate installed successfully via elevated PowerShell!");
        return Ok(());
    }
    
    let error_msg = String::from_utf8_lossy(&output.stderr);
    let ps_error_msg = String::from_utf8_lossy(&ps_output.stderr);
    Err(format!(
        "All automatic installation methods failed:\n- Certutil: {}\n- PowerShell: {}",
        error_msg, ps_error_msg
    ))
}

#[command]
fn install_ssl_certificate() -> Result<String, String> {
    // Use the new install_ca_certificate function from proxy module
    proxy::install_ca_certificate()
}

#[command]
fn install_ca_certificate() -> Result<String, String> {
    // Get the certificate path from proxy module
    let cert_path_str = proxy::get_certificate_path()?;
    let cert_path = std::path::Path::new(&cert_path_str);
    
    // Check if certificate file exists
    if !cert_path.exists() {
        return Err("SSL certificate not found. Please start the proxy first to generate the certificate.".to_string());
    }
    
    #[cfg(target_os = "windows")]
    {
        // Try automatic installation first
        match auto_install_certificate(&cert_path) {
            Ok(_) => {
                return Ok(format!("🎉 SSL Certificate installed automatically for ALL domains!\n\n✅ The certificate has been added to your system's trusted root certificates.\n🌐 HTTPS interception is now enabled for ALL game domains and websites.\n🔒 No more certificate warnings!"));
            }
            Err(auto_error) => {
                println!("Automatic installation failed: {}", auto_error);
                
                // Fallback to manual installation
                match Command::new("certlm.msc")
                    .spawn()
                {
                    Ok(_) => Ok(format!("Certificate saved to: {}\n\nAutomatic installation failed. Opened Certificate Manager.\n\nPlease manually:\n1. Navigate to 'Trusted Root Certification Authorities' > 'Certificates'\n2. Right-click and select 'All Tasks' > 'Import'\n3. Import the certificate file\n4. This will enable HTTPS interception for game domains", cert_path.display())),
                    Err(_) => {
                        // Final fallback: try to open the certificate file directly
                        match Command::new("cmd")
                            .args(["/C", "start", "", &cert_path.to_string_lossy()])
                            .spawn()
                        {
                            Ok(_) => Ok(format!("Certificate saved to: {}\n\nAutomatic installation failed. Please install this certificate manually as a trusted root certificate to enable HTTPS interception.", cert_path.display())),
                            Err(e) => Err(format!("Failed to open certificate: {}\n\nCertificate saved to: {}\nPlease manually install it as a trusted root certificate.", e, cert_path.display()))
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(format!("Certificate saved to: {}\n\nPlease install this certificate as a trusted root certificate to enable HTTPS interception.\n\nOn macOS: Double-click the certificate and add it to Keychain\nOn Linux: Copy to /usr/local/share/ca-certificates/ and run update-ca-certificates", cert_path.display()))
    }
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
fn launch_game_with_engine(
    game_id: Number,
    game_title: String,
    _engine_id: Number,
    engine_name: String,
    version: String,
    game_folder_path: String,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Use the provided game folder path from frontend settings
        if game_folder_path.is_empty() {
            return Err(format!("Game folder path not set for {} version {}. Please configure it in game settings.", game_title, version));
        }
        
        // Check if game folder exists
        if !std::path::Path::new(&game_folder_path).exists() {
            return Err(format!("Game folder not found: {}. Please verify the path in game settings.", game_folder_path));
        }
        
        // Start game monitoring instead of just starting proxy
        // This will automatically manage proxy based on game state
        if let Err(e) = start_game_monitor(game_id.clone()) {
            return Err(format!("Failed to start game monitoring: {}", e));
        }
        
        // Determine game executable name based on game ID
        let game_exe_name = match game_id.as_u64() {
            Some(1) => "GenshinImpact.exe",
            Some(2) => "StarRail.exe", // Common names: StarRail.exe, HonkaiStarRail.exe, or StarRail_Data.exe
            Some(3) => "BlueArchive.exe",
            _ => return Err(format!("Unsupported game ID: {}", game_id)),
        };

        // Construct full path to game executable
        let game_exe_path = std::path::Path::new(&game_folder_path).join(game_exe_name);
        
        // Check if game executable exists
        if !game_exe_path.exists() {
            return Err(format!("Game executable not found: {} = {}. Please verify the game installation.", game_exe_path.display(),game_id));
        }
        
        // Launch the game executable
        match Command::new(&game_exe_path)
            .current_dir(&game_folder_path)
            .spawn()
        {
            Ok(child) => {
                let process_id = child.id();
                let result = LaunchResult {
                    message: format!("Successfully launched {} ({}) with {} from folder {}. HTTP/HTTPS proxy is active on 127.0.0.1:??? with automatic Windows proxy configuration - game traffic redirected to ps.yuuki.me", game_title, game_id, engine_name, game_folder_path),
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
            Some(3) => vec!["BlueArchive.exe"],
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

#[command]
fn check_game_running(game_id: Number) -> Result<bool, String> {
    check_game_running_internal(&game_id)
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
                             // Game just started - start proxy if not already running
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
                            // Game just stopped - force stop proxy regardless of who started it
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
    .invoke_handler(tauri::generate_handler![launch_game_with_engine, get_game_folder_path, show_game_folder, check_game_installed, check_game_running, start_game_monitor, stop_game_monitor, is_game_monitor_active, stop_game_process, stop_game, open_directory, start_proxy, stop_proxy, check_proxy_status, force_stop_proxy, check_and_disable_windows_proxy, install_ssl_certificate, install_ca_certificate, check_certificate_status, check_ssl_certificate_installed, check_admin_privileges, proxy::set_proxy_addr, proxy::get_proxy_addr, proxy::get_proxy_logs, proxy::clear_proxy_logs, proxy::get_proxy_domains, proxy::add_proxy_domain, proxy::remove_proxy_domain, proxy::set_proxy_port, proxy::get_proxy_port, proxy::find_available_port, proxy::start_proxy_with_port])
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
