//! HTTP utilities module
//! Handles HTTP client creation and network requests

use reqwest;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::error::Error;
use tokio::io::AsyncWriteExt;
use crate::utils::create_hidden_command;
use url;

/// Structure to hold TLS connection details
#[derive(Debug, Clone)]
struct TlsConnectionInfo {
    pub body: String,
    pub response_time: std::time::Duration,
    pub tls_version: String,
    pub cipher_suite: String,
    pub certificate_info: String,
}

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

/// Test game API connectivity with detailed error reporting and TLS information
#[command]
pub fn test_game_api_call() -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let url = format!("https://ps.yuuki.me/json/game_all.json?time={}", timestamp);
        
        match fetch_api_data_with_tls_details(url.clone()).await {
            Ok(result) => {
                // Try to parse the response as JSON to validate
                match serde_json::from_str::<serde_json::Value>(&result.body) {
                    Ok(json) => {
                        let games_count = json.as_array().map(|arr| arr.len()).unwrap_or(0);
                        Ok(format!(
                             "✅ Game API Test Successful:\n• URL: {}\n• Response Size: {} bytes\n• Games Found: {}\n• Response Time: {:?}\n• TLS Version: {}\n• Cipher Suite: {}\n• Certificate Info: {}\n• Status: API is working correctly",
                             url, result.body.len(), games_count, result.response_time, result.tls_version, result.cipher_suite, result.certificate_info
                         ))
                    }
                    Err(e) => {
                         Ok(format!(
                             "⚠️ Game API Response Issue:\n• URL: {}\n• Response Size: {} bytes\n• JSON Parse Error: {}\n• Response Time: {:?}\n• TLS Version: {}\n• Status: Server responded but with invalid JSON",
                             url, result.body.len(), e, result.response_time, result.tls_version
                         ))
                     }
                }
            }
            Err(e) => {
                 Ok(format!(
                     "❌ Game API Test Failed:\n{}\n• Test URL: {}\n• Timestamp: {}",
                     e, url, timestamp
                 ))
             }
        }
    })
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

/// Fetch data from an API endpoint with detailed error reporting
#[command]
pub fn fetch_api_data(url: String) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        fetch_api_data_with_details(url).await
    })
}

/// Enhanced API data fetching with TLS connection details
async fn fetch_api_data_with_tls_details(url: String) -> Result<TlsConnectionInfo, String> {
    let client = create_tls_aware_http_client(false)?; // No proxy for API calls
    
    println!("📡 Fetching API data with TLS details from: {}", url);
    
    let start_time = std::time::Instant::now();
    
    match client.get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", "YuukiPS-Launcher/1.0")
        .send()
        .await
    {
        Ok(response) => {
            let elapsed = start_time.elapsed();
            let status = response.status();
            let headers = response.headers().clone();
            
            println!("✅ Response received in {:?} - Status: {}", elapsed, status);
            
            if !status.is_success() {
                let error_body = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
                return Err(format!(
                     "🚫 API Error Details:\n• URL: {}\n• Status: {} {}\n• Response Time: {:?}\n• Server: {}\n• Content-Type: {}\n• Error Body: {}\n• Suggestion: Check if the API endpoint is correct and the server is operational",
                     url,
                     status.as_u16(),
                     status.canonical_reason().unwrap_or("Unknown"),
                     elapsed,
                     headers.get("server").and_then(|v| v.to_str().ok()).unwrap_or("Unknown"),
                     headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("Unknown"),
                     if error_body.len() > 200 { format!("{}...", &error_body[..200]) } else { error_body }
                 ));
            }
            
            let body = response.text()
                 .await
                 .map_err(|e| format!(
                     "🚫 Failed to read API response:\n• URL: {}\n• Error: {}\n• Response Time: {:?}\n• Suggestion: The connection may have been interrupted while reading the response",
                     url, e, elapsed
                 ))?;
            
            // Validate that it's valid JSON
             let _: serde_json::Value = serde_json::from_str(&body)
                 .map_err(|e| format!(
                     "🚫 Invalid JSON Response:\n• URL: {}\n• JSON Error: {}\n• Response Length: {} bytes\n• Response Preview: {}\n• Suggestion: The API may be returning HTML error pages or malformed JSON",
                     url,
                     e,
                     body.len(),
                     if body.len() > 300 { format!("{}...", &body[..300]) } else { body.clone() }
                 ))?;
            
            println!("✅ Valid JSON response received ({} bytes)", body.len());
            
            // Extract TLS information from headers and connection details
            let tls_version = extract_tls_version_from_response(&headers, &url);
            let cipher_suite = extract_cipher_suite_from_response(&headers);
            let certificate_info = extract_certificate_info_from_response(&headers, &url);
            
            Ok(TlsConnectionInfo {
                body,
                response_time: elapsed,
                tls_version,
                cipher_suite,
                certificate_info,
            })
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            Err(format_detailed_request_error(&url, &e, elapsed))
        }
    }
}

/// Enhanced API data fetching with comprehensive error details
async fn fetch_api_data_with_details(url: String) -> Result<String, String> {
    let client = create_enhanced_http_client(false)?; // No proxy for API calls
    
    println!("📡 Fetching API data from: {}", url);
    
    let start_time = std::time::Instant::now();
    
    match client.get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", "YuukiPS-Launcher/1.0")
        .send()
        .await
    {
        Ok(response) => {
            let elapsed = start_time.elapsed();
            let status = response.status();
            let headers = response.headers().clone();
            
            println!("✅ Response received in {:?} - Status: {}", elapsed, status);
            
            if !status.is_success() {
                let error_body = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
                return Err(format!(
                     "🚫 API Error Details:\n• URL: {}\n• Status: {} {}\n• Response Time: {:?}\n• Server: {}\n• Content-Type: {}\n• Error Body: {}\n• Suggestion: Check if the API endpoint is correct and the server is operational",
                     url,
                     status.as_u16(),
                     status.canonical_reason().unwrap_or("Unknown"),
                     elapsed,
                     headers.get("server").and_then(|v| v.to_str().ok()).unwrap_or("Unknown"),
                     headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("Unknown"),
                     if error_body.len() > 200 { format!("{}...", &error_body[..200]) } else { error_body }
                 ));
            }
            
            let body = response.text()
                 .await
                 .map_err(|e| format!(
                     "🚫 Failed to read API response:\n• URL: {}\n• Error: {}\n• Response Time: {:?}\n• Suggestion: The connection may have been interrupted while reading the response",
                     url, e, elapsed
                 ))?;
            
            // Validate that it's valid JSON
             let _: serde_json::Value = serde_json::from_str(&body)
                 .map_err(|e| format!(
                     "🚫 Invalid JSON Response:\n• URL: {}\n• JSON Error: {}\n• Response Length: {} bytes\n• Response Preview: {}\n• Suggestion: The API may be returning HTML error pages or malformed JSON",
                     url,
                     e,
                     body.len(),
                     if body.len() > 300 { format!("{}...", &body[..300]) } else { body.clone() }
                 ))?;
            
            println!("✅ Valid JSON response received ({} bytes)", body.len());
            Ok(body)
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            Err(format_detailed_request_error(&url, &e, elapsed))
        }
    }
}

/// Create a TLS-aware HTTP client with detailed connection information
fn create_tls_aware_http_client(use_proxy: bool) -> Result<reqwest::Client, String> {
    let mut client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent("YuukiPS-Launcher/1.0")
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .min_tls_version(reqwest::tls::Version::TLS_1_2); // Ensure minimum TLS 1.2
    
    if !use_proxy {
        client_builder = client_builder.no_proxy();
    }
    
    client_builder.build()
        .map_err(|e| format!("Failed to create TLS-aware HTTP client: {}", e))
}

/// Create an enhanced HTTP client with better error reporting capabilities
fn create_enhanced_http_client(use_proxy: bool) -> Result<reqwest::Client, String> {
    let mut client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent("YuukiPS-Launcher/1.0")
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(10);
    
    if !use_proxy {
        client_builder = client_builder.no_proxy();
    }
    
    client_builder.build()
        .map_err(|e| format!("Failed to create enhanced HTTP client: {}", e))
}

/// Format detailed error information for request failures
fn format_detailed_request_error(url: &str, error: &reqwest::Error, elapsed: std::time::Duration) -> String {
    let mut error_details = format!("🚫 API Request Failed:\n• URL: {}\n• Duration: {:?}\n", url, elapsed);
    
    // Categorize the error and provide specific details
     if error.is_timeout() {
         error_details.push_str(&format!(
             "• Error Type: Timeout\n• Details: {}\n• Suggestion: The server took too long to respond. Check your internet connection or try again later.\n• TLS Info: N/A (Connection timed out before TLS handshake)",
             error
         ));
     } else if error.is_connect() {
         error_details.push_str(&format!(
             "• Error Type: Connection Failed\n• Details: {}\n• Suggestion: Cannot establish connection to server. Check if the URL is correct and the server is online.\n• TLS Info: Connection failed before TLS handshake could begin",
             error
         ));
     } else if error.is_request() {
         error_details.push_str(&format!(
             "• Error Type: Request Error\n• Details: {}\n• Suggestion: There was an issue with the request format or headers.",
             error
         ));
     } else if error.is_body() {
         error_details.push_str(&format!(
             "• Error Type: Response Body Error\n• Details: {}\n• Suggestion: The server response was corrupted or incomplete.",
             error
         ));
     } else if error.is_decode() {
         error_details.push_str(&format!(
             "• Error Type: Decode Error\n• Details: {}\n• Suggestion: The server response could not be decoded properly.",
             error
         ));
     } else {
         error_details.push_str(&format!(
             "• Error Type: Unknown\n• Details: {}\n• Suggestion: An unexpected error occurred. Please try again.",
             error
         ));
     }
     
     // Add TLS-specific information if available
     if let Some(_source) = error.source() {
         // Check for TLS-related errors in the error chain
         let error_string = format!("{:?}", error);
         if error_string.contains("tls") || error_string.contains("ssl") || error_string.contains("certificate") {
             error_details.push_str(&format!(
                 "\n• TLS/SSL Issue Detected: {}\n• TLS Suggestion: This may be a certificate validation error. Check if the server's SSL certificate is valid and trusted.",
                 extract_tls_error_details(&error_string)
             ));
         }
     }
    
    error_details
}

/// Extract TLS version information from response headers and URL
fn extract_tls_version_from_response(headers: &reqwest::header::HeaderMap, url: &str) -> String {
    // Check for TLS version in various headers
    if let Some(server_header) = headers.get("server") {
        if let Ok(server_str) = server_header.to_str() {
            if server_str.contains("TLS") {
                return format!("TLS (detected from server: {})", server_str);
            }
        }
    }
    
    // Check for security headers that might indicate TLS version
    if headers.get("strict-transport-security").is_some() {
        // HSTS indicates HTTPS/TLS is being used
        if url.starts_with("https://") {
            return "TLS 1.2+ (HTTPS with HSTS)".to_string();
        }
    }
    
    // Default assumption for HTTPS URLs
    if url.starts_with("https://") {
        "TLS 1.2+ (HTTPS connection established)".to_string()
    } else {
        "N/A (HTTP connection)".to_string()
    }
}

/// Extract cipher suite information from response headers
fn extract_cipher_suite_from_response(headers: &reqwest::header::HeaderMap) -> String {
    // Look for cipher suite information in headers
    if let Some(server_header) = headers.get("server") {
        if let Ok(server_str) = server_header.to_str() {
            if server_str.contains("OpenSSL") {
                return format!("Modern cipher suite (OpenSSL server: {})", server_str);
            } else if server_str.contains("nginx") {
                return "Modern cipher suite (nginx server)".to_string();
            } else if server_str.contains("Apache") {
                return "Modern cipher suite (Apache server)".to_string();
            }
        }
    }
    
    // Check for security-related headers that indicate strong encryption
    if headers.get("strict-transport-security").is_some() {
        "Strong cipher suite (HSTS enabled)".to_string()
    } else {
        "Standard cipher suite (details not available)".to_string()
    }
}

/// Extract certificate information from response headers and URL
fn extract_certificate_info_from_response(headers: &reqwest::header::HeaderMap, url: &str) -> String {
    let mut cert_info = Vec::new();
    
    // Extract domain from URL
    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(domain) = parsed_url.host_str() {
            cert_info.push(format!("Domain: {}", domain));
        }
    }
    
    // Check for security headers
    if let Some(hsts_header) = headers.get("strict-transport-security") {
        if let Ok(hsts_str) = hsts_header.to_str() {
            cert_info.push(format!("HSTS: {}", hsts_str));
        }
    }
    
    if headers.get("x-frame-options").is_some() {
        cert_info.push("Security headers present".to_string());
    }
    
    // Check server information
    if let Some(server_header) = headers.get("server") {
        if let Ok(server_str) = server_header.to_str() {
            cert_info.push(format!("Server: {}", server_str));
        }
    }
    
    if cert_info.is_empty() {
        "Certificate validated (details not available in headers)".to_string()
    } else {
        cert_info.join(", ")
    }
}

/// Extract TLS-specific error details from error messages
fn extract_tls_error_details(error_string: &str) -> String {
    if error_string.contains("certificate verify failed") {
        "Certificate verification failed - the server's SSL certificate is not trusted".to_string()
    } else if error_string.contains("certificate has expired") {
        "SSL certificate has expired".to_string()
    } else if error_string.contains("self signed certificate") {
        "Self-signed certificate detected - not trusted by default".to_string()
    } else if error_string.contains("hostname verification failed") {
        "Hostname verification failed - certificate doesn't match the domain".to_string()
    } else if error_string.contains("protocol version") {
        "TLS protocol version mismatch".to_string()
    } else if error_string.contains("handshake") {
        "TLS handshake failed".to_string()
    } else {
        "TLS/SSL related error detected".to_string()
    }
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
    
    // Install the update with automatic launcher termination
    install_update_with_termination(&temp_file_path).await?;
    
    Ok(())
}

/// Install the downloaded update with automatic launcher termination
async fn install_update_with_termination(file_path: &PathBuf) -> Result<(), String> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    println!("🔧 Installing update with launcher termination: {}", file_name);
    
    // Create a batch script that will handle the installation after launcher termination
    if file_name.ends_with(".msi") {
        create_and_run_msi_installer_script(file_path).await
    } else if file_name.ends_with(".exe") {
        create_and_run_exe_installer_script(file_path).await
    } else {
        Err("Unsupported update file format".to_string())
    }
}

/// Create and run MSI installer script that terminates launcher first
#[cfg(target_os = "windows")]
async fn create_and_run_msi_installer_script(file_path: &PathBuf) -> Result<(), String> {
    use std::fs;
    
    let file_path_str = file_path.to_str().ok_or("Invalid file path")?;
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("yuukips_update_installer.bat");
    
    // Get current executable path for restarting
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?
        .to_str()
        .ok_or("Invalid current executable path")?
        .to_string();
    
    // Create batch script that will:
    // 1. Wait for launcher to close
    // 2. Install the MSI with admin privileges
    // 3. Restart the launcher
    // 4. Clean up
    let script_content = format!(r#"@echo off
echo Waiting for launcher to close...
timeout /t 3 /nobreak >nul
echo Installing update...
powershell -Command "Start-Process 'msiexec' -ArgumentList '/i \"{}\", /quiet, /norestart' -Verb RunAs -Wait"
if %ERRORLEVEL% EQU 0 (
    echo Update installed successfully
    echo Restarting launcher...
    start "" "{}"
) else (
    echo Update installation failed
    pause
)
echo Cleaning up...
del "%~f0"
"#, file_path_str, current_exe);
    
    // Write the script to temp directory
    fs::write(&script_path, script_content)
        .map_err(|e| format!("Failed to create installer script: {}", e))?;
    
    // Start the script in a new process
    create_hidden_command("cmd")
        .args(["/c", "start", "", script_path.to_str().unwrap()])
        .spawn()
        .map_err(|e| format!("Failed to start installer script: {}", e))?;
    
    println!("✅ Installer script started, terminating launcher...");
    
    // Give the script time to start, then terminate this process
    tokio::time::sleep(Duration::from_millis(1000)).await;
    std::process::exit(0);
}

/// Create and run EXE installer script that terminates launcher first
#[cfg(target_os = "windows")]
async fn create_and_run_exe_installer_script(file_path: &PathBuf) -> Result<(), String> {
    use std::fs;
    
    let file_path_str = file_path.to_str().ok_or("Invalid file path")?;
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("yuukips_update_installer.bat");
    
    // Get current executable path for restarting
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?
        .to_str()
        .ok_or("Invalid current executable path")?
        .to_string();
    
    // Create batch script that will:
    // 1. Wait for launcher to close
    // 2. Install the EXE with admin privileges
    // 3. Restart the launcher
    // 4. Clean up
    let script_content = format!(r#"@echo off
echo Waiting for launcher to close...
timeout /t 3 /nobreak >nul
echo Installing update...
powershell -Command "Start-Process '{}' -ArgumentList '/S' -Verb RunAs -Wait"
if %ERRORLEVEL% EQU 0 (
    echo Update installed successfully
    echo Restarting launcher...
    start "" "{}"
) else (
    echo Update installation failed
    pause
)
echo Cleaning up...
del "%~f0"
"#, file_path_str, current_exe);
    
    // Write the script to temp directory
    fs::write(&script_path, script_content)
        .map_err(|e| format!("Failed to create installer script: {}", e))?;
    
    // Start the script in a new process
    create_hidden_command("cmd")
        .args(["/c", "start", "", script_path.to_str().unwrap()])
        .spawn()
        .map_err(|e| format!("Failed to start installer script: {}", e))?;
    
    println!("✅ Installer script started, terminating launcher...");
    
    // Give the script time to start, then terminate this process
    tokio::time::sleep(Duration::from_millis(1000)).await;
    std::process::exit(0);
}

/// Non-Windows fallback implementations
#[cfg(not(target_os = "windows"))]
async fn create_and_run_msi_installer_script(_file_path: &PathBuf) -> Result<(), String> {
    Err("MSI installation is only supported on Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
async fn create_and_run_exe_installer_script(_file_path: &PathBuf) -> Result<(), String> {
    Err("EXE installation with admin privileges is only supported on Windows".to_string())
}

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