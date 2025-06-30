## 🔄 Patch Workflow

### Automatic Patching (Game Launch)
1. **API Call**: Fetch patch info from `https://ps.yuuki.me/game/patch/{game_id}/{version}/{channel}/{md5}.json`
2. **Method Check**: 
   - Method 0: Skip patching
   - Method 1: Apply file patches
3. **Proxy Control**: Stop proxy if running, respect patch response proxy setting
4. **Cache Check**: Use cached `.patch` files if MD5 matches
5. **Download**: Download new patches with progress reporting
6. **Backup**: Create `.backup` files for original files
7. **Apply**: Replace files with patches
8. **Cache**: Save patches as `.patch` files for future use
9. **Launch**: Start game with updated files

### Automatic Cleanup (Game Stop) in libs.rs
1. **Restoration**: Restore original files (API first, then backups),
2. **Cleanup Patch**: Rename file patch (which is in the API list) that doesn't have `.patch` so the game can run normally without the patch
3. **Proxy**: Stop proxy if running

### Manual Operations
- **Status Check**: Check current patch status
- **Manual Restore**: Restore files without game launch
- **Progress Monitor**: Real-time download tracking

## 🛡️ Security & Error Handling

### Security Measures
- **MD5 Verification**: All downloads verified before application
- **Backup Creation**: Original files always backed up
- **Graceful Fallbacks**: Multiple restoration methods
- **Timeout Protection**: 5-minute download timeout

### Error Handling
- **Download Failures**: Automatic retry and fallback mechanisms
- **MD5 Mismatches**: Reject corrupted downloads
- **Network Issues**: Graceful degradation
- **File System Errors**: Comprehensive error reporting
- **Launch Abortion**: Any patch failure aborts game launch

## 📁 File Organization

### Game Folder Structure
```
GameFolder/
├── game.exe
├── config.json
├── config.json.backup    # Original backup
├── config.json.patch     # Cached patch
├── assets/
│   ├── data.pak
│   ├── data.pak.backup   # Original backup
│   └── data.pak.patch    # Cached patch
└── ...
```

### Cache Management
- **Patch Files**: Stored with `.patch` extension
- **Backup Files**: Stored with `.backup` extension
- **Automatic Cleanup**: Old caches cleaned during restoration
- **Reuse Logic**: Cached patches reused if MD5 matches