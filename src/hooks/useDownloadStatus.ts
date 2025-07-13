import { useState, useEffect } from 'react';
import { DownloadItem, DownloadStats } from '../types';
import { DownloadService } from '../services/downloadService';

export interface DownloadStatusInfo {
  activeDownloads: DownloadItem[];
  stats: DownloadStats;
  totalActiveCount: number;
  totalCompletedCount: number;
  totalSpeed: number;
  isLoading: boolean;
  error: string | null;
}

export const useDownloadStatus = (refreshInterval: number = 1000) => {
  const [downloadStatus, setDownloadStatus] = useState<DownloadStatusInfo>({
    activeDownloads: [],
    stats: {
      total_downloads: 0,
      active_downloads: 0,
      completed_downloads: 0,
      total_downloaded_size: 0,
      average_speed: 0
    },
    totalActiveCount: 0,
    totalCompletedCount: 0,
    totalSpeed: 0,
    isLoading: true,
    error: null
  });

  const fetchDownloadStatus = async () => {
    try {
      const [activeDownloads, stats] = await Promise.all([
        DownloadService.getActiveDownloads(),
        DownloadService.getDownloadStats()
      ]);

      // Calculate total speed from active downloads
      const totalSpeed = activeDownloads
        .filter(download => download.status === 'downloading')
        .reduce((sum, download) => sum + download.speed, 0);

      const activeCount = activeDownloads.filter(
        download => download.status === 'downloading' || download.status === 'paused'
      ).length;

      setDownloadStatus({
        activeDownloads,
        stats,
        totalActiveCount: activeCount,
        totalCompletedCount: stats.completed_downloads,
        totalSpeed,
        isLoading: false,
        error: null
      });
    } catch (error) {
      console.error('Failed to fetch download status:', error);
      setDownloadStatus(prev => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      }));
    }
  };

  useEffect(() => {
    fetchDownloadStatus();
    
    const interval = setInterval(fetchDownloadStatus, refreshInterval);
    
    return () => clearInterval(interval);
  }, [refreshInterval]);

  return downloadStatus;
};

export const formatSpeed = (bytesPerSecond: number): string => {
  if (bytesPerSecond === 0) return '0 B/s';
  
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const base = 1024;
  const digitGroups = Math.floor(Math.log(bytesPerSecond) / Math.log(base));
  
  return `${(bytesPerSecond / Math.pow(base, digitGroups)).toFixed(1)} ${units[digitGroups]}`;
};

export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const base = 1024;
  const digitGroups = Math.floor(Math.log(bytes) / Math.log(base));
  
  return `${(bytes / Math.pow(base, digitGroups)).toFixed(1)} ${units[digitGroups]}`;
};