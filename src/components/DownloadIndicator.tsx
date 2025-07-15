import React, { useState } from 'react';
import { createPortal } from 'react-dom';
import { Download } from 'lucide-react';
import { useDownloadStatus, formatSpeed, formatFileSize } from '../hooks/useDownloadStatus';

interface DownloadIndicatorProps {
  onClick: () => void;
}

export const DownloadIndicator: React.FC<DownloadIndicatorProps> = ({ onClick }) => {
  const [showTooltip, setShowTooltip] = useState(false);
  const downloadStatus = useDownloadStatus(1000);

  const { totalActiveCount, totalSpeed, activeDownloads, stats } = downloadStatus;
  const hasActiveDownloads = totalActiveCount > 0;
  const downloadingCount = activeDownloads.filter(d => d.status === 'downloading').length;

  return (
    <div 
      className="relative"
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
    >
      <button 
        onClick={onClick}
        className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors duration-200 relative"
        title="Downloads"
      >
        <Download className="w-5 h-5" />
        
        {/* Download count badge */}
        {hasActiveDownloads && (
          <div className="absolute -top-1 -right-1 min-w-[18px] h-[18px] bg-blue-500 rounded-full flex items-center justify-center text-xs font-bold text-white">
            <span className={downloadingCount > 0 ? 'animate-pulse' : ''}>
              {totalActiveCount}
            </span>
          </div>
        )}
        
        {/* Downloading animation ring */}
        {downloadingCount > 0 && (
          <div className="absolute inset-0 rounded-lg">
            <div className="absolute inset-0 rounded-lg border-2 border-blue-400 animate-ping opacity-30"></div>
            <div className="absolute inset-0 rounded-lg border-2 border-blue-500 animate-pulse"></div>
          </div>
        )}
      </button>

      {/* Tooltip Portal */}
      {showTooltip && createPortal(
        <div className="fixed inset-0 pointer-events-none z-[99999]">
          <div className="absolute top-16 right-4 w-80 bg-gray-800 border border-gray-600 rounded-lg shadow-xl pointer-events-auto p-4">
            <div className="space-y-3">
              {/* Header */}
              <div className="flex items-center justify-between border-b border-gray-600 pb-2">
                <h3 className="text-white font-semibold flex items-center gap-2">
                  <Download className="w-4 h-4" />
                  Download Status
                </h3>
                {downloadingCount > 0 && (
                  <div className="flex items-center gap-1">
                    <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                    <span className="text-xs text-green-400">Active</span>
                  </div>
                )}
              </div>

              {/* Stats */}
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div className="bg-gray-700 rounded p-2">
                  <div className="text-gray-400 text-xs">Active Downloads</div>
                  <div className="text-white font-semibold">{totalActiveCount}</div>
                </div>
                <div className="bg-gray-700 rounded p-2">
                  <div className="text-gray-400 text-xs">Completed</div>
                  <div className="text-white font-semibold">{stats.completed_downloads}</div>
                </div>
                <div className="bg-gray-700 rounded p-2">
                  <div className="text-gray-400 text-xs">Total Speed</div>
                  <div className="text-white font-semibold">{formatSpeed(totalSpeed)}</div>
                </div>
                <div className="bg-gray-700 rounded p-2">
                  <div className="text-gray-400 text-xs">Downloaded</div>
                  <div className="text-white font-semibold">{formatFileSize(stats.total_downloaded_size)}</div>
                </div>
              </div>

              {/* Active downloads list */}
              {activeDownloads.length > 0 && (
                <div className="space-y-2">
                  <div className="text-gray-400 text-xs font-medium">Current Downloads:</div>
                  <div className="max-h-32 overflow-y-auto space-y-1">
                    {activeDownloads.slice(0, 1).map((download) => (
                      <div key={download.id} className="bg-gray-700 rounded p-2 text-xs">
                        <div className="flex items-center justify-between mb-1">
                          <span className="text-white truncate flex-1 mr-2">
                            {download.fileName}
                          </span>
                          <span className={`px-1.5 py-0.5 rounded text-xs font-medium ${
                            download.status === 'downloading' 
                              ? 'bg-blue-500 text-white' 
                              : download.status === 'paused'
                              ? 'bg-yellow-500 text-black'
                              : 'bg-gray-500 text-white'
                          }`}>
                            {download.status}
                          </span>
                        </div>
                        <div className="flex items-center justify-between text-gray-400">
                          <span>{download.progress.toFixed(1)}%</span>
                          {download.status === 'downloading' && (
                            <span>{formatSpeed(download.speed)}</span>
                          )}
                        </div>
                        {/* Progress bar */}
                        <div className="w-full bg-gray-600 rounded-full h-1 mt-1">
                          <div 
                            className={`h-1 rounded-full transition-all duration-300 ${
                              download.status === 'downloading' 
                                ? 'bg-blue-500' 
                                : download.status === 'paused'
                                ? 'bg-yellow-500'
                                : 'bg-gray-500'
                            }`}
                            style={{ width: `${download.progress}%` }}
                          ></div>
                        </div>
                      </div>
                    ))}
                    {activeDownloads.length > 1 && (
                      <div className="text-center text-gray-400 text-xs py-1">
                        +{activeDownloads.length - 1} more downloads
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* No downloads message */}
              {totalActiveCount === 0 && (
                <div className="text-center text-gray-400 text-sm py-2">
                  No active downloads
                </div>
              )}

              {/* Footer */}
              <div className="border-t border-gray-600 pt-2 text-center">
                <span className="text-gray-400 text-xs">Click to open Download Manager</span>
              </div>
            </div>
          </div>
        </div>,
        document.body
      )}
    </div>
  );
};