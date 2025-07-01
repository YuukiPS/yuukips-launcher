//! Download management module
//! Handles file downloads with progress tracking, pause/resume functionality, and download history

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::command;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use reqwest::Client;
use uuid::Uuid;

// Global download manager state
static DOWNLOAD_MANAGER: once_cell::sync::Lazy<Arc<Mutex<DownloadManager>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(DownloadManager::new())));

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadItem {
    pub id: String,
    pub file_name: String,
    pub file_extension: String,
    pub total_size: u64,
    pub downloaded_size: u64,
    pub progress: f64,
    pub speed: u64, // bytes per second
    pub status: DownloadStatus,
    pub time_remaining: u64, // seconds
    pub url: String,
    pub file_path: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadHistory {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub download_date: String,
    pub status: String,
    pub file_path: String,
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
    client: Client,
}

impl DownloadManager {
    fn new() -> Self {
        let download_directory = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("./downloads"));
        
        Self {
            downloads: HashMap::new(),
            history: Vec::new(),
            download_directory,
            client: Client::new(),
        }
    }

    fn add_download(&mut self, url: String, file_path: String, file_name: Option<String>) -> String {
        let id = Uuid::new_v4().to_string();
        let path = Path::new(&file_path);
        let actual_file_name = file_name.unwrap_or_else(|| {
            path.file_stem()
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
            url,
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
                
                // Add to history
                let history_item = DownloadHistory {
                    id: download.id.clone(),
                    file_name: download.file_name.clone(),
                    file_size: download.total_size,
                    download_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
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
    let download_id = {
        let mut manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| format!("Failed to lock download manager: {}", e))?;
        manager.add_download(url.clone(), file_path.clone(), file_name)
    };

    // Start the download in a background task
    let download_id_clone = download_id.clone();
    tokio::spawn(async move {
        if let Err(e) = perform_download(download_id_clone, url, file_path).await {
            eprintln!("Download failed: {}", e);
        }
    });

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
        let total_size = download.total_size;
        
        if let Some(download) = manager.downloads.get_mut(&download_id) {
            download.status = DownloadStatus::Paused;
        }
        manager.update_download_progress(&download_id, 0, total_size, 0);
    }
    
    Ok(())
}

/// Resume a paused download
#[command]
pub fn resume_download(download_id: String) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    if let Some(download) = manager.downloads.get(&download_id) {
        if download.status == DownloadStatus::Paused {
            let url = download.url.clone();
            let file_path = download.file_path.clone();
            let download_id_clone = download_id.clone();
            
            manager.set_download_status(&download_id, DownloadStatus::Downloading, None);
            
            // Restart the download
            tokio::spawn(async move {
                if let Err(e) = perform_download(download_id_clone, url, file_path).await {
                    eprintln!("Resume download failed: {}", e);
                }
            });
        }
    }
    
    Ok(())
}

/// Cancel a download
#[command]
pub fn cancel_download(download_id: String) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.set_download_status(&download_id, DownloadStatus::Cancelled, None);
    Ok(())
}

/// Restart a failed download
#[command]
pub fn restart_download(download_id: String) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    if let Some(download) = manager.downloads.get(&download_id) {
        if matches!(download.status, DownloadStatus::Error | DownloadStatus::Cancelled) {
            let total_size = download.total_size;
            let url = download.url.clone();
            let file_path = download.file_path.clone();
            let download_id_clone = download_id.clone();
            
            manager.set_download_status(&download_id, DownloadStatus::Downloading, None);
            
            // Reset progress
            manager.update_download_progress(&download_id, 0, total_size, 0);
            
            // Restart the download
            tokio::spawn(async move {
                if let Err(e) = perform_download(download_id_clone, url, file_path).await {
                    eprintln!("Restart download failed: {}", e);
                }
            });
        }
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
pub fn bulk_resume_downloads(download_ids: Vec<String>) -> Result<(), String> {
    for id in download_ids {
        resume_download(id)?;
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
    let client = reqwest::Client::new();
    match client.head(&url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[command]
pub async fn get_file_size_from_url(url: String) -> Result<u64, String> {
    let client = reqwest::Client::new();
    let response = client.head(&url).send().await
        .map_err(|e| format!("Failed to get file info: {}", e))?;
    
    if let Some(content_length) = response.headers().get("content-length") {
        let size_str = content_length.to_str()
            .map_err(|e| format!("Invalid content-length header: {}", e))?;
        let size = size_str.parse::<u64>()
            .map_err(|e| format!("Failed to parse content-length: {}", e))?;
        Ok(size)
    } else {
        Err("Content-Length header not found".to_string())
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

/// Core download function
async fn perform_download(download_id: String, url: String, file_path: String) -> Result<(), String> {
    let client = {
        let manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| format!("Failed to lock download manager: {}", e))?;
        manager.client.clone()
    };

    // Check if download is still active
    let should_continue = {
        let manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| format!("Failed to lock download manager: {}", e))?;
        manager.downloads.get(&download_id)
            .map(|d| matches!(d.status, DownloadStatus::Downloading))
            .unwrap_or(false)
    };

    if !should_continue {
        return Ok(());
    }

    let response = client.get(&url).send().await
        .map_err(|e| {
            let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
            manager.set_download_status(&download_id, DownloadStatus::Error, Some(e.to_string()));
            format!("Failed to start download: {}", e)
        })?;

    if !response.status().is_success() {
        let error_msg = format!("HTTP error: {}", response.status());
        let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
        manager.set_download_status(&download_id, DownloadStatus::Error, Some(error_msg.clone()));
        return Err(error_msg);
    }

    let total_size = response.content_length().unwrap_or(0);
    
    // Update total size
    {
        let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
        manager.update_download_progress(&download_id, 0, total_size, 0);
    }

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    let mut file = File::create(&file_path).await
        .map_err(|e| {
            let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
            manager.set_download_status(&download_id, DownloadStatus::Error, Some(e.to_string()));
            format!("Failed to create file: {}", e)
        })?;

    let mut downloaded = 0u64;
    let mut last_update = std::time::Instant::now();
    let mut speed = 0u64;
    
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    
    while let Some(chunk_result) = stream.next().await {
        // Check if download should be paused or cancelled
        let should_continue = {
            let manager = DOWNLOAD_MANAGER.lock()
                .map_err(|e| format!("Failed to lock download manager: {}", e))?;
            manager.downloads.get(&download_id)
                .map(|d| matches!(d.status, DownloadStatus::Downloading))
                .unwrap_or(false)
        };

        if !should_continue {
            break;
        }

        let chunk = chunk_result
            .map_err(|e| {
                let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
                manager.set_download_status(&download_id, DownloadStatus::Error, Some(e.to_string()));
                format!("Failed to read chunk: {}", e)
            })?;

        file.write_all(&chunk).await
            .map_err(|e| {
                let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
                manager.set_download_status(&download_id, DownloadStatus::Error, Some(e.to_string()));
                format!("Failed to write to file: {}", e)
            })?;

        downloaded += chunk.len() as u64;

        // Update progress every 500ms
        let now = std::time::Instant::now();
        if now.duration_since(last_update).as_millis() >= 500 {
            let elapsed_secs = now.duration_since(last_update).as_secs_f64();
            if elapsed_secs > 0.0 {
                speed = (chunk.len() as f64 / elapsed_secs) as u64;
            }
            
            let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
            manager.update_download_progress(&download_id, downloaded, total_size, speed);
            last_update = now;
        }
    }

    file.flush().await
        .map_err(|e| format!("Failed to flush file: {}", e))?;

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
            let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
            manager.update_download_progress(&download_id, downloaded, total_size, 0);
            manager.set_download_status(&download_id, DownloadStatus::Completed, None);
        }
        _ => {
            // Download was paused or cancelled
        }
    }

    Ok(())
}