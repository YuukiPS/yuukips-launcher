use winreg::enums::*;
use winreg::RegKey;
use serde_json::Number;



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

/// Get game folder path using name_code directly from HoyoPlay registry
#[tauri::command]
pub fn get_hoyoplay_game_folder(name_code: String) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let registry_path = format!("Software\\Cognosphere\\HYP\\1_0\\{}", name_code);
    
    let game_key = hkcu
        .open_subkey(&registry_path)
        .map_err(|e| format!("Failed to open registry key for {}: {}", name_code, e))?;
    
    let install_path = game_key
        .get_value::<String, _>("GameInstallPath")
        .map_err(|e| format!("Failed to get GameInstallPath for {}: {}", name_code, e))?;
    
    Ok(install_path)
}