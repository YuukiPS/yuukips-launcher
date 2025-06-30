//! Utility functions module
//! Contains common helper functions used across the application

use std::fs;
use std::path::Path;
use serde_json::Number;

/// Calculate MD5 hash of a file
pub fn calculate_md5(file_path: &Path) -> Result<String, String> {
    let file_contents = fs::read(file_path)
        .map_err(|e| format!("Failed to read file for MD5 calculation: {}", e))?;
    
    let md5 = md5::compute(&file_contents);
    Ok(format!("{:x}", md5))
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

/// Get possible executable names for a game ID
pub fn get_game_executable_names(game_id: &Number) -> Result<Vec<&'static str>, String> {
    match game_id.as_u64() {
        Some(1) => Ok(vec!["GenshinImpact.exe"]),
        Some(2) => Ok(vec!["StarRail.exe"]),
        Some(3) => Ok(vec!["ZenlessZoneZero.exe"]),
        Some(4) => Ok(vec!["WutheringWaves.exe"]),
        _ => Err(format!("Unsupported game ID: {}", game_id)),
    }
}

/// Get game name from game ID
pub fn get_game_name(game_id: &Number) -> Result<&'static str, String> {
    match game_id.as_u64() {
        Some(1) => Ok("Genshin Impact"),
        Some(2) => Ok("Honkai: Star Rail"),
        Some(3) => Ok("Zenless Zone Zero"),
        Some(4) => Ok("Wuthering Waves"),
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
    
    // Check if game executable exists
    let game_exe_names = get_game_executable_names(game_id)?;
    let mut found_executable = false;
    
    for exe_name in game_exe_names {
        let exe_path = path.join(exe_name);
        if exe_path.exists() && exe_path.is_file() {
            found_executable = true;
            break;
        }
    }
    
    if !found_executable {
        let game_name = get_game_name(game_id)?;
        return Err(format!(
            "Game executable not found in folder: {}. Please verify that {} is properly installed.", 
            game_folder_path, game_name
        ));
    }
    
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
                            println!("🧹 Cleaned up temp file: {}", file_path.display());
                        }
                        Err(e) => {
                            eprintln!("⚠️ Failed to remove temp file {}: {}", file_path.display(), e);
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