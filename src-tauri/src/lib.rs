#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::process::Command;
use serde_json::Number;
use tauri::command;

#[command]
fn launch_game(_game_id: Number, game_title: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // For demonstration, we'll show a Windows notification
        // In a real launcher, you would launch the actual game executable
        
        // Example of launching a Windows application
        // You would replace this with the actual game executable path
        match Command::new("cmd")
            .args(["/C", "echo", &format!("Starting {}", game_title)])
            .output()
        {
            Ok(_) => Ok(format!("Successfully launched {}", game_title)),
            Err(e) => Err(format!("Failed to launch game: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Game launching is only supported on Windows".to_string())
    }
}

#[command]
fn get_game_folder_path(game_id: Number, version: String) -> Result<String, String> {
    // This would typically read from a config file or database
    // For now, we'll return an error indicating the path should be set via frontend
    Err(format!("Game folder path not configured for game {} version {}. Please set it in game settings.", game_id, version))
}

#[command]
fn launch_game_with_engine(
    game_id: Number,
    game_title: String,
    _engine_id: Number,
    engine_name: String,
    version: String,
    game_folder_path: String,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Use the provided game folder path from frontend settings
        if game_folder_path.is_empty() {
            return Err(format!("Game folder path not set for {} version {}. Please configure it in game settings.", game_title, version));
        }
        
        // Check if game folder exists
        if !std::path::Path::new(&game_folder_path).exists() {
            return Err(format!("Game folder not found: {}. Please verify the path in game settings.", game_folder_path));
        }
        
        // Determine game executable name based on game ID
        let game_exe_name = match game_id.as_u64() {
            Some(1) => "GenshinImpact.exe",
            Some(2) => "StarRail.exe",
            _ => return Err(format!("Unsupported game ID: {}", game_id)),
        };
        
        // Construct full path to game executable
        let game_exe_path = std::path::Path::new(&game_folder_path).join(game_exe_name);
        
        // Check if game executable exists
        if !game_exe_path.exists() {
            return Err(format!("Game executable not found: {}. Please verify the game installation.", game_exe_path.display()));
        }
        
        // Launch the game executable
        match Command::new(&game_exe_path)
            .current_dir(&game_folder_path)
            .spawn()
        {
            Ok(_) => Ok(format!("Successfully launched {} with {} from folder {}", game_title, engine_name, game_folder_path)),
            Err(e) => Err(format!("Failed to launch game: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Game launching is only supported on Windows".to_string())
    }
}

#[command]
fn show_game_folder(game_id: Number) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Open the game folder in Windows Explorer
        let game_path = format!("C:\\Games\\{}", game_id); // Example path
        match Command::new("explorer")
            .arg(&game_path)
            .spawn()
        {
            Ok(_) => Ok(format!("Opened folder for {}", game_id)),
            Err(e) => Err(format!("Failed to open folder: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Folder opening is only supported on Windows".to_string())
    }
}

#[command]
fn open_directory(path: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        match Command::new("explorer")
            .arg(&path)
            .spawn()
        {
            Ok(_) => Ok(format!("Opened directory: {}", path)),
            Err(e) => Err(format!("Failed to open directory: {}", e)),
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        match Command::new("open")
            .arg(&path)
            .spawn()
        {
            Ok(_) => Ok(format!("Opened directory: {}", path)),
            Err(e) => Err(format!("Failed to open directory: {}", e)),
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        match Command::new("xdg-open")
            .arg(&path)
            .spawn()
        {
            Ok(_) => Ok(format!("Opened directory: {}", path)),
            Err(e) => Err(format!("Failed to open directory: {}", e)),
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Directory opening is not supported on this platform".to_string())
    }
}

#[command]
fn check_game_installed(game_id: Number, version: String, game_folder_path: String) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if game is installed by verifying the configured folder path exists
        if game_folder_path.is_empty() {
            return false; // No path configured means not installed
        }
        
        // Check if the configured game folder exists
        std::path::Path::new(&game_folder_path).exists()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![launch_game, launch_game_with_engine, get_game_folder_path, show_game_folder, check_game_installed, open_directory])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
