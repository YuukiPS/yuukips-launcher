#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::process::Command;
use tauri::command;

#[command]
fn launch_game(_game_id: String, game_title: String) -> Result<String, String> {
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
fn show_game_folder(game_id: String) -> Result<String, String> {
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
fn check_game_installed(game_id: String) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if game is installed by looking for the executable
        let game_path = format!("C:\\Games\\{}\\game.exe", game_id);
        std::path::Path::new(&game_path).exists()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![launch_game, show_game_folder, check_game_installed, open_directory])
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
