# Update Notif
On startup, check for launcher updates by following these steps:

1. After disabling Windows proxy, fetch the latest release information from the GitHub API endpoint: https://github.com/YuukiPS/yuukips-launcher/releases

2. Compare the fetched version number with the current version in package.json

3. If a newer version is available:
   - Display a modal popup with:
     - Title: "Update Available"
     - Message: "A new version (v{latest_version}) of YuukiPS Launcher is available. Your current version is v{current_version}."
     - Release notes from GitHub (if available)
     - Two buttons:
       - "Update Now" - Primary action
       - "Remind Me Later" - Secondary action

4. When "Update Now" is clicked:
   - Download the latest release package
   - Show download progress
   - Automatically install the update when download completes
   - Restart the launcher to apply changes

5. Handle potential errors:
   - Network connectivity issues
   - Download failures
   - Installation problems
   
Ensure the update process runs in the background without blocking the main application functionality.