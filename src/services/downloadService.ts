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
    try {
      const downloadId = await invoke<string>('start_download', {
        url,
        filePath,
        fileName
      });
      return downloadId;
    } catch (error) {
      throw new Error(`Failed to start download: ${error}`);
    }
  }

  /**
   * Pause a download
   */
  static async pauseDownload(downloadId: string): Promise<void> {
    try {
      await invoke('pause_download', { downloadId });
    } catch (error) {
      throw new Error(`Failed to pause download: ${error}`);
    }
  }

  /**
   * Resume a paused download
   */
  static async resumeDownload(downloadId: string): Promise<void> {
    try {
      await invoke('resume_download', { downloadId });
    } catch (error) {
      throw new Error(`Failed to resume download: ${error}`);
    }
  }

  /**
   * Cancel a download
   */
  static async cancelDownload(downloadId: string): Promise<void> {
    try {
      await invoke('cancel_download', { downloadId });
    } catch (error) {
      throw new Error(`Failed to cancel download: ${error}`);
    }
  }

  /**
   * Restart a failed download
   */
  static async restartDownload(downloadId: string): Promise<void> {
    try {
      await invoke('restart_download', { downloadId });
    } catch (error) {
      throw new Error(`Failed to restart download: ${error}`);
    }
  }

  /**
   * Get all active downloads
   */
  static async getActiveDownloads(): Promise<DownloadItem[]> {
    try {
      const downloads = await invoke<DownloadItem[]>('get_active_downloads');
      return downloads;
    } catch (error) {
      throw new Error(`Failed to get active downloads: ${error}`);
    }
  }

  /**
   * Get download status for a specific download
   */
  static async getDownloadStatus(downloadId: string): Promise<DownloadItem> {
    try {
      const download = await invoke<DownloadItem>('get_download_status', { downloadId });
      return download;
    } catch (error) {
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
    try {
      await invoke('clear_completed_downloads');
    } catch (error) {
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

  /**
   * Validate download URL
   */
  static async validateDownloadUrl(url: string): Promise<boolean> {
    try {
      const isValid = await invoke<boolean>('validate_download_url', { url });
      return isValid;
    } catch (error) {
      throw new Error(`Failed to validate download URL: ${error}`);
    }
  }

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
}