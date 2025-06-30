//! System utilities module
//! Handles Windows-specific operations like admin privileges, certificates, and proxy settings

use std::process::Command;
use tauri::command;

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

#[cfg(not(target_os = "windows"))]
pub fn is_running_as_admin() -> bool {
    false
}

/// Check administrator privileges
#[command]
pub fn check_admin_privileges() -> Result<bool, String> {
    Ok(is_running_as_admin())
}

/// Check and disable Windows proxy settings
#[command]
pub fn check_and_disable_windows_proxy() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Check current proxy settings
        let check_output = Command::new("reg")
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
                let disable_output = Command::new("reg")
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
pub fn install_ssl_certificate(cert_path: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        if !std::path::Path::new(&cert_path).exists() {
            return Err(format!("Certificate file not found: {}", cert_path));
        }

        // Use certutil to install the certificate
        let output = Command::new("certutil")
            .args(["-addstore", "Root", &cert_path])
            .output()
            .map_err(|e| format!("Failed to execute certutil: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("completed successfully") {
                Ok(format!(
                    "SSL certificate installed successfully: {}",
                    cert_path
                ))
            } else {
                // Check if certificate is already installed
                if output_str.contains("already exists") || output_str.contains("duplicate") {
                    Ok(format!(
                        "SSL certificate is already installed: {}",
                        cert_path
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

/// Check certificate installation status
#[command]
pub fn check_certificate_status(cert_name: String) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("certutil")
            .args(["-store", "Root", &cert_name])
            .output()
            .map_err(|e| format!("Failed to execute certutil: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Check if the certificate is found in the output
            Ok(output_str.contains(&cert_name) && !output_str.contains("not found"))
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

/// Check if SSL certificate is installed
#[command]
pub fn check_ssl_certificate_installed(cert_thumbprint: String) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("certutil")
            .args(["-store", "Root"])
            .output()
            .map_err(|e| format!("Failed to execute certutil: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Check if the certificate thumbprint is found in the Root store
            Ok(output_str
                .to_lowercase()
                .contains(&cert_thumbprint.to_lowercase()))
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

/// Get system information
#[command]
pub fn get_system_info() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let mut info = std::collections::HashMap::new();

        // Get OS version
        if let Ok(output) = Command::new("ver").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                info.insert("os_version", version);
            }
        }

        // Get computer name
        if let Ok(output) = Command::new("hostname").output() {
            if output.status.success() {
                let hostname = String::from_utf8_lossy(&output.stdout).trim().to_string();
                info.insert("hostname", hostname);
            }
        }

        // Get current user
        if let Ok(output) = Command::new("whoami").output() {
            if output.status.success() {
                let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                info.insert("username", username);
            }
        }

        // Add admin status
        info.insert("is_admin", is_running_as_admin().to_string());

        // Add architecture
        info.insert("architecture", std::env::consts::ARCH.to_string());

        serde_json::to_string(&info).map_err(|e| format!("Failed to serialize system info: {}", e))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let info = std::collections::HashMap::from([
            ("os", std::env::consts::OS),
            ("architecture", std::env::consts::ARCH),
            ("is_admin", "false"),
        ]);

        serde_json::to_string(&info).map_err(|e| format!("Failed to serialize system info: {}", e))
    }
}

/// Check Windows Defender real-time protection status
#[command]
pub fn check_windows_defender_status() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-MpPreference | Select-Object DisableRealtimeMonitoring",
            ])
            .output()
            .map_err(|e| format!("Failed to check Windows Defender status: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            let is_disabled = output_str.contains("True");
            let status = if is_disabled {
                "Windows Defender real-time protection is disabled"
            } else {
                "Windows Defender real-time protection is enabled"
            };

            Ok(status.to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!(
                "Failed to check Windows Defender status: {}",
                error_msg
            ))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Windows Defender check is only supported on Windows".to_string())
    }
}

/// Get installed .NET Framework versions
#[command]
pub fn get_dotnet_versions() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("reg")
            .args([
                "query",
                "HKLM\\SOFTWARE\\Microsoft\\NET Framework Setup\\NDP",
                "/s",
                "/v",
                "Version",
            ])
            .output()
            .map_err(|e| format!("Failed to query .NET versions: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut versions = Vec::new();

            for line in output_str.lines() {
                if line.contains("Version") && line.contains("REG_SZ") {
                    if let Some(version) = line.split_whitespace().last() {
                        versions.push(version.to_string());
                    }
                }
            }

            let result = serde_json::json!({
                "versions": versions,
                "count": versions.len()
            });

            serde_json::to_string(&result)
                .map_err(|e| format!("Failed to serialize .NET versions: {}", e))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to query .NET versions: {}", error_msg))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("{\"versions\": [], \"count\": 0}".to_string())
    }
}

/// Check if a Windows service is running
#[command]
pub fn check_service_status(service_name: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("sc")
            .args(["query", &service_name])
            .output()
            .map_err(|e| format!("Failed to query service status: {}", e))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            let status = if output_str.contains("RUNNING") {
                "running"
            } else if output_str.contains("STOPPED") {
                "stopped"
            } else if output_str.contains("PAUSED") {
                "paused"
            } else {
                "unknown"
            };

            let result = serde_json::json!({
                "service_name": service_name,
                "status": status,
                "exists": true
            });

            serde_json::to_string(&result)
                .map_err(|e| format!("Failed to serialize service status: {}", e))
        } else {
            // Service doesn't exist or access denied
            let result = serde_json::json!({
                "service_name": service_name,
                "status": "not_found",
                "exists": false
            });

            serde_json::to_string(&result)
                .map_err(|e| format!("Failed to serialize service status: {}", e))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let result = serde_json::json!({
            "service_name": service_name,
            "status": "not_supported",
            "exists": false
        });

        serde_json::to_string(&result)
            .map_err(|e| format!("Failed to serialize service status: {}", e))
    }
}
