import React, { useState, useEffect, useCallback } from 'react';
import {
  Download,
  Pause,
  Play,
  X,
  RotateCcw,
  Trash2,
  Search,
  Filter,
  FolderOpen,
  CheckCircle,
  AlertCircle,
  Clock,
  ArrowUpDown,
  Plus,
  Settings,
  RefreshCw
} from 'lucide-react';
import { DownloadItem, DownloadStats, ActivityEntry } from '../types';
import { DownloadService } from '../services/downloadService';
import { useDownloadSettingsContext } from '../hooks/useDownloadSettingsContext';
import { invoke } from '@tauri-apps/api/core';
import { open, confirm } from '@tauri-apps/plugin-dialog';

interface DownloadManagerProps {
  isOpen: boolean;
  onClose: () => void;
}

type SortField = 'fileName' | 'progress' | 'size' | 'speed' | 'status' | 'startTime';
type SortDirection = 'asc' | 'desc';
type FilterStatus = 'all' | 'downloading' | 'paused' | 'completed' | 'error' | 'queued';

export const DownloadManager: React.FC<DownloadManagerProps> = ({ isOpen, onClose }) => {
  const { settings: globalSettings, updateSettings: updateGlobalSettings } = useDownloadSettingsContext();
  
  // Debug: Log context values whenever they change
  useEffect(() => {
    console.log('🔍 DownloadManager: globalSettings changed:', globalSettings);
  }, [globalSettings]);
  
  const [downloads, setDownloads] = useState<DownloadItem[]>([]);
  const [activities, setActivities] = useState<ActivityEntry[]>([]);
  const [stats, setStats] = useState<DownloadStats>({
    total_downloads: 0,
    active_downloads: 0,
    completed_downloads: 0,
    total_downloaded_size: 0,
    average_speed: 0
  });
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState<FilterStatus>('all');
  const [sortField, setSortField] = useState<SortField>('fileName');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [selectedDownloads, setSelectedDownloads] = useState<Set<string>>(new Set());
  const [activeTab, setActiveTab] = useState<'active' | 'activity'>('active');

  // New download form state
  const [newDownloadUrl, setNewDownloadUrl] = useState('');
  const [newDownloadFolder, setNewDownloadFolder] = useState('');
  const [isAddingDownload, setIsAddingDownload] = useState(false);
  const [urlError, setUrlError] = useState('');
  const [folderError, setFolderError] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);

  // Settings modal state
  const [showSettingsModal, setShowSettingsModal] = useState(false);
  const [tempSpeedLimit, setTempSpeedLimit] = useState(0);
  const [tempDivideSpeedEnabled, setTempDivideSpeedEnabled] = useState(false);
  const [tempMaxSimultaneousDownloads, setTempMaxSimultaneousDownloads] = useState(3);
  const [tempDisableRangeRequests, setTempDisableRangeRequests] = useState(false);

  // Column width customization state
  const [columnWidths, setColumnWidths] = useState({
    checkbox: '40px',
    fileName: '2fr',
    size: '120px',
    progress: '200px',
    speed: '100px',
    status: '120px',
    started: '140px',
    actions: '120px'
  });
  const [isResizing, setIsResizing] = useState(false);
  const [resizingColumn, setResizingColumn] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    try {
      const [activeDownloads, downloadStats, activityEntries] = await Promise.all([
        DownloadService.getActiveDownloads(),
        DownloadService.getDownloadStats(),
        loadActivities()
      ]);
      
      setDownloads(activeDownloads);
      setStats(downloadStats);
      setActivities(activityEntries);
    } catch (error) {
      console.error('Failed to load download data:', error);
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      console.log('📂 DownloadManager opened, initializing downloads and data...');
      // Resume interrupted downloads first, then load data
      const initializeDownloads = async () => {
        await loadData();
      };
      
      initializeDownloads();
      loadDefaultDownloadFolder();
      
      // Set up polling for download updates
      const interval = setInterval(loadData, 1000);
      return () => clearInterval(interval);
    }
  }, [isOpen, loadData]);

  // Simple sync when modal opens - no complex dependencies
  useEffect(() => {
    if (showSettingsModal) {
      console.log('🔧 Settings modal opened, using current global settings:', globalSettings);
      setTempSpeedLimit(globalSettings.speedLimit);
      setTempDivideSpeedEnabled(globalSettings.divideSpeedEnabled);
      setTempMaxSimultaneousDownloads(globalSettings.maxSimultaneousDownloads);
      setTempDisableRangeRequests(globalSettings.disableRangeRequests);
      
      console.log('🔧 Temp settings initialized:', {
        tempSpeedLimit: globalSettings.speedLimit,
        tempDivideSpeedEnabled: globalSettings.divideSpeedEnabled,
        tempMaxSimultaneousDownloads: globalSettings.maxSimultaneousDownloads,
        tempDisableRangeRequests: globalSettings.disableRangeRequests
      });
    }
  }, [showSettingsModal, globalSettings]); // Include globalSettings dependency

  const loadActivities = async (): Promise<ActivityEntry[]> => {
    try {
      return await invoke('get_activities');
    } catch (error) {
      console.error('Failed to load activities:', error);
      return [];
    }
  };

  const clearActivities = async () => {
    try {
      await invoke('clear_activities');
      setActivities([]);
      // Track the clear action itself
      await invoke('add_user_interaction_activity', {
        action: 'Clear Activities',
        details: 'All activity entries cleared by user'
      });
      // Reload activities to get the new clear action
      const newActivities = await loadActivities();
      setActivities(newActivities);
    } catch (error) {
      console.error('Failed to clear activities:', error);
    }
  };

  const addUserInteraction = async (action: string, details?: string) => {
    try {
      await invoke('add_user_interaction_activity', { action, details });
      // Reload activities to show the new entry
      const newActivities = await loadActivities();
      setActivities(newActivities);
    } catch (error) {
      console.error('Failed to add user interaction:', error);
    }
  };

  const loadDefaultDownloadFolder = async () => {
    try {
      const defaultFolder = await DownloadService.getDownloadDirectory();
      setNewDownloadFolder(defaultFolder);
    } catch (error) {
      console.error('Failed to load default download folder:', error);
      setNewDownloadFolder('C:\\Downloads');
    }
  };

  // Column resizing functions
  const getGridTemplateColumns = () => {
    return `${columnWidths.checkbox} ${columnWidths.fileName} ${columnWidths.size} ${columnWidths.progress} ${columnWidths.speed} ${columnWidths.status} ${columnWidths.started} ${columnWidths.actions}`;
  };

  const handleColumnResize = (columnKey: string, newWidth: string) => {
    setColumnWidths(prev => ({
      ...prev,
      [columnKey]: newWidth
    }));
  };

  const resetColumnWidths = () => {
    setColumnWidths({
      checkbox: '40px',
      fileName: '2fr',
      size: '120px',
      progress: '200px',
      speed: '100px',
      status: '120px',
      started: '140px',
      actions: '120px'
    });
  };

  const handleMouseDown = (e: React.MouseEvent, columnKey: string) => {
    e.preventDefault();
    e.stopPropagation(); // Prevent sorting when resizing
    setIsResizing(true);
    setResizingColumn(columnKey);
    
    const startX = e.clientX;
    const currentWidth = columnWidths[columnKey as keyof typeof columnWidths];
    
    // Parse current width (handle both px and fr units)
    let startWidth: number;
    if (currentWidth.includes('fr')) {
      startWidth = parseFloat(currentWidth.replace('fr', '')) * 100; // Convert fr to approximate px for calculation
    } else {
      startWidth = parseInt(currentWidth.replace('px', ''));
    }
    
    const handleMouseMove = (e: MouseEvent) => {
      const deltaX = e.clientX - startX;
      const newWidth = Math.max(50, startWidth + deltaX); // Minimum width of 50px
      
      if (columnKey === 'fileName') {
        // For fileName, use fractional units to maintain flexibility
        const frValue = Math.max(0.5, newWidth / 100);
        handleColumnResize(columnKey, `${frValue}fr`);
      } else {
        // For other columns, use fixed pixel widths
        handleColumnResize(columnKey, `${newWidth}px`);
      }
    };
    
    const handleMouseUp = () => {
      setIsResizing(false);
      setResizingColumn(null);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };





  const handleAddDownload = async () => {
    console.log('[DownloadManager] Starting add download process', {
      url: newDownloadUrl,
      folder: newDownloadFolder
    });
    
    setUrlError('');
    setFolderError('');

    // Basic URL check - only ensure it's not empty
    if (!newDownloadUrl.trim()) {
      console.log('[DownloadManager] URL validation failed: empty URL');
      setUrlError('URL is required');
      return;
    }

    // Validate folder
    if (!newDownloadFolder.trim()) {
      console.log('[DownloadManager] Folder validation failed: empty folder');
      setFolderError('Destination folder is required');
      return;
    }

    setIsAddingDownload(true);
    console.log('[DownloadManager] Starting backend validation and download process');

    try {
      // URL validation removed - proceeding directly to download
      console.log('[DownloadManager] Starting download directly without URL validation');

      // Extract filename from URL
      const urlObj = new URL(newDownloadUrl);
      const fileName = urlObj.pathname.split('/').pop() || 'download';
      const filePath = `${newDownloadFolder}\\${fileName}`;
      console.log('[DownloadManager] Extracted filename:', fileName, 'Full path:', filePath);

      // Check if file already exists
      console.log('[DownloadManager] Checking if file exists:', filePath);
      const fileExists = await DownloadService.checkFileExists(filePath);
      console.log('[DownloadManager] File exists check result:', fileExists);
      
      if (fileExists) {
        const overwrite = await confirm(`File ${fileName} already exists. Do you want to overwrite it?`, {
          title: 'File Already Exists',
          kind: 'warning'
        });
        console.log('[DownloadManager] User overwrite decision:', overwrite);
        if (!overwrite) {
          console.log('[DownloadManager] User cancelled overwrite - aborting download');
          setIsAddingDownload(false);
          return;
        }
      }

      // Start the actual download
      console.log('[DownloadManager] Starting download with backend');
      const downloadId = await DownloadService.startDownload(newDownloadUrl, filePath, fileName);
      console.log('[DownloadManager] Download started successfully with ID:', downloadId);

      // Log user interaction for adding download
      await addUserInteraction(`Added new download: ${fileName}`);

      // Only clear form and close modal if download started successfully
      setNewDownloadUrl('');
      setShowAddModal(false);
      console.log('[DownloadManager] Modal closed, download initiated successfully');
      
      // Refresh data to show new download
      await loadData();
      console.log('[DownloadManager] Download data refreshed');

    } catch (error) {
      console.error('[DownloadManager] Failed to start download:', error);
      
      // Extract meaningful error message
      let errorMessage = 'Failed to start download. Please try again.';
      if (error instanceof Error) {
        if (error.message.includes('network') || error.message.includes('fetch')) {
          errorMessage = 'Network error occurred. Please check your internet connection and try again.';
        } else if (error.message.includes('permission') || error.message.includes('access')) {
          errorMessage = 'Permission denied. Please check folder permissions and try again.';
        } else {
          errorMessage = `Download failed: ${error.message}`;
        }
      }
      
      setUrlError(errorMessage);
      console.log('[DownloadManager] Error message set:', errorMessage);
    } finally {
      setIsAddingDownload(false);
      console.log('[DownloadManager] Add download process completed');
    }
  };

  const selectDownloadFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: newDownloadFolder
      });
      
      if (selected && typeof selected === 'string') {
        setNewDownloadFolder(selected);
        // Update the default download directory
        await DownloadService.setDownloadDirectory(selected);
      }
    } catch (error) {
      console.error('Failed to select folder:', error);
    }
  };


  const formatFileSize = (bytes: number): string => {
    if (bytes === 0 || isNaN(bytes) || bytes === undefined || bytes === null) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatSpeed = (bytesPerSecond: number): string => {
    return formatFileSize(bytesPerSecond) + '/s';
  };

  const formatTime = (seconds: number): string => {
    if (seconds === 0) return '--';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    } else {
      return `${secs}s`;
    }
  };

  const getFileIcon = (extension: string) => {
    // Add null/undefined check for extension
    const ext = extension || '';
    switch (ext.toLowerCase()) {
      case '.zip':
      case '.rar':
      case '.7z':
        return '📦';
      case '.exe':
      case '.msi':
        return '⚙️';
      case '.pak':
      case '.dat':
        return '🎮';
      case '.mp4':
      case '.avi':
      case '.mkv':
        return '🎬';
      case '.mp3':
      case '.wav':
      case '.flac':
        return '🎵';
      default:
        return '📄';
    }
  };

  const getStatusIcon = (status: DownloadItem['status']) => {
    switch (status) {
      case 'downloading':
        return <Download className="w-4 h-4 text-blue-500" />;
      case 'paused':
        return <Pause className="w-4 h-4 text-yellow-500" />;
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
      case 'cancelled':
        return <X className="w-4 h-4 text-gray-500" />;
      case 'queued':
        return <Clock className="w-4 h-4 text-orange-500" />;
      default:
        return <Clock className="w-4 h-4 text-gray-500" />;
    }
  };

  const handlePauseResume = async (id: string) => {
    try {
      const download = downloads.find(d => d.id === id);
      if (!download) return;

      if (download.status === 'downloading') {
        await DownloadService.pauseDownload(id);
        await addUserInteraction(`Paused download: ${download.fileName || download.id}`);
      } else if (download.status === 'paused') {
        await DownloadService.resumeDownload(id);
        await addUserInteraction(`Resumed download: ${download.fileName || download.id}`);
      }
      
      // Refresh data to show updated status
      await loadData();
    } catch (error) {
      console.error('Failed to pause/resume download:', error);
    }
  };

  const handleCancel = async (id: string) => {
    try {
      const download = downloads.find(d => d.id === id);
      const fileName = download?.fileName || 'Unknown file';
      
      // Show confirmation dialog with delete warning
      const shouldCancel = await confirm(
        `Are you sure you want to cancel the download of "${fileName}"?\n\n⚠️ Warning: This will permanently delete the download progress and any partially downloaded file.`,
        {
          title: 'Cancel Download',
          kind: 'warning'
        }
      );
      
      if (!shouldCancel) {
        return; // User chose not to cancel
      }
      
      await DownloadService.cancelAndDeleteDownload(id);
      await addUserInteraction(`Cancelled download: ${fileName}`);
      // Refresh data to show updated status
      await loadData();
    } catch (error) {
      console.error('Failed to cancel download:', error);
    }
  };

  const handleRestart = async (id: string) => {
    try {
      const download = downloads.find(d => d.id === id);
      await DownloadService.restartDownload(id);
      await addUserInteraction(`Restarted download: ${download?.fileName || id}`);
      // Refresh data to show updated status
      await loadData();
    } catch (error) {
      console.error('Failed to restart download:', error);
    }
  };

  const handleClearCompleted = async () => {
    try {
      const completedCount = downloads.filter(d => d.status === 'completed').length;
      await DownloadService.clearCompletedDownloads();
      await addUserInteraction(`Cleared ${completedCount} completed downloads`);
      // Refresh data to show updated list
      await loadData();
    } catch (error) {
      console.error('Failed to clear completed downloads:', error);
    }
  };

  const handleCheckStalledDownloads = async () => {
    try {
      console.log('Checking for stalled downloads...');
      await invoke('check_and_fix_stalled_downloads');
      await addUserInteraction('Checked and fixed stalled downloads');
      // Refresh data to show updated list
      await loadData();
    } catch (error) {
      console.error('Failed to check stalled downloads:', error);
    }
  };

  const handleOpenLocation = async (filePath: string) => {
    try {
      await DownloadService.openDownloadLocation(filePath);
    } catch (error) {
      console.error('Failed to open download location:', error);
    }
  };

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('asc');
    }
  };

  const handleSelectDownload = (id: string) => {
    setSelectedDownloads(prev => {
      const newSet = new Set(prev);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return newSet;
    });
  };

  const handleSelectAll = () => {
    if (selectedDownloads.size === filteredDownloads.length) {
      setSelectedDownloads(new Set());
    } else {
      setSelectedDownloads(new Set(filteredDownloads.map(d => d.id)));
    }
  };

  const handleBulkAction = async (action: 'pause' | 'resume' | 'cancel') => {
    try {
      const downloadIds = Array.from(selectedDownloads);
      const count = downloadIds.length;
      
      switch (action) {
        case 'pause':
          await DownloadService.bulkPauseDownloads(downloadIds);
          await addUserInteraction(`Bulk paused ${count} downloads`);
          break;
        case 'resume':
          await DownloadService.bulkResumeDownloads(downloadIds);
          await addUserInteraction(`Bulk resumed ${count} downloads`);
          break;
        case 'cancel': {
          // Show confirmation dialog with delete warning for bulk cancel
          const shouldCancel = await confirm(
            `Are you sure you want to cancel ${count} selected download${count > 1 ? 's' : ''}?\n\n⚠️ Warning: This will permanently delete the download progress and any partially downloaded files for all selected downloads.`,
            {
              title: 'Cancel Downloads',
              kind: 'warning'
            }
          );
          
          if (!shouldCancel) {
            return; // User chose not to cancel
          }
          
          await DownloadService.bulkCancelAndDeleteDownloads(downloadIds);
          await addUserInteraction(`Bulk cancelled ${count} downloads`);
          break;
        }
      }
      
      setSelectedDownloads(new Set());
      // Refresh data to show updated statuses
      await loadData();
    } catch (error) {
      console.error('Failed to perform bulk action:', error);
    }
  };

  // Settings modal handlers
  const handleSaveSettings = async () => {
    const settingsToSave = {
      speedLimit: tempSpeedLimit,
      divideSpeedEnabled: tempDivideSpeedEnabled,
      maxSimultaneousDownloads: tempMaxSimultaneousDownloads,
      disableRangeRequests: tempDisableRangeRequests
    };
    console.log('💾 User clicked Save Settings button. Saving:', settingsToSave);
    
    try {
      await updateGlobalSettings(settingsToSave);
      console.log('✅ Settings saved successfully, closing modal');
      setShowSettingsModal(false);
      await addUserInteraction(`Updated settings: speed limit ${tempSpeedLimit === 0 ? 'unlimited' : `${tempSpeedLimit} MB/s`}, divide speed ${tempDivideSpeedEnabled ? 'enabled' : 'disabled'}, max downloads ${tempMaxSimultaneousDownloads}, range requests ${tempDisableRangeRequests ? 'disabled' : 'enabled'}`);
    } catch (error) {
      console.error('❌ Failed to save settings:', error);
    }
  };

  const handleCancelSettings = () => {
    setTempSpeedLimit(globalSettings.speedLimit);
    setTempDivideSpeedEnabled(globalSettings.divideSpeedEnabled);
    setTempMaxSimultaneousDownloads(globalSettings.maxSimultaneousDownloads);
    setTempDisableRangeRequests(globalSettings.disableRangeRequests);
    setShowSettingsModal(false);
  };

  // Filter and sort downloads
  const filteredDownloads = downloads
    .filter(download => {
      // Add null/undefined checks for fileName
      const fileName = download.fileName || '';
      const matchesSearch = fileName.toLowerCase().includes(searchTerm.toLowerCase());
      const matchesFilter = filterStatus === 'all' || download.status === filterStatus;
      return matchesSearch && matchesFilter;
    })
    .sort((a, b) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      let aValue: any, bValue: any;

      switch (sortField) {
        case 'fileName':
          aValue = (a.fileName || '').toLowerCase();
          bValue = (b.fileName || '').toLowerCase();
          break;
        case 'progress':
          aValue = a.progress;
          bValue = b.progress;
          break;
        case 'size':
          aValue = a.totalSize;
          bValue = b.totalSize;
          break;
        case 'speed':
          aValue = a.speed;
          bValue = b.speed;
          break;
        case 'status':
          aValue = a.status;
          bValue = b.status;
          break;
        case 'startTime':
          aValue = a.startTime;
          bValue = b.startTime;
          break;
        default:
          return 0;
      }

      if (aValue < bValue) return sortDirection === 'asc' ? -1 : 1;
      if (aValue > bValue) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });

  if (!isOpen) return null;

  return (
    <div className={`fixed inset-0 bg-gray-900 z-50 ${isResizing ? 'cursor-col-resize' : ''}`}>
      <div className="w-full h-full flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700" data-tauri-drag-region>
          <div className="flex items-center space-x-3">
            <Download className="w-6 h-6 text-blue-500" />
            <h2 className="text-xl font-bold text-white">Download Manager</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Stats */}
        <div className="p-6 border-b border-gray-700">
          <div className="grid grid-cols-2 md:grid-cols-5 gap-5">
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Total Downloads</div>
              <div className="text-lg font-bold text-white">{stats.total_downloads}</div>
            </div>
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Completed</div>
              <div className="text-lg font-bold text-green-400">{stats.completed_downloads}</div>
            </div>
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Active</div>
              <div className="text-lg font-bold text-blue-400">{stats.active_downloads}</div>
            </div>
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Total Downloaded</div>
              <div className="text-lg font-bold text-white">{formatFileSize(stats.total_downloaded_size)}</div>
            </div>
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Average Speed</div>
              <div className="text-lg font-bold text-white">{formatSpeed(stats.average_speed)}</div>
            </div>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-gray-700">
          <button
            onClick={() => setActiveTab('active')}
            className={`px-6 py-3 font-medium transition-colors ${activeTab === 'active'
              ? 'text-blue-400 border-b-2 border-blue-400'
              : 'text-gray-400 hover:text-white'
              }`}
          >
            Active Downloads
          </button>
          <button
            onClick={() => setActiveTab('activity')}
            className={`px-6 py-3 font-medium transition-colors ${activeTab === 'activity'
              ? 'text-blue-400 border-b-2 border-blue-400'
              : 'text-gray-400 hover:text-white'
              }`}
          >
            Activity
          </button>
        </div>

        {/* Controls */}
        {activeTab === 'active' && (
          <div className="p-4 border-b border-gray-700">
            <div className="flex flex-col md:flex-row gap-4 items-center justify-between">
              <div className="flex flex-1 gap-4">
                {/* Search */}
                <div className="relative flex-1 max-w-md">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                  <input
                    type="text"
                    placeholder="Search downloads..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="w-full pl-10 pr-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                  />
                </div>

                {/* Filter */}
                <div className="relative">
                  <Filter className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                  <select
                    value={filterStatus}
                    onChange={(e) => setFilterStatus(e.target.value as FilterStatus)}
                    className="pl-10 pr-8 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:border-blue-500 appearance-none"
                  >
                    <option value="all">All Status</option>
                    <option value="downloading">Downloading</option>
                    <option value="paused">Paused</option>
                    <option value="queued">Queued</option>
                    <option value="completed">Completed</option>
                    <option value="error">Error</option>
                  </select>
                </div>
              </div>

              {/* Bulk Actions */}
              {selectedDownloads.size > 0 && (
                <div className="flex gap-2">
                  <button
                    onClick={() => handleBulkAction('pause')}
                    className="px-3 py-2 bg-yellow-600 hover:bg-yellow-700 text-white rounded-lg transition-colors text-sm"
                  >
                    Pause Selected
                  </button>
                  <button
                    onClick={() => handleBulkAction('resume')}
                    className="px-3 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors text-sm"
                  >
                    Resume Selected
                  </button>
                  <button
                    onClick={() => handleBulkAction('cancel')}
                    className="px-3 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors text-sm"
                  >
                    Cancel Selected
                  </button>
                </div>
              )}

              {/* Action Buttons */}
              <div className="flex gap-2">
                <button
                  onClick={() => setShowAddModal(true)}
                  className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors flex items-center gap-2"
                >
                  <Plus className="w-4 h-4" />
                  Add New Download
                </button>
                <button
                  onClick={() => setShowSettingsModal(true)}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors flex items-center gap-2"
                >
                  <Settings className="w-4 h-4" />
                  Settings
                </button>
                <button
                  onClick={handleCheckStalledDownloads}
                  className="px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg transition-colors flex items-center gap-2"
                  title="Check for downloads that appear stalled but are actually complete"
                >
                  <RefreshCw className="w-4 h-4" />
                  Fix Stalled
                </button>
                <button
                  onClick={handleClearCompleted}
                  className="px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors flex items-center gap-2"
                >
                  <Trash2 className="w-4 h-4" />
                  Clear Completed
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-hidden">
          {activeTab === 'active' ? (
            <div className="h-full flex flex-col">
              {/* Table Header */}
              <div className="bg-gray-700 border-b border-gray-600">
                {/* Column Width Reset Button */}
                <div className="px-4 pt-2 pb-1 border-b border-gray-600">
                  <button
                    onClick={resetColumnWidths}
                    className="text-xs text-blue-400 hover:text-blue-300 transition-colors"
                    title="Reset column widths to default"
                  >
                    Reset Column Widths
                  </button>
                </div>
                <div className="grid gap-4 p-4 text-sm font-medium text-gray-300 relative" style={{gridTemplateColumns: getGridTemplateColumns()}}>
                  {/* Resizing indicator */}
                  {isResizing && resizingColumn && (
                    <div className="absolute top-0 left-0 right-0 bottom-0 bg-blue-500 bg-opacity-10 pointer-events-none" />
                  )}
                  <div className="flex items-center">
                    <input
                      type="checkbox"
                      checked={selectedDownloads.size === filteredDownloads.length && filteredDownloads.length > 0}
                      onChange={handleSelectAll}
                      className="w-4 h-4 text-blue-600 bg-gray-600 border-gray-500 rounded focus:ring-blue-500"
                    />
                  </div>
                  <div className="flex items-center gap-2 cursor-pointer relative" onClick={() => handleSort('fileName')}>
                    File Name
                    <ArrowUpDown className="w-3 h-3" />
                    <div 
                      className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors"
                      onMouseDown={(e) => handleMouseDown(e, 'fileName')}
                      title="Drag to resize column"
                    />
                  </div>
                  <div className="flex items-center gap-2 cursor-pointer relative" onClick={() => handleSort('size')}>
                    Size
                    <ArrowUpDown className="w-3 h-3" />
                    <div 
                      className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors"
                      onMouseDown={(e) => handleMouseDown(e, 'size')}
                      title="Drag to resize column"
                    />
                  </div>
                  <div className="flex items-center gap-2 cursor-pointer relative" onClick={() => handleSort('progress')}>
                    Progress
                    <ArrowUpDown className="w-3 h-3" />
                    <div 
                      className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors"
                      onMouseDown={(e) => handleMouseDown(e, 'progress')}
                      title="Drag to resize column"
                    />
                  </div>
                  <div className="flex items-center gap-2 cursor-pointer relative" onClick={() => handleSort('speed')}>
                    Speed
                    <ArrowUpDown className="w-3 h-3" />
                    <div 
                      className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors"
                      onMouseDown={(e) => handleMouseDown(e, 'speed')}
                      title="Drag to resize column"
                    />
                  </div>
                  <div className="flex items-center gap-2 cursor-pointer relative" onClick={() => handleSort('status')}>
                    Status
                    <ArrowUpDown className="w-3 h-3" />
                    <div 
                      className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors"
                      onMouseDown={(e) => handleMouseDown(e, 'status')}
                      title="Drag to resize column"
                    />
                  </div>
                  <div className="flex items-center gap-2 cursor-pointer relative" onClick={() => handleSort('startTime')}>
                    Started
                    <ArrowUpDown className="w-3 h-3" />
                    <div 
                      className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors"
                      onMouseDown={(e) => handleMouseDown(e, 'started')}
                      title="Drag to resize column"
                    />
                  </div>
                  <div>Actions</div>
                </div>
              </div>

              {/* Table Body */}
              <div className="flex-1 overflow-y-auto">
                {filteredDownloads.length === 0 ? (
                  <div className="flex items-center justify-center h-full">
                    <div className="text-center">
                      <Download className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                      <p className="text-gray-400 text-lg">No downloads found</p>
                      <p className="text-gray-500 text-sm">Downloads will appear here when you start downloading files</p>
                    </div>
                  </div>
                ) : (
                  filteredDownloads.map((download) => (
                    <div key={download.id} className="grid gap-4 p-4 border-b border-gray-700 hover:bg-gray-750 transition-colors" style={{gridTemplateColumns: getGridTemplateColumns()}}>
                      <div className="flex items-center">
                        <input
                          type="checkbox"
                          checked={selectedDownloads.has(download.id)}
                          onChange={() => handleSelectDownload(download.id)}
                          className="w-4 h-4 text-blue-600 bg-gray-600 border-gray-500 rounded focus:ring-blue-500"
                        />
                      </div>
                      <div className="flex items-center gap-3">
                        <span className="text-2xl">{getFileIcon(download.fileExtension)}</span>
                        <div>
                          <div className="text-white font-medium truncate">{download.fileName}</div>
                          <div className="text-gray-400 text-sm">{download.fileExtension || 'Unknown'}</div>
                        </div>
                      </div>
                      <div className="flex items-center">
                        <div>
                          {download.totalSize > 0 ? (
                            <>
                              <div className="text-white">{formatFileSize(download.totalSize)}</div>
                              <div className="text-gray-400 text-sm">{formatFileSize(download.downloadedSize)} downloaded</div>
                            </>
                          ) : (
                            <>
                              <div className="text-white">Unknown size</div>
                              <div className="text-gray-400 text-sm">{formatFileSize(download.downloadedSize)} downloaded</div>
                            </>
                          )}
                        </div>
                      </div>
                      <div className="flex items-center">
                        <div className="w-full">
                          {download.totalSize > 0 ? (
                            <>
                              <div className="flex justify-between text-sm mb-1">
                                <span className="text-white">{download.status === 'completed' ? '100.0' : download.progress.toFixed(1)}%</span>
                                <span className="text-gray-400">{formatTime(download.timeRemaining)}</span>
                              </div>
                              <div className="w-full bg-gray-600 rounded-full h-2">
                                <div
                                  className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                                  style={{ width: `${download.status === 'completed' ? 100 : download.progress}%` }}
                                />
                              </div>
                            </>
                          ) : (
                            <>
                              <div className="flex justify-between text-sm mb-1">
                                <span className="text-white">{formatFileSize(download.downloadedSize)}</span>
                                <span className="text-gray-400">Unknown time</span>
                              </div>
                              <div className="w-full bg-gray-600 rounded-full h-2">
                                <div className="bg-blue-500 h-2 rounded-full transition-all duration-300 animate-pulse" style={{ width: '100%' }} />
                              </div>
                            </>
                          )}
                        </div>
                      </div>
                      <div className="flex items-center">
                        <div className="text-white">{formatSpeed(download.speed)}</div>
                      </div>
                      <div className="flex items-center">
                        <div className="flex items-center gap-2">
                          {getStatusIcon(download.status)}
                          <span className="text-sm text-gray-300 capitalize">{download.status}</span>
                        </div>
                      </div>
                      <div className="flex items-center">
                        <div>
                          <div className="text-white text-sm">{download.startTime ? new Date(download.startTime * 1000).toLocaleDateString() : 'Unknown'}</div>
                          <div className="text-gray-400 text-xs">{download.startTime ? new Date(download.startTime * 1000).toLocaleTimeString() : '--'}</div>
                        </div>
                      </div>
                      <div className="flex items-center gap-1">
                        {download.status === 'downloading' && (
                          <button
                            onClick={() => handlePauseResume(download.id)}
                            className="p-1 text-yellow-400 hover:text-yellow-300 transition-colors"
                            title="Pause"
                          >
                            <Pause className="w-4 h-4" />
                          </button>
                        )}
                        {download.status === 'paused' && (
                          <button
                            onClick={() => handlePauseResume(download.id)}
                            className="p-1 text-green-400 hover:text-green-300 transition-colors"
                            title="Resume"
                          >
                            <Play className="w-4 h-4" />
                          </button>
                        )}
                        {(download.status === 'error' || download.status === 'cancelled') && (
                          <button
                            onClick={() => handleRestart(download.id)}
                            className="p-1 text-blue-400 hover:text-blue-300 transition-colors"
                            title="Restart"
                          >
                            <RotateCcw className="w-4 h-4" />
                          </button>
                        )}
                        {download.status !== 'completed' && (
                          <button
                            onClick={() => handleCancel(download.id)}
                            className="p-1 text-red-400 hover:text-red-300 transition-colors"
                            title="Cancel"
                          >
                            <X className="w-4 h-4" />
                          </button>
                        )}
                        <button
                          onClick={() => handleOpenLocation(download.filePath)}
                          className="p-1 text-gray-400 hover:text-gray-300 transition-colors"
                          title="Open Location"
                        >
                          <FolderOpen className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          ) : (
            /* Activity Tab */
            <div className="h-full flex flex-col">
              {/* Activity Controls */}
              <div className="p-4 border-b border-gray-700">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-medium text-white">Activity Log</h3>
                  <button
                    onClick={async () => {
                      const confirmed = await confirm('Are you sure you want to clear all activity entries? This action cannot be undone.', {
                        title: 'Clear All Activities',
                        kind: 'warning'
                      });
                      if (confirmed) {
                        clearActivities();
                        addUserInteraction('Clear Activities Confirmed', 'User confirmed clearing all activity entries');
                      } else {
                        addUserInteraction('Clear Activities Cancelled', 'User cancelled clearing activity entries');
                      }
                    }}
                    className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors flex items-center gap-2"
                  >
                    <Trash2 className="w-4 h-4" />
                    Clear All
                  </button>
                </div>
              </div>
              
              {/* Activity List */}
              <div className="flex-1 overflow-y-auto">
                {activities.length === 0 ? (
                  <div className="flex items-center justify-center h-full">
                    <div className="text-center">
                      <Clock className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                      <p className="text-gray-400 text-lg">No activity recorded</p>
                      <p className="text-gray-500 text-sm">User actions and download events will appear here</p>
                    </div>
                  </div>
                ) : (
                  <div className="p-4">
                    {activities.map((activity) => {
                      const getActivityIcon = (actionType: string) => {
                        switch (actionType) {
                          case 'DownloadStarted': return <Download className="w-4 h-4 text-blue-500" />;
                          case 'DownloadCompleted': return <CheckCircle className="w-4 h-4 text-green-500" />;
                          case 'DownloadPaused': return <Pause className="w-4 h-4 text-yellow-500" />;
                          case 'DownloadResumed': return <Play className="w-4 h-4 text-green-500" />;
                          case 'DownloadCancelled': return <X className="w-4 h-4 text-red-500" />;
                          case 'DownloadError': return <AlertCircle className="w-4 h-4 text-red-500" />;
                          case 'FileAdded': return <Plus className="w-4 h-4 text-blue-500" />;
                          case 'StatusChanged': return <RotateCcw className="w-4 h-4 text-orange-500" />;
                          case 'UserInteraction': return <Clock className="w-4 h-4 text-purple-500" />;
                          default: return <Clock className="w-4 h-4 text-gray-500" />;
                        }
                      };
                      
                      const getActivityColor = (actionType: string) => {
                        switch (actionType) {
                          case 'DownloadStarted': return 'border-l-blue-500';
                          case 'DownloadCompleted': return 'border-l-green-500';
                          case 'DownloadPaused': return 'border-l-yellow-500';
                          case 'DownloadResumed': return 'border-l-green-500';
                          case 'DownloadCancelled': return 'border-l-red-500';
                          case 'DownloadError': return 'border-l-red-500';
                          case 'FileAdded': return 'border-l-blue-500';
                          case 'StatusChanged': return 'border-l-orange-500';
                          case 'UserInteraction': return 'border-l-purple-500';
                          default: return 'border-l-gray-500';
                        }
                      };
                      
                      return (
                        <div key={activity.id} className={`bg-gray-700 rounded-lg p-4 mb-3 border-l-4 ${getActivityColor(activity.actionType)}`}>
                          <div className="flex items-start gap-3">
                            <div className="flex-shrink-0 mt-1">
                              {getActivityIcon(activity.actionType)}
                            </div>
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center justify-between mb-1">
                                <div className="text-white font-medium">
                                  {activity.actionType.replace(/([A-Z])/g, ' $1').trim()}
                                </div>
                                <div className="text-gray-400 text-xs">
                                  {new Date(activity.timestamp).toLocaleString()}
                                </div>
                              </div>
                              {activity.fileName && (
                                <div className="text-gray-300 text-sm mb-1">
                                  <span className="font-medium">File:</span> {activity.fileName}
                                </div>
                              )}
                              {activity.identifier && (
                                <div className="text-gray-300 text-sm mb-1">
                                  <span className="font-medium">ID:</span> {activity.identifier}
                                </div>
                              )}
                              {activity.status && (
                                <div className="text-gray-300 text-sm mb-1">
                                  <span className="font-medium">Status:</span> {activity.status}
                                </div>
                              )}
                              {activity.details && (
                                <div className="text-gray-400 text-sm">
                                  {activity.details}
                                </div>
                              )}
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Add Download Modal */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
            {/* Modal Header */}
            <div className="flex items-center justify-between p-6 border-b border-gray-700">
              <h3 className="text-xl font-semibold text-white flex items-center gap-2">
                <Plus className="w-6 h-6" />
                Add New Download
              </h3>
              <button
                onClick={() => {
                  setShowAddModal(false);
                  setNewDownloadUrl('');
                  setNewDownloadFolder('');
                  setUrlError('');
                  setFolderError('');
                }}
                className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* Modal Body */}
            <div className="p-6">
              <div className="space-y-6">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Download URL *
                  </label>
                  <input
                    type="url"
                    value={newDownloadUrl}
                    onChange={(e) => {
                      setNewDownloadUrl(e.target.value);
                      if (urlError) setUrlError('');
                    }}
                    placeholder="https://example.com/file.zip"
                    className={`w-full px-4 py-3 bg-gray-700 border rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:ring-opacity-20 ${urlError ? 'border-red-500' : 'border-gray-600'
                      }`}
                  />
                  {urlError && (
                    <p className="text-red-400 text-sm mt-2 flex items-center gap-1">
                      <AlertCircle className="w-4 h-4" />
                      {urlError}
                    </p>
                  )}
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Destination Folder *
                  </label>
                  <div className="flex gap-3">
                    <input
                      type="text"
                      value={newDownloadFolder}
                      onChange={(e) => {
                        setNewDownloadFolder(e.target.value);
                        if (folderError) setFolderError('');
                      }}
                      placeholder="C:\\Downloads"
                      className={`flex-1 px-4 py-3 bg-gray-700 border rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:ring-opacity-20 ${folderError ? 'border-red-500' : 'border-gray-600'
                        }`}
                    />
                    <button
                      onClick={selectDownloadFolder}
                      className="px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors flex items-center gap-2 whitespace-nowrap"
                    >
                      <FolderOpen className="w-4 h-4" />
                      Browse
                    </button>
                  </div>
                  {folderError && (
                    <p className="text-red-400 text-sm mt-2 flex items-center gap-1">
                      <AlertCircle className="w-4 h-4" />
                      {folderError}
                    </p>
                  )}
                </div>




              </div>
            </div>

            {/* Modal Footer */}
            <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-700">
              <button
                onClick={() => {
                  setShowAddModal(false);
                  setNewDownloadUrl('');
                  setNewDownloadFolder('');
                  setUrlError('');
                  setFolderError('');
                }}
                className="px-6 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleAddDownload}
                disabled={isAddingDownload || !newDownloadUrl.trim() || !newDownloadFolder.trim()}
                className="px-6 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg transition-colors flex items-center gap-2"
              >
                {isAddingDownload ? (
                  <>
                    <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    Adding...
                  </>
                ) : (
                  <>
                    <Download className="w-4 h-4" />
                    Add Download
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Settings Modal */}
      {showSettingsModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4">
            {/* Modal Header */}
            <div className="flex items-center justify-between p-6 border-b border-gray-700">
              <h3 className="text-xl font-semibold text-white flex items-center gap-2">
                <Settings className="w-6 h-6" />
                Download Settings
              </h3>
              <button
                onClick={handleCancelSettings}
                className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* Modal Body */}
            <div className="p-6">
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Download Speed Limit (MB/s)
                  </label>
                  <input
                    type="number"
                    min="0"
                    step="0.1"
                    value={tempSpeedLimit}
                    onChange={(e) => {
                      const value = e.target.value;
                      
                      if (value === '' || value === '.') {
                        setTempSpeedLimit(0);
                      } else {
                        const numValue = parseFloat(value);
                        if (!isNaN(numValue) && numValue >= 0) {
                          setTempSpeedLimit(numValue);
                        }
                      }
                    }}
                    placeholder="0 = Unlimited"
                    className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:ring-opacity-20"
                  />
                  <p className="text-gray-400 text-sm mt-2">
                    Set to 0 for unlimited speed. Current: {globalSettings.speedLimit === 0 ? 'Unlimited' : `${globalSettings.speedLimit} MB/s`}
                  </p>
                </div>
                
                <div>
                  <label className="flex items-center space-x-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={tempDivideSpeedEnabled}
                      onChange={(e) => setTempDivideSpeedEnabled(e.target.checked)}
                      className="w-5 h-5 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500 focus:ring-2"
                    />
                    <div>
                      <span className="text-sm font-medium text-gray-300">
                        Divide Speed Among Downloads
                      </span>
                      <p className="text-gray-400 text-xs mt-1">
                        When enabled, the speed limit will be divided equally among all active downloads.
                        For example: 2MB limit with 2 downloads = 1MB per download.
                      </p>
                    </div>
                  </label>
                  <p className="text-gray-400 text-sm mt-2">
                    Current: {globalSettings.divideSpeedEnabled ? 'Enabled' : 'Disabled'}
                  </p>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Max Simultaneous Downloads
                  </label>
                  <input
                    type="number"
                    min="1"
                    max="10"
                    value={tempMaxSimultaneousDownloads}
                    onChange={(e) => {
                      const value = parseInt(e.target.value);
                      if (!isNaN(value) && value >= 1 && value <= 10) {
                        setTempMaxSimultaneousDownloads(value);
                      }
                    }}
                    className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:ring-opacity-20"
                  />
                  <p className="text-gray-400 text-sm mt-2">
                    Maximum number of downloads that can run simultaneously (1-10). Current: {globalSettings.maxSimultaneousDownloads}
                  </p>
                </div>
                
                <div>
                  <label className="flex items-center space-x-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={tempDisableRangeRequests}
                      onChange={(e) => setTempDisableRangeRequests(e.target.checked)}
                      className="w-5 h-5 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500 focus:ring-2"
                    />
                    <div>
                      <span className="text-sm font-medium text-gray-300">
                        Disable Range Requests (Direct Download)
                      </span>
                      <p className="text-gray-400 text-xs mt-1">
                        When enabled, downloads will always start from the beginning without using HTTP Range headers.
                        This can help resolve file corruption issues but prevents resuming interrupted downloads.
                      </p>
                    </div>
                  </label>
                  <p className="text-gray-400 text-sm mt-2">
                    Current: {globalSettings.disableRangeRequests ? 'Disabled (Direct Download)' : 'Enabled (Resumable)'}
                  </p>
                </div>
              </div>
            </div>

            {/* Modal Footer */}
            <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-700">
              <button
                onClick={handleCancelSettings}
                className="px-6 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveSettings}
                className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors flex items-center gap-2"
              >
                <Settings className="w-4 h-4" />
                Save Settings
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};