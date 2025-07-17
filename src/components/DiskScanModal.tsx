import React, { useState, useEffect } from 'react';
import { X, HardDrive, Search, CheckCircle, Loader2, AlertTriangle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { DriveInfo, ScanProgress } from '../types/tauri';

interface DiskScanModalProps {
  isOpen: boolean;
  onClose: () => void;
  onPathSelected: (path: string) => void;
  gameId: number;
  channel: number;
}

type ScanState = 'selecting' | 'scanning' | 'results' | 'error';

interface MD5CheckResult {
  found: boolean;
  data?: {
    game_id: number;
    version: string;
    channel: number;
  };
  error?: string;
}

interface PathCheckResult {
  path: string;
  md5?: string;
  checkResult?: MD5CheckResult;
  isChecking: boolean;
}

export const DiskScanModal: React.FC<DiskScanModalProps> = ({
  isOpen,
  onClose,
  onPathSelected,
  gameId,
  channel
}) => {
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [selectedDrive, setSelectedDrive] = useState<string>('');
  const [scanState, setScanState] = useState<ScanState>('selecting');
  const [scanProgress, setScanProgress] = useState<ScanProgress | null>(null);
  const [foundPaths, setFoundPaths] = useState<string[]>([]);
  const [pathCheckResults, setPathCheckResults] = useState<PathCheckResult[]>([]);
  const [error, setError] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);

  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return 'Unknown';
    
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  // Check MD5 on server (similar to PatchErrorModal)
  const checkMd5OnServer = async (md5: string): Promise<MD5CheckResult> => {
    try {
      const response = await fetch(`https://ps.yuuki.me/api/v1/patch/find/${md5}`);
      const data = await response.json();
      
      if (response.ok && data.retcode !== -1) {
        return {
          found: true,
          data: {
            game_id: data.game_id,
            version: data.version,
            channel: data.channel
          }
        };
      } else {
        return {
          found: false,
          error: data.message || 'Patch file not found or invalid format'
        };
      }
    } catch (error) {
      console.error('Failed to check MD5 on server:', error);
      return {
        found: false,
        error: 'Failed to connect to patch server'
      };
    }
  };

  // Get MD5 hash for a game path
  const getGameMd5 = async (path: string): Promise<string | null> => {
    try {
      const md5 = await invoke('get_game_md5', { path }) as string;
      return md5;
    } catch (error) {
      console.error('Failed to get MD5 for path:', path, error);
      return null;
    }
  };

  // Check all found paths for MD5 and server support
  const checkAllPaths = async (paths: string[]) => {
    const initialResults: PathCheckResult[] = paths.map(path => ({
      path,
      isChecking: true
    }));
    setPathCheckResults(initialResults);

    for (let i = 0; i < paths.length; i++) {
      const path = paths[i];
      try {
        const md5 = await getGameMd5(path);
        if (md5) {
          const checkResult = await checkMd5OnServer(md5);
          setPathCheckResults(prev => prev.map((result, index) => 
            index === i ? { ...result, md5, checkResult, isChecking: false } : result
          ));
        } else {
          setPathCheckResults(prev => prev.map((result, index) => 
            index === i ? { ...result, isChecking: false } : result
          ));
        }
      } catch (error) {
        console.error('Failed to check MD5 for path:', path, error);
        setPathCheckResults(prev => prev.map((result, index) => 
          index === i ? { ...result, isChecking: false } : result
        ));
      }
    }
  };

  // Load available drives
  useEffect(() => {
    if (isOpen && scanState === 'selecting') {
      loadDrives();
    }
  }, [isOpen, scanState]);

  // Listen for scan progress events
  useEffect(() => {
    if (!isOpen) return;

    const unlisten = listen<ScanProgress>('scan-progress', (event) => {
      setScanProgress(event.payload);
      if (event.payload.found_paths.length > 0) {
        setFoundPaths(event.payload.found_paths);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [isOpen]);

  const loadDrives = async () => {
    try {
      setIsLoading(true);
      const driveList = await invoke('get_available_drives') as DriveInfo[];
      setDrives(driveList.filter(drive => drive.drive_type === 'Fixed')); // Only show fixed drives
    } catch (err) {
      setError(`Failed to load drives: ${err}`);
      setScanState('error');
    } finally {
      setIsLoading(false);
    }
  };

  const startScan = async () => {
    if (!selectedDrive) return;
    
    try {
      setScanState('scanning');
      setScanProgress(null);
      setFoundPaths([]);
      setError('');
      
      const results = await invoke('scan_drive_for_games', {
        drive: selectedDrive,
        gameId,
        channel
      }) as string[];
      
      setFoundPaths(results);
      setScanState('results');
      
      // Start MD5 checking for all found paths
      if (results.length > 0) {
        checkAllPaths(results);
      }
    } catch (err) {
      setError(`Scan failed: ${err}`);
      setScanState('error');
    }
  };

  const handlePathSelect = (path: string) => {
    onPathSelected(path);
    onClose();
  };

  const handleManualFolderSelect = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Game Installation Folder'
      });
      
      if (selected && typeof selected === 'string') {
        onPathSelected(selected);
        onClose();
      }
    } catch (error) {
      console.error('Failed to select folder:', error);
      setError(`Failed to open folder dialog: ${error}`);
    }
  };

  const resetModal = () => {
    setScanState('selecting');
    setSelectedDrive('');
    setScanProgress(null);
    setFoundPaths([]);
    setPathCheckResults([]);
    setError('');
  };

  const handleClose = () => {
    resetModal();
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-lg w-full max-w-2xl mx-4 max-h-[80vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between bg-gray-700 px-4 py-3">
          <h2 className="text-xl font-semibold text-white flex items-center gap-2">
            <HardDrive size={20} />
            Disk Scanner
          </h2>
          <button 
            onClick={handleClose} 
            className="text-gray-400 hover:text-white"
            disabled={scanState === 'scanning'}
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div className="px-4 py-4 overflow-y-auto max-h-[60vh]">
          {scanState === 'selecting' && (
            <div>
              <p className="text-gray-300 mb-4">
                Select a drive to scan for game installations:
              </p>
              
              {isLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="animate-spin" size={24} />
                  <span className="ml-2 text-gray-400">Loading drives...</span>
                </div>
              ) : (
                <div className="space-y-2">
                  {drives.map((drive) => (
                    <div
                      key={drive.letter}
                      className={`p-3 rounded border cursor-pointer transition-colors ${
                        selectedDrive === drive.letter
                          ? 'border-blue-500 bg-blue-900 bg-opacity-20'
                          : 'border-gray-600 hover:border-gray-500 bg-gray-700'
                      }`}
                      onClick={() => setSelectedDrive(drive.letter)}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <HardDrive size={20} className="text-gray-400" />
                          <div>
                            <div className="text-white font-medium">
                              {drive.name}
                            </div>
                            <div className="text-gray-400 text-sm">
                              {formatFileSize(drive.free_size)} free of {formatFileSize(drive.total_size)} • {drive.drive_type}
                            </div>
                          </div>
                        </div>
                        {selectedDrive === drive.letter && (
                          <CheckCircle size={20} className="text-blue-500" />
                        )}
                      </div>
                      
                      {/* Usage bar */}
                      {drive.total_size > 0 && (
                        <div className="mt-2">
                          <div className="w-full bg-gray-600 rounded-full h-2">
                            <div 
                              className="bg-blue-600 h-2 rounded-full" 
                              style={{ 
                                width: `${((drive.total_size - drive.free_size) / drive.total_size) * 100}%` 
                              }}
                            />
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {scanState === 'scanning' && (
            <div>
              <div className="flex items-center gap-2 mb-4">
                <Search className="animate-pulse" size={20} />
                <span className="text-white font-medium">
                  Scanning {drives.find(d => d.letter === selectedDrive)?.name || `Drive ${selectedDrive}`}...
                </span>
              </div>
              
              {scanProgress && (
                <div className="space-y-3">
                  <div className="text-gray-300 text-sm">
                    <div>Current: {scanProgress.current_path}</div>
                    <div className="mt-1">
                      Files scanned: {scanProgress.files_scanned.toLocaleString()} | 
                      Directories: {scanProgress.directories_scanned.toLocaleString()}
                    </div>
                    {scanProgress.found_paths.length > 0 && (
                      <div className="mt-1 text-green-400">
                        Found {scanProgress.found_paths.length} potential installation(s)
                      </div>
                    )}
                  </div>
                  
                  {/* Progress indicator */}
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div className="bg-blue-600 h-2 rounded-full animate-pulse" style={{ width: '100%' }} />
                  </div>
                </div>
              )}
            </div>
          )}

          {scanState === 'results' && (
            <div>
              <h3 className="text-white font-medium mb-3">
                Scan Results ({foundPaths.length} found)
              </h3>
              
              {foundPaths.length === 0 ? (
                <div className="text-center py-8">
                  <div className="text-gray-400 mb-4">No game installations found on this drive.</div>
                  <div className="flex flex-col gap-3 items-center">
                    <button
                      onClick={() => setScanState('selecting')}
                      className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                    >
                      Select Another Drive
                    </button>
                    <button
                      onClick={handleManualFolderSelect}
                      className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 transition-colors"
                    >
                      Set Manual Folder
                    </button>
                  </div>
                </div>
              ) : (
                <div className="space-y-2">
                  {foundPaths.map((path, index) => {
                    const pathResult = pathCheckResults.find(result => result.path === path);
                    const isSupported = pathResult?.checkResult?.found;
                    const isChecking = pathResult?.isChecking;
                    
                    return (
                      <div
                        key={index}
                        className={`p-3 rounded border cursor-pointer transition-colors ${
                          isSupported
                            ? 'border-green-500 bg-green-900 bg-opacity-20 hover:border-green-400'
                            : pathResult?.checkResult && !isSupported
                            ? 'border-yellow-500 bg-yellow-900 bg-opacity-20 hover:border-yellow-400'
                            : 'border-gray-600 hover:border-gray-500 bg-gray-700'
                        }`}
                        onClick={() => handlePathSelect(path)}
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="text-white font-medium">{path}</div>
                            
                            {isChecking ? (
                              <div className="flex items-center gap-2 mt-2 text-blue-400 text-sm">
                                <Loader2 className="animate-spin" size={14} />
                                <span>Checking version compatibility...</span>
                              </div>
                            ) : pathResult?.checkResult ? (
                              <div className="mt-2 space-y-1">
                                {isSupported ? (
                                  <div className="flex items-center gap-2">
                                    <CheckCircle size={16} className="text-green-400" />
                                    <span className="text-green-400 text-sm font-medium">Supported Version</span>
                                  </div>
                                ) : (
                                  <div className="flex items-center gap-2">
                                    <AlertTriangle size={16} className="text-yellow-400" />
                                    <span className="text-yellow-400 text-sm font-medium">Unsupported Version</span>
                                  </div>
                                )}
                                
                                {pathResult.checkResult.data && (
                                  <div className="text-xs text-gray-300 ml-6">
                                    Game ID: {pathResult.checkResult.data.game_id} • 
                                    Version: {pathResult.checkResult.data.version} • 
                                    Channel: {pathResult.checkResult.data.channel}
                                  </div>
                                )}
                                
                                {pathResult.md5 && (
                                  <div className="text-xs text-gray-400 ml-6 font-mono">
                                    MD5: {pathResult.md5}
                                  </div>
                                )}
                              </div>
                            ) : pathResult && !pathResult.md5 ? (
                              <div className="flex items-center gap-2 mt-2 text-gray-400 text-sm">
                                <AlertTriangle size={14} />
                                <span>Unable to read game files</span>
                              </div>
                            ) : null}
                            
                            <div className="text-gray-400 text-sm mt-1">
                              Click to select this path
                            </div>
                          </div>
                          
                          {isSupported && (
                            <CheckCircle size={20} className="text-green-500 flex-shrink-0" />
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}

          {scanState === 'error' && (
            <div className="text-center py-8">
              <div className="text-red-400 mb-4">{error}</div>
              <button
                onClick={resetModal}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
              >
                Try Again
              </button>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="bg-gray-700 px-4 py-3 flex justify-end space-x-3">
          {scanState === 'selecting' && (
            <>
              <button
                onClick={handleClose}
                className="px-4 py-2 text-gray-300 hover:text-white"
              >
                Cancel
              </button>
              <button
                onClick={startScan}
                disabled={!selectedDrive || isLoading}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
              >
                <Search size={16} />
                Start Scan
              </button>
            </>
          )}
          
          {scanState === 'scanning' && (
            <div className="text-gray-400 text-sm">
              Scanning in progress... Please wait.
            </div>
          )}
          
          {scanState === 'results' && foundPaths.length > 0 && (
            <button
              onClick={() => setScanState('selecting')}
              className="px-4 py-2 text-gray-300 hover:text-white"
            >
              Scan Another Drive
            </button>
          )}
        </div>
      </div>
    </div>
  );
};