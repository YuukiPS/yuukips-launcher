# YuukiPS Launcher

A modern game launcher built with React, TypeScript, Tailwind CSS, and Tauri for cross-platform desktop functionality.

## Features

- 🎮 Modern game launcher interface
- 🖥️ Native desktop application with Tauri
- 🎯 Windows-specific game launching functionality
- 📁 Game folder management
- ⚙️ Game settings and configuration
- 🔄 Real-time game installation status
- 🎨 Beautiful UI with Tailwind CSS

## Development

### Prerequisites

- Node.js (v16 or higher)
- Rust (latest stable)
- Windows (for Windows-specific features)

### Setup

1. Install dependencies:
```bash
npm install
```

2. Run in development mode (web version):
```bash
npm run dev
```

3. Run Tauri development mode (desktop version):
```bash
npm run tauri:dev
```

### Building

1. Build web version:
```bash
npm run build
```

2. Build desktop application:
```bash
npm run tauri:build
```

## Tauri Integration

The launcher includes the following Tauri commands for Windows functionality:

- `launch_game(game_id, game_title)` - Launch a game executable
- `show_game_folder(game_id)` - Open game folder in Windows Explorer
- `check_game_installed(game_id)` - Check if a game is installed

### Game Installation Detection

The launcher checks for games in the following directory structure:
```
C:\Games\{game_id}\game.exe
```

You can modify the game paths in `src-tauri/src/lib.rs` to match your game installation directories.

## Features

### Desktop-Specific Features

- **Custom Window Controls**: Minimize, maximize, and close buttons
- **Window Dragging**: Drag the header to move the window
- **Game Launching**: Direct integration with Windows to launch game executables
- **Folder Management**: Open game folders in Windows Explorer
- **Installation Detection**: Automatically detect installed games

### Cross-Platform Compatibility

The launcher works both as a web application and as a native desktop application:

- Web version: Shows demo messages for desktop-specific features
- Desktop version: Full functionality with native Windows integration

## Configuration

### Tauri Configuration

The Tauri configuration is located in `src-tauri/tauri.conf.json`. Key settings:

- Window size: 1200x800 (minimum 1000x700)
- Decorations: Disabled (custom window controls)
- Transparency: Enabled for modern UI effects

### Game Data

Game information is stored in `src/data/games.ts`. Each game includes:

- Basic info (title, description, developer)
- Version and status information
- Installation and play time data
- Visual assets (images, backgrounds)

## Development Notes

- The launcher automatically detects if it's running in Tauri or web mode
- Windows-specific functionality is conditionally compiled for the target OS
- All Tauri commands include error handling and user feedback
- The UI adapts based on the runtime environment (web vs desktop)

## License

MIT License - see LICENSE file for details.