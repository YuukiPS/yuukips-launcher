//! System utilities module
//! Handles Windows-specific operations like admin privileges, certificates, and proxy settings

use std::env;
use std::path::PathBuf;
use tauri::command;

use crate::utils::create_hidden_command;
use crate::proxy::generate_ca_files;
use crate::game::is_any_game_running;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri::Manager;

// Helper function to get data directory
pub fn get_data_dir() -> Result<PathBuf, String> {
    if let Some(home) = env::var_os("USERPROFILE") {
        Ok(PathBuf::from(home).join("AppData").join("Local"))
    } else {
        Err("Could not determine data directory".to_string())
    }
}

/// Check if the application is running with administrator privileges
#[cfg(target_os = "windows")]
pub fn is_running_as_admin() -> bool {
    use std::mem;
    use std::ptr;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};

    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        let current_process = GetCurrentProcess();

        if OpenProcessToken(current_process, TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = 0u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );

        CloseHandle(token);

        result != 0 && elevation.TokenIsElevated != 0
    }
}

/// Check if the application is running with administrator privileges (non-Windows)
#[cfg(not(target_os = "windows"))]
pub fn is_running_as_admin() -> bool {
    // On non-Windows systems, we assume the user has appropriate permissions
    // or we can check if running as root
    unsafe {
        libc::geteuid() == 0
    }
}

/// Check if running as admin without returning error
#[command]
pub fn is_admin() -> bool {
    is_running_as_admin()
}

/// Check and disable Windows proxy settings
#[command]
pub fn check_and_disable_windows_proxy() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Check current proxy settings
        let check_output = create_hidden_command("reg")
            .args([
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable",
            ])
            .output()
            .map_err(|e| format!("Failed to check proxy settings: {}", e))?;

        if check_output.status.success() {
            let output_str = String::from_utf8_lossy(&check_output.stdout);

            // Check if proxy is enabled (ProxyEnable = 1)
            if output_str.contains("ProxyEnable") && output_str.contains("0x1") {
                println!("🔧 Windows proxy is enabled, attempting to disable...");

                // Disable proxy
                let disable_output = create_hidden_command("reg")
                    .args([
                        "add",
                        "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                        "/v",
                        "ProxyEnable",
                        "/t",
                        "REG_DWORD",
                        "/d",
                        "0",
                        "/f",
                    ])
                    .output()
                    .map_err(|e| format!("Failed to disable proxy: {}", e))?;

                if disable_output.status.success() {
                    Ok("Windows proxy has been disabled successfully".to_string())
                } else {
                    let error_msg = String::from_utf8_lossy(&disable_output.stderr);
                    Err(format!("Failed to disable Windows proxy: {}", error_msg))
                }
            } else {
                Ok("Windows proxy is already disabled".to_string())
            }
        } else {
            let error_msg = String::from_utf8_lossy(&check_output.stderr);
            Err(format!("Failed to check proxy settings: {}", error_msg))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Proxy management is only supported on Windows".to_string())
    }
}

/// Install SSL certificate
#[command]
pub fn install_ssl_certificate() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Get the certificate path from the data directory
        let cert_path = get_data_dir()?
            .join("yuukips")
            .join("ca")
            .join("cert.crt");

        if !cert_path.exists() {
            // Generate CA files if they don't exist
            let yuukips_dir = get_data_dir()?.join("yuukips");
            println!("Certificate file not found, generating CA files at: {}", yuukips_dir.display());
            generate_ca_files(&yuukips_dir);
            
            // Check again if the certificate was created
            if !cert_path.exists() {
                return Err(format!("Failed to generate certificate file: {}. Certificate generation may have failed.", cert_path.display()));
            }
        }

        let cert_path_str = cert_path.to_string_lossy();
        
        // Use certutil to install the certificate
        let output = create_hidden_command("certutil")
            .args(["-addstore", "Root", &cert_path_str])
            .output()
            .map_err(|e| format!("Failed to execute certutil: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("completed successfully") {
                Ok(format!(
                    "SSL certificate installed successfully: {}",
                    cert_path.display()
                ))
            } else {
                // Check if certificate is already installed
                if output_str.contains("already exists") || output_str.contains("duplicate") {
                    Ok(format!(
                        "SSL certificate is already installed: {}",
                        cert_path.display()
                    ))
                } else {
                    Err(format!(
                        "Certificate installation may have failed: {}",
                        output_str
                    ))
                }
            }
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to install SSL certificate: {}", error_msg))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("SSL certificate installation is only supported on Windows".to_string())
    }
}

/// Open a directory in the system file explorer
#[command]
pub fn open_directory(path: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        
        // Validate that the path exists
        if !Path::new(&path).exists() {
            return Err(format!("Directory does not exist: {}", path));
        }
        
        // Use explorer.exe to open the directory
        // Note: explorer.exe doesn't always return proper exit codes, so we use spawn instead of output
        let result = create_hidden_command("explorer")
            .arg(&path)
            .spawn();
        
        match result {
            Ok(_) => Ok(format!("Directory opened successfully: {}", path)),
            Err(e) => Err(format!("Failed to open directory: {}", e))
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::path::Path;
        
        if !Path::new(&path).exists() {
            return Err(format!("Directory does not exist: {}", path));
        }
        
        let output = create_hidden_command("open")
            .arg(&path)
            .output()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
        
        if output.status.success() {
            Ok(format!("Directory opened successfully: {}", path))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to open directory: {}", error_msg))
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::path::Path;
        
        if !Path::new(&path).exists() {
            return Err(format!("Directory does not exist: {}", path));
        }
        
        // Try xdg-open first, then fallback to other common file managers
        let commands = ["xdg-open", "nautilus", "dolphin", "thunar", "pcmanfm"];
        
        for cmd in &commands {
            let output = create_hidden_command(cmd)
                .arg(&path)
                .output();
                
            match output {
                Ok(result) if result.status.success() => {
                    return Ok(format!("Directory opened successfully: {}", path));
                }
                _ => continue,
            }
        }
        
        Err("Failed to open directory: No suitable file manager found".to_string())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Opening directories is not supported on this platform".to_string())
    }
}

/// Check if SSL certificate is installed
#[command]
pub fn check_ssl_certificate_installed() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = create_hidden_command("certutil")
            .args(["-store", "Root"])
            .output()
            .map_err(|e| format!("Failed to execute certutil: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Check if the YuukiPS certificate is found in the Root store
            Ok(output_str
                .to_lowercase()
                .contains("yuukips"))
        } else {
            // If certutil fails, assume certificate is not installed
            Ok(false)
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(false)
    }
}

/// Get YuukiPS data directory path
#[command]
pub fn get_yuukips_data_path() -> Result<String, String> {
    let yuukips_dir = get_data_dir()?.join("yuukips");
    Ok(yuukips_dir.to_string_lossy().to_string())
}

/// Get Tauri app data directory path
#[command]
pub fn get_app_data_path() -> Result<String, String> {
    let app_data_dir = get_data_dir()?.join("com.yuukips.launcher");
    Ok(app_data_dir.to_string_lossy().to_string())
}

/// Get temporary files directory path
#[command]
pub fn get_temp_files_path() -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join("yuukips");
    Ok(temp_dir.to_string_lossy().to_string())
}

/// Helper function to selectively clear a directory while preserving specified files
fn clear_directory_selective(dir_path: &std::path::Path, preserve_files: &[&str]) -> Result<usize, String> {
    use std::fs;
    
    let mut cleared_count = 0;
    
    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        
        // Skip files that should be preserved
        if preserve_files.contains(&file_name) {
            println!("🔒 Preserving essential file: {}", file_name);
            continue;
        }
        
        // Remove file or directory
        let result = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
        
        match result {
            Ok(_) => {
                cleared_count += 1;
                println!("🗑️ Removed: {}", path.display());
            }
            Err(e) => {
                eprintln!("⚠️ Failed to remove {}: {}", path.display(), e);
            }
        }
    }
    
    Ok(cleared_count)
}

/// Clear all launcher data and reset settings
#[command]
pub fn clear_launcher_data() -> Result<String, String> {
    use std::fs;
    
    let mut cleared_items = Vec::new();
    
    // Clear YuukiPS data directory (preserve essential launcher files)
    let yuukips_dir = get_data_dir()?.join("yuukips");
    if yuukips_dir.exists() {
        // Files to preserve (essential launcher files)
        let preserve_files = ["yuukips-launcher.exe", "uninstall.exe"];
        
        match clear_directory_selective(&yuukips_dir, &preserve_files) {
            Ok(cleared_count) => {
                if cleared_count > 0 {
                    cleared_items.push(format!("YuukiPS data directory ({} items)", cleared_count));
                    println!("🧹 Cleared {} items from YuukiPS data directory: {}", cleared_count, yuukips_dir.display());
                }
            }
            Err(e) => {
                eprintln!("⚠️ Failed to clear YuukiPS data directory: {}", e);
            }
        }
    }
    
    // Clear Tauri app data directory (in AppData/Local/com.yuukips.launcher)
    // Preserve EBWebView folder as it contains browser data
    let app_data_dir = get_data_dir()?.join("com.yuukips.launcher");
    if app_data_dir.exists() {
        // Folders to preserve (browser data)
        let preserve_folders = ["EBWebView"];
        
        match clear_directory_selective(&app_data_dir, &preserve_folders) {
            Ok(cleared_count) => {
                if cleared_count > 0 {
                    cleared_items.push(format!("Tauri app data ({} items)", cleared_count));
                    println!("🧹 Cleared {} items from Tauri app data directory: {}", cleared_count, app_data_dir.display());
                }
            }
            Err(e) => {
                eprintln!("⚠️ Failed to clear Tauri app data: {}", e);
            }
        }
    }
    
    // Clear temporary files
    if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
        let yuukips_temp = temp_dir.join("yuukips");
        if yuukips_temp.exists() {
            match fs::remove_dir_all(&yuukips_temp) {
                Ok(_) => {
                    cleared_items.push("temporary files".to_string());
                    println!("🧹 Cleared temporary files: {}", yuukips_temp.display());
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to clear temporary files: {}", e);
                }
            }
        }
    }
    
    if cleared_items.is_empty() {
        Ok("No launcher data found to clear".to_string())
    } else {
        Ok(format!("Successfully cleared: {}", cleared_items.join(", ")))
    }
}

// Global state for Task Manager monitoring
static TASK_MANAGER_MONITOR_STATE: Mutex<Option<TaskManagerMonitorHandle>> = Mutex::new(None);

struct TaskManagerMonitorHandle {
    should_stop: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

/// Check if Task Manager is currently running
#[cfg(target_os = "windows")]
fn is_task_manager_running() -> Result<bool, String> {
    let output = create_hidden_command("tasklist")
        .args(["/FI", "IMAGENAME eq Taskmgr.exe"])
        .output()
        .map_err(|e| format!("Failed to check Task Manager process: {}", e))?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.contains("Taskmgr.exe"))
    } else {
        Err("Failed to execute tasklist command".to_string())
    }
}

/// Start monitoring for Task Manager while a game is running (internal version)
pub fn start_task_manager_monitor_internal(app_handle: tauri::AppHandle) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let mut monitor_state = TASK_MANAGER_MONITOR_STATE
            .lock()
            .map_err(|e| format!("Failed to lock Task Manager monitor state: {}", e))?;

        // Stop existing monitor if running
        if let Some(mut handle) = monitor_state.take() {
            *handle.should_stop.lock().unwrap() = true;
            if let Some(thread_handle) = handle.thread_handle.take() {
                let _ = thread_handle.join();
            }
        }

        let should_stop = Arc::new(Mutex::new(false));
        let should_stop_clone = Arc::clone(&should_stop);

        let thread_handle = thread::spawn(move || {
            let mut last_task_manager_state = false;
            let mut warning_shown = false;

            loop {
                // Check if we should stop monitoring
                if *should_stop_clone.lock().unwrap() {
                    break;
                }

                // Check if any game is running
                let game_running = match is_any_game_running() {
                    Ok(running) => running,
                    Err(_) => false,
                };

                if !game_running {
                    // No game running, stop monitoring
                    break;
                }

                // Check Task Manager status
                let task_manager_running = match is_task_manager_running() {
                    Ok(running) => running,
                    Err(_) => false,
                };

                // If Task Manager just started running
                if task_manager_running && !last_task_manager_state && !warning_shown {
                     // Show warning dialog on client side
                     let _ = app_handle.dialog()
                         .message("Task Manager detected while game is running!\n\nWarning: Closing game through Task Manager may cause issues:\n• Proxy settings may not be deactivated automatically\n• Remaining patch files may not be deleted\n• Game may not run normally on official servers\n\nPlease use the launcher to properly close the game instead.")
                         .title("Task Manager Warning")
                         .kind(MessageDialogKind::Warning)
                         .show(|_| {});
                     warning_shown = true;
                 }

                // Reset warning flag when Task Manager is closed
                if !task_manager_running {
                    warning_shown = false;
                }

                last_task_manager_state = task_manager_running;

                // Sleep for a short interval before checking again
                thread::sleep(Duration::from_millis(1000));
            }
        });

        *monitor_state = Some(TaskManagerMonitorHandle {
            should_stop,
            thread_handle: Some(thread_handle),
        });

        Ok("Task Manager monitoring started".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Task Manager monitoring is only supported on Windows".to_string())
    }
}

/// Start monitoring for Task Manager while a game is running (command version)
#[command]
pub fn start_task_manager_monitor(app_handle: tauri::AppHandle) -> Result<String, String> {
    start_task_manager_monitor_internal(app_handle)
}

/// Stop Task Manager monitoring
#[command]
pub fn stop_task_manager_monitor() -> Result<String, String> {
    let mut monitor_state = TASK_MANAGER_MONITOR_STATE
        .lock()
        .map_err(|e| format!("Failed to lock Task Manager monitor state: {}", e))?;

    if let Some(mut handle) = monitor_state.take() {
        *handle.should_stop.lock().unwrap() = true;
        if let Some(thread_handle) = handle.thread_handle.take() {
            let _ = thread_handle.join();
        }
        Ok("Task Manager monitoring stopped".to_string())
    } else {
        Ok("Task Manager monitoring was not active".to_string())
    }
}

/// Check if Task Manager monitoring is active
#[command]
pub fn is_task_manager_monitor_active() -> Result<bool, String> {
    let monitor_state = TASK_MANAGER_MONITOR_STATE
        .lock()
        .map_err(|e| format!("Failed to lock Task Manager monitor state: {}", e))?;
    
    Ok(monitor_state.is_some())
}

/// Minimize the launcher window
#[command]
pub fn minimize_launcher_window(app_handle: tauri::AppHandle) -> Result<String, String> {
    match app_handle.get_webview_window("main") {
        Some(window) => {
            match window.minimize() {
                Ok(_) => Ok("Launcher window minimized".to_string()),
                Err(e) => Err(format!("Failed to minimize window: {}", e)),
            }
        }
        None => Err("Main window not found".to_string()),
    }
}

/// Restore/show the launcher window
#[command]
pub fn restore_launcher_window(app_handle: tauri::AppHandle) -> Result<String, String> {
    match app_handle.get_webview_window("main") {
        Some(window) => {
            match window.unminimize() {
                Ok(_) => {
                    // Also bring window to front
                    let _ = window.set_focus();
                    Ok("Launcher window restored".to_string())
                }
                Err(e) => Err(format!("Failed to restore window: {}", e)),
            }
        }
        None => Err("Main window not found".to_string()),
    }
}