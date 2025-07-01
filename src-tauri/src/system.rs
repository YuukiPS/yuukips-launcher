//! System utilities module
//! Handles Windows-specific operations like admin privileges, certificates, and proxy settings

use tauri::command;

use crate::utils::create_hidden_command;

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
pub fn install_ssl_certificate(_cert_path: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        if !std::path::Path::new(&_cert_path).exists() {
            return Err(format!("Certificate file not found: {}", _cert_path));
        }

        // Use certutil to install the certificate
        let output = create_hidden_command("certutil")
            .args(["-addstore", "Root", &_cert_path])
            .output()
            .map_err(|e| format!("Failed to execute certutil: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("completed successfully") {
                Ok(format!(
                    "SSL certificate installed successfully: {}",
                    _cert_path
                ))
            } else {
                // Check if certificate is already installed
                if output_str.contains("already exists") || output_str.contains("duplicate") {
                    Ok(format!(
                        "SSL certificate is already installed: {}",
                        _cert_path
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