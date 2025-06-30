import React from 'react';
import { X, AlertTriangle, RefreshCw } from 'lucide-react';

interface UpdateErrorModalProps {
  isOpen: boolean;
  onClose: () => void;
  onRetry: () => void;
  errorMessage: string;
}

export const UpdateErrorModal: React.FC<UpdateErrorModalProps> = ({
  isOpen,
  onClose,
  onRetry,
  errorMessage
}) => {
  if (!isOpen) return null;

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
            Failed to check for updates. This might be due to network connectivity issues or server problems.
          </p>
          
          <div className="bg-gray-900 rounded-lg p-4 border border-gray-600">
            <p className="text-red-400 text-sm font-mono">
              {errorMessage}
            </p>
          </div>
          
          <p className="text-gray-400 text-sm">
            You can continue using the application normally. The update check will be retried automatically on the next startup.
          </p>
        </div>

        {/* Actions */}
        <div className="flex justify-end space-x-3 p-6 border-t border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          >
            Dismiss
          </button>
          <button
            onClick={onRetry}
            className="flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            <RefreshCw className="w-4 h-4" />
            <span>Retry</span>
          </button>
        </div>
      </div>
    </div>
  );
};