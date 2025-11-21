//! Utility functions module
//! Contains common helper functions used across the application

use std::fs;
use std::path::Path;
use std::process::Command;
use serde_json::Number;
use tauri::Emitter;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Create a command with hidden window on Windows
pub fn create_hidden_command(program: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new(program);
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new(program)
    }
}

/// Calculate MD5 hash of a file
pub fn calculate_md5(file_path: &Path) -> Result<String, String> {
    let file_contents = fs::read(file_path)
        .map_err(|e| format!("Failed to read file for MD5 calculation: {}", e))?;
    
    let md5 = md5::compute(&file_contents);
    Ok(format!("{:x}", md5))
}

/// Calculate MD5 hash of a file in chunks (async, non-blocking)
pub async fn calculate_md5_chunked_with_progress(
    file_path: &Path,
    window: tauri::Window,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<String, String> {
    use tokio::fs::File;
    use tokio::io::{AsyncReadExt};
    use tokio::time::{sleep, Duration, timeout, Instant};
    use std::sync::atomic::Ordering;
    
    log::info!("[MD5] calculate_md5_chunked_with_progress starting for: {:?}", file_path);
    
    // Get file size for progress tracking
    let file_size = match std::fs::metadata(file_path) {
        Ok(metadata) => {
            let size = metadata.len();
            log::info!("[MD5] File size: {} bytes ({:.2} MB)", size, size as f64 / (1024.0 * 1024.0));
            size
        },
        Err(e) => {
            log::error!("[MD5] Failed to get file metadata: {}", e);
            return Err(format!("Failed to get file metadata: {}", e));
        }
    };
    
    // Add timeout to prevent indefinite blocking
    log::info!("[MD5] Starting timeout wrapper (5 minutes)");
    let result = timeout(Duration::from_secs(300), async {
        log::info!("[MD5] Opening file: {:?}", file_path);
        let mut file = File::open(file_path).await
            .map_err(|e| {
                log::error!("[MD5] Failed to open file: {}", e);
                format!("Failed to open file for MD5 calculation: {}", e)
            })?;
        
        log::info!("[MD5] File opened successfully, initializing MD5 context");
        let mut context = md5::Context::new();
        // Use larger chunks for better performance while maintaining UI responsiveness
        let chunk_size = if file_size > 500 * 1024 * 1024 { 
            256 * 1024  // 256KB for files > 500MB
        } else if file_size > 100 * 1024 * 1024 {
            128 * 1024  // 128KB for files > 100MB
        } else if file_size > 10 * 1024 * 1024 {
            64 * 1024   // 64KB for files > 10MB
        } else {
            32 * 1024   // 32KB for smaller files
        };
        log::info!("[MD5] Using chunk size: {} KB", chunk_size / 1024);
        let mut buffer = vec![0u8; chunk_size];
        let mut total_read = 0u64;
        let mut chunk_count = 0u64;
        let start_time = Instant::now();
        let mut last_progress_time = start_time;
        
        log::info!("[MD5] Starting chunk processing loop");
        loop {
            // Check for cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                log::info!("[MD5] MD5 calculation cancelled by user");
                let error_msg = "MD5 calculation cancelled by user".to_string();
                let error_data = serde_json::json!({
                    "error": error_msg.clone()
                });
                if let Err(e) = window.emit("md5-error", error_data) {
                    log::warn!("Failed to emit MD5 error event: {}", e);
                }
                return Err(error_msg);
            }
            
            log::debug!("[MD5] Reading chunk {} (total read: {} bytes)", chunk_count, total_read);
            let bytes_read = file.read(&mut buffer).await
                .map_err(|e| {
                    log::error!("[MD5] Failed to read file chunk {}: {}", chunk_count, e);
                    let error_msg = format!("Failed to read file chunk: {}", e);
                    let error_data = serde_json::json!({
                        "error": error_msg.clone()
                    });
                    if let Err(emit_err) = window.emit("md5-error", error_data) {
                        log::warn!("Failed to emit MD5 error event: {}", emit_err);
                    }
                    error_msg
                })?;
            
            log::debug!("[MD5] Chunk {} read: {} bytes", chunk_count, bytes_read);
            
            if bytes_read == 0 {
                log::info!("[MD5] End of file reached after {} chunks", chunk_count);
                break; // End of file
            }
            
            log::debug!("[MD5] Consuming {} bytes into MD5 context", bytes_read);
            context.consume(&buffer[..bytes_read]);
            
            // Use tokio::task::yield_now() for better async cooperation
            tokio::task::yield_now().await;
            total_read += bytes_read as u64;
            chunk_count += 1;
            
            // Emit progress every 1000ms or every 50MB for better performance
            let now = Instant::now();
            let should_emit_progress = now.duration_since(last_progress_time).as_millis() >= 1000
                || total_read % (50 * 1024 * 1024) == 0;
            
            if should_emit_progress {
                let progress = (total_read as f64 / file_size as f64) * 100.0;
                let elapsed_secs = now.duration_since(start_time).as_secs_f64();
                let speed_mbps = if elapsed_secs > 0.0 {
                    (total_read as f64 / (1024.0 * 1024.0)) / elapsed_secs
                } else {
                    0.0
                };
                
                let progress_data = crate::game::Md5Progress {
                    file_path: file_path.to_string_lossy().to_string(),
                    progress,
                    bytes_processed: total_read,
                    total_bytes: file_size,
                    speed_mbps,
                };
                
                if let Err(e) = window.emit("md5-progress", &progress_data) {
                    log::warn!("[MD5] Failed to emit progress event: {}", e);
                }
                
                log::info!("[MD5] Progress: {:.1}% ({} MB / {:.1} MB) - Speed: {:.1} MB/s", 
                    progress, total_read / (1024 * 1024), file_size as f64 / (1024.0 * 1024.0), speed_mbps);
                
                last_progress_time = now;
            }
            
            // Optimized yielding for better performance while maintaining UI responsiveness
            // Yield less frequently and for shorter durations to improve speed
            if file_size > 500 * 1024 * 1024 {
                // For very large files, yield every 5 chunks with minimal sleep
                if chunk_count % 5 == 0 {
                    log::debug!("[MD5] Yielding control (2ms) for very large file - chunk {}", chunk_count);
                    sleep(Duration::from_millis(2)).await;
                }
            } else if file_size > 100 * 1024 * 1024 {
                // For large files, yield every 10 chunks with minimal sleep
                if chunk_count % 10 == 0 {
                    log::debug!("[MD5] Yielding control (1ms) for large file - chunk {}", chunk_count);
                    sleep(Duration::from_millis(1)).await;
                }
            } else if chunk_count % 20 == 0 {
                // For smaller files, yield every 20 chunks
                log::debug!("[MD5] Yielding control (1ms) - every 20 chunks");
                sleep(Duration::from_millis(1)).await;
            }
        }
        
        log::info!("[MD5] Computing final digest");
        let digest = context.compute();
        let hash = format!("{:x}", digest);
        log::info!("[MD5] MD5 calculation completed: {}", hash);
        
        // Emit completion event with hash
        let completion_data = serde_json::json!({
            "hash": hash.clone()
        });
        if let Err(e) = window.emit("md5-complete", completion_data) {
            log::warn!("[MD5] Failed to emit completion event: {}", e);
        }
        
        Ok(hash)
    }).await;
    
    match result {
        Ok(md5_result) => {
            log::info!("[MD5] Timeout wrapper completed successfully");
            md5_result
        },
        Err(_) => {
            log::error!("[MD5] Timeout occurred after 5 minutes");
            let error_msg = format!("MD5 calculation timed out after 5 minutes for file: {:?}", file_path);
            let error_data = serde_json::json!({
                "error": error_msg.clone()
            });
            if let Err(e) = window.emit("md5-error", error_data) {
                log::warn!("Failed to emit MD5 error event: {}", e);
            }
            Err(error_msg)
        }
    }
}

pub async fn calculate_md5_chunked(file_path: &Path) -> Result<String, String> {
    use tokio::fs::File;
    use tokio::io::{AsyncReadExt};
    use tokio::time::{sleep, Duration, timeout};
    
    log::info!("[MD5] calculate_md5_chunked starting for: {:?}", file_path);
    
    // Get file size for progress tracking
    let file_size = match std::fs::metadata(file_path) {
        Ok(metadata) => {
            let size = metadata.len();
            log::info!("[MD5] File size: {} bytes ({:.2} MB)", size, size as f64 / (1024.0 * 1024.0));
            size
        },
        Err(e) => {
            log::info!("[MD5] Failed to get file metadata: {}", e);
            return Err(format!("Failed to get file metadata: {}", e));
        }
    };
    
    // Add timeout to prevent indefinite blocking
    log::info!("[MD5] Starting timeout wrapper (5 minutes)");
    let result = timeout(Duration::from_secs(300), async {
        log::info!("[MD5] Opening file: {:?}", file_path);
        let mut file = File::open(file_path).await
            .map_err(|e| {
                log::info!("[MD5] Failed to open file: {}", e);
                format!("Failed to open file for MD5 calculation: {}", e)
            })?;
        
        log::info!("[MD5] File opened successfully, initializing MD5 context");
        let mut context = md5::Context::new();
        // Use smaller chunks for better UI responsiveness, especially for large files
        let chunk_size = if file_size > 100 * 1024 * 1024 { 
            8 * 1024  // 8KB for files > 100MB
        } else if file_size > 10 * 1024 * 1024 {
            16 * 1024 // 16KB for files > 10MB
        } else {
            32 * 1024 // 32KB for smaller files
        };
        log::info!("[MD5] Using chunk size: {} KB", chunk_size / 1024);
        let mut buffer = vec![0u8; chunk_size];
        let mut total_read = 0u64;
        let mut chunk_count = 0u64;
        
        log::info!("[MD5] Starting chunk processing loop");
        loop {
            log::info!("[MD5] Reading chunk {} (total read: {} bytes)", chunk_count, total_read);
            let bytes_read = file.read(&mut buffer).await
                .map_err(|e| {
                    log::info!("[MD5] Failed to read file chunk {}: {}", chunk_count, e);
                    format!("Failed to read file chunk: {}", e)
                })?;
            
            log::info!("[MD5] Chunk {} read: {} bytes", chunk_count, bytes_read);
            
            if bytes_read == 0 {
                log::info!("[MD5] End of file reached after {} chunks", chunk_count);
                break; // End of file
            }
            
            log::info!("[MD5] Consuming {} bytes into MD5 context", bytes_read);
            context.consume(&buffer[..bytes_read]);
            
            // Use tokio::task::yield_now() for better async cooperation
            tokio::task::yield_now().await;
            total_read += bytes_read as u64;
            chunk_count += 1;
            
            // Log progress every 1MB for better tracking
            if total_read % (1024 * 1024) == 0 {
                let progress = (total_read as f64 / file_size as f64) * 100.0;
                log::info!("[MD5] Progress: {:.1}% ({} MB / {:.1} MB)", 
                    progress, total_read / (1024 * 1024), file_size as f64 / (1024.0 * 1024.0));
            }
            
            // Yield control more aggressively to prevent UI blocking
            // Yield every chunk for files > 50MB, every 10 chunks for smaller files
            if file_size > 50 * 1024 * 1024 {
                // For large files, yield every chunk with longer sleep
                log::info!("[MD5] Yielding control (10ms) for large file - chunk {}", chunk_count);
                sleep(Duration::from_millis(10)).await;
            } else if chunk_count % 10 == 0 {
                // For smaller files, yield every 10 chunks
                log::info!("[MD5] Yielding control (5ms) - every 10 chunks");
                sleep(Duration::from_millis(5)).await;
            }
            
            // Additional yield for very large files (>500MB) every 5 chunks
            if file_size > 500 * 1024 * 1024 && chunk_count % 5 == 0 {
                log::info!("[MD5] Extra yield (15ms) for very large file - chunk {}", chunk_count);
                sleep(Duration::from_millis(15)).await;
            }
        }
        
        log::info!("[MD5] Computing final digest");
        let digest = context.compute();
        let hash = format!("{:x}", digest);
        log::info!("[MD5] MD5 calculation completed: {}", hash);
        Ok(hash)
    }).await;
    
    match result {
        Ok(md5_result) => {
            log::info!("[MD5] Timeout wrapper completed successfully");
            md5_result
        },
        Err(_) => {
            log::info!("[MD5] Timeout occurred after 5 minutes");
            Err(format!("MD5 calculation timed out after 5 minutes for file: {:?}", file_path))
        }
    }
}

/// Create parent directories for a file path if they don't exist
pub fn create_parent_directories(file_path: &Path) -> Result<(), String> {
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directories for {}: {}", file_path.display(), e))?;
        }
    }
    Ok(())
}

// Functions moved to hoyoplay.rs for better organization

/// Get game name from game ID
pub fn get_game_name(game_id: &Number) -> Result<&'static str, String> {
    match game_id.as_u64() {
        Some(1) => Ok("Genshin Impact"),
        Some(2) => Ok("Honkai: Star Rail"),
        Some(3) => Ok("Blue Archive"),
        Some(4) => Ok("Stella Sora"),
        _ => Err(format!("Unsupported game ID: {}", game_id)),
    }
}

/// Check if a file exists and is readable
pub fn is_file_accessible(file_path: &Path) -> bool {
    file_path.exists() && file_path.is_file() && fs::metadata(file_path).is_ok()
}

/// Check if a directory exists and is accessible
pub fn is_directory_accessible(dir_path: &Path) -> bool {
    dir_path.exists() && dir_path.is_dir() && fs::read_dir(dir_path).is_ok()
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Validate game folder path
pub fn validate_game_folder_path(game_id: &Number, game_folder_path: &str) -> Result<(), String> {
    if game_folder_path.is_empty() {
        return Err(format!(
            "Game folder path not set for game {}. Please configure it in game settings.", 
            game_id
        ));
    }
    
    let path = Path::new(game_folder_path);
    if !path.exists() {
        return Err(format!(
            "Game folder not found: {}. Please verify the path in game settings.", 
            game_folder_path
        ));
    }
    
    if !path.is_dir() {
        return Err(format!(
            "Game folder path is not a directory: {}. Please verify the path in game settings.", 
            game_folder_path
        ));
    }
    
    // Note: This function now requires channel_id to check for specific executable
    // For backward compatibility, we'll skip the executable check here
    // The caller should use get_game_executable_names with both game_id and channel_id
    
    Ok(())
}

/// Clean up temporary files in a directory
pub fn cleanup_temp_files(dir_path: &str, pattern: &str) -> Result<usize, String> {
    let path = Path::new(dir_path);
    if !path.exists() {
        return Ok(0);
    }
    
    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory {}: {}", dir_path, e))?;
    
    let mut cleaned_count = 0;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let file_path = entry.path();
        
        if file_path.is_file() {
            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(pattern) {
                    match fs::remove_file(&file_path) {
                        Ok(_) => {
                            cleaned_count += 1;
                            log::info!("🧹 Cleaned up temp file: {}", file_path.display());
                        }
                        Err(e) => {
                            log::error!("⚠️ Failed to remove temp file {}: {}", file_path.display(), e);
                        }
                    }
                }
            }
        }
    }
    
    Ok(cleaned_count)
}

/// Get file extension without the dot
pub fn get_file_extension(file_path: &Path) -> Option<String> {
    file_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Check if a string is a valid URL
pub fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Sanitize filename for safe file operations
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            '/' | '\\' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

/// Convert Windows path separators to forward slashes
pub fn normalize_path_separators(path: &str) -> String {
    path.replace('\\', "/")
}

/// Get the current timestamp as a formatted string
pub fn get_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Parse version string into components
pub fn parse_version(version: &str) -> Result<(u32, u32, u32), String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Invalid version format: {}. Expected format: x.y.z", version));
    }
    
    let major = parts[0].parse::<u32>()
        .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
    let minor = parts[1].parse::<u32>()
        .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
    let patch = parts[2].parse::<u32>()
        .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;
    
    Ok((major, minor, patch))
}

/// Compare two version strings
pub fn compare_versions(version1: &str, version2: &str) -> Result<std::cmp::Ordering, String> {
    let (maj1, min1, pat1) = parse_version(version1)?;
    let (maj2, min2, pat2) = parse_version(version2)?;
    
    Ok((maj1, min1, pat1).cmp(&(maj2, min2, pat2)))
}