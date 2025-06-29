#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::process::Command;

use serde_json::Number;
use tauri::command;
// Import the proxy module
mod proxy;

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
fn launch_game(_game_id: Number, game_title: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // For demonstration, we'll show a Windows notification
        // In a real launcher, you would launch the actual game executable
        
        // Example of launching a Windows application
        // You would replace this with the actual game executable path
        match Command::new("cmd")
            .args(["/C", "echo", &format!("Starting {}", game_title)])
            .output()
        {
            Ok(_) => Ok(format!("Successfully launched {}", game_title)),
            Err(e) => Err(format!("Failed to launch game: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Game launching is only supported on Windows".to_string())
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
        
        // Start HTTP proxy before launching the game (will automatically stop existing proxy if running)
        if let Err(e) = proxy::start_proxy() {
            return Err(format!("Failed to start HTTP proxy: {}", e));
        }
        
        // Determine game executable name based on game ID
        let game_exe_name = match game_id.as_u64() {
            Some(1) => "GenshinImpact.exe",
            Some(2) => "StarRail.exe",
            _ => return Err(format!("Unsupported game ID: {}", game_id)),
        };
        
        // Construct full path to game executable
        let game_exe_path = std::path::Path::new(&game_folder_path).join(game_exe_name);
        
        // Check if game executable exists
        if !game_exe_path.exists() {
            return Err(format!("Game executable not found: {}. Please verify the game installation.", game_exe_path.display()));
        }
        
        // Launch the game executable
        match Command::new(&game_exe_path)
            .current_dir(&game_folder_path)
            .spawn()
        {
            Ok(_) => Ok(format!("Successfully launched {} with {} from folder {}. HTTP/HTTPS proxy is active on 127.0.0.1:8080 with automatic Windows proxy configuration - game traffic redirected to ps.yuuki.me", game_title, engine_name, game_folder_path)),
            Err(e) => {
                // If game launch fails, try to clean up proxy
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

pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![launch_game, launch_game_with_engine, get_game_folder_path, show_game_folder, check_game_installed, open_directory, start_proxy, stop_proxy, check_proxy_status, force_stop_proxy, check_and_disable_windows_proxy, install_ssl_certificate, install_ca_certificate, check_certificate_status, check_ssl_certificate_installed, check_admin_privileges])
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
