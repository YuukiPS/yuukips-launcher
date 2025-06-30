import { useState, useEffect } from 'react';
import { X, Download, RefreshCw } from 'lucide-react';
import { UpdateService, UpdateInfo, DownloadProgress } from '../services/updateService';

interface UpdateModalProps {
  isOpen: boolean;
  onClose: () => void;
  updateInfo: UpdateInfo;
}

export function UpdateModal({ isOpen, onClose, updateInfo }: UpdateModalProps) {
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    // Listen for download progress events
    const handleProgress = (event: CustomEvent<DownloadProgress>) => {
      setDownloadProgress(event.detail);
    };
    
    window.addEventListener('updateDownloadProgress', handleProgress as EventListener);
    
    return () => {
      window.removeEventListener('updateDownloadProgress', handleProgress as EventListener);
    };
  }, []);
  
  const handleDownload = async () => {
    if (!updateInfo.downloadUrl) {
      setError('Download URL not available');
      return;
    }
    
    try {
      setIsDownloading(true);
      setError(null);
      
      await UpdateService.downloadAndInstallUpdate(
        updateInfo.downloadUrl,
        (progress) => setDownloadProgress(progress)
      );
      
      // After successful download and installation, restart the app
      await UpdateService.restartApplication();
    } catch (err) {
      setError(`Update failed: ${err}`);
      setIsDownloading(false);
    }
  };
  
  if (!isOpen) return null;
  
  // Format file size
  const formatFileSize = (bytes?: number): string => {
    if (!bytes) return 'Unknown size';
    
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(2)} ${units[unitIndex]}`;
  };
  
  // Format download speed
  const formatSpeed = (bytesPerSecond: number): string => {
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    let speed = bytesPerSecond;
    let unitIndex = 0;
    
    while (speed >= 1024 && unitIndex < units.length - 1) {
      speed /= 1024;
      unitIndex++;
    }
    
    return `${speed.toFixed(2)} ${units[unitIndex]}`;
  };
  
  // Format release notes (convert markdown to simple HTML)
  const formatReleaseNotes = (notes?: string): string => {
    if (!notes) return '';
    
    // Simple markdown to HTML conversion
    return notes
      .replace(/\r\n/g, '\n')
      .replace(/\n\n/g, '</p><p>')
      .replace(/\n/g, '<br>')
      .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.*?)\*/g, '<em>$1</em>')
      .replace(/^### (.*?)$/gm, '<h3>$1</h3>')
      .replace(/^## (.*?)$/gm, '<h2>$1</h2>')
      .replace(/^# (.*?)$/gm, '<h1>$1</h1>');
  };
  
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-lg w-full max-w-md mx-4 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between bg-gray-700 px-4 py-3">
          <h2 className="text-xl font-semibold text-white">Update Available</h2>
          <button 
            onClick={onClose} 
            className="text-gray-400 hover:text-white"
            disabled={isDownloading}
          >
            <X size={20} />
          </button>
        </div>
        
        {/* Content */}
        <div className="px-4 py-4">
          <p className="text-gray-300 mb-4">
            A new version (v{updateInfo.latestVersion}) of YuukiPS Launcher is available. 
            Your current version is v{updateInfo.currentVersion}.
          </p>
          
          {updateInfo.assetSize && (
            <p className="text-gray-400 text-sm mb-4">
              Download size: {formatFileSize(updateInfo.assetSize)}
            </p>
          )}
          
          {/* Release Notes */}
          {updateInfo.releaseNotes && (
            <div className="mb-4">
              <h3 className="text-white font-medium mb-2">Release Notes:</h3>
              <div 
                className="bg-gray-900 rounded p-3 text-gray-300 text-sm max-h-40 overflow-y-auto"
                dangerouslySetInnerHTML={{ __html: `<p>${formatReleaseNotes(updateInfo.releaseNotes)}</p>` }}
              />
            </div>
          )}
          
          {/* Download Progress */}
          {isDownloading && downloadProgress && (
            <div className="mb-4">
              <div className="flex justify-between text-sm text-gray-400 mb-1">
                <span>
                  {formatFileSize(downloadProgress.downloaded)} / {formatFileSize(downloadProgress.total)}
                </span>
                <span>{formatSpeed(downloadProgress.speed)}</span>
              </div>
              <div className="w-full bg-gray-700 rounded-full h-2.5">
                <div 
                  className="bg-blue-600 h-2.5 rounded-full" 
                  style={{ width: `${downloadProgress.percentage}%` }}
                />
              </div>
            </div>
          )}
          
          {/* Error Message */}
          {error && (
            <div className="text-red-500 text-sm mb-4 p-2 bg-red-900 bg-opacity-20 rounded">
              {error}
            </div>
          )}
        </div>
        
        {/* Footer */}
        <div className="bg-gray-700 px-4 py-3 flex justify-end space-x-3">
          {!isDownloading && (
            <button
              onClick={onClose}
              className="px-4 py-2 text-gray-300 hover:text-white"
              disabled={isDownloading}
            >
              Remind Me Later
            </button>
          )}
          
          <button
            onClick={handleDownload}
            disabled={isDownloading || !updateInfo.downloadUrl}
            className={`px-4 py-2 rounded flex items-center space-x-2 ${isDownloading 
              ? 'bg-blue-700 text-blue-200 cursor-not-allowed' 
              : 'bg-blue-600 text-white hover:bg-blue-500'}`}
          >
            {isDownloading ? (
              <>
                <RefreshCw size={18} className="animate-spin" />
                <span>Downloading...</span>
              </>
            ) : (
              <>
                <Download size={18} />
                <span>Update Now</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}