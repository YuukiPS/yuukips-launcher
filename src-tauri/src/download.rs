//! Download management module
//! 
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::command;
use uuid::Uuid;
use url;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncWriteExt;

static DOWNLOAD_MANAGER: once_cell::sync::Lazy<Arc<Mutex<DownloadManager>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(DownloadManager::new())));

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadItem {
    pub id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileExtension")]
    pub file_extension: String,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    #[serde(rename = "downloadedSize")]
    pub downloaded_size: u64,
    pub progress: f64,
    pub speed: u64, // bytes per second
    pub status: DownloadStatus,
    #[serde(rename = "timeRemaining")]
    pub time_remaining: u64, // seconds
    pub url: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "startTime")]
    pub start_time: u64,
    #[serde(rename = "endTime")]
    pub end_time: Option<u64>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadHistory {
    pub id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileSize")]
    pub file_size: u64,
    #[serde(rename = "downloadDate")]
    pub download_date: String,
    pub status: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadStats {
    pub total_downloads: u32,
    pub active_downloads: u32,
    pub completed_downloads: u32,
    pub total_downloaded_size: u64,
    pub average_speed: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Downloading,
    Paused,
    Completed,
    Error,
    Cancelled,
}

struct DownloadManager {
    downloads: HashMap<String, DownloadItem>,
    history: Vec<DownloadHistory>,
    download_directory: PathBuf,
    cancellation_tokens: HashMap<String, Arc<AtomicBool>>,
}

impl DownloadManager {
    fn new() -> Self {
        let download_directory = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("./downloads"));
        
        Self {
            downloads: HashMap::new(),
            history: Vec::new(),
            download_directory,
            cancellation_tokens: HashMap::new(),
        }
    }

    fn add_download(&mut self, url: String, file_path: String, file_name: Option<String>) -> String {
        let id = Uuid::new_v4().to_string();
        
        // Clean the URL by trimming whitespace and removing trailing commas/semicolons
        let cleaned_url = url.trim().trim_end_matches(',').trim_end_matches(';').to_string();
        println!("[Rust] Cleaned URL from '{}' to '{}'", url, cleaned_url);
        
        let path = Path::new(&file_path);
        let actual_file_name = file_name.unwrap_or_else(|| {
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("download")
                .to_string()
        });
        let file_extension = path.extension()
            .and_then(|s| s.to_str())
            .map(|s| format!(".{}", s))
            .unwrap_or_default();

        let download = DownloadItem {
            id: id.clone(),
            file_name: actual_file_name,
            file_extension,
            total_size: 0,
            downloaded_size: 0,
            progress: 0.0,
            speed: 0,
            status: DownloadStatus::Downloading,
            time_remaining: 0,
            url: cleaned_url,
            file_path,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            end_time: None,
            error_message: None,
        };

        self.downloads.insert(id.clone(), download);
        id
    }

    fn update_download_progress(&mut self, id: &str, downloaded: u64, total: u64, speed: u64) {
        if let Some(download) = self.downloads.get_mut(id) {
            download.downloaded_size = downloaded;
            download.total_size = total;
            download.progress = if total > 0 {
                (downloaded as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            download.speed = speed;
            download.time_remaining = if speed > 0 && total > downloaded {
                (total - downloaded) / speed
            } else {
                0
            };
        }
    }

    fn set_download_status(&mut self, id: &str, status: DownloadStatus, error_message: Option<String>) {
        if let Some(download) = self.downloads.get_mut(id) {
            download.status = status.clone();
            download.error_message = error_message;
            
            if matches!(status, DownloadStatus::Completed | DownloadStatus::Error | DownloadStatus::Cancelled) {
                download.end_time = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );
                
                // Clean up cancellation token only for final states
                self.cancellation_tokens.remove(id);
                
                // Add to history
                let history_item = DownloadHistory {
                    id: download.id.clone(),
                    file_name: download.file_name.clone(),
                    file_size: download.total_size,
                    download_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    status: match status {
                        DownloadStatus::Completed => "completed".to_string(),
                        DownloadStatus::Error => "error".to_string(),
                        DownloadStatus::Cancelled => "cancelled".to_string(),
                        _ => "unknown".to_string(),
                    },
                    file_path: download.file_path.clone(),
                    error_message: download.error_message.clone(),
                };
                
                self.history.push(history_item);
            }
        }
    }
    
    fn set_download_status_no_cleanup(&mut self, id: &str, status: DownloadStatus, error_message: Option<String>) {
        if let Some(download) = self.downloads.get_mut(id) {
            download.status = status;
            download.error_message = error_message;
        }
    }

    fn get_stats(&self) -> DownloadStats {
        let active_downloads = self.downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Downloading | DownloadStatus::Paused))
            .count() as u32;
        
        let completed_downloads = self.downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Completed))
            .count() as u32;
        
        let total_downloaded_size = self.downloads.values()
            .map(|d| d.downloaded_size)
            .sum();
        
        let active_speeds: Vec<u64> = self.downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Downloading))
            .map(|d| d.speed)
            .collect();
        
        let average_speed = if !active_speeds.is_empty() {
            active_speeds.iter().sum::<u64>() / active_speeds.len() as u64
        } else {
            0
        };

        DownloadStats {
            total_downloads: self.downloads.len() as u32,
            active_downloads,
            completed_downloads,
            total_downloaded_size,
            average_speed,
        }
    }
}

/// Start a new download
#[command]
pub async fn start_download(
    url: String,
    file_path: String,
    file_name: Option<String>,
) -> Result<String, String> {
    println!("[Rust] Starting new download: url={}, file_path={}, file_name={:?}", url, file_path, file_name);
    
    let download_id = {
        let mut manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| {
                let error_msg = format!("Failed to lock download manager: {}", e);
                println!("[Rust] Error: {}", error_msg);
                error_msg
            })?;
        let id = manager.add_download(url.clone(), file_path.clone(), file_name);
        println!("[Rust] Download added to manager with ID: {}", id);
        id
    };

    // Start the download in a background task
    let download_id_clone = download_id.clone();
    let url_clone = url.clone();
    let file_path_clone = file_path.clone();
    
    println!("[Rust] Spawning background task for download ID: {}", download_id);
    tauri::async_runtime::spawn(async move {
        println!("[Rust] Background task started for download ID: {}", download_id_clone);
        if let Err(e) = perform_download(download_id_clone.clone(), url_clone, file_path_clone).await {
            println!("[Rust] Download failed for ID {}: {}", download_id_clone, e);
        } else {
            println!("[Rust] Download completed successfully for ID: {}", download_id_clone);
        }
    });

    println!("[Rust] Download initiation completed, returning ID: {}", download_id);
    Ok(download_id)
}

/// Pause a download
#[command]
pub fn pause_download(download_id: String) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    if let Some(download) = manager.downloads.get(&download_id) {
        if download.status != DownloadStatus::Downloading {
            return Err("Download is not active".to_string());
        }
        
        // Signal cancellation to stop current download
        if let Some(token) = manager.cancellation_tokens.get(&download_id) {
            token.store(true, Ordering::Relaxed);
        }
        
        // Set status to paused without cleaning up cancellation token
        manager.set_download_status_no_cleanup(&download_id, DownloadStatus::Paused, None);
    }
    
    Ok(())
}

/// Resume a paused download
#[command]
pub async fn resume_download(download_id: String) -> Result<(), String> {
    let download_info = {
        let mut manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| format!("Failed to lock download manager: {}", e))?;
        
        if let Some(download) = manager.downloads.get(&download_id) {
            if download.status == DownloadStatus::Paused {
                let url = download.url.clone();
                let file_path = download.file_path.clone();
                
                // Remove old cancellation token and set status to downloading
                manager.cancellation_tokens.remove(&download_id);
                manager.set_download_status_no_cleanup(&download_id, DownloadStatus::Downloading, None);
                
                Some((url, file_path))
            } else {
                None
            }
        } else {
            None
        }
    };
    
    // If we have download info, perform the download outside the lock
    if let Some((url, file_path)) = download_info {
        // Use a separate task with tauri's async runtime
        tauri::async_runtime::spawn(async move {
            if let Err(e) = perform_download(download_id.clone(), url, file_path).await {
                eprintln!("Resume download failed: {}", e);
            }
        });
    }
    
    Ok(())
}

/// Cancel a download
#[command]
pub fn cancel_download(download_id: String) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    // Signal cancellation
    if let Some(token) = manager.cancellation_tokens.get(&download_id) {
        token.store(true, Ordering::Relaxed);
    }
    
    manager.set_download_status(&download_id, DownloadStatus::Cancelled, None);
    Ok(())
}

/// Restart a failed download
#[command]
pub async fn restart_download(download_id: String) -> Result<(), String> {
    let download_info = {
        let mut manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| format!("Failed to lock download manager: {}", e))?;
        
        if let Some(download) = manager.downloads.get(&download_id) {
            if matches!(download.status, DownloadStatus::Error | DownloadStatus::Cancelled) {
                let total_size = download.total_size;
                let url = download.url.clone();
                let file_path = download.file_path.clone();
                
                // Clean up any existing cancellation token and set status
                manager.cancellation_tokens.remove(&download_id);
                manager.set_download_status_no_cleanup(&download_id, DownloadStatus::Downloading, None);
                
                // Reset progress
                manager.update_download_progress(&download_id, 0, total_size, 0);
                
                Some((url, file_path))
            } else {
                None
            }
        } else {
            None
        }
    };
    
    // If we have download info, perform the download outside the lock
    if let Some((url, file_path)) = download_info {
        // Use a separate task with tauri's async runtime
        tauri::async_runtime::spawn(async move {
            if let Err(e) = perform_download(download_id.clone(), url, file_path).await {
                eprintln!("Restart download failed: {}", e);
            }
        });
    }
    
    Ok(())
}

/// Get all active downloads
#[command]
pub fn get_active_downloads() -> Result<Vec<DownloadItem>, String> {
    let manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    Ok(manager.downloads.values().cloned().collect())
}

/// Get download status for a specific download
#[command]
pub fn get_download_status(download_id: String) -> Result<Option<DownloadItem>, String> {
    let manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    Ok(manager.downloads.get(&download_id).cloned())
}

/// Get download history
#[command]
pub fn get_download_history() -> Result<Vec<DownloadHistory>, String> {
    let manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    Ok(manager.history.clone())
}

/// Clear completed downloads
#[command]
pub fn clear_completed_downloads() -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.downloads.retain(|_, download| {
        !matches!(download.status, DownloadStatus::Completed)
    });
    
    Ok(())
}

/// Clear download history
#[command]
pub fn clear_download_history() -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.history.clear();
    Ok(())
}

/// Get download statistics
#[command]
pub fn get_download_stats() -> Result<DownloadStats, String> {
    let manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    Ok(manager.get_stats())
}

/// Open download location in file explorer
#[command]
pub fn open_download_location(file_path: String) -> Result<(), String> {
    let path = Path::new(&file_path);
    
    if let Some(parent) = path.parent() {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open file explorer: {}", e))?;
        }
        
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open finder: {}", e))?;
        }
        
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open file manager: {}", e))?;
        }
    }
    
    Ok(())
}

/// Set download directory
#[command]
pub fn set_download_directory(directory: String) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.download_directory = PathBuf::from(directory);
    Ok(())
}

/// Get download directory
#[command]
pub fn get_download_directory() -> Result<String, String> {
    let manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    Ok(manager.download_directory.to_string_lossy().to_string())
}

/// Bulk operations
#[command]
pub fn bulk_pause_downloads(download_ids: Vec<String>) -> Result<(), String> {
    for id in download_ids {
        pause_download(id)?;
    }
    Ok(())
}

#[command]
pub async fn bulk_resume_downloads(download_ids: Vec<String>) -> Result<(), String> {
    for id in download_ids {
        resume_download(id).await?;
    }
    Ok(())
}

#[command]
pub fn bulk_cancel_downloads(download_ids: Vec<String>) -> Result<(), String> {
    for id in download_ids {
        cancel_download(id)?;
    }
    Ok(())
}

/// Utility functions
#[command]
pub async fn validate_download_url(url: String) -> Result<bool, String> {
    validate_download_url_with_options(url, false).await
}

/// Enhanced URL validation with option to skip HEAD request
#[command]
pub async fn validate_download_url_with_options(url: String, skip_head_check: bool) -> Result<bool, String> {
    // Clean the URL by trimming whitespace and removing trailing commas/semicolons
    let cleaned_url = url.trim().trim_end_matches(',').trim_end_matches(';');
    println!("[Rust] Validating download URL: {} -> {} (skip_head_check: {})", url, cleaned_url, skip_head_check);
    
    // First validate URL format
    if url::Url::parse(cleaned_url).is_ok() {
        println!("[Rust] URL format validation passed: {}", cleaned_url);
            
            if skip_head_check {
                println!("[Rust] Skipping HEAD request validation as requested");
                return Ok(true);
            }
            
            // Then check if URL is accessible with timeout and retry
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .connect_timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            
            println!("[Rust] Sending HEAD request to validate URL accessibility");
            
            // Retry logic for network requests
            let max_retries = 2;
            let mut last_error = None;
            let mut connection_errors = 0;
            
            for attempt in 1..=max_retries {
                println!("[Rust] HEAD request attempt {} of {}", attempt, max_retries);
                
                match client.head(cleaned_url).send().await {
                    Ok(response) => {
                        let status = response.status();
                        let is_success = status.is_success();
                        
                        println!("[Rust] URL validation response: status={}, success={}", status, is_success);
                        
                        // Log headers for debugging
                        println!("[Rust] Response headers: {:?}", response.headers());
                        
                        if is_success {
                            println!("[Rust] URL validation successful on attempt {}: {}", attempt, cleaned_url);
                            return Ok(true);
                        } else {
                            println!("[Rust] URL validation failed - HTTP status: {} on attempt {}", status, attempt);
                            if attempt == max_retries {
                                return Ok(false);
                            }
                        }
                    },
                    Err(e) => {
                        let error_string = e.to_string();
                        let error_type = if e.is_timeout() {
                            "Timeout error"
                        } else if e.is_connect() {
                            connection_errors += 1;
                            "Connection error"
                        } else if e.is_request() {
                            "Request error"
                        } else if error_string.contains("SSL") || error_string.contains("ssl") || error_string.contains("SSL_ERROR_SYSCALL") {
                            "SSL error"
                        } else {
                            "Unknown error"
                        };
                        
                        println!("[Rust] URL validation error on attempt {}: {}: {}", attempt, error_type, e);
                        last_error = Some((error_type, error_string));
                        
                        if attempt < max_retries {
                            println!("[Rust] Retrying in 1 second...");
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                    },
                }
            }
            
            // If HEAD requests failed, try a range request as fallback
            if last_error.is_some() {
                println!("[Rust] HEAD requests failed with SSL errors. Trying range request fallback (bytes=0-0)...");
                
                // Try a range request for bytes 0-0 (similar to curl -r 0-0)
                match client.get(cleaned_url)
                    .header("Range", "bytes=0-0")
                    .send()
                    .await {
                    Ok(response) => {
                        let status = response.status();
                        println!("[Rust] Range request fallback response: status={}", status);
                        
                        // 206 Partial Content is the expected response for a successful range request
                        if status == reqwest::StatusCode::PARTIAL_CONTENT || status.is_success() {
                            println!("[Rust] URL validation successful via range request fallback: {}", cleaned_url);
                            return Ok(true);
                        } else {
                            println!("[Rust] URL validation failed via range request - HTTP status: {}", status);
                        }
                    },
                    Err(e) => {
                        println!("[Rust] Range request fallback also failed: {}", e);
                    }
                }
            }
            
            if let Some((error_type, error_msg)) = last_error {
                println!("[Rust] URL validation failed after {} attempts: {} - {}", max_retries, error_type, error_msg);
                
                // If we have connection errors, suggest skipping HEAD check
                if connection_errors >= max_retries {
                    println!("[Rust] All attempts failed with connection errors. Consider using skip_head_check=true for this URL.");
                    return Err(format!("Connection failed: {}. You can try skipping URL validation by enabling 'Skip URL Check' option.", error_msg));
                }
        }
        Ok(false)
    } else {
        println!("[Rust] URL format validation failed: invalid URL format");
        Ok(false)
    }
}

#[command]
pub async fn get_file_size_from_url(url: String) -> Result<u64, String> {
    // Clean the URL by trimming whitespace and removing trailing commas/semicolons
    let cleaned_url = url.trim().trim_end_matches(',').trim_end_matches(';');
    println!("[Rust] Getting file size from URL: {} -> {}", url, cleaned_url);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    
    // Try HEAD request first
    match client.head(cleaned_url).send().await {
        Ok(response) => {
            if let Some(content_length) = response.headers().get("content-length") {
                let size_str = content_length.to_str()
                    .map_err(|e| format!("Invalid content-length header: {}", e))?;
                let size = size_str.parse::<u64>()
                    .map_err(|e| format!("Failed to parse content-length: {}", e))?;
                return Ok(size);
            } else {
                return Err("Content-Length header not found".to_string());
            }
        }
        Err(e) => {
             let error_string = e.to_string();
             // If HEAD request failed, try range request fallback
             println!("[Rust] HEAD request failed, trying range request fallback for file size");
             
             match client.get(cleaned_url)
                 .header("Range", "bytes=0-0")
                 .send()
                 .await {
                 Ok(response) => {
                     // Look for Content-Range header which contains the total size
                     if let Some(content_range) = response.headers().get("content-range") {
                         let range_str = content_range.to_str()
                             .map_err(|e| format!("Invalid content-range header: {}", e))?;
                         
                         // Parse "bytes 0-0/total_size" format
                         if let Some(total_part) = range_str.split('/').nth(1) {
                             let size = total_part.parse::<u64>()
                                 .map_err(|e| format!("Failed to parse total size from content-range: {}", e))?;
                             return Ok(size);
                         }
                     }
                     return Err("Could not determine file size from range request".to_string());
                 }
                 Err(range_err) => {
                     return Err(format!("Both HEAD and range requests failed: HEAD: {}, Range: {}", error_string, range_err));
                 }
             }
        }
    }
}

#[command]
pub fn check_file_exists(file_path: String) -> Result<bool, String> {
    Ok(Path::new(&file_path).exists())
}

#[command]
pub fn get_available_disk_space(path: String) -> Result<u64, String> {
    use std::fs;
    
    // This is a simplified implementation
    // In a real implementation, you'd use platform-specific APIs
    match fs::metadata(&path) {
        Ok(_) => Ok(1024 * 1024 * 1024 * 10), // Return 10GB as placeholder
        Err(e) => Err(format!("Failed to get disk space: {}", e)),
    }
}

/// Core download function using reqwest with robust error handling
async fn perform_download(download_id: String, url: String, file_path: String) -> Result<(), String> {
    println!("[Rust] Starting perform_download for ID: {}, URL: {}, Path: {}", download_id, url, file_path);
    
    // Create cancellation token
    let cancellation_token = Arc::new(AtomicBool::new(false));
    println!("[Rust] Created cancellation token for download ID: {}", download_id);
    
    // Store cancellation token
    {
        let mut manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| {
                let error_msg = format!("Failed to lock download manager: {}", e);
                println!("[Rust] Error storing cancellation token: {}", error_msg);
                error_msg
            })?;
        manager.cancellation_tokens.insert(download_id.clone(), cancellation_token.clone());
        println!("[Rust] Stored cancellation token for download ID: {}", download_id);
    }

    // Check if download is still active
    let should_continue = {
        let manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| {
                let error_msg = format!("Failed to lock download manager: {}", e);
                println!("[Rust] Error checking download status: {}", error_msg);
                error_msg
            })?;
        let status = manager.downloads.get(&download_id)
            .map(|d| d.status.clone())
            .unwrap_or(DownloadStatus::Error);
        let should_continue = matches!(status, DownloadStatus::Downloading);
        println!("[Rust] Download status check for ID {}: status={:?}, should_continue={}", download_id, status, should_continue);
        should_continue
    };

    if !should_continue {
        println!("[Rust] Download ID {} is not in downloading state, aborting", download_id);
        return Ok(());
    }

    // Try to get file info with HEAD request, but don't fail if it doesn't work
    println!("[Rust] Attempting to get file info with HEAD request for ID: {}", download_id);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    
    let mut total_size = 0u64;
    let max_retries = 3;
    let mut head_request_successful = false;
    
    for attempt in 1..=max_retries {
        println!("[Rust] HEAD request attempt {} of {} for ID: {}", attempt, max_retries, download_id);
        
        match client.head(&url).send().await {
            Ok(resp) => {
                total_size = resp.content_length().unwrap_or(0);
                println!("[Rust] File size determined via HEAD request for ID {}: {} bytes", download_id, total_size);
                head_request_successful = true;
                break;
            }
            Err(e) => {
                println!("[Rust] HEAD request failed on attempt {} for ID {}: {}", attempt, download_id, e);
                if attempt == max_retries {
                    println!("[Rust] HEAD request failed after {} attempts for ID {}. Will proceed with download and determine size during transfer.", max_retries, download_id);
                } else {
                    println!("[Rust] Retrying HEAD request in 2 seconds...");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    
    if !head_request_successful {
        println!("[Rust] Could not determine file size via HEAD request for ID {}. Size will be determined during download.", download_id);
    }
    
    // Update total size
    {
        let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
        manager.update_download_progress(&download_id, 0, total_size, 0);
        println!("[Rust] Updated download progress for ID {}: 0/{} bytes", download_id, total_size);
    }

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(&file_path).parent() {
        println!("[Rust] Creating parent directories for ID {}: {:?}", download_id, parent);
        fs::create_dir_all(parent)
            .map_err(|e| {
                let error_msg = format!("Failed to create directories: {}", e);
                println!("[Rust] Error creating directories for ID {}: {}", download_id, error_msg);
                error_msg
            })?;
        println!("[Rust] Parent directories created successfully for ID: {}", download_id);
    }

     // Use custom implementation for download
     let download_id_clone2 = download_id.clone();
     let url_clone = url.clone();
     let file_path_clone = file_path.clone();
     let cancellation_token_clone2 = cancellation_token.clone();
     
     let custom_result = tauri::async_runtime::spawn(async move {
         let client = reqwest::Client::builder()
             .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
            
        let mut downloaded = 0u64;
        let mut total_size = 0u64;
        let mut last_update = std::time::Instant::now();
        let mut last_downloaded = 0u64;
        let mut consecutive_errors = 0u32;
        let max_consecutive_errors = 5;
        let mut last_progress_time = std::time::Instant::now();
        let progress_timeout = std::time::Duration::from_secs(30);
        
        // Check if file already exists (for resume functionality)
        if let Ok(metadata) = std::fs::metadata(&file_path_clone) {
            downloaded = metadata.len();
            println!("[Rust] Found existing file for ID {}: {} bytes already downloaded", download_id_clone2, downloaded);
        }
        
        // Get file size with HEAD request
        match client.head(&url_clone).send().await {
            Ok(response) => {
                if let Some(content_length) = response.content_length() {
                    total_size = content_length;
                    println!("[Rust] File size determined from HEAD request for ID {}: {} bytes", download_id_clone2, total_size);
                } else {
                    println!("[Rust] Content-Length not available from HEAD request for ID {}", download_id_clone2);
                }
            }
            Err(e) => {
                println!("[Rust] HEAD request failed for ID {}: {}", download_id_clone2, e);
            }
        }
        
        loop {
             // Check for cancellation
             if cancellation_token_clone2.load(Ordering::Relaxed) {
                 println!("[Rust] Download cancellation detected for ID: {}", download_id_clone2);
                 break;
             }

             // Check if download should be paused or cancelled
             let should_continue = {
                 let manager = DOWNLOAD_MANAGER.lock()
                     .map_err(|e| format!("Failed to lock download manager: {}", e))?;
                 manager.downloads.get(&download_id_clone2)
                     .map(|d| matches!(d.status, DownloadStatus::Downloading))
                     .unwrap_or(false)
             };

             if !should_continue {
                 println!("[Rust] Download should not continue for ID: {}", download_id_clone2);
                 break;
             }
            
            // Check for progress timeout
             if last_progress_time.elapsed() > progress_timeout {
                 let error_msg = format!("Download timeout: No progress for {} seconds", progress_timeout.as_secs());
                 println!("[Rust] Download timeout for ID {}: {}", download_id_clone2, error_msg);
                 return Err(error_msg);
             }
             
             // Create range request for resumable download
             let mut request = client.get(&url_clone);
             if downloaded > 0 {
                  let range_header = format!("bytes={}-", downloaded);
                  request = request.header("Range", &range_header);
                  println!("[Rust] Using range request for resume: {} for ID {}", range_header, download_id_clone2);
              } else {
                  println!("[Rust] Starting fresh download for ID {}", download_id_clone2);
              }
            
            match request.send().await {
                Ok(mut response) => {
                     if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                         consecutive_errors += 1;
                         println!("[Rust] HTTP error {} for ID {}: attempt {}", response.status(), download_id_clone2, consecutive_errors);
                         
                         if consecutive_errors >= max_consecutive_errors {
                             return Err(format!("HTTP error after {} attempts: {}", max_consecutive_errors, response.status()));
                         }
                         
                         tokio::time::sleep(std::time::Duration::from_millis(1000 * consecutive_errors as u64)).await;
                         continue;
                     }
                     
                     // Reset error counter on successful response
                     consecutive_errors = 0;
                     
                     // Handle total size for both fresh downloads and resumes
                     if total_size == 0 {
                         if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                             // For partial content, check Content-Range header for total size
                             if let Some(content_range) = response.headers().get("content-range") {
                                 if let Ok(range_str) = content_range.to_str() {
                                     // Parse "bytes start-end/total" format
                                     if let Some(total_part) = range_str.split('/').nth(1) {
                                         if let Ok(parsed_total) = total_part.parse::<u64>() {
                                             total_size = parsed_total;
                                             println!("[Rust] Total file size from Content-Range for ID {}: {} bytes", download_id_clone2, total_size);
                                         }
                                     }
                                 }
                             }
                         } else if let Some(content_length) = response.content_length() {
                             // For fresh downloads, use Content-Length
                             total_size = if downloaded > 0 {
                                 // If we're resuming, add the already downloaded bytes
                                 content_length + downloaded
                             } else {
                                 content_length
                             };
                             println!("[Rust] File size determined from GET response for ID {}: {} bytes (downloaded: {})", download_id_clone2, total_size, downloaded);
                         } else {
                             println!("[Rust] Content-Length not available for ID {}. Download size will be unknown.", download_id_clone2);
                         }
                         
                         // Update the download manager with the new total size
                         if total_size > 0 {
                             let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
                             manager.update_download_progress(&download_id_clone2, downloaded, total_size, 0);
                         }
                     }
                    
                    // Open/create file for writing
                     let mut file = if downloaded > 0 {
                         tokio::fs::OpenOptions::new()
                             .write(true)
                             .append(true)
                             .open(&file_path_clone)
                             .await
                             .map_err(|e| format!("Failed to open file for append: {}", e))?
                     } else {
                         tokio::fs::File::create(&file_path_clone).await
                             .map_err(|e| format!("Failed to create file: {}", e))?
                     };
                    
                    // Download chunks
                     while let Some(chunk_result) = response.chunk().await.transpose() {
                         // Check for cancellation
                         if cancellation_token_clone2.load(Ordering::Relaxed) {
                             println!("[Rust] Download cancellation detected during chunk read for ID: {}", download_id_clone2);
                             return Ok(());
                         }
                         
                         // Check if download should continue
                         let should_continue = {
                             let manager = DOWNLOAD_MANAGER.lock()
                                 .map_err(|e| format!("Failed to lock download manager: {}", e))?;
                             manager.downloads.get(&download_id_clone2)
                                 .map(|d| matches!(d.status, DownloadStatus::Downloading))
                                 .unwrap_or(false)
                         };

                         if !should_continue {
                             println!("[Rust] Download should not continue during chunk read for ID: {}", download_id_clone2);
                             return Ok(());
                         }
                        
                        match chunk_result {
                            Ok(chunk) => {
                                last_progress_time = std::time::Instant::now();
                                
                                use tokio::io::AsyncWriteExt;
                                file.write_all(&chunk).await
                                    .map_err(|e| format!("Failed to write to file: {}", e))?;

                                downloaded += chunk.len() as u64;

                                // Update progress every 500ms
                                let now = std::time::Instant::now();
                                if now.duration_since(last_update).as_millis() >= 500 {
                                    let elapsed_secs = now.duration_since(last_update).as_secs_f64();
                                    let speed = if elapsed_secs > 0.0 {
                                        ((downloaded - last_downloaded) as f64 / elapsed_secs) as u64
                                    } else {
                                        0
                                    };
                                    
                                    let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
                                     manager.update_download_progress(&download_id_clone2, downloaded, total_size, speed);
                                     
                                     last_update = now;
                                     last_downloaded = downloaded;
                                 }
                                 
                                 // Check if download is complete
                                 if total_size > 0 && downloaded >= total_size {
                                     println!("[Rust] Download completed: {} bytes for ID: {}", downloaded, download_id_clone2);
                                     file.flush().await
                                         .map_err(|e| format!("Failed to flush file: {}", e))?;
                                     return Ok(());
                                 }
                             }
                             Err(e) => {
                                 consecutive_errors += 1;
                                 println!("[Rust] Chunk read error {} for ID {}: {}", consecutive_errors, download_id_clone2, e);
                                
                                if consecutive_errors >= max_consecutive_errors {
                                    return Err(format!("Download failed after {} consecutive chunk errors: {}", max_consecutive_errors, e));
                                }
                                
                                // Break from chunk loop to retry request
                                break;
                            }
                        }
                    }
                    
                    // If we reach here and total_size is 0 or unknown, consider download complete
                     if total_size == 0 {
                         println!("[Rust] Download completed (unknown size): {} bytes for ID: {}", downloaded, download_id_clone2);
                         file.flush().await
                             .map_err(|e| format!("Failed to flush file: {}", e))?;
                         return Ok(());
                     }
                 }
                 Err(e) => {
                     consecutive_errors += 1;
                     println!("[Rust] Request error {} for ID {}: {}", consecutive_errors, download_id_clone2, e);
                    
                    if consecutive_errors >= max_consecutive_errors {
                        return Err(format!("Download failed after {} consecutive request errors: {}", max_consecutive_errors, e));
                    }
                    
                    // Exponential backoff
                    tokio::time::sleep(std::time::Duration::from_millis(1000 * consecutive_errors as u64)).await;
                }
            }
        }
        
        Ok::<(), String>(())
    }).await;

     // Clean up cancellation token
     {
         let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
         manager.cancellation_tokens.remove(&download_id);
     }

     match custom_result {
        Ok(Ok(())) => {
            // Check final status
            let final_status = {
                let manager = DOWNLOAD_MANAGER.lock()
                    .map_err(|e| format!("Failed to lock download manager: {}", e))?;
                manager.downloads.get(&download_id)
                    .map(|d| d.status.clone())
                    .unwrap_or(DownloadStatus::Error)
            };

            match final_status {
                DownloadStatus::Downloading => {
                    // Download completed successfully
                    println!("[Rust] Download completed successfully for ID: {}", download_id);
                    let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
                    manager.set_download_status(&download_id, DownloadStatus::Completed, None);
                    println!("[Rust] Download status set to Completed for ID: {}", download_id);
                }
                _ => {
                    // Download was paused or cancelled
                    println!("[Rust] Download was paused or cancelled for ID: {}", download_id);
                }
            }
            Ok(())
        }
        Ok(Err(e)) => {
            let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
            manager.set_download_status(&download_id, DownloadStatus::Error, Some(e.clone()));
            Err(e)
        }
        Err(e) => {
            let error_msg = format!("Download task failed: {}", e);
            let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
            manager.set_download_status(&download_id, DownloadStatus::Error, Some(error_msg.clone()));
            Err(error_msg)
        }
    }
}