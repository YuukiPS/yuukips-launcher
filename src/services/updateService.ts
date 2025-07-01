import { invoke } from '@tauri-apps/api/core';

export interface GitHubRelease {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  assets: {
    name: string;
    browser_download_url: string;
    size: number;
  }[];
}

export interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseNotes?: string;
  downloadUrl?: string;
  assetSize?: number;
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
  percentage: number;
  speed: number; // bytes per second
}

export class UpdateService {
  private static readonly GITHUB_API_URL = 'https://book-api.yuuki.me/app/yuukips-launcher/latest';
  private static updateCheckPromise: Promise<UpdateInfo> | null = null;
  
  /**
   * Check for available updates
   * @param force - If true, bypasses version comparison and always returns update info
   */
  static async checkForUpdates(force: boolean = false): Promise<UpdateInfo> {
    // If there's already a pending request and not forcing, return it
    if (this.updateCheckPromise && !force) {
      return this.updateCheckPromise;
    }
    
    // Create and store the promise
    this.updateCheckPromise = this.performUpdateCheck(force);
    
    try {
      const result = await this.updateCheckPromise;
      return result;
    } finally {
      // Clear the promise after completion (success or failure)
      this.updateCheckPromise = null;
    }
  }
  
  private static async performUpdateCheck(force: boolean = false): Promise<UpdateInfo> {
    try {
      // Get current version from package.json
      const currentVersion = await invoke('get_current_version') as string;
      
      // Fetch latest release from GitHub API
      const release = await invoke('fetch_latest_release', {
        url: this.GITHUB_API_URL
      }) as GitHubRelease;
      
      const latestVersion = release.tag_name.replace(/^v/, ''); // Remove 'v' prefix if present
      const isUpdateAvailable = this.compareVersions(currentVersion, latestVersion) < 0;
      
      if (!isUpdateAvailable && !force) {
        return {
          available: false,
          currentVersion,
          latestVersion
        };
      }
      
      // Find the appropriate asset for the current platform
      const asset = this.findPlatformAsset(release.assets);
      
      return {
        available: true,
        currentVersion,
        latestVersion,
        releaseNotes: release.body,
        downloadUrl: asset?.browser_download_url,
        assetSize: asset?.size
      };
    } catch (error) {
      console.error('Failed to check for updates:', error);
      throw new Error(`Update check failed: ${error}`);
    }
  }
  
  /**
   * Download and install update with admin privileges
   */
  static async downloadAndInstallUpdate(
    downloadUrl: string,
    onProgress?: (progress: DownloadProgress) => void
  ): Promise<void> {
    try {
      await invoke('download_and_install_update', {
        downloadUrl,
        progressCallback: onProgress ? 'update_progress' : null
      });
    } catch (error) {
      console.error('Failed to download and install update:', error);
      
      // Check if the error is related to file access issues
      const errorMessage = String(error);
      if (errorMessage.includes('being used by another process') || 
          errorMessage.includes('access is denied') ||
          errorMessage.includes('os error 32')) {
        throw new Error(
          'Update installation requires administrator privileges. ' +
          'The installer will request admin access to replace the application files. ' +
          'Please approve the UAC prompt when it appears.'
        );
      }
      
      throw new Error(`Update installation failed: ${error}`);
    }
  }
  
  /**
   * Restart the application to apply updates
   */
  static async restartApplication(): Promise<void> {
    try {
      await invoke('restart_application');
    } catch (error) {
      console.error('Failed to restart application:', error);
      throw new Error(`Application restart failed: ${error}`);
    }
  }

  /**
   * Terminate the application to allow installer to replace files
   * This is used when the installer needs to replace the running executable
   */
  static async terminateForUpdate(): Promise<void> {
    try {
      await invoke('terminate_for_update');
    } catch (error) {
      console.error('Failed to terminate application for update:', error);
      throw new Error(`Application termination failed: ${error}`);
    }
  }
  
  /**
   * Compare two version strings
   * Returns: -1 if v1 < v2, 0 if v1 === v2, 1 if v1 > v2
   */
  private static compareVersions(v1: string, v2: string): number {
    const parts1 = v1.split('.').map(Number);
    const parts2 = v2.split('.').map(Number);
    
    const maxLength = Math.max(parts1.length, parts2.length);
    
    for (let i = 0; i < maxLength; i++) {
      const part1 = parts1[i] || 0;
      const part2 = parts2[i] || 0;
      
      if (part1 < part2) return -1;
      if (part1 > part2) return 1;
    }
    
    return 0;
  }
  
  /**
   * Find the appropriate asset for the current platform
   */
  private static findPlatformAsset(assets: GitHubRelease['assets']) {
    // For Windows, look for .msi or .exe files
    const windowsAsset = assets.find(asset => 
      asset.name.includes('windows') || 
      asset.name.endsWith('.msi') || 
      asset.name.endsWith('.exe')
    );
    
    if (windowsAsset) return windowsAsset;
    
    // Fallback to the first asset if no platform-specific asset is found
    return assets[0];
  }
}

// Event listener for download progress updates
if (typeof window !== 'undefined') {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  window.addEventListener('update_progress', (event: any) => {
    const progress = event.detail as DownloadProgress;
    // This will be handled by the component that initiated the download
    window.dispatchEvent(new CustomEvent('updateDownloadProgress', { detail: progress }));
  });
}