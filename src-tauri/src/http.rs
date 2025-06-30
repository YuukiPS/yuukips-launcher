//! HTTP utilities module
//! Handles HTTP client creation and network requests

use reqwest;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Create an HTTP client with optional proxy bypass
pub fn create_http_client(use_proxy: bool) -> Result<reqwest::Client, String> {
    let mut client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("YuukiPS-Launcher/1.0");
    
    if !use_proxy {
        client_builder = client_builder.no_proxy();
    }
    
    client_builder.build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

/// Test proxy bypass functionality
#[command]
pub fn test_proxy_bypass(url: String) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let client = create_http_client(false)?; // No proxy
        
        println!("🌐 Testing proxy bypass for: {}", url);
        
        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        let status = response.status();
        let headers = response.headers().clone();
        
        let body = response.text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;
        
        let result = serde_json::json!({
            "status": status.as_u16(),
            "headers": headers.iter().map(|(k, v)| {
                (k.as_str(), v.to_str().unwrap_or("<invalid>"))
            }).collect::<std::collections::HashMap<_, _>>(),
            "body_length": body.len(),
            "body_preview": if body.len() > 200 {
                format!("{}...", &body[..200])
            } else {
                body
            }
        });
        
        serde_json::to_string(&result)
            .map_err(|e| format!("Failed to serialize response: {}", e))
    })
}

/// Fetch data from an API endpoint
#[command]
pub fn fetch_api_data(url: String) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let client = create_http_client(false)?; // No proxy for API calls
        
        println!("📡 Fetching API data from: {}", url);
        
        let response = client.get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("API returned error status: {}", response.status()));
        }
        
        let body = response.text()
            .await
            .map_err(|e| format!("Failed to read API response: {}", e))?;
        
        // Validate that it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("API response is not valid JSON: {}", e))?;
        
        Ok(body)
    })
}

/// Test network connectivity
#[command]
pub fn test_network_connectivity() -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let client = create_http_client(false)?;
        
        // Test multiple endpoints to ensure connectivity
        let test_urls = vec![
            "https://httpbin.org/get",
            "https://api.github.com",
            "https://www.google.com",
        ];
        
        let mut results = Vec::new();
        
        for url in test_urls {
            let start_time = std::time::Instant::now();
            
            match client.get(url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(response) => {
                    let duration = start_time.elapsed();
                    results.push(serde_json::json!({
                        "url": url,
                        "status": response.status().as_u16(),
                        "success": true,
                        "duration_ms": duration.as_millis(),
                        "error": null
                    }));
                }
                Err(e) => {
                    let duration = start_time.elapsed();
                    results.push(serde_json::json!({
                        "url": url,
                        "status": null,
                        "success": false,
                        "duration_ms": duration.as_millis(),
                        "error": e.to_string()
                    }));
                }
            }
        }
        
        let summary = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "total_tests": results.len(),
            "successful_tests": results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count(),
            "results": results
        });
        
        serde_json::to_string(&summary)
            .map_err(|e| format!("Failed to serialize connectivity test results: {}", e))
    })
}

/// Get the current version from Cargo.toml
#[command]
pub fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// Fetch the latest release information from GitHub API
#[command]
pub async fn fetch_latest_release(url: String) -> Result<GitHubRelease, String> {
    let client = create_http_client(false)?; // Bypass proxy for GitHub API
    
    println!("🔍 Fetching latest release from: {}", url);
    
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "YuukiPS-Launcher")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }
    
    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release JSON: {}", e))?;
    
    println!("✅ Found release: {} ({})", release.name, release.tag_name);
    Ok(release)
}

/// Download and install update
#[command]
pub async fn download_and_install_update(
    app_handle: AppHandle,
    download_url: String,
    progress_callback: Option<String>,
) -> Result<(), String> {
    let client = create_http_client(false)?;
    
    println!("📥 Starting download from: {}", download_url);
    
    // Get the response
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    
    // Create temporary file for download
    let temp_dir = std::env::temp_dir();
    let file_name = download_url
        .split('/')
        .last()
        .unwrap_or("yuukips_launcher_update")
        .to_string();
    let temp_file_path = temp_dir.join(&file_name);
    
    let mut file = tokio::fs::File::create(&temp_file_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    let start_time = Instant::now();
    
    // Download with progress tracking
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        
        downloaded += chunk.len() as u64;
        
        // Emit progress event if callback is provided
        if let Some(ref callback) = progress_callback {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { downloaded as f64 / elapsed } else { 0.0 };
            
            let progress = serde_json::json!({
                "downloaded": downloaded,
                "total": total_size,
                "percentage": if total_size > 0 { (downloaded as f64 / total_size as f64) * 100.0 } else { 0.0 },
                "speed": speed
            });
            
            let _ = app_handle.emit(callback, progress);
        }
    }
    
    file.flush().await.map_err(|e| format!("Failed to flush file: {}", e))?;
    
    println!("✅ Download completed: {} bytes", downloaded);
    
    // Install the update (for Windows, this would typically be an MSI or EXE)
    install_update(&temp_file_path).await?;
    
    Ok(())
}

/// Install the downloaded update with admin privileges
async fn install_update(file_path: &PathBuf) -> Result<(), String> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    println!("🔧 Installing update: {}", file_name);
    
    // Try to install with admin privileges to handle file access issues
    if file_name.ends_with(".msi") {
        install_msi_with_admin(file_path).await
    } else if file_name.ends_with(".exe") {
        install_exe_with_admin(file_path).await
    } else {
        Err("Unsupported update file format".to_string())
    }
}

/// Install MSI package with administrator privileges
#[cfg(target_os = "windows")]
async fn install_msi_with_admin(file_path: &PathBuf) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::um::shellapi::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use winapi::um::synchapi::WaitForSingleObject;
    use winapi::um::winbase::INFINITE;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::GetExitCodeProcess;
    
    let file_path_str = file_path.to_str().ok_or("Invalid file path")?;
    let parameters = format!("/i \"{}\" /quiet /norestart", file_path_str);
    
    // Convert strings to wide strings for Windows API
    let verb: Vec<u16> = OsStr::new("runas").encode_wide().chain(std::iter::once(0)).collect();
    let file: Vec<u16> = OsStr::new("msiexec").encode_wide().chain(std::iter::once(0)).collect();
    let params: Vec<u16> = OsStr::new(&parameters).encode_wide().chain(std::iter::once(0)).collect();
    
    unsafe {
        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            hwnd: ptr::null_mut(),
            lpVerb: verb.as_ptr(),
            lpFile: file.as_ptr(),
            lpParameters: params.as_ptr(),
            lpDirectory: ptr::null(),
            nShow: 0, // SW_HIDE
            hInstApp: ptr::null_mut(),
            lpIDList: ptr::null_mut(),
            lpClass: ptr::null(),
            hkeyClass: ptr::null_mut(),
            dwHotKey: 0,
            hMonitor: ptr::null_mut(),
            hProcess: ptr::null_mut(),
        };
        
        if ShellExecuteExW(&mut sei) == 0 {
            return Err("Failed to start MSI installer with admin privileges".to_string());
        }
        
        if sei.hProcess.is_null() {
            return Err("Failed to get installer process handle".to_string());
        }
        
        // Wait for the installer to complete
        WaitForSingleObject(sei.hProcess, INFINITE);
        
        // Check exit code
        let mut exit_code: u32 = 0;
        if GetExitCodeProcess(sei.hProcess, &mut exit_code) != 0 {
            CloseHandle(sei.hProcess);
            if exit_code == 0 {
                println!("✅ MSI update installed successfully");
                Ok(())
            } else {
                Err(format!("MSI installer failed with exit code: {}", exit_code))
            }
        } else {
            CloseHandle(sei.hProcess);
            Err("Failed to get installer exit code".to_string())
        }
    }
}

/// Install EXE package with administrator privileges
#[cfg(target_os = "windows")]
async fn install_exe_with_admin(file_path: &PathBuf) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::um::shellapi::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use winapi::um::synchapi::WaitForSingleObject;
    use winapi::um::winbase::INFINITE;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::GetExitCodeProcess;
    
    let file_path_str = file_path.to_str().ok_or("Invalid file path")?;
    let parameters = "/S"; // Silent install flag
    
    // Convert strings to wide strings for Windows API
    let verb: Vec<u16> = OsStr::new("runas").encode_wide().chain(std::iter::once(0)).collect();
    let file: Vec<u16> = OsStr::new(file_path_str).encode_wide().chain(std::iter::once(0)).collect();
    let params: Vec<u16> = OsStr::new(parameters).encode_wide().chain(std::iter::once(0)).collect();
    
    unsafe {
        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            hwnd: ptr::null_mut(),
            lpVerb: verb.as_ptr(),
            lpFile: file.as_ptr(),
            lpParameters: params.as_ptr(),
            lpDirectory: ptr::null(),
            nShow: 0, // SW_HIDE
            hInstApp: ptr::null_mut(),
            lpIDList: ptr::null_mut(),
            lpClass: ptr::null(),
            hkeyClass: ptr::null_mut(),
             dwHotKey: 0,
             hMonitor: ptr::null_mut(),
             hProcess: ptr::null_mut(),
         };
        
        if ShellExecuteExW(&mut sei) == 0 {
            return Err("Failed to start EXE installer with admin privileges".to_string());
        }
        
        if sei.hProcess.is_null() {
            return Err("Failed to get installer process handle".to_string());
        }
        
        // Wait for the installer to complete
        WaitForSingleObject(sei.hProcess, INFINITE);
        
        // Check exit code
        let mut exit_code: u32 = 0;
        if GetExitCodeProcess(sei.hProcess, &mut exit_code) != 0 {
            CloseHandle(sei.hProcess);
            if exit_code == 0 {
                println!("✅ EXE update installed successfully");
                Ok(())
            } else {
                Err(format!("EXE installer failed with exit code: {}", exit_code))
            }
        } else {
            CloseHandle(sei.hProcess);
            Err("Failed to get installer exit code".to_string())
        }
    }
}

/// Non-Windows fallback implementations
#[cfg(not(target_os = "windows"))]
async fn install_msi_with_admin(_file_path: &PathBuf) -> Result<(), String> {
    Err("MSI installation is only supported on Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
async fn install_exe_with_admin(_file_path: &PathBuf) -> Result<(), String> {
    Err("EXE installation with admin privileges is only supported on Windows".to_string())
}

/// Restart the application
#[command]
pub async fn restart_application(app_handle: AppHandle) -> Result<(), String> {
    println!("🔄 Restarting application...");
    
    // Give a small delay to ensure the response is sent
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    app_handle.restart();
    // Note: This function will never return as the app restarts
}

/// Terminate the current application process to allow installer to replace files
#[command]
pub async fn terminate_for_update() -> Result<(), String> {
    println!("🔄 Terminating application for update installation...");
    
    // Give a small delay to ensure the response is sent
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    #[cfg(target_os = "windows")]
    {
        use std::process;
        process::exit(0);
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        std::process::exit(0);
    }
}