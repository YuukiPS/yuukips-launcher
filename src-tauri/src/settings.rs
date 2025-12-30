use crate::system::get_yuukips_data_path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub speed_limit_mbps: f64,
    pub divide_speed_enabled: bool,
    pub max_simultaneous_downloads: u32,
    pub proxy_port: u16,
    #[serde(default)]
    pub game_playtime: std::collections::HashMap<u64, u64>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            speed_limit_mbps: 0.0,
            divide_speed_enabled: false,
            max_simultaneous_downloads: 3,
            proxy_port: 8080,
            game_playtime: std::collections::HashMap::new(),
        }
    }
}

impl AppSettings {
    fn get_settings_file_path() -> PathBuf {
        let config_dir = get_yuukips_data_path().unwrap_or_else(|_| ".".to_string());
        PathBuf::from(config_dir).join("app_settings.json")
    }

    pub fn load() -> Self {
        let file_path = Self::get_settings_file_path();
        log::debug!("Loading settings from: {:?}", file_path);

        match fs::read_to_string(&file_path) {
            Ok(content) => {
                match serde_json::from_str::<AppSettings>(&content) {
                    Ok(settings) => {
                        log::debug!("Successfully loaded settings: speed_limit={}, divide_speed={}, max_downloads={}", 
                                  settings.speed_limit_mbps, settings.divide_speed_enabled, settings.max_simultaneous_downloads);
                        settings
                    }
                    Err(e) => {
                        log::error!("Failed to parse settings JSON: {}", e);
                        let default_settings = Self::default();
                        log::info!("Using default settings: speed_limit={}, divide_speed={}, max_downloads={}", 
                                  default_settings.speed_limit_mbps, default_settings.divide_speed_enabled, default_settings.max_simultaneous_downloads);
                        default_settings
                    }
                }
            }
            Err(e) => {
                log::info!("Settings file not found or unreadable: {}", e);
                let default_settings = Self::default();
                log::info!(
                    "Using default settings: speed_limit={}, divide_speed={}, max_downloads={}",
                    default_settings.speed_limit_mbps,
                    default_settings.divide_speed_enabled,
                    default_settings.max_simultaneous_downloads
                );
                default_settings
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_settings_file_path();

        // Create directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&file_path, json)?;

        log::info!(
            "Settings saved successfully: speed_limit={}, divide_speed={}, max_downloads={}, proxy_port={}",
            self.speed_limit_mbps,
            self.divide_speed_enabled,
            self.max_simultaneous_downloads,
            self.proxy_port
        );

        Ok(())
    }
}

// Global settings instance
pub static SETTINGS: once_cell::sync::Lazy<std::sync::Mutex<AppSettings>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(AppSettings::load()));

#[command]
pub fn get_app_speed_limit() -> Result<f64, String> {
    let settings = SETTINGS.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(settings.speed_limit_mbps)
}

#[command]
pub fn set_app_speed_limit(speed_limit_mbps: f64) -> Result<(), String> {
    let mut settings = SETTINGS.lock().map_err(|e| format!("Lock error: {}", e))?;
    settings.speed_limit_mbps = speed_limit_mbps;
    settings.save().map_err(|e| format!("Save error: {}", e))?;
    Ok(())
}

#[command]
pub fn get_app_divide_speed_enabled() -> Result<bool, String> {
    let settings = SETTINGS.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(settings.divide_speed_enabled)
}

#[command]
pub fn set_app_divide_speed_enabled(enabled: bool) -> Result<(), String> {
    let mut settings = SETTINGS.lock().map_err(|e| format!("Lock error: {}", e))?;
    settings.divide_speed_enabled = enabled;
    settings.save().map_err(|e| format!("Save error: {}", e))?;
    Ok(())
}

#[command]
pub fn get_app_max_simultaneous_downloads() -> Result<u32, String> {
    let settings = SETTINGS.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(settings.max_simultaneous_downloads)
}

#[command]
pub fn set_app_max_simultaneous_downloads(max_downloads: u32) -> Result<(), String> {
    // Validate the input (minimum 1, maximum 64 for reasonable limits)
    if max_downloads < 1 || max_downloads > 64 {
        return Err("Max simultaneous downloads must be between 1 and 64".to_string());
    }
    
    let old_limit = {
        let settings = SETTINGS.lock().map_err(|e| format!("Lock error: {}", e))?;
        settings.max_simultaneous_downloads
    };
    
    let mut settings = SETTINGS.lock().map_err(|e| format!("Lock error: {}", e))?;
    settings.max_simultaneous_downloads = max_downloads;
    settings.save().map_err(|e| format!("Save error: {}", e))?;
    
    // Release the settings lock before calling download manager
    drop(settings);
    
    // Trigger queue management in download manager
    crate::download::trigger_queue_management_on_settings_change(max_downloads, old_limit)
        .map_err(|e| format!("Failed to update download queue: {}", e))?;
    
    Ok(())
}



#[command]
pub fn get_all_app_settings() -> Result<AppSettings, String> {
    let settings = SETTINGS.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(settings.clone())
}
