import React, { useState, useEffect } from 'react';
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
  Plus
} from 'lucide-react';
import { DownloadItem, DownloadHistory, DownloadStats } from '../types';
import { DownloadService } from '../services/downloadService';
import { open } from '@tauri-apps/plugin-dialog';

interface DownloadManagerProps {
  isOpen: boolean;
  onClose: () => void;
}

type SortField = 'fileName' | 'progress' | 'size' | 'speed' | 'status';
type SortDirection = 'asc' | 'desc';
type FilterStatus = 'all' | 'downloading' | 'paused' | 'completed' | 'error';

export const DownloadManager: React.FC<DownloadManagerProps> = ({ isOpen, onClose }) => {
  const [downloads, setDownloads] = useState<DownloadItem[]>([]);
  const [history, setHistory] = useState<DownloadHistory[]>([]);
  const [stats, setStats] = useState<DownloadStats>({
    totalDownloads: 0,
    activeDownloads: 0,
    completedDownloads: 0,
    totalDownloadedSize: 0,
    averageSpeed: 0
  });
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState<FilterStatus>('all');
  const [sortField, setSortField] = useState<SortField>('fileName');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [selectedDownloads, setSelectedDownloads] = useState<Set<string>>(new Set());
  const [activeTab, setActiveTab] = useState<'active' | 'history'>('active');

  // New download form state
  const [newDownloadUrl, setNewDownloadUrl] = useState('');
  const [newDownloadFolder, setNewDownloadFolder] = useState('');
  const [isAddingDownload, setIsAddingDownload] = useState(false);
  const [urlError, setUrlError] = useState('');
  const [folderError, setFolderError] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [skipUrlCheck, setSkipUrlCheck] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadData();
      loadDefaultDownloadFolder();
      
      // Set up polling for download updates
      const interval = setInterval(loadData, 1000);
      return () => clearInterval(interval);
    }
  }, [isOpen]);

  const loadData = async () => {
    try {
      const [activeDownloads, downloadHistory, downloadStats] = await Promise.all([
        DownloadService.getActiveDownloads(),
        DownloadService.getDownloadHistory(),
        DownloadService.getDownloadStats()
      ]);
      
      setDownloads(activeDownloads);
      setHistory(downloadHistory);
      setStats(downloadStats);
    } catch (error) {
      console.error('Failed to load download data:', error);
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



  const validateUrl = (url: string): boolean => {
    try {
      new URL(url);
      return url.startsWith('http://') || url.startsWith('https://');
    } catch {
      return false;
    }
  };

  const handleAddDownload = async () => {
    console.log('[DownloadManager] Starting add download process', {
      url: newDownloadUrl,
      folder: newDownloadFolder
    });
    
    setUrlError('');
    setFolderError('');

    // Validate URL
    if (!newDownloadUrl.trim()) {
      console.log('[DownloadManager] URL validation failed: empty URL');
      setUrlError('URL is required');
      return;
    }

    if (!validateUrl(newDownloadUrl)) {
      console.log('[DownloadManager] URL validation failed: invalid format');
      setUrlError('Please enter a valid HTTP/HTTPS URL');
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
      // Validate URL with backend (with skip option)
      console.log('[DownloadManager] Validating URL with backend:', newDownloadUrl, 'skipUrlCheck:', skipUrlCheck);
      
      if (skipUrlCheck) {
        console.log('[DownloadManager] Skipping URL validation as requested by user');
      } else {
        try {
          const isValidUrl = await DownloadService.validateDownloadUrlWithOptions(newDownloadUrl, false);
          console.log('[DownloadManager] Backend URL validation result:', isValidUrl);
          
          if (!isValidUrl) {
            console.log('[DownloadManager] Backend validation failed - URL not accessible');
            setUrlError('URL is not accessible or invalid. You can try enabling "Skip URL Check" if you\'re sure the URL is correct.');
            setIsAddingDownload(false);
            return;
          }
        } catch (error) {
          console.log('[DownloadManager] URL validation failed with error:', error);
          // If validation fails with connection error, suggest skipping
          if (error instanceof Error && error.message.includes('Connection failed')) {
            setUrlError(error.message);
          } else {
            setUrlError('URL validation failed. You can try enabling "Skip URL Check" if you\'re sure the URL is correct.');
          }
          setIsAddingDownload(false);
          return;
        }
      }

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
        const overwrite = confirm(`File ${fileName} already exists. Do you want to overwrite it?`);
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
        if (error.message.includes('validate')) {
          errorMessage = 'Unable to validate the download URL. Please check the URL and your internet connection.';
        } else if (error.message.includes('network') || error.message.includes('fetch')) {
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
      } else if (download.status === 'paused') {
        await DownloadService.resumeDownload(id);
      }
      
      // Refresh data to show updated status
      await loadData();
    } catch (error) {
      console.error('Failed to pause/resume download:', error);
    }
  };

  const handleCancel = async (id: string) => {
    try {
      await DownloadService.cancelDownload(id);
      // Refresh data to show updated status
      await loadData();
    } catch (error) {
      console.error('Failed to cancel download:', error);
    }
  };

  const handleRestart = async (id: string) => {
    try {
      await DownloadService.restartDownload(id);
      // Refresh data to show updated status
      await loadData();
    } catch (error) {
      console.error('Failed to restart download:', error);
    }
  };

  const handleClearCompleted = async () => {
    try {
      await DownloadService.clearCompletedDownloads();
      // Refresh data to show updated list
      await loadData();
    } catch (error) {
      console.error('Failed to clear completed downloads:', error);
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
      
      switch (action) {
        case 'pause':
          await DownloadService.bulkPauseDownloads(downloadIds);
          break;
        case 'resume':
          await DownloadService.bulkResumeDownloads(downloadIds);
          break;
        case 'cancel':
          await DownloadService.bulkCancelDownloads(downloadIds);
          break;
      }
      
      setSelectedDownloads(new Set());
      // Refresh data to show updated statuses
      await loadData();
    } catch (error) {
      console.error('Failed to perform bulk action:', error);
    }
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
        default:
          return 0;
      }

      if (aValue < bValue) return sortDirection === 'asc' ? -1 : 1;
      if (aValue > bValue) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-gray-900 z-50">
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
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Total Downloads</div>
              <div className="text-lg font-bold text-white">{stats.totalDownloads}</div>
            </div>
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Completed</div>
              <div className="text-lg font-bold text-green-400">{stats.completedDownloads}</div>
            </div>
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Active</div>
              <div className="text-lg font-bold text-blue-400">{stats.activeDownloads}</div>
            </div>
            <div className="bg-gray-700 rounded-lg p-3">
              <div className="text-sm text-gray-400">Total Downloaded</div>
              <div className="text-lg font-bold text-white">{formatFileSize(stats.totalDownloadedSize)}</div>
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
            onClick={() => setActiveTab('history')}
            className={`px-6 py-3 font-medium transition-colors ${activeTab === 'history'
              ? 'text-blue-400 border-b-2 border-blue-400'
              : 'text-gray-400 hover:text-white'
              }`}
          >
            History
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
                <div className="grid grid-cols-12 gap-4 p-4 text-sm font-medium text-gray-300">
                  <div className="col-span-1 flex items-center">
                    <input
                      type="checkbox"
                      checked={selectedDownloads.size === filteredDownloads.length && filteredDownloads.length > 0}
                      onChange={handleSelectAll}
                      className="w-4 h-4 text-blue-600 bg-gray-600 border-gray-500 rounded focus:ring-blue-500"
                    />
                  </div>
                  <div className="col-span-3 flex items-center gap-2 cursor-pointer" onClick={() => handleSort('fileName')}>
                    File Name
                    <ArrowUpDown className="w-3 h-3" />
                  </div>
                  <div className="col-span-2 flex items-center gap-2 cursor-pointer" onClick={() => handleSort('size')}>
                    Size
                    <ArrowUpDown className="w-3 h-3" />
                  </div>
                  <div className="col-span-2 flex items-center gap-2 cursor-pointer" onClick={() => handleSort('progress')}>
                    Progress
                    <ArrowUpDown className="w-3 h-3" />
                  </div>
                  <div className="col-span-2 flex items-center gap-2 cursor-pointer" onClick={() => handleSort('speed')}>
                    Speed
                    <ArrowUpDown className="w-3 h-3" />
                  </div>
                  <div className="col-span-1 flex items-center gap-2 cursor-pointer" onClick={() => handleSort('status')}>
                    Status
                    <ArrowUpDown className="w-3 h-3" />
                  </div>
                  <div className="col-span-1">Actions</div>
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
                    <div key={download.id} className="grid grid-cols-12 gap-4 p-4 border-b border-gray-700 hover:bg-gray-750 transition-colors">
                      <div className="col-span-1 flex items-center">
                        <input
                          type="checkbox"
                          checked={selectedDownloads.has(download.id)}
                          onChange={() => handleSelectDownload(download.id)}
                          className="w-4 h-4 text-blue-600 bg-gray-600 border-gray-500 rounded focus:ring-blue-500"
                        />
                      </div>
                      <div className="col-span-3 flex items-center gap-3">
                        <span className="text-2xl">{getFileIcon(download.fileExtension)}</span>
                        <div>
                          <div className="text-white font-medium truncate">{download.fileName}</div>
                          <div className="text-gray-400 text-sm">{download.fileExtension || 'Unknown'}</div>
                        </div>
                      </div>
                      <div className="col-span-2 flex items-center">
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
                      <div className="col-span-2 flex items-center">
                        <div className="w-full">
                          {download.totalSize > 0 ? (
                            <>
                              <div className="flex justify-between text-sm mb-1">
                                <span className="text-white">{download.progress.toFixed(1)}%</span>
                                <span className="text-gray-400">{formatTime(download.timeRemaining)}</span>
                              </div>
                              <div className="w-full bg-gray-600 rounded-full h-2">
                                <div
                                  className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                                  style={{ width: `${download.progress}%` }}
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
                      <div className="col-span-2 flex items-center">
                        <div className="text-white">{formatSpeed(download.speed)}</div>
                      </div>
                      <div className="col-span-1 flex items-center">
                        <div className="flex items-center gap-2">
                          {getStatusIcon(download.status)}
                          <span className="text-sm text-gray-300 capitalize">{download.status}</span>
                        </div>
                      </div>
                      <div className="col-span-1 flex items-center gap-1">
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
            /* History Tab */
            <div className="h-full overflow-y-auto">
              {history.length === 0 ? (
                <div className="flex items-center justify-center h-full">
                  <div className="text-center">
                    <Clock className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                    <p className="text-gray-400 text-lg">No download history</p>
                    <p className="text-gray-500 text-sm">Completed downloads will appear here</p>
                  </div>
                </div>
              ) : (
                <div className="p-4">
                  {history.map((item) => (
                    <div key={item.id} className="bg-gray-700 rounded-lg p-4 mb-3 flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <span className="text-2xl">📄</span>
                        <div>
                          <div className="text-white font-medium">{item.fileName}</div>
                          <div className="text-gray-400 text-sm">
                            {formatFileSize(item.fileSize)} • {item.downloadDate}
                          </div>
                          {item.errorMessage && (
                            <div className="text-red-400 text-sm">{item.errorMessage}</div>
                          )}
                        </div>
                      </div>
                      <div className="flex items-center gap-3">
                        <div className="flex items-center gap-2">
                          {item.status === 'completed' && <CheckCircle className="w-4 h-4 text-green-500" />}
                          {item.status === 'error' && <AlertCircle className="w-4 h-4 text-red-500" />}
                          {item.status === 'cancelled' && <X className="w-4 h-4 text-gray-500" />}
                          <span className="text-sm text-gray-300 capitalize">{item.status}</span>
                        </div>
                        <button
                          onClick={() => handleOpenLocation(item.filePath)}
                          className="p-2 text-gray-400 hover:text-gray-300 transition-colors"
                          title="Open Location"
                        >
                          <FolderOpen className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
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

                <div className="flex items-center gap-3">
                  <input
                    type="checkbox"
                    id="skipUrlCheck"
                    checked={skipUrlCheck}
                    onChange={(e) => setSkipUrlCheck(e.target.checked)}
                    className="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500 focus:ring-2"
                  />
                  <label htmlFor="skipUrlCheck" className="text-sm text-gray-300 cursor-pointer">
                    Skip URL validation (use if URL validation fails due to connection issues)
                  </label>
                </div>
                <div className="text-xs text-gray-400 bg-gray-700 p-3 rounded-lg">
                  <strong>Note:</strong> Enabling "Skip URL validation" will bypass the initial URL accessibility check. 
                  Use this option if you're experiencing connection issues during validation but are confident the URL is correct. 
                  The download will still fail if the URL is actually invalid.
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
                  setSkipUrlCheck(false);
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
    </div>
  );
};