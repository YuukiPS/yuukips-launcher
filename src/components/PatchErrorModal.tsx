import React from 'react';
import { X, AlertTriangle, Copy } from 'lucide-react';

export interface PatchErrorInfo {
  game_id: string;
  version: string;
  channel: string;
  md5: string;
  url: string;
  status_code: number;
  error_type: string;
}

interface PatchErrorModalProps {
  isOpen: boolean;
  onClose: () => void;
  errorInfo: PatchErrorInfo | null;
  gameTitle: string;
}

export const PatchErrorModal: React.FC<PatchErrorModalProps> = ({
  isOpen,
  onClose,
  errorInfo,
  gameTitle
}) => {
  if (!isOpen) return null;

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  };

  if (!errorInfo) return null;

  const errorDetails = `Game: ${gameTitle}
Game ID: ${errorInfo.game_id}
Version: ${errorInfo.version}
Channel: ${errorInfo.channel}
MD5: ${errorInfo.md5}
URL: ${errorInfo.url}
Status Code: ${errorInfo.status_code}
Error Type: ${errorInfo.error_type}`;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <AlertTriangle className="w-6 h-6 text-red-400" />
            <h2 className="text-xl font-semibold text-white">Patch Not Found</h2>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        <div className="space-y-4">
          <div className="bg-red-900/20 border border-red-500/30 rounded-lg p-4">
            <p className="text-red-200 mb-2">
              The patch information for <strong>{gameTitle}</strong> could not be found on the server.
            </p>
            <p className="text-red-200">
              This usually means the game version or configuration is not supported yet.
            </p>
          </div>

          <div className="bg-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-lg font-medium text-white">Error Details</h3>
              <button
                onClick={() => copyToClipboard(errorDetails)}
                className="flex items-center gap-2 px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm transition-colors"
              >
                <Copy className="w-4 h-4" />
                Copy Details
              </button>
            </div>
            
            <div className="space-y-2 text-sm">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                <div>
                  <span className="text-gray-400">Game ID:</span>
                  <span className="text-white ml-2 font-mono">{errorInfo.game_id}</span>
                </div>
                <div>
                  <span className="text-gray-400">Version:</span>
                  <span className="text-white ml-2 font-mono">{errorInfo.version}</span>
                </div>
                <div>
                  <span className="text-gray-400">Channel:</span>
                  <span className="text-white ml-2 font-mono">{errorInfo.channel}</span>
                </div>
                <div>
                  <span className="text-gray-400">Status Code:</span>
                  <span className="text-red-400 ml-2 font-mono">{errorInfo.status_code}</span>
                </div>
              </div>
              
              <div>
                <span className="text-gray-400">MD5:</span>
                <span className="text-white ml-2 font-mono text-xs break-all">{errorInfo.md5}</span>
              </div>
              
              <div>
                <span className="text-gray-400">Request URL:</span>
                <span className="text-blue-400 ml-2 font-mono text-xs break-all">{errorInfo.url}</span>
              </div>
            </div>
          </div>

          <div className="bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-4">
            <h3 className="text-yellow-200 font-medium mb-2">What to do next:</h3>
            <ul className="text-yellow-200 text-sm space-y-1 list-disc list-inside">
              <li>Please notify the administrator about this missing patch</li>
              <li>Include the error details above in your report</li>
              <li>Try again later as the patch may be added soon</li>
              <li>Check if you're using the correct game version and channel</li>
            </ul>
          </div>
        </div>

        <div className="flex justify-end gap-3 mt-6">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};