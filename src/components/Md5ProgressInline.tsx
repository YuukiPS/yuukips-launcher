/* eslint-disable @typescript-eslint/no-explicit-any */
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { X, Clock, FileText } from 'lucide-react';

interface Md5Progress {
  file_path: string;
  bytes_processed: number;
  total_bytes: number;
  progress: number;
  speed_mbps: number;
}

interface Md5ProgressInlineProps {
  isVisible: boolean;
  fileName: string;
  currentFileIndex: number;
  totalFiles: number;
  onComplete: (hash: string) => void;
  onError: (error: string) => void;
  onCancel: () => void;
}

const Md5ProgressInline: React.FC<Md5ProgressInlineProps> = ({
  isVisible,
  fileName,
  currentFileIndex,
  totalFiles,
  onComplete,
  onError,
  onCancel
}) => {
  const [progress, setProgress] = useState<Md5Progress | null>(null);
  const [estimatedTimeRemaining, setEstimatedTimeRemaining] = useState<string>('Calculating...');

  useEffect(() => {
    if (!isVisible) {
      setProgress(null);
      setEstimatedTimeRemaining('Calculating...');
      return;
    }

    const currentStartTime = Date.now();

    const unlisten = listen('md5-progress', (event: any) => {
      const progressData = event.payload as Md5Progress;
      setProgress(progressData);
      
      // Calculate estimated time remaining
      if (progressData.progress > 0) {
        const elapsed = (Date.now() - currentStartTime) / 1000; // seconds
        const totalEstimated = (elapsed / progressData.progress) * 100;
        const remaining = totalEstimated - elapsed;
        
        if (remaining > 60) {
          setEstimatedTimeRemaining(`${Math.ceil(remaining / 60)}m ${Math.ceil(remaining % 60)}s`);
        } else {
          setEstimatedTimeRemaining(`${Math.ceil(remaining)}s`);
        }
      }
    });

    const unlistenComplete = listen('md5-complete', (event: any) => {
      onComplete(event.payload.hash);
    });

    const unlistenError = listen('md5-error', (event: any) => {
      onError(event.payload.error);
    });

    return () => {
      unlisten.then(fn => fn());
      unlistenComplete.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, [isVisible, fileName, onComplete, onError]);

  const handleCancel = async () => {
    try {
      await invoke('cancel_md5_calculation');
      onCancel();
    } catch (error) {
      console.error('Failed to cancel MD5 calculation:', error);
      onCancel();
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  if (!isVisible) return null;

  return (
    <div className="bg-blue-900/20 border border-blue-500/30 rounded-lg p-4 mt-3">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          <FileText className="w-4 h-4 text-blue-400" />
          <span className="text-white font-medium text-sm">MD5 Verification</span>
          <span className="text-blue-400 text-sm">({currentFileIndex}/{totalFiles})</span>
        </div>
        <button
          onClick={handleCancel}
          className="text-gray-400 hover:text-white transition-colors"
          title="Cancel MD5 calculation"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
      
      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm">
          <span className="text-gray-300 truncate max-w-[60%]" title={fileName}>
            {fileName}
          </span>
          <span className="text-blue-400">
            {progress && typeof progress.progress === 'number' ? `${progress.progress.toFixed(1)}%` : '0%'}
          </span>
        </div>
        
        <div className="w-full bg-gray-700 rounded-full h-2">
          <div 
            className="bg-blue-500 h-2 rounded-full transition-all duration-300 ease-out"
            style={{ width: `${progress?.progress?.toFixed(1) || 0}%` }}
          />
        </div>
        
        <div className="flex items-center justify-between text-xs text-gray-400">
          <div className="flex items-center space-x-4">
            <span>
              {progress?.progress?.toFixed(1) || 0}% • {formatBytes(progress?.bytes_processed || 0)} / {formatBytes(progress?.total_bytes || 0)}
            </span>
            <span>
              {progress && typeof progress.speed_mbps === 'number' ? `${progress.speed_mbps.toFixed(1)} MB/s` : '0 MB/s'}
            </span>
          </div>
          <div className="flex items-center space-x-1">
            <Clock className="w-3 h-3" />
            <span>{estimatedTimeRemaining}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Md5ProgressInline;