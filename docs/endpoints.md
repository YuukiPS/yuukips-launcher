# Endpoint Index

Comprehensive list of all external endpoints and domains used in YuukiPS Launcher.

---

## Domain Summary

| Domain | Purpose |
|--------|---------|
| `ps.yuuki.me` | Gate |
| `login.yuuki.me` | Game authentication proxy target |
| `git.yuuki.me` | Git repository (Gitea) for source code & releases |
| `discord.yuuki.me` | Discord community invite link |
| `t.me/YuukiPSz` | Telegram community |
| `youtube.com/c/MeYuuki` | YouTube channel |
| `objectstore.e2-delivery.com` | CDN for game patch file downloads |

---

## API Endpoints (`ps.yuuki.me`)

### 1. Game List

```
GET https://ps.yuuki.me/json/game_all.json?time={timestamp}
```

- **Source**: `src/services/gameApi.ts`
- **Method**: GET (fetch)
- **Purpose**: Fetches the complete list of all supported games with metadata (title, description, engine, versions, images, status, etc.)
- **Cache**: Cache-busted with a `?time={timestamp}` query parameter on each request
- **Response**: JSON array of game objects
- **Used on**: Launcher startup / game list loading

---

### 2. Patch Info

```
GET https://ps.yuuki.me/game/patch/{game_id}/{version}/{channel}/{md5}.json
```

- **Source**: `src-tauri/src/patch.rs`
- **Method**: GET (reqwest, Rust backend)
- **Purpose**: Fetches patch information for a specific game version. Returns which files need to be patched, proxy settings, and patch method instructions.
- **Parameters**:
  - `game_id` - Numeric game identifier
  - `version` - Game version string
  - `channel` - Game channel number
  - `md5` - MD5 hash of the game file being patched
- **Response**: `PatchResponse` JSON with fields: `patch`, `proxy`, `method`, `list` (file patches)
- **Error handling**: 404 returns `PATCH_NOT_FOUND` error with detailed info for frontend popup
- **Used when**: Applying game patches via the patcher

---

### 3. Game Download Manifest

```
GET https://ps.yuuki.me/game/download/pc/{game_id}/{channel}/{version}.json
```

- **Source**: `src/components/GameSettingsModal.tsx`
- **Method**: GET (fetch)
- **Purpose**: Retrieves download manifest for a specific game, including file lists with sizes and checksums for downloading game files.
- **Parameters**:
  - `game_id` - Numeric game identifier
  - `channel` - Game channel number
  - `version` - Game version string
- **Response**: JSON with `retcode`, file list with download URLs, sizes, checksums
- **Error**: `retcode: -1` indicates download not available
- **Used when**: User initiates game download from settings modal

---

### 4. MD5 Patch Lookup

```
GET https://ps.yuuki.me/api/v1/patch/find/{md5}
```

- **Source**: `src/components/DiskScanModal.tsx`
- **Method**: GET (fetch)
- **Purpose**: Looks up a file by its MD5 hash to find matching patch information. Used during disk scanning to identify game files.
- **Parameters**:
  - `md5` - MD5 hash of the file to look up
- **Response**: JSON with `game_id`, `version`, `channel` if found; `retcode: -1` if not found
- **Used when**: Scanning disk for existing game files

---

### 5. Launcher Update Check

```
GET https://ps.yuuki.me/app/yuukips-launcher/latest
```

- **Source**: `src/services/updateService.ts`
- **Method**: GET (reqwest, via Tauri backend `fetch_latest_release`)
- **Purpose**: Checks for launcher self-updates. Returns latest release info.
- **Response**: JSON with `tag_name`, `name`, `body` (release notes), `published_at`, `assets` (download URLs, sizes)
- **Used on**: Application startup / manual update check
- **Fallback**: Manual download from `https://git.yuuki.me/YuukiTeam/YuukiPS_Launcher_PC/releases`

---

### 6. Default Server URL

```
https://login.yuuki.me
```

- **Source**: `src/services/gameApi.ts`
- **Method**: N/A (configuration value)
- **Purpose**: Default `serverUrl` for games when no explicit server URL is provided in the game API response. Used as the base URL for game server connections.

---

## Proxy Target (`login.yuuki.me`)

### 7. Game Authentication Proxy

```
https://login.yuuki.me
```

- **Source**: `src-tauri/src/proxy.rs`
- **Purpose**: Default proxy target server for game authentication. The launcher runs a local MITM proxy that intercepts game traffic and redirects authentication requests to this server.
- **Configuration**: Stored in global `SERVER` variable, can be changed at runtime based on patch response settings
- **Used when**: Proxy is active and game makes authentication requests

---

## Repository & Community Links

### 8. Git Repository (Gitea)

```
https://git.yuuki.me/YuukiTeam/YuukiPS_Launcher_PC
```

- **Source**: `src-tauri/Cargo.toml`, `src/services/updateService.ts`
- **Purpose**: Source code repository and release downloads for the launcher
- **Releases URL**: `https://git.yuuki.me/YuukiTeam/YuukiPS_Launcher_PC/releases`

### 9. Discord

```
https://discord.yuuki.me/
```

- **Source**: `src/data/socialLinks.ts`
- **Purpose**: Discord community invite link

### 10. Telegram

```
https://t.me/YuukiPSz
```

- **Source**: `src/data/socialLinks.ts`
- **Purpose**: Telegram community group

### 11. YouTube

```
https://www.youtube.com/c/MeYuuki
```

- **Source**: `src/data/socialLinks.ts`
- **Purpose**: YouTube channel

---

## CDN (External)

### 12. Patch File Downloads

```
https://objectstore.e2-delivery.com/...
```

- **Source**: `src-tauri/src/patch.rs`
- **Purpose**: CDN endpoint where actual patch binary files are hosted. Download URLs are provided in the patch info response from endpoint #2.
- **Used when**: Downloading and applying game patches
