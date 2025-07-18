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
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncWriteExt;
use chrono::Utc;
use sha2::{Sha256, Digest};
use crate::settings::AppSettings;
use crate::system::get_yuukips_data_path;

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
    #[serde(rename = "userPaused", default)]
    pub user_paused: bool, // Track if pause was initiated by user
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
    Queued,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActivityEntry {
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "actionType")]
    pub action_type: ActivityType,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    pub identifier: Option<String>,
    pub status: Option<String>,
    pub details: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    DownloadStarted,
    DownloadPaused,
    DownloadResumed,
    DownloadCancelled,
    DownloadCompleted,
    DownloadError,
    FileAdded,
    StatusChanged,
    UserInteraction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadState {
    pub downloads: HashMap<String, DownloadItem>,
    pub history: Vec<DownloadHistory>,
    pub activities: Vec<ActivityEntry>,
    pub download_directory: String,
    pub version: u32,
    pub timestamp: u64,
    pub checksum: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartialDownloadInfo {
    pub id: String,
    pub url: String,
    pub file_path: String,
    pub downloaded_size: u64,
    pub total_size: u64,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
    pub resume_supported: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StateBackup {
    pub primary: DownloadState,
    pub backup: Option<DownloadState>,
    pub corruption_count: u32,
}

struct DownloadManager {
    downloads: HashMap<String, DownloadItem>,
    history: Vec<DownloadHistory>,
    activities: Vec<ActivityEntry>,
    download_directory: PathBuf,
    cancellation_tokens: HashMap<String, Arc<AtomicBool>>,
    partial_downloads: HashMap<String, PartialDownloadInfo>,
    auto_save_enabled: bool,
    last_save_time: SystemTime,
    state_version: u32,
}

impl DownloadManager {
    fn new() -> Self {
        log::info!("Initializing DownloadManager");
        let download_directory = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("./downloads"));
        
        let mut manager = Self {
            downloads: HashMap::new(),
            history: Vec::new(),
            activities: Vec::new(),
            download_directory,
            cancellation_tokens: HashMap::new(),
            partial_downloads: HashMap::new(),
            auto_save_enabled: true,
            last_save_time: SystemTime::now(),
            state_version: 1,
        };
        
        // Load persisted state (includes activities, downloads, and history)
        match manager.load_state() {
            Ok(_) => {
                log::info!("Successfully loaded download state");
            },
            Err(e) => {
                log::error!("Failed to load state: {}", e);
                // Fallback to loading just activities for backward compatibility
                if let Err(e) = manager.load_activities() {
                    log::error!("Failed to load activities: {}", e);
                }
            }
        }
        
        // Resume interrupted downloads
        if let Err(e) = manager.resume_interrupted_downloads() {
            log::error!("Failed to resume interrupted downloads: {}", e);
        }
        
        // Ensure state file exists
        if let Err(e) = manager.save_state() {
            log::error!("Failed to save initial state: {}", e);
        } else {
            log::info!("Initial state saved successfully");
        }
        
        manager
    }
    
    fn add_activity(&mut self, action_type: ActivityType, file_name: Option<String>, identifier: Option<String>, status: Option<String>, details: Option<String>) {
        let activity = ActivityEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            action_type,
            file_name,
            identifier,
            status,
            details,
        };
        
        self.activities.push(activity);
        
        // Persist activities immediately
        if let Err(e) = self.save_activities() {
            log::error!("Failed to save activities: {}", e);
        }
    }
    
    fn get_activities_file_path() -> PathBuf {
        let yuukips_dir = get_yuukips_data_path()
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(yuukips_dir).join("activities.json")
    }
    
    fn save_activities(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_activities_file_path();
        
        // Create directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(&self.activities)?;
        fs::write(file_path, json)?;
        Ok(())
    }
    
    fn load_activities(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_activities_file_path();
        
        if file_path.exists() {
            let json = fs::read_to_string(file_path)?;
            self.activities = serde_json::from_str(&json)?;
        }
        
        Ok(())
    }
    
    fn clear_activities(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.activities.clear();
        self.save_activities()?;
        self.auto_save_state()?;
        Ok(())
    }
    
    // State persistence methods
    fn get_state_file_path() -> PathBuf {
        let yuukips_dir = get_yuukips_data_path()
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(yuukips_dir).join("download_state.json")
    }
    
    fn get_backup_state_file_path() -> PathBuf {
        let yuukips_dir = get_yuukips_data_path()
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(yuukips_dir).join("download_state_backup.json")
    }
    
    fn calculate_state_checksum(state: &DownloadState) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_string(state)?;
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    fn create_download_state(&self) -> Result<DownloadState, Box<dyn std::error::Error>> {
        let mut state = DownloadState {
            downloads: self.downloads.clone(),
            history: self.history.clone(),
            activities: self.activities.clone(),
            download_directory: self.download_directory.to_string_lossy().to_string(),
            version: self.state_version,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            checksum: String::new(),
        };
        
        state.checksum = Self::calculate_state_checksum(&state)?;
        Ok(state)
    }
    
    fn save_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.create_download_state()?;
        let file_path = Self::get_state_file_path();
        let backup_path = Self::get_backup_state_file_path();
        
        // Create directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Create backup of current state if it exists
        if file_path.exists() {
            if let Err(e) = fs::copy(&file_path, &backup_path) {
                log::warn!("Warning: Failed to create backup: {}", e);
            }
        }
        
        // Write new state
        let json = serde_json::to_string_pretty(&state)?;
        fs::write(&file_path, json)?;
        
        self.last_save_time = SystemTime::now();
        
        // Update partial download info for active downloads
        // Update partial download info for all active downloads
        let active_downloads: Vec<_> = self.downloads.iter()
            .filter(|(_, download)| matches!(download.status, DownloadStatus::Downloading | DownloadStatus::Paused))
            .map(|(id, download)| (id.clone(), download.downloaded_size, download.total_size))
            .collect();
        
        for (id, downloaded, total) in active_downloads {
            self.update_partial_download_info(&id, downloaded, total);
        }
        
        Ok(())
    }
    
    fn load_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_state_file_path();
        let backup_path = Self::get_backup_state_file_path();
        
        log::info!("Attempting to load state from: {:?}", file_path);
        log::info!("Backup state file path: {:?}", backup_path);
        
        // Try to load primary state file
        let state = match self.try_load_state_file(&file_path) {
            Ok(state) => {
                log::info!("Successfully loaded primary state file");
                state
            },
            Err(e) => {
                log::error!("Failed to load primary state file: {}", e);
                
                // Try backup file
                match self.try_load_state_file(&backup_path) {
                    Ok(state) => {
                        log::info!("Loaded from backup state file");
                        state
                    }
                    Err(backup_e) => {
                        log::error!("Failed to load backup state file: {}", backup_e);
                        return Err(format!("Both primary and backup state files failed: {} | {}", e, backup_e).into());
                    }
                }
            }
        };
        
        // Apply loaded state
        self.downloads = state.downloads;
        self.history = state.history;
        self.activities = state.activities;
        self.download_directory = PathBuf::from(state.download_directory);
        self.state_version = state.version;
        
        Ok(())
    }
    
    fn try_load_state_file(&self, file_path: &Path) -> Result<DownloadState, Box<dyn std::error::Error>> {
        if !file_path.exists() {
            return Err("State file does not exist".into());
        }
        
        let json = fs::read_to_string(file_path)?;
        
        // Try to deserialize with the current structure first
        match serde_json::from_str::<DownloadState>(&json) {
            Ok(mut state) => {
                // Verify checksum
                let stored_checksum = state.checksum.clone();
                state.checksum = String::new();
                let calculated_checksum = Self::calculate_state_checksum(&state)?;
                
                if stored_checksum == calculated_checksum {
                    state.checksum = stored_checksum;
                    return Ok(state);
                }
                // If checksum doesn't match, fall through to legacy handling
            }
            Err(_) => {
                // If deserialization fails, fall through to legacy handling
            }
        }
        
        // Try to load as legacy format (without speed_limit_mbps)
        #[derive(Deserialize)]
        struct LegacyDownloadState {
            pub downloads: HashMap<String, DownloadItem>,
            pub history: Vec<DownloadHistory>,
            pub activities: Vec<ActivityEntry>,
            pub download_directory: String,
            pub version: u32,
            pub timestamp: u64,
            pub checksum: String,
        }
        
        let legacy_state: LegacyDownloadState = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse state file as legacy format: {}", e))?;
        
        // Convert legacy state to current format
        let state = DownloadState {
            downloads: legacy_state.downloads,
            history: legacy_state.history,
            activities: legacy_state.activities,
            download_directory: legacy_state.download_directory,
            version: legacy_state.version,
            timestamp: legacy_state.timestamp,
            checksum: legacy_state.checksum,

        };
        
        Ok(state)
    }
    
    fn auto_save_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.auto_save_enabled {
            return Ok(());
        }
        
        // Auto-save every 30 seconds or when significant changes occur
        let should_save = self.last_save_time.elapsed()
            .map(|d| d.as_secs() >= 30)
            .unwrap_or(true);
        
        if should_save {
            self.save_state()?;
        }
        
        Ok(())
    }
    
    fn update_partial_download_info(&mut self, id: &str, downloaded: u64, total: u64) {
        if let Some(download) = self.downloads.get(id) {
            if matches!(download.status, DownloadStatus::Downloading | DownloadStatus::Paused) {
                let partial_info = PartialDownloadInfo {
                    id: id.to_string(),
                    url: download.url.clone(),
                    file_path: download.file_path.clone(),
                    downloaded_size: downloaded,
                    total_size: total,
                    last_modified: None, // Will be populated during download
                    etag: None, // Will be populated during download
                    resume_supported: true, // Assume true, will be verified during resume
                };
                
                self.partial_downloads.insert(id.to_string(), partial_info);
            }
        }
    }
    
    fn resume_interrupted_downloads(&mut self) -> Result<Vec<String>, String> {
        // Debug: Log all downloads and their status
        log::debug!("=== Resume Interrupted Downloads Debug ===");
        log::debug!("Total downloads in manager: {}", self.downloads.len());
        for (id, download) in &self.downloads {
            log::debug!("Download {}: status={:?}, downloaded={}, total={}, user_paused={}", 
                id, download.status, download.downloaded_size, download.total_size, download.user_paused);
        }
        
        let interrupted_downloads: Vec<_> = self.downloads
            .iter()
            .filter(|(_, download)| {
                let is_interrupted = matches!(download.status, DownloadStatus::Downloading) && 
                    download.downloaded_size > 0 && 
                    download.downloaded_size < download.total_size;
                log::debug!("Checking download for interruption: status={:?}, downloaded={}, total={}, is_interrupted={}", 
                    download.status, download.downloaded_size, download.total_size, is_interrupted);
                is_interrupted
            })
            .map(|(id, download)| (id.clone(), download.clone()))
            .collect();
        
        let paused_downloads: Vec<_> = self.downloads
            .iter()
            .filter(|(_, download)| {
                let is_auto_resumable = matches!(download.status, DownloadStatus::Paused) && 
                    !download.user_paused;
                log::debug!("Checking download for auto-resume: status={:?}, user_paused={}, is_auto_resumable={}", 
                    download.status, download.user_paused, is_auto_resumable);
                is_auto_resumable
            })
            .map(|(id, download)| (id.clone(), download.clone()))
            .collect();
        
        log::debug!("Found {} interrupted downloads and {} paused downloads", 
            interrupted_downloads.len(), paused_downloads.len());
        
        let mut resumed_ids = Vec::new();
        
        // Handle interrupted downloads (set to paused and mark for auto-resume)
        for (id, download) in interrupted_downloads {
            // Mark as not user-paused since this was an interruption, not user action
            if let Some(download_mut) = self.downloads.get_mut(&id) {
                download_mut.user_paused = false;
            }
            
            // Set status to paused initially
            self.set_download_status_no_cleanup(&id, DownloadStatus::Paused, None);
            
            // Add activity for interrupted download detection
            self.add_activity(
                ActivityType::StatusChanged,
                Some(download.file_name.clone()),
                Some(id.clone()),
                Some("paused".to_string()),
                Some("Download was interrupted and will be auto-resumed.".to_string())
            );
            
            log::info!("Detected interrupted download: {} ({} bytes downloaded)", download.file_name, download.downloaded_size);
            resumed_ids.push(id.clone());
            
            // Mark for auto-resume but don't immediately start downloading
            // The actual resumption will be handled by the frontend when it calls resume_download
            log::info!("Marked interrupted download for auto-resume: {} ({} bytes downloaded)", download.file_name, download.downloaded_size);
        }
        
        // Mark paused downloads that were not manually paused for auto-resume
        for (id, download) in paused_downloads {
            // Reset user_paused flag but keep status as paused
            // The frontend will handle the actual resumption
            if let Some(download_mut) = self.downloads.get_mut(&id) {
                download_mut.user_paused = false;
            }
            
            log::info!("Marked paused download for auto-resume: {} ({} bytes downloaded)", download.file_name, download.downloaded_size);
            
            resumed_ids.push(id);
        }
        
        Ok(resumed_ids)
    }

    fn add_download(&mut self, url: String, file_path: String, file_name: Option<String>) -> String {
        let id = Uuid::new_v4().to_string();
        
        // Clean the URL by trimming whitespace and removing trailing commas/semicolons
        let cleaned_url = url.trim().trim_end_matches(',').trim_end_matches(';').to_string();
        log::debug!("[Rust] Cleaned URL from '{}' to '{}'", url, cleaned_url);
        
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

        // Check if we've reached the max simultaneous downloads limit
        let downloading_count = self.count_downloading_only();
        let max_downloads = crate::settings::get_app_max_simultaneous_downloads().unwrap_or(3);
        let initial_status = if downloading_count >= max_downloads {
            DownloadStatus::Queued
        } else {
            DownloadStatus::Downloading
        };

        let download = DownloadItem {
            id: id.clone(),
            file_name: actual_file_name.clone(),
            file_extension,
            total_size: 0,
            downloaded_size: 0,
            progress: 0.0,
            speed: 0,
            status: initial_status.clone(),
            time_remaining: 0,
            url: cleaned_url.clone(),
            file_path: file_path.clone(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            end_time: None,
            error_message: None,
            user_paused: false,
        };

        // Add activity entry for file addition
        self.add_activity(
            ActivityType::FileAdded,
            Some(actual_file_name.clone()),
            Some(id.clone()),
            None,
            Some(format!("Added download from URL: {}", cleaned_url))
        );
        
        // Add activity entry for download start or queue
        if matches!(initial_status, DownloadStatus::Downloading) {
            self.add_activity(
                ActivityType::DownloadStarted,
                Some(actual_file_name),
                Some(id.clone()),
                Some("downloading".to_string()),
                Some(format!("Download started for file: {}", file_path))
            );
        } else {
            self.add_activity(
                ActivityType::StatusChanged,
                Some(actual_file_name),
                Some(id.clone()),
                Some("queued".to_string()),
                Some(format!("Download queued (max {} simultaneous downloads reached): {}", max_downloads, file_path))
            );
        }

        self.downloads.insert(id.clone(), download);
        
        // Auto-save state after adding download
        if let Err(e) = self.auto_save_state() {
            log::error!("Failed to auto-save state: {}", e);
        }
        
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
            
            // Update partial download info
            self.update_partial_download_info(id, downloaded, total);
            
            // Auto-save state periodically during progress updates
            if let Err(e) = self.auto_save_state() {
                log::error!("Failed to auto-save state during progress update: {}", e);
            }
        }
    }

    fn set_download_status(&mut self, id: &str, status: DownloadStatus, error_message: Option<String>) {
        if let Some(download) = self.downloads.get_mut(id) {
            let _old_status = download.status.clone();
            download.status = status.clone();
            download.error_message = error_message.clone();
            
            // Activity logging will be handled after releasing the borrow
            
            // Release the mutable borrow before calling add_activity
        }
        
        // Now we can safely call add_activity without borrowing conflicts
        if self.downloads.contains_key(id) {
            let download = &self.downloads[id];
            let file_name = download.file_name.clone();
            let old_status = download.status.clone();
            
            let activity_type = match status {
                DownloadStatus::Completed => ActivityType::DownloadCompleted,
                DownloadStatus::Error => ActivityType::DownloadError,
                DownloadStatus::Cancelled => ActivityType::DownloadCancelled,
                DownloadStatus::Paused => ActivityType::DownloadPaused,
                DownloadStatus::Queued => ActivityType::StatusChanged,
                DownloadStatus::Downloading => {
                    if matches!(old_status, DownloadStatus::Paused) {
                        ActivityType::DownloadResumed
                    } else {
                        ActivityType::StatusChanged
                    }
                }
            };
            
            let status_str = match status {
                DownloadStatus::Completed => "completed".to_string(),
                DownloadStatus::Error => "error".to_string(),
                DownloadStatus::Cancelled => "cancelled".to_string(),
                DownloadStatus::Paused => "paused".to_string(),
                DownloadStatus::Downloading => "downloading".to_string(),
                DownloadStatus::Queued => "queued".to_string(),
            };
            
            let details = if let Some(ref err_msg) = error_message {
                Some(format!("Status changed to {} - {}", status_str, err_msg))
            } else {
                Some(format!("Status changed to {}", status_str))
            };
            
            self.add_activity(
                activity_type,
                Some(file_name),
                Some(id.to_string()),
                Some(status_str),
                details
            );
        }
        
        // Handle final state cleanup and queue management
        if let Some(download) = self.downloads.get_mut(id) {
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
                
                // Remove from partial downloads when completed/cancelled/error
                self.partial_downloads.remove(id);
                
                // Start next queued download if a download slot is now available
                self.start_next_queued_download();
            }
        }
        
        // Auto-save state after status change
        if let Err(e) = self.auto_save_state() {
            log::error!("Failed to auto-save state after status change: {}", e);
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
    
    fn count_downloading_only(&self) -> u32 {
        self.downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Downloading))
            .count() as u32
    }
    
    fn start_next_queued_download(&mut self) {
        // Check if we have capacity for more downloads
        let mut downloading_count = self.count_downloading_only();
        let max_downloads = crate::settings::get_app_max_simultaneous_downloads().unwrap_or(3);
        
        // Start multiple downloads to fill available slots
        while downloading_count < max_downloads {
            // Find the oldest queued download
            let next_queued = self.downloads.iter()
                .filter(|(_, download)| matches!(download.status, DownloadStatus::Queued))
                .min_by_key(|(_, download)| download.start_time)
                .map(|(id, _)| id.clone());
            
            if let Some(download_id) = next_queued {
                if let Some(download) = self.downloads.get(&download_id) {
                    let url = download.url.clone();
                    let file_path = download.file_path.clone();
                    
                    // Update status to downloading
                    self.set_download_status_no_cleanup(&download_id, DownloadStatus::Downloading, None);
                    
                    // Start the download in a background task
                    let download_id_clone = download_id.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = perform_download(download_id_clone.clone(), url, file_path).await {
                            log::error!("[Rust] Queued download failed for ID {}: {}", download_id_clone, e);
                            // Update status to error
                            if let Ok(mut manager) = DOWNLOAD_MANAGER.lock() {
                                manager.set_download_status(&download_id_clone, DownloadStatus::Error, Some(e));
                            }
                        }
                    });
                    
                    log::info!("[Rust] Started queued download with ID: {}", download_id);
                    downloading_count += 1; // Update our local count
                } else {
                    break; // Download not found, stop trying
                }
            } else {
                break; // No more queued downloads
            }
        }
    }
    
    fn enforce_download_limit(&mut self) {
        let max_downloads = crate::settings::get_app_max_simultaneous_downloads().unwrap_or(3);
        let downloading_only: Vec<_> = self.downloads.iter()
            .filter(|(_, download)| matches!(download.status, DownloadStatus::Downloading))
            .map(|(id, download)| (id.clone(), download.start_time))
            .collect();
        
        if downloading_only.len() > max_downloads as usize {
            // Sort by start time to queue the newest downloads first
            let mut sorted_downloads = downloading_only;
            sorted_downloads.sort_by_key(|(_, start_time)| *start_time);
            
            // Queue the excess downloads (newest ones)
            let excess_count = sorted_downloads.len() - max_downloads as usize;
            for (download_id, _) in sorted_downloads.iter().rev().take(excess_count) {
                if let Some(download) = self.downloads.get(download_id) {
                    // Only queue downloads that are currently downloading (not paused by user)
                    if matches!(download.status, DownloadStatus::Downloading) {
                        // Cancel the download task if it's running
                        if let Some(token) = self.cancellation_tokens.get(download_id) {
                            token.store(true, Ordering::Relaxed);
                        }
                        
                        // Set status to queued
                        self.set_download_status_no_cleanup(download_id, DownloadStatus::Queued, None);
                        
                        log::info!("[Rust] Download {} moved to queue due to reduced simultaneous download limit", download_id);
                    }
                }
            }
        }
    }
}

/// Save current download state manually
#[command]
pub fn save_download_state() -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.save_state()
        .map_err(|e| format!("Failed to save state: {}", e))
}

/// Load download state manually
#[command]
pub fn load_download_state() -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.load_state()
        .map_err(|e| format!("Failed to load state: {}", e))
}

/// Resume all interrupted downloads
#[command]
pub fn resume_interrupted_downloads() -> Result<Vec<String>, String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.resume_interrupted_downloads()
        .map_err(|e| format!("Failed to resume interrupted downloads: {}", e))
}

/// Get current state version
#[command]
pub fn get_state_version() -> Result<u32, String> {
    let manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    Ok(manager.state_version)
}

/// Enable or disable auto-save
#[command]
pub fn set_auto_save_enabled(enabled: bool) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.auto_save_enabled = enabled;
    Ok(())
}

/// Get partial download information
#[command]
pub fn get_partial_downloads() -> Result<std::collections::HashMap<String, PartialDownloadInfo>, String> {
    let manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    Ok(manager.partial_downloads.clone())
}

/// Get current download speed limit in MB/s
#[command]
pub fn get_speed_limit() -> Result<f64, String> {
    let settings = AppSettings::load();
    Ok(settings.speed_limit_mbps)
}

/// Set download speed limit in MB/s (0 = unlimited)
#[command]
pub fn set_speed_limit(speed_limit_mbps: f64) -> Result<(), String> {
    let mut settings = AppSettings::load();
    settings.speed_limit_mbps = speed_limit_mbps.max(0.0); // Ensure non-negative
    log::info!("[Rust] Speed limit set to {} MB/s", settings.speed_limit_mbps);
    
    settings.save()
        .map_err(|e| format!("Failed to save settings after setting speed limit: {}", e))?;
    
    Ok(())
}

#[command]
pub fn get_divide_speed_enabled() -> Result<bool, String> {
    let settings = AppSettings::load();
    Ok(settings.divide_speed_enabled)
}

#[command]
pub fn set_divide_speed_enabled(enabled: bool) -> Result<(), String> {
    let mut settings = AppSettings::load();
    settings.divide_speed_enabled = enabled;
    log::info!("[Rust] Divide speed setting set to {}", settings.divide_speed_enabled);
    
    settings.save()
        .map_err(|e| format!("Failed to save settings after setting divide speed: {}", e))?;
    
    Ok(())
}

#[command]
pub fn get_max_simultaneous_downloads() -> Result<u32, String> {
    let settings = AppSettings::load();
    Ok(settings.max_simultaneous_downloads)
}

#[command]
pub fn set_max_simultaneous_downloads(max_downloads: u32) -> Result<(), String> {
    // Validate the input (minimum 1, maximum 64 for reasonable limits)
    if max_downloads < 1 || max_downloads > 64 {
        return Err("Max simultaneous downloads must be between 1 and 64".to_string());
    }
    
    let mut settings = AppSettings::load();
    let old_limit = settings.max_simultaneous_downloads;
    settings.max_simultaneous_downloads = max_downloads;
    log::info!("[Rust] Max simultaneous downloads set to {}", settings.max_simultaneous_downloads);
    
    settings.save()
        .map_err(|e| format!("Failed to save settings after setting max downloads: {}", e))?;
    
    trigger_queue_management_on_settings_change(max_downloads, old_limit)
}

/// Trigger queue management when download limit settings change
/// This function is called from both the download manager and settings module
pub fn trigger_queue_management_on_settings_change(max_downloads: u32, old_limit: u32) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    if max_downloads < old_limit {
        // If we reduced the limit, enforce the new limit by queuing excess downloads
        manager.enforce_download_limit();
    } else if max_downloads > old_limit {
        // If we increased the limit, try to start queued downloads
        manager.start_next_queued_download();
    }
    
    Ok(())
}

/// Get all activity entries
#[command]
pub fn get_activities() -> Result<Vec<ActivityEntry>, String> {
    let manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    // Return activities in reverse chronological order (newest first)
    let mut activities = manager.activities.clone();
    activities.reverse();
    Ok(activities)
}

/// Clear all activity entries
#[command]
pub fn clear_activities() -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.clear_activities()
        .map_err(|e| format!("Failed to clear activities: {}", e))
}

/// Add a user interaction activity entry
#[command]
pub fn add_user_interaction_activity(action: String, details: Option<String>) -> Result<(), String> {
    let mut manager = DOWNLOAD_MANAGER.lock()
        .map_err(|e| format!("Failed to lock download manager: {}", e))?;
    
    manager.add_activity(
        ActivityType::UserInteraction,
        None,
        None,
        None,
        Some(details.unwrap_or(action))
    );
    
    Ok(())
}

/// Start a new download
#[command]
pub async fn start_download(
    url: String,
    file_path: String,
    file_name: Option<String>,
) -> Result<String, String> {
    log::info!("[Rust] Starting new download: url={}, file_path={}, file_name={:?}", url, file_path, file_name);
    
    let (download_id, should_start_immediately) = {
        let mut manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| {
                let error_msg = format!("Failed to lock download manager: {}", e);
                log::error!("[Rust] Error: {}", error_msg);
                error_msg
            })?;
        let id = manager.add_download(url.clone(), file_path.clone(), file_name);
        log::debug!("[Rust] Download added to manager with ID: {}", id);
        
        // Check if the download should start immediately or is queued
        let should_start = if let Some(download) = manager.downloads.get(&id) {
            matches!(download.status, DownloadStatus::Downloading)
        } else {
            false
        };
        
        (id, should_start)
    };

    // Start the download in a background task only if not queued
    if should_start_immediately {
        let download_id_clone = download_id.clone();
        let url_clone = url.clone();
        let file_path_clone = file_path.clone();
        
        log::debug!("[Rust] Spawning background task for download ID: {}", download_id);
        let _spawn_result = tauri::async_runtime::spawn(async move {
            log::debug!("[Rust] Background task started for download ID: {}", download_id_clone);
            if let Err(e) = perform_download(download_id_clone.clone(), url_clone, file_path_clone).await {
                log::error!("[Rust] Download failed for ID {}: {}", download_id_clone, e);
                // Update status to error
                if let Ok(mut manager) = DOWNLOAD_MANAGER.lock() {
                    manager.set_download_status(&download_id_clone, DownloadStatus::Error, Some(e));
                }
            } else {
                log::info!("[Rust] Download completed successfully for ID: {}", download_id_clone);
            }
        });
        log::debug!("[Rust] Background task spawned successfully for download ID: {}, task handle created", download_id);
    } else {
        log::info!("[Rust] Download queued with ID: {} (max simultaneous downloads reached)", download_id);
    }

    log::debug!("[Rust] Download initiation completed, returning ID: {}", download_id);
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
        
        // Set user_paused to true for manual pause
        if let Some(download) = manager.downloads.get_mut(&download_id) {
            download.user_paused = true;
        }
        
        // Set status to paused without cleaning up cancellation token
        manager.set_download_status_no_cleanup(&download_id, DownloadStatus::Paused, None);
        
        // Try to start next queued download since we freed up a slot
        manager.start_next_queued_download();
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
                // Check if we have capacity to resume this download
                let downloading_count = manager.count_downloading_only();
                let max_downloads = crate::settings::get_app_max_simultaneous_downloads().unwrap_or(3);
                
                if downloading_count >= max_downloads {
                    // No capacity, set to queued instead
                    manager.set_download_status_no_cleanup(&download_id, DownloadStatus::Queued, None);
                    return Ok(());
                }
                
                let url = download.url.clone();
                let file_path = download.file_path.clone();
                
                // Reset user_paused flag when manually resuming
                if let Some(download_mut) = manager.downloads.get_mut(&download_id) {
                    download_mut.user_paused = false;
                }
                
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
                log::error!("Resume download failed: {}", e);
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
    
    // Check if this was an active download before cancelling
    let was_downloading = manager.downloads.get(&download_id)
        .map(|d| d.status == DownloadStatus::Downloading)
        .unwrap_or(false);
    
    // Signal cancellation
    if let Some(token) = manager.cancellation_tokens.get(&download_id) {
        token.store(true, Ordering::Relaxed);
    }
    
    manager.set_download_status(&download_id, DownloadStatus::Cancelled, None);
    
    // If we cancelled an active download, try to start next queued download
    if was_downloading {
        manager.start_next_queued_download();
    }
    
    Ok(())
}

/// Cancel a download and delete the partially downloaded file
#[command]
pub fn cancel_and_delete_download(download_id: String) -> Result<(), String> {
    let file_path = {
        let mut manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| format!("Failed to lock download manager: {}", e))?;
        
        // Signal cancellation
        if let Some(token) = manager.cancellation_tokens.get(&download_id) {
            token.store(true, Ordering::Relaxed);
        }
        
        // Get file path and file name before removing from downloads
        let (file_path, file_name) = manager.downloads.get(&download_id)
            .map(|download| (download.file_path.clone(), download.file_name.clone()))
            .unwrap_or_else(|| (String::new(), String::new()));
        
        // Add activity for deletion before removing
        if !file_name.is_empty() {
            manager.add_activity(
                ActivityType::DownloadCancelled,
                Some(file_name),
                Some(download_id.clone()),
                Some("deleted".to_string()),
                Some("Download cancelled and file deleted".to_string())
            );
        }
        
        // Remove from downloads and cancellation tokens
         manager.downloads.remove(&download_id);
         manager.cancellation_tokens.remove(&download_id);
         
         file_path
    };
    
    // Delete the file if it exists
    if !file_path.is_empty() {
        let path = Path::new(&file_path);
        if path.exists() {
            match fs::remove_file(path) {
                Ok(_) => log::info!("Successfully deleted file: {}", file_path),
        Err(e) => log::error!("Failed to delete file {}: {}", file_path, e),
            }
        }
    }
    
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
                log::error!("Restart download failed: {}", e);
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

#[command]
pub fn bulk_cancel_and_delete_downloads(download_ids: Vec<String>) -> Result<(), String> {
    for id in download_ids {
        cancel_and_delete_download(id)?;
    }
    Ok(())
}

/// Utility functions
// URL validation functions removed - downloads now proceed directly

#[command]
pub async fn get_file_size_from_url(url: String) -> Result<u64, String> {
    // Clean the URL by trimming whitespace and removing trailing commas/semicolons
    let cleaned_url = url.trim().trim_end_matches(',').trim_end_matches(';');
    log::debug!("[Rust] Getting file size from URL: {} -> {}", url, cleaned_url);
    
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
                Ok(size)
            } else {
                Err("Content-Length header not found".to_string())
            }
        }
        Err(e) => {
             let error_string = e.to_string();
             // If HEAD request failed, try range request fallback
             log::debug!("[Rust] HEAD request failed, trying range request fallback for file size");
             
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
                     Err("Could not determine file size from range request".to_string())
                 }
                 Err(range_err) => {
                     Err(format!("Both HEAD and range requests failed: HEAD: {}, Range: {}", error_string, range_err))
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
    log::info!("\n=== PERFORM_DOWNLOAD STARTED ===\nID: {}\nURL: {}\nPath: {}\n=== PERFORM_DOWNLOAD STARTED ===", download_id, url, file_path);
    
    // Create cancellation token
    let cancellation_token = Arc::new(AtomicBool::new(false));
    log::debug!("[Rust] Created cancellation token for download ID: {}", download_id);
    
    // Store cancellation token
    {
        let mut manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| {
                let error_msg = format!("Failed to lock download manager: {}", e);
                log::error!("[Rust] Error storing cancellation token: {}", error_msg);
                error_msg
            })?;
        manager.cancellation_tokens.insert(download_id.clone(), cancellation_token.clone());
        log::debug!("[Rust] Stored cancellation token for download ID: {}", download_id);
    }

    // Check if download is still active
    let should_continue = {
        let manager = DOWNLOAD_MANAGER.lock()
            .map_err(|e| {
                let error_msg = format!("Failed to lock download manager: {}", e);
                log::error!("[Rust] Error checking download status: {}", error_msg);
                error_msg
            })?;
        let status = manager.downloads.get(&download_id)
            .map(|d| d.status.clone())
            .unwrap_or(DownloadStatus::Error);
        let should_continue = matches!(status, DownloadStatus::Downloading);
        log::debug!("[Rust] Download status check for ID {}: status={:?}, should_continue={}", download_id, status, should_continue);
        should_continue
    };

    if !should_continue {
        log::info!("[Rust] Download ID {} is not in downloading state, aborting", download_id);
        return Ok(());
    }

    // Try to get file info with HEAD request, but don't fail if it doesn't work
    log::debug!("[Rust] Attempting to get file info with HEAD request for ID: {}", download_id);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    
    let mut total_size = 0u64;
    let max_retries = 3;
    let mut head_request_successful = false;
    
    for attempt in 1..=max_retries {
        log::debug!("[Rust] HEAD request attempt {} of {} for ID: {}", attempt, max_retries, download_id);
        
        match client.head(&url).send().await {
            Ok(resp) => {
                total_size = resp.content_length().unwrap_or(0);
                log::debug!("[Rust] File size determined via HEAD request for ID {}: {} bytes", download_id, total_size);
                head_request_successful = true;
                break;
            }
            Err(e) => {
                log::debug!("[Rust] HEAD request failed on attempt {} for ID {}: {}", attempt, download_id, e);
                if attempt == max_retries {
                    log::debug!("[Rust] HEAD request failed after {} attempts for ID {}. Will proceed with download and determine size during transfer.", max_retries, download_id);
                } else {
                    log::debug!("[Rust] Retrying HEAD request in 2 seconds...");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    
    if !head_request_successful {
        log::debug!("[Rust] Could not determine file size via HEAD request for ID {}. Size will be determined during download.", download_id);
    }
    
    // Update total size
    {
        let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
        manager.update_download_progress(&download_id, 0, total_size, 0);
        log::debug!("[Rust] Updated download progress for ID {}: 0/{} bytes", download_id, total_size);
    }

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(&file_path).parent() {
        log::debug!("[Rust] Creating parent directories for ID {}: {:?}", download_id, parent);
        fs::create_dir_all(parent)
            .map_err(|e| {
                let error_msg = format!("Failed to create directories: {}", e);
                log::error!("[Rust] Error creating directories for ID {}: {}", download_id, error_msg);
                error_msg
            })?;
        log::debug!("[Rust] Parent directories created successfully for ID: {}", download_id);
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
            log::debug!("[Rust] Found existing file for ID {}: {} bytes already downloaded", download_id_clone2, downloaded);
        }
        
        // Get file size with HEAD request
        match client.head(&url_clone).send().await {
            Ok(response) => {
                if let Some(content_length) = response.content_length() {
                    total_size = content_length;
                    log::debug!("[Rust] File size determined from HEAD request for ID {}: {} bytes", download_id_clone2, total_size);
                } else {
                    log::debug!("[Rust] Content-Length not available from HEAD request for ID {}", download_id_clone2);
                }
            }
            Err(e) => {
                log::debug!("[Rust] HEAD request failed for ID {}: {}", download_id_clone2, e);
            }
        }
        
        loop {
             // Check for cancellation
             if cancellation_token_clone2.load(Ordering::Relaxed) {
                 log::info!("[Rust] Download cancellation detected for ID: {}", download_id_clone2);
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
                 log::info!("[Rust] Download should not continue for ID: {}", download_id_clone2);
                 break;
             }
            
            // Check for progress timeout
             if last_progress_time.elapsed() > progress_timeout {
                 let error_msg = format!("Download timeout: No progress for {} seconds", progress_timeout.as_secs());
                 log::error!("[Rust] Download timeout for ID {}: {}", download_id_clone2, error_msg);
                 return Err(error_msg);
             }
             
             // Create range request for resumable download
             let mut request = client.get(&url_clone);
             if downloaded > 0 {
                  let range_header = format!("bytes={}-", downloaded);
                  request = request.header("Range", &range_header);
                  log::debug!("[Rust] Using range request for resume: {} for ID {}", range_header, download_id_clone2);
              } else {
                  log::debug!("[Rust] Starting fresh download for ID {}", download_id_clone2);
              }
            
            match request.send().await {
                Ok(mut response) => {
                     if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                         consecutive_errors += 1;
                         log::error!("[Rust] HTTP error {} for ID {}: attempt {}", response.status(), download_id_clone2, consecutive_errors);
                         
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
                                             log::debug!("[Rust] Total file size from Content-Range for ID {}: {} bytes", download_id_clone2, total_size);
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
                             log::debug!("[Rust] File size determined from GET response for ID {}: {} bytes (downloaded: {})", download_id_clone2, total_size, downloaded);
                         } else {
                             log::debug!("[Rust] Content-Length not available for ID {}. Download size will be unknown.", download_id_clone2);
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
                    
                    // Initialize rate limiting variables
                     let mut rate_limiter_start = std::time::Instant::now();
                     let mut bytes_downloaded_in_window = 0u64;
                     
                    // Download chunks
                     while let Some(chunk_result) = response.chunk().await.transpose() {
                         // Check for cancellation
                         if cancellation_token_clone2.load(Ordering::Relaxed) {
                             log::info!("[Rust] Download cancellation detected during chunk read for ID: {}", download_id_clone2);
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
                             log::info!("[Rust] Download should not continue during chunk read for ID: {}", download_id_clone2);
                             return Ok(());
                         }
                        
                        match chunk_result {
                            Ok(chunk) => {
                                last_progress_time = std::time::Instant::now();
                                
                                use tokio::io::AsyncWriteExt;
                                file.write_all(&chunk).await
                                    .map_err(|e| format!("Failed to write to file: {}", e))?;

                                downloaded += chunk.len() as u64;

                                // Apply speed limiting if enabled using a sliding window approach
                                let settings = AppSettings::load();
                                let (speed_limit, divide_speed_enabled) = (settings.speed_limit_mbps, settings.divide_speed_enabled);
                                
                                let effective_speed_limit = if divide_speed_enabled && speed_limit > 0.0 {
                                    // Count active downloads and divide speed limit among them
                                    let active_downloads_count = {
                                        let manager = DOWNLOAD_MANAGER.lock().unwrap();
                                        manager.downloads.values()
                                            .filter(|d| matches!(d.status, DownloadStatus::Downloading))
                                            .count()
                                    };
                                    
                                    if active_downloads_count > 0 {
                                        speed_limit / active_downloads_count as f64
                                    } else {
                                        speed_limit
                                    }
                                } else {
                                    speed_limit
                                };
                                
                                if effective_speed_limit > 0.0 {
                                    // Convert speed limit from MB/s to bytes/s
                                    let speed_limit_bytes_per_sec = effective_speed_limit * 1024.0 * 1024.0;
                                    
                                    // Add current chunk to the rate limiting window
                                    bytes_downloaded_in_window += chunk.len() as u64;
                                    
                                    // Calculate elapsed time in the current window
                                    let elapsed = rate_limiter_start.elapsed();
                                    let elapsed_secs = elapsed.as_secs_f64();
                                    
                                    // If we have data for at least 100ms, check if we need to throttle
                                    if elapsed_secs >= 0.1 {
                                        let current_rate = bytes_downloaded_in_window as f64 / elapsed_secs;
                                        
                                        // If we're exceeding the speed limit, calculate how long to sleep
                                        if current_rate > speed_limit_bytes_per_sec {
                                            let excess_bytes = bytes_downloaded_in_window as f64 - (speed_limit_bytes_per_sec * elapsed_secs);
                                            let sleep_time_secs = excess_bytes / speed_limit_bytes_per_sec;
                                            let sleep_time_ms = (sleep_time_secs * 1000.0).max(1.0) as u64;
                                            
                                            if sleep_time_ms > 0 {
                                                tokio::time::sleep(std::time::Duration::from_millis(sleep_time_ms)).await;
                                            }
                                        }
                                        
                                        // Reset the rate limiting window
                                        rate_limiter_start = std::time::Instant::now();
                                        bytes_downloaded_in_window = 0;
                                    }
                                }

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
                                     log::info!("[Rust] Download completed: {} bytes for ID: {}", downloaded, download_id_clone2);
                                     file.flush().await
                                         .map_err(|e| format!("Failed to flush file: {}", e))?;
                                     return Ok(());
                                 }
                             }
                             Err(e) => {
                                 consecutive_errors += 1;
                                 log::error!("[Rust] Chunk read error {} for ID {}: {}", consecutive_errors, download_id_clone2, e);
                                
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
                         log::info!("[Rust] Download completed (unknown size): {} bytes for ID: {}", downloaded, download_id_clone2);
                         file.flush().await
                             .map_err(|e| format!("Failed to flush file: {}", e))?;
                         return Ok(());
                     }
                 }
                 Err(e) => {
                     consecutive_errors += 1;
                     log::error!("[Rust] Request error {} for ID {}: {}", consecutive_errors, download_id_clone2, e);
                    
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
                    log::info!("[Rust] Download completed successfully for ID: {}", download_id);
                    let mut manager = DOWNLOAD_MANAGER.lock().unwrap();
                    manager.set_download_status(&download_id, DownloadStatus::Completed, None);
                    log::debug!("[Rust] Download status set to Completed for ID: {}", download_id);
                }
                _ => {
                    // Download was paused or cancelled
                    log::info!("[Rust] Download was paused or cancelled for ID: {}", download_id);
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