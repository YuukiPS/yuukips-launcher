//! Patch management module
//! Handles game patching, file restoration, and patch verification

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde_json::Number;
use serde::{Deserialize, Serialize};
use tauri::command;

use crate::http::create_http_client;
use crate::utils::{calculate_md5, create_parent_directories};

// Global download progress state
static DOWNLOAD_PROGRESS: once_cell::sync::Lazy<Arc<Mutex<DownloadProgress>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(DownloadProgress::default())));

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DownloadProgress {
    pub total_size: u64,
    pub downloaded: u64,
    pub percentage: f64,
    pub status: String,
    pub current_file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatchResponse {
    pub patch: bool,
    pub proxy: bool,
    pub message: String,
    pub metode: u32,
    #[serde(default)]
    pub patched: Vec<PatchFile>,
    #[serde(default)]
    pub original: Vec<PatchFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatchFile {
    pub location: String, // location file in game folder, so use path_folder+location
    pub md5: String, // for check match file 
    pub file: String, // file url to download
}

/// Get current download progress
#[command]
pub fn get_download_progress() -> Result<String, String> {
    let progress = DOWNLOAD_PROGRESS.lock()
        .map_err(|e| format!("Failed to lock download progress: {}", e))?;
    
    serde_json::to_string(&*progress)
        .map_err(|e| format!("Failed to serialize download progress: {}", e))
}

/// Clear download progress
#[command]
pub fn clear_download_progress() -> Result<String, String> {
    let mut progress = DOWNLOAD_PROGRESS.lock()
        .map_err(|e| format!("Failed to lock download progress: {}", e))?;
    
    *progress = DownloadProgress::default();
    Ok("Download progress cleared".to_string())
}

/// Check patch status for a game
#[command]
pub fn check_patch_status(
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        match fetch_patch_info(game_id, version, channel, md5).await {
            Ok(patch_response) => {
                serde_json::to_string(&patch_response)
                    .map_err(|e| format!("Failed to serialize patch response: {}", e))
            }
            Err(e) => Err(format!("Failed to fetch patch info: {}", e))
        }
    })
}

/// Restore game files to original state
#[command]
pub fn restore_game_files(
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
    game_folder_path: String,
) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        // First, fetch patch info to get original file URLs
        let patch_response = fetch_patch_info(game_id, version, channel, md5).await
            .map_err(|e| format!("Failed to fetch patch info for restoration: {}", e))?;
        
        restore_original_files(&patch_response, &game_folder_path)
    })
}

/// Check and apply patches if needed
pub fn check_and_apply_patches(
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
    game_folder_path: String,
) -> Result<(String, Option<PatchResponse>, Vec<String>), String> {
    // Ensure proxy is stopped before patching
    if crate::proxy::is_proxy_running() {
        println!("🔧 Stopping proxy before applying patches...");
        crate::proxy::stop_proxy()
            .map_err(|e| format!("Failed to stop proxy before patching: {}", e))?;
    }
    
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        // Fetch patch information
        let patch_response = fetch_patch_info(game_id.clone(), version, channel, md5).await
            .map_err(|e| format!("Failed to fetch patch info: {}", e))?;
        
        // Check if game is running and try to kill it if needed
        if crate::game::check_game_running_internal(&game_id)? {
            println!("🎮 Game is running, attempting to close it for patching...");
            match crate::game::kill_game_processes(&game_id) {
                Ok(message) => println!("🔪 {}", message),
                Err(e) => return Err(format!("Cannot patch while game is running. Failed to close game: {}", e)),
            }
            
            // Wait a moment for the game to fully close
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            
            // Verify game is actually closed
            if crate::game::check_game_running_internal(&game_id)? {
                return Err("Cannot patch: Game is still running after close attempt. Please close the game manually.".to_string());
            }
        }
        
        // Apply patches based on method
        let patched_files = match patch_response.metode {
            0 => {
                // Method 0: No patching needed
                println!("✅ No patches needed for this game version");
                Vec::new()
            }
            1 => {
                // Method 1: Apply file patches
                apply_file_patches(&patch_response, &game_folder_path).await?
            }
            _ => {
                return Err(format!("Unsupported patch method: {}", patch_response.metode));
            }
        };
        
        let message = if patched_files.is_empty() {
            "No patches applied".to_string()
        } else {
            format!("Applied {} patches successfully", patched_files.len())
        };
        
        Ok((message, Some(patch_response), patched_files))
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatchErrorInfo {
    pub game_id: String,
    pub version: String,
    pub channel: String,
    pub md5: String,
    pub url: String,
    pub status_code: u16,
    pub error_type: String,
}

/// Fetch patch information from API
async fn fetch_patch_info(
    game_id: Number,
    version: String,
    channel: Number,
    md5: String,
) -> Result<PatchResponse, String> {
    let client = create_http_client(false)?;
    
    let url = format!(
        "https://ps.yuuki.me/game/patch/{}/{}/{}/{}.json",
        game_id, version, channel, md5
    );
    
    println!("🔍 Checking for patches: {}", url);
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch patch info: {}", e))?;
    
    if !response.status().is_success() {
        let status_code = response.status().as_u16();
        
        // For 404 errors, create detailed error info for frontend popup
        if status_code == 404 {
            let error_info = PatchErrorInfo {
                game_id: game_id.to_string(),
                version: version.clone(),
                channel: channel.to_string(),
                md5: md5.clone(),
                url: url.clone(),
                status_code,
                error_type: "PATCH_NOT_FOUND".to_string(),
            };
            
            let error_json = serde_json::to_string(&error_info)
                .unwrap_or_else(|_| "Failed to serialize error info".to_string());
            
            return Err(format!("PATCH_ERROR_404:{}", error_json));
        }
        
        return Err(format!("Patch API returned error: {}", response.status()));
    }
    
    let patch_response: PatchResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse patch response: {}", e))?;
    
    println!("📦 Patch info received: method={}, proxy={}, files={}", 
             patch_response.metode, patch_response.proxy, patch_response.patched.len());
    
    Ok(patch_response)
}

/// Apply file patches
async fn apply_file_patches(
    patch_response: &PatchResponse,
    game_folder_path: &str,
) -> Result<Vec<String>, String> {
    let mut patched_files = Vec::new();
    let cache_dir = Path::new(game_folder_path).join(".patch_cache");
    
    // Create cache directory if it doesn't exist
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create patch cache directory: {}", e))?;
    }
    
    for (index, patch_file) in patch_response.patched.iter().enumerate() {
        let file_path = Path::new(game_folder_path).join(&patch_file.location);
        let cache_file_path = cache_dir.join(format!("{}.patch", patch_file.location.replace(['/', '\\'], "_")));
        
        println!("🔧 Processing patch {}/{}: {}", index + 1, patch_response.patched.len(), patch_file.location);
        
        // Check if we have a cached version with matching MD5
        let mut use_cached = false;
        if cache_file_path.exists() {
            match calculate_md5(&cache_file_path) {
                Ok(cached_md5) if cached_md5.to_uppercase() == patch_file.md5.to_uppercase() => {
                    println!("📦 Using cached patch for: {}", patch_file.location);
                    use_cached = true;
                }
                Ok(_) => {
                    println!("🔄 Cache MD5 mismatch for {}, expected {} but got {}", 
                        patch_file.location, patch_file.md5.to_uppercase(), calculate_md5(&cache_file_path).unwrap_or_default());
                }
                Err(e) => {
                    println!("⚠️ Failed to verify cached file {}: {}", patch_file.location, e);
                }
            }
        }
        
        if !use_cached {
            // Download the patch file
            download_and_verify_file(&patch_file.file, &cache_file_path, &patch_file.md5.to_uppercase()).await
                .map_err(|e| format!("Failed to download patch for {}: {}", patch_file.location, e))?;
        }
        
        // Create backup of original file if it exists
        if file_path.exists() {
            let backup_path = file_path.with_extension(format!("{}.backup", 
                file_path.extension().and_then(|s| s.to_str()).unwrap_or("")));
            
            if !backup_path.exists() {
                fs::copy(&file_path, &backup_path)
                    .map_err(|e| format!("Failed to create backup for {}: {}", patch_file.location, e))?;
                println!("💾 Created backup: {}", backup_path.display());
            }
        }
        
        // Apply the patch (copy from cache to game folder)
        create_parent_directories(&file_path)?;
        fs::copy(&cache_file_path, &file_path)
            .map_err(|e| format!("Failed to apply patch for {}: {}", patch_file.location, e))?;
        
        patched_files.push(patch_file.location.clone());
        println!("✅ Applied patch: {}", patch_file.location);
    }
    
    if !patched_files.is_empty() {
        println!("🎉 Successfully applied {} patches", patched_files.len());
    }
    
    Ok(patched_files)
}

/// Download and verify a file
async fn download_and_verify_file(
    url: &str,
    file_path: &Path,
    expected_md5: &str,
) -> Result<(), String> {
    let client = create_http_client(false)?;
    
    // Update progress
    {
        let mut progress = DOWNLOAD_PROGRESS.lock()
            .map_err(|e| format!("Failed to lock download progress: {}", e))?;
        progress.current_file = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
        progress.status = "downloading".to_string();
        progress.downloaded = 0;
        progress.percentage = 0.0;
    }
    
    println!("⬇️ Downloading: {} -> {}", url, file_path.display());
    
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    
    // Update total size
    {
        let mut progress = DOWNLOAD_PROGRESS.lock()
            .map_err(|e| format!("Failed to lock download progress: {}", e))?;
        progress.total_size = total_size;
    }
    
    // Create parent directories
    create_parent_directories(file_path)?;
    
    // Download the entire response
    let file_contents = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let downloaded = file_contents.len() as u64;
    
    // Update progress to verifying
    {
        if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
            progress.downloaded = downloaded;
            progress.total_size = total_size;
            progress.percentage = 100.0;
            progress.status = "verifying".to_string();
            progress.current_file = file_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();
        }
    }
    
    // Write to file
    fs::write(file_path, &file_contents)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    // Verify MD5
    let actual_md5 = format!("{:x}", md5::compute(&file_contents));
    if actual_md5.to_uppercase() != expected_md5.to_uppercase() {
        // Update progress with failed status
        if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
            progress.downloaded = downloaded;
            progress.total_size = total_size;
            progress.percentage = 100.0;
            progress.status = "failed".to_string();
            progress.current_file = file_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();
        }
        return Err(format!(
            "MD5 mismatch for {}: expected {}, got {}",
            file_path.display(),
            expected_md5,
            actual_md5
        ));
    }
    
    // Update progress to completed
    {
        if let Ok(mut progress) = DOWNLOAD_PROGRESS.lock() {
            progress.downloaded = downloaded;
            progress.total_size = total_size;
            progress.percentage = 100.0;
            progress.status = "completed".to_string();
            progress.current_file = file_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();
        }
    }
    
    println!("✅ Download verified: {}", file_path.display());
    Ok(())
}

/// Restore original files by downloading them
pub fn restore_original_files(
    patch_response: &PatchResponse,
    game_folder_path: &str,
) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let mut restored_files = Vec::new();
        
        for patch_file in &patch_response.patched {
            let file_path = Path::new(game_folder_path).join(&patch_file.file);
            
            // Save current file as .patch if it exists
            if file_path.exists() {
                let patch_backup_path = file_path.with_extension(format!("{}.patch", 
                    file_path.extension().and_then(|s| s.to_str()).unwrap_or("")));
                
                fs::copy(&file_path, &patch_backup_path)
                    .map_err(|e| format!("Failed to backup patched file {}: {}", patch_file.file, e))?;
            }
            
            // Download original file (assuming the API provides original file URLs)
            // This would need to be implemented based on your API structure
            // For now, we'll try to restore from backup
            let backup_path = file_path.with_extension(format!("{}.backup", 
                file_path.extension().and_then(|s| s.to_str()).unwrap_or("")));
            
            if backup_path.exists() {
                fs::copy(&backup_path, &file_path)
                    .map_err(|e| format!("Failed to restore from backup {}: {}", patch_file.file, e))?;
                
                // Remove backup after successful restoration
                fs::remove_file(&backup_path)
                    .map_err(|e| format!("Failed to remove backup {}: {}", backup_path.display(), e))?;
                
                restored_files.push(patch_file.file.clone());
            }
        }
        
        if restored_files.is_empty() {
            Ok("No files were restored (no backups found)".to_string())
        } else {
            Ok(format!("Restored {} files from backups", restored_files.len()))
        }
    })
}

/// Restore files from .backup files
pub fn restore_from_backups(
    game_folder_path: &str,
    patched_files: &[String],
) -> Result<String, String> {
    let mut restored_files = Vec::new();
    
    for file_name in patched_files {
        let file_path = Path::new(game_folder_path).join(file_name);
        let backup_path = file_path.with_extension(format!("{}.backup", 
            file_path.extension().and_then(|s| s.to_str()).unwrap_or("")));
        
        if backup_path.exists() {
            // Save current file as .patch before restoring
            if file_path.exists() {
                let patch_backup_path = file_path.with_extension(format!("{}.patch", 
                    file_path.extension().and_then(|s| s.to_str()).unwrap_or("")));
                
                fs::copy(&file_path, &patch_backup_path)
                    .map_err(|e| format!("Failed to backup current file {}: {}", file_name, e))?;
            }
            
            // Restore from backup
            fs::copy(&backup_path, &file_path)
                .map_err(|e| format!("Failed to restore from backup {}: {}", file_name, e))?;
            
            // Remove backup after successful restoration
            fs::remove_file(&backup_path)
                .map_err(|e| format!("Failed to remove backup {}: {}", backup_path.display(), e))?;
            
            restored_files.push(file_name.clone());
        }
    }
    
    if restored_files.is_empty() {
        Ok("No backup files found to restore".to_string())
    } else {
        Ok(format!("Restored {} files from backups", restored_files.len()))
    }
}

/// Clean up remaining patch files
pub fn cleanup_remaining_patches(
    game_folder_path: &str,
    patched_files: &[String],
) -> Result<String, String> {
    let mut cleaned_files = Vec::new();
    
    for file_name in patched_files {
        let file_path = Path::new(game_folder_path).join(file_name);
        
        if file_path.exists() {
            let patch_path = file_path.with_extension(format!("{}.patch", 
                file_path.extension().and_then(|s| s.to_str()).unwrap_or("")));
            
            // If a .patch file already exists, remove it first
            if patch_path.exists() {
                fs::remove_file(&patch_path)
                    .map_err(|e| format!("Failed to remove existing patch file {}: {}", patch_path.display(), e))?;
            }
            
            // Rename current file to .patch
            fs::rename(&file_path, &patch_path)
                .map_err(|e| format!("Failed to rename {} to patch: {}", file_name, e))?;
            
            cleaned_files.push(file_name.clone());
        }
    }
    
    if cleaned_files.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("Cleaned up {} remaining patch files", cleaned_files.len()))
    }
}