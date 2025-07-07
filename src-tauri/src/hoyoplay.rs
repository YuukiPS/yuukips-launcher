#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;
use serde_json::Number;

/*  TODO
https://github.com/babalae/better-genshin-impact/raw/refs/heads/main/BetterGenshinImpact/Genshin/Paths/RegistryGameLocator.cs
*/

/// Get game executable names based on game_id and channel_id
#[tauri::command]
pub fn get_game_executable_names(game_id: Number, channel_id: Number) -> Result<String, String> {
    match (game_id.as_u64(), channel_id.as_u64()) {
        (Some(1), Some(1)) => Ok("GenshinImpact.exe".to_string()),
        (Some(1), Some(2)) => Ok("YuanShen.exe".to_string()),
        (Some(2), Some(1)) => Ok("StarRail.exe".to_string()),
        (Some(2), Some(2)) => Ok("StarRail.exe".to_string()),
        (Some(3), Some(1)) => Ok("BlueArchive.exe".to_string()),
        _ => Err(format!("Unsupported game ID: {} with channel ID: {}", game_id, channel_id)),
    }
}

/// Get game data folder name for a game ID and channel ID
#[tauri::command]
pub fn get_game_folder(game_id: Number, channel_id: Number) -> Result<String, String> {
    match (game_id.as_u64(), channel_id.as_u64()) {
        (Some(1), Some(1)) => Ok("GenshinImpact_Data".to_string()),
        (Some(1), Some(2)) => Ok("YuanShen_Data".to_string()),
        (Some(2), Some(1)) => Ok("StarRail_Data".to_string()),
        (Some(2), Some(2)) => Ok("StarRail_Data".to_string()),
        (Some(3), Some(1)) => Ok("BlueArchive_Data".to_string()),
        _ => Err(format!("Unsupported game ID: {} with channel ID: {}", game_id, channel_id)),
    }
}



/// Get list of all installed games from HoyoPlay registry
#[tauri::command]
#[cfg(windows)]
pub fn get_hoyoplay_list_game() -> Result<Vec<(String, String)>, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hyp_key = hkcu
        .open_subkey("Software\\Cognosphere\\HYP\\1_0")
        .map_err(|e| format!("Failed to open HYP registry key: {}", e))?;
    
    let mut games = Vec::new();
    
    // Enumerate all subkeys under HYP\1_0
    for subkey_name in hyp_key.enum_keys().map(|x| x.unwrap()) {
        if let Ok(game_key) = hyp_key.open_subkey(&subkey_name) {
            // Try to get GameInstallPath
            if let Ok(install_path) = game_key.get_value::<String, _>("GameInstallPath") {
                // Try to get GameBiz for name code, fallback to subkey name
                let name_code = game_key
                    .get_value::<String, _>("GameBiz")
                    .unwrap_or_else(|_| subkey_name.clone());
                
                games.push((name_code, install_path));
            }
        }
    }
    
    Ok(games)
}

#[cfg(not(windows))]
#[tauri::command]
pub fn get_hoyoplay_list_game() -> Result<Vec<(String, String)>, String> {
    Err("HoyoPlay registry access is only available on Windows".to_string())
}

/// Get game folder path using name_code directly from HoyoPlay registry
#[tauri::command]
#[cfg(windows)]
pub fn get_hoyoplay_game_folder(_name_code: String) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let registry_path = format!("Software\\Cognosphere\\HYP\\1_0\\{}", _name_code);
    
    let game_key = hkcu
        .open_subkey(&registry_path)
        .map_err(|e| format!("Failed to open registry key for {}: {}", _name_code, e))?;
    
    let install_path = game_key
        .get_value::<String, _>("GameInstallPath")
        .map_err(|e| format!("Failed to get GameInstallPath for {}: {}", _name_code, e))?;
    
    Ok(install_path)
}

#[cfg(not(windows))]
#[tauri::command]
pub fn get_hoyoplay_game_folder(_name_code: String) -> Result<String, String> {
    Err("HoyoPlay registry access is only available on Windows".to_string())
}

/// Remove all HoyoPass-related registry entries from miHoYo game folders
#[tauri::command]
#[cfg(windows)]
pub fn remove_all_hoyo_pass() -> Result<Vec<String>, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let mihoyo_key = hkcu
        .open_subkey("Software\\miHoYo")
        .map_err(|e| format!("Failed to open miHoYo registry key: {}", e))?;
    
    let mut deleted_entries = Vec::new();
    
    // Enumerate all subkeys under Software\miHoYo
    for subkey_name in mihoyo_key.enum_keys().map(|x| x.unwrap()) {
        if let Ok(game_key) = mihoyo_key.open_subkey_with_flags(&subkey_name, KEY_ALL_ACCESS) {
            // Get all value names in this game folder
            let value_names: Vec<String> = game_key.enum_values()
                .filter_map(|result| result.ok())
                .map(|(name, _)| name)
                .collect();
            
            // Look for HoyoPass-related entries
            for value_name in value_names {
                let should_delete = value_name.contains("HOYO_ACCOUNTS_MIGRATED_TO_HOYOPASS_PROD_OVERSEA_h") ||
                                  value_name.contains("HOYO_PASS_ENABLE") ||
                                  value_name.contains("HOYO_NEW_USERCENTER_ABTEST");
                
                if should_delete {
                    match game_key.delete_value(&value_name) {
                        Ok(_) => {
                            let full_path = format!("HKEY_CURRENT_USER\\Software\\miHoYo\\{}\\{}", subkey_name, value_name);
                            deleted_entries.push(full_path);
                        },
                        Err(e) => {
                            // Log error but continue with other entries
                            eprintln!("Failed to delete {}: {}", value_name, e);
                        }
                    }
                }
            }
        }
    }
    
    Ok(deleted_entries)
}

#[cfg(not(windows))]
#[tauri::command]
pub fn remove_all_hoyo_pass() -> Result<Vec<String>, String> {
    Err("Registry access is only available on Windows".to_string())
}