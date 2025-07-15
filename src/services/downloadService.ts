import { invoke } from '@tauri-apps/api/core';
import { DownloadItem, DownloadHistory, DownloadStats } from '../types';

/**
 * Service for managing downloads through Tauri backend
 */
export class DownloadService {
  /**
   * Start a new download
   */
  static async startDownload(url: string, filePath: string, fileName?: string): Promise<string> {
    console.log('[DownloadService] Starting download:', { url, filePath, fileName });
    try {
      const downloadId = await invoke<string>('start_download', {
        url,
        filePath,
        fileName
      });
      console.log('[DownloadService] Download started successfully with ID:', downloadId);
      return downloadId;
    } catch (error) {
      console.error('[DownloadService] Failed to start download:', error);
      throw new Error(`Failed to start download: ${error}`);
    }
  }

  /**
   * Pause a download
   */
  static async pauseDownload(downloadId: string): Promise<void> {
    console.log('[DownloadService] Pausing download:', downloadId);
    try {
      await invoke('pause_download', { downloadId });
      console.log('[DownloadService] Download paused successfully:', downloadId);
    } catch (error) {
      console.error('[DownloadService] Failed to pause download:', downloadId, error);
      throw new Error(`Failed to pause download: ${error}`);
    }
  }

  /**
   * Resume a paused download
   */
  static async resumeDownload(downloadId: string): Promise<void> {
    console.log('[DownloadService] Resuming download:', downloadId);
    try {
      await invoke('resume_download', { downloadId });
      console.log('[DownloadService] Download resumed successfully:', downloadId);
    } catch (error) {
      console.error('[DownloadService] Failed to resume download:', downloadId, error);
      throw new Error(`Failed to resume download: ${error}`);
    }
  }

  /**
   * Restart a download
   */
  static async restartDownload(downloadId: string): Promise<void> {
    console.log('[DownloadService] Restarting download:', downloadId);
    try {
      await invoke('restart_download', { downloadId });
      console.log('[DownloadService] Download restarted successfully:', downloadId);
    } catch (error) {
      console.error('[DownloadService] Failed to restart download:', downloadId, error);
      throw new Error(`Failed to restart download: ${error}`);
    }
  }

  /**
   * Cancel a download
   */
  static async cancelDownload(downloadId: string): Promise<void> {
    console.log('[DownloadService] Cancelling download:', downloadId);
    try {
      await invoke('cancel_download', { downloadId });
      console.log('[DownloadService] Download cancelled successfully:', downloadId);
    } catch (error) {
      console.error('[DownloadService] Failed to cancel download:', downloadId, error);
      throw new Error(`Failed to cancel download: ${error}`);
    }
  }

  /**
   * Cancel a download and delete the partially downloaded file
   */
  static async cancelAndDeleteDownload(downloadId: string): Promise<void> {
    console.log('[DownloadService] Cancelling and deleting download:', downloadId);
    try {
      await invoke('cancel_and_delete_download', { downloadId });
      console.log('[DownloadService] Download cancelled and deleted successfully:', downloadId);
    } catch (error) {
      console.error('[DownloadService] Failed to cancel and delete download:', downloadId, error);
      throw new Error(`Failed to cancel and delete download: ${error}`);
    }
  }

  /**
   * Remove a download
   */
  static async removeDownload(downloadId: string): Promise<void> {
    console.log('[DownloadService] Removing download:', downloadId);
    try {
      await invoke('remove_download', { downloadId });
      console.log('[DownloadService] Download removed successfully:', downloadId);
    } catch (error) {
      console.error('[DownloadService] Failed to remove download:', downloadId, error);
      throw new Error(`Failed to remove download: ${error}`);
    }
  }

  /**
   * Get all active downloads
   */
  static async getActiveDownloads(): Promise<DownloadItem[]> {
    console.log('[DownloadService] Getting active downloads');
    try {
      const downloads = await invoke<DownloadItem[]>('get_active_downloads');
      console.log('[DownloadService] Retrieved active downloads count:', downloads.length);
      return downloads;
    } catch (error) {
      console.error('[DownloadService] Failed to get active downloads:', error);
      throw new Error(`Failed to get active downloads: ${error}`);
    }
  }

  /**
   * Get download status for a specific download
   */
  static async getDownloadStatus(downloadId: string): Promise<DownloadItem> {
    console.log('[DownloadService] Getting download status for:', downloadId);
    try {
      const download = await invoke<DownloadItem>('get_download_status', { downloadId });
      console.log('[DownloadService] Download status retrieved:', downloadId, download.status);
      return download;
    } catch (error) {
      console.error('[DownloadService] Failed to get download status:', downloadId, error);
      throw new Error(`Failed to get download status: ${error}`);
    }
  }

  /**
   * Get download history
   */
  static async getDownloadHistory(): Promise<DownloadHistory[]> {
    try {
      const history = await invoke<DownloadHistory[]>('get_download_history');
      return history;
    } catch (error) {
      throw new Error(`Failed to get download history: ${error}`);
    }
  }

  /**
   * Clear completed downloads
   */
  static async clearCompletedDownloads(): Promise<void> {
    console.log('[DownloadService] Clearing completed downloads');
    try {
      await invoke('clear_completed_downloads');
      console.log('[DownloadService] Completed downloads cleared successfully');
    } catch (error) {
      console.error('[DownloadService] Failed to clear completed downloads:', error);
      throw new Error(`Failed to clear completed downloads: ${error}`);
    }
  }

  /**
   * Clear download history
   */
  static async clearDownloadHistory(): Promise<void> {
    try {
      await invoke('clear_download_history');
    } catch (error) {
      throw new Error(`Failed to clear download history: ${error}`);
    }
  }

  /**
   * Get download statistics
   */
  static async getDownloadStats(): Promise<DownloadStats> {
    try {
      const stats = await invoke<DownloadStats>('get_download_stats');
      return stats;
    } catch (error) {
      throw new Error(`Failed to get download stats: ${error}`);
    }
  }

  /**
   * Open download location in file explorer
   */
  static async openDownloadLocation(filePath: string): Promise<void> {
    try {
      await invoke('open_download_location', { filePath });
    } catch (error) {
      throw new Error(`Failed to open download location: ${error}`);
    }
  }

  /**
   * Search downloads by name
   */
  static async searchDownloads(query: string): Promise<DownloadItem[]> {
    try {
      const downloads = await invoke<DownloadItem[]>('search_downloads', { query });
      return downloads;
    } catch (error) {
      throw new Error(`Failed to search downloads: ${error}`);
    }
  }

  /**
   * Get downloads by status
   */
  static async getDownloadsByStatus(status: string): Promise<DownloadItem[]> {
    try {
      const downloads = await invoke<DownloadItem[]>('get_downloads_by_status', { status });
      return downloads;
    } catch (error) {
      throw new Error(`Failed to get downloads by status: ${error}`);
    }
  }

  /**
   * Bulk pause downloads
   */
  static async bulkPauseDownloads(downloadIds: string[]): Promise<void> {
    try {
      await invoke('bulk_pause_downloads', { downloadIds });
    } catch (error) {
      throw new Error(`Failed to bulk pause downloads: ${error}`);
    }
  }

  /**
   * Bulk resume downloads
   */
  static async bulkResumeDownloads(downloadIds: string[]): Promise<void> {
    try {
      await invoke('bulk_resume_downloads', { downloadIds });
    } catch (error) {
      throw new Error(`Failed to bulk resume downloads: ${error}`);
    }
  }

  /**
   * Bulk cancel downloads
   */
  static async bulkCancelDownloads(downloadIds: string[]): Promise<void> {
    try {
      await invoke('bulk_cancel_downloads', { downloadIds });
    } catch (error) {
      throw new Error(`Failed to bulk cancel downloads: ${error}`);
    }
  }

  /**
   * Bulk cancel downloads and delete the partially downloaded files
   */
  static async bulkCancelAndDeleteDownloads(downloadIds: string[]): Promise<void> {
    try {
      await invoke('bulk_cancel_and_delete_downloads', { downloadIds });
    } catch (error) {
      throw new Error(`Failed to bulk cancel and delete downloads: ${error}`);
    }
  }

  /**
   * Set download directory
   */
  static async setDownloadDirectory(directory: string): Promise<void> {
    try {
      await invoke('set_download_directory', { directory });
    } catch (error) {
      throw new Error(`Failed to set download directory: ${error}`);
    }
  }

  /**
   * Get download directory
   */
  static async getDownloadDirectory(): Promise<string> {
    try {
      const directory = await invoke<string>('get_download_directory');
      return directory;
    } catch (error) {
      throw new Error(`Failed to get download directory: ${error}`);
    }
  }

  /**
   * Subscribe to download progress updates
   */
  static async subscribeToDownloadUpdates(callback: (download: DownloadItem) => void): Promise<() => void> {
    try {
      // This would typically use Tauri's event system
      // For now, we'll implement a polling mechanism
      const interval = setInterval(async () => {
        try {
          const downloads = await this.getActiveDownloads();
          downloads.forEach(callback);
        } catch (error) {
          console.error('Failed to get download updates:', error);
        }
      }, 1000);

      return () => clearInterval(interval);
    } catch (error) {
      throw new Error(`Failed to subscribe to download updates: ${error}`);
    }
  }

  // URL validation functions removed - downloads now proceed directly

  /**
   * Get file size from URL without downloading
   */
  static async getFileSizeFromUrl(url: string): Promise<number> {
    try {
      const size = await invoke<number>('get_file_size_from_url', { url });
      return size;
    } catch (error) {
      throw new Error(`Failed to get file size: ${error}`);
    }
  }

  /**
   * Check if file already exists
   */
  static async checkFileExists(filePath: string): Promise<boolean> {
    try {
      const exists = await invoke<boolean>('check_file_exists', { filePath });
      return exists;
    } catch (error) {
      throw new Error(`Failed to check if file exists: ${error}`);
    }
  }

  /**
   * Get available disk space
   */
  static async getAvailableDiskSpace(path: string): Promise<number> {
    try {
      const space = await invoke<number>('get_available_disk_space', { path });
      return space;
    } catch (error) {
      throw new Error(`Failed to get available disk space: ${error}`);
    }
  }

  /**
   * Resume interrupted downloads and auto-resume them
   */
  static async resumeInterruptedDownloads(): Promise<string[]> {
    console.log('[DownloadService] Resuming interrupted downloads');
    try {
      const interruptedIds = await invoke<string[]>('resume_interrupted_downloads');
      console.log('[DownloadService] Found interrupted downloads:', interruptedIds);
      
      // Auto-resume each interrupted download
      for (const downloadId of interruptedIds) {
        try {
          await this.resumeDownload(downloadId);
          console.log('[DownloadService] Auto-resumed interrupted download:', downloadId);
        } catch (error) {
          console.error('[DownloadService] Failed to auto-resume download:', downloadId, error);
        }
      }
      
      return interruptedIds;
    } catch (error) {
      console.error('[DownloadService] Failed to resume interrupted downloads:', error);
      throw new Error(`Failed to resume interrupted downloads: ${error}`);
    }
  }

  /**
   * Get current download speed limit in MB/s
   */
  static async getSpeedLimit(): Promise<number> {
    console.log('[DownloadService] Getting speed limit from backend...');
    try {
      const speedLimit = await invoke<number>('get_speed_limit');
      console.log('[DownloadService] Retrieved speed limit:', speedLimit);
      return speedLimit;
    } catch (error) {
      console.error('[DownloadService] Failed to get speed limit:', error);
      throw new Error(`Failed to get speed limit: ${error}`);
    }
  }

  /**
   * Set download speed limit in MB/s (0 = unlimited)
   */
  static async setSpeedLimit(speedLimitMbps: number): Promise<void> {
    console.log('[DownloadService] Setting speed limit to backend:', speedLimitMbps);
    try {
      await invoke('set_speed_limit', { speedLimitMbps: speedLimitMbps });
      console.log('[DownloadService] Speed limit set successfully:', speedLimitMbps);
    } catch (error) {
      console.error('[DownloadService] Failed to set speed limit:', error);
      throw new Error(`Failed to set speed limit: ${error}`);
    }
  }

  /**
   * Get whether speed limit should be divided among active downloads
   */
  static async getDivideSpeedEnabled(): Promise<boolean> {
    console.log('[DownloadService] Getting divide speed enabled from backend...');
    try {
      const enabled = await invoke<boolean>('get_divide_speed_enabled');
      console.log('[DownloadService] Retrieved divide speed enabled:', enabled);
      return enabled;
    } catch (error) {
      console.error('[DownloadService] Failed to get divide speed enabled:', error);
      throw new Error(`Failed to get divide speed enabled: ${error}`);
    }
  }

  /**
   * Set whether speed limit should be divided among active downloads
   */
  static async setDivideSpeedEnabled(enabled: boolean): Promise<void> {
    console.log('[DownloadService] Setting divide speed enabled to backend:', enabled);
    try {
      await invoke('set_divide_speed_enabled', { enabled });
      console.log('[DownloadService] Divide speed enabled set successfully:', enabled);
    } catch (error) {
      console.error('[DownloadService] Failed to set divide speed enabled:', error);
      throw new Error(`Failed to set divide speed enabled: ${error}`);
    }
  }

  /**
   * Get current max simultaneous downloads setting
   */
  static async getMaxSimultaneousDownloads(): Promise<number> {
    console.log('[DownloadService] Getting max simultaneous downloads from backend...');
    try {
      const maxDownloads = await invoke<number>('get_max_simultaneous_downloads');
      console.log('[DownloadService] Retrieved max simultaneous downloads:', maxDownloads);
      return maxDownloads;
    } catch (error) {
      console.error('[DownloadService] Failed to get max simultaneous downloads:', error);
      throw new Error(`Failed to get max simultaneous downloads: ${error}`);
    }
  }

  /**
   * Set max simultaneous downloads (1-10)
   */
  static async setMaxSimultaneousDownloads(maxDownloads: number): Promise<void> {
    console.log('[DownloadService] Setting max simultaneous downloads to backend:', maxDownloads);
    try {
      await invoke('set_max_simultaneous_downloads', { maxDownloads });
      console.log('[DownloadService] Max simultaneous downloads set successfully:', maxDownloads);
    } catch (error) {
      console.error('[DownloadService] Failed to set max simultaneous downloads:', error);
      throw new Error(`Failed to set max simultaneous downloads: ${error}`);
    }
  }
}