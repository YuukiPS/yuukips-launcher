import React from 'react';
import { X, AlertTriangle, ExternalLink, Power, RefreshCw } from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface UpdateFailureModalProps {
  isOpen: boolean;
  onClose: () => void;
  onRetry?: () => void;
  errorMessage: string;
}

export const UpdateFailureModal: React.FC<UpdateFailureModalProps> = ({
  isOpen,
  onClose,
  onRetry,
  errorMessage
}) => {
  if (!isOpen) return null;

  const handleExitAndOpenReleases = async () => {
    try {
      // Open the GitHub releases page
      await openUrl('https://github.com/YuukiPS/yuukips-launcher/releases');
    } catch (error) {
      console.error('Failed to open releases page:', error);
      // Fallback for web or if Tauri is not available
      window.open('https://github.com/YuukiPS/yuukips-launcher/releases', '_blank');
    }
    
    try {
      // Exit the application
      const appWindow = getCurrentWindow();
      await appWindow.close();
    } catch (error) {
      console.error('Failed to exit application:', error);
      // Fallback to window.close for web environment
      window.close();
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4 border border-gray-700">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-red-600 rounded-lg">
              <AlertTriangle className="w-5 h-5 text-white" />
            </div>
            <h2 className="text-xl font-semibold text-white">Update Check Failed</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          <p className="text-gray-300">
            Failed to check for updates. Please download the latest launcher manually.
          </p>
          
          <div className="bg-gray-900 rounded-lg p-4 border border-gray-600">
            <p className="text-red-400 text-sm font-mono">
              {errorMessage}
            </p>
          </div>
          
          <div className="bg-blue-900/30 border border-blue-600/30 rounded-lg p-4">
            <p className="text-blue-300 text-sm">
              <strong>Please download the latest launcher at:</strong><br />
              <span className="font-mono text-blue-200">https://github.com/YuukiPS/yuukips-launcher/releases</span>
            </p>
            <p className="text-blue-400 text-xs mt-2">
              If you can't access the link, it will open automatically when you click "Exit & Download".
            </p>
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-end space-x-3 p-6 border-t border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          >
            Continue
          </button>
          {onRetry && (
            <button
              onClick={onRetry}
              className="flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
              <span>Retry</span>
            </button>
          )}
          <button
            onClick={handleExitAndOpenReleases}
            className="flex items-center space-x-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
          >
            <Power className="w-4 h-4" />
            <span>Exit & Download</span>
            <ExternalLink className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
};