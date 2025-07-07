import React, { useState, useEffect } from 'react';
import { X, AlertTriangle, Copy, Search, CheckCircle, XCircle } from 'lucide-react';

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

interface MD5CheckResult {
  found: boolean;
  data?: {
    game_id: number;
    version: string;
    channel: number;
  };
  error?: string;
}

export const PatchErrorModal: React.FC<PatchErrorModalProps> = ({
  isOpen,
  onClose,
  errorInfo,
  gameTitle
}) => {
  const [md5CheckResult, setMd5CheckResult] = useState<MD5CheckResult | null>(null);
  const [isCheckingMd5, setIsCheckingMd5] = useState(false);
  const [hasCheckedMd5, setHasCheckedMd5] = useState(false);

  const checkMd5OnServer = async (md5: string) => {
    setIsCheckingMd5(true);
    try {
      const response = await fetch(`https://ps.yuuki.me/api/v1/patch/find/${md5}`);
      const data = await response.json();
      
      if (response.ok && data.retcode !== -1) {
        // Patch found on server
        setMd5CheckResult({
          found: true,
          data: {
            game_id: data.game_id,
            version: data.version,
            channel: data.channel
          }
        });
      } else {
        // Patch not found or error response
        setMd5CheckResult({
          found: false,
          error: data.message || 'Patch file not found or invalid format'
        });
      }
    } catch (error) {
      console.error('Failed to check MD5 on server:', error);
      setMd5CheckResult({
        found: false,
        error: 'Failed to connect to patch server'
      });
    } finally {
      setIsCheckingMd5(false);
      setHasCheckedMd5(true);
    }
  };

  // Auto-check MD5 when modal opens
  useEffect(() => {
    if (isOpen && errorInfo && !hasCheckedMd5) {
      checkMd5OnServer(errorInfo.md5);
    }
  }, [isOpen, errorInfo, hasCheckedMd5]);

  // Reset state when modal closes
  useEffect(() => {
    if (!isOpen) {
      setMd5CheckResult(null);
      setIsCheckingMd5(false);
      setHasCheckedMd5(false);
    }
  }, [isOpen]);

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  };

  if (!isOpen) return null;
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

          {/* MD5 Check Results */}
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-lg font-medium text-white">MD5 Verification</h3>
              {!hasCheckedMd5 && (
                <button
                  onClick={() => checkMd5OnServer(errorInfo.md5)}
                  disabled={isCheckingMd5}
                  className="flex items-center gap-2 px-3 py-1 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-800 text-white rounded text-sm transition-colors"
                >
                  <Search className={`w-4 h-4 ${isCheckingMd5 ? 'animate-spin' : ''}`} />
                  {isCheckingMd5 ? 'Checking...' : 'Check MD5'}
                </button>
              )}
            </div>
            
            {isCheckingMd5 && (
              <div className="text-blue-200 text-sm">
                Checking MD5 hash on patch server...
              </div>
            )}
            
            {md5CheckResult && (
              <div className="space-y-3">
                {md5CheckResult.found ? (
                  <div className="bg-green-900/20 border border-green-500/30 rounded-lg p-3">
                    <div className="flex items-center gap-2 mb-2">
                      <CheckCircle className="w-5 h-5 text-green-400" />
                      <span className="text-green-200 font-medium">MD5 Found on Server</span>
                    </div>
                    <p className="text-green-200 text-sm mb-3">
                      The file hash was found on the server, but the game file you are currently using does not match the one you selected in the game settings.
                    </p>
                    
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                      <div>
                        <h4 className="text-white font-medium mb-2">Your Selection:</h4>
                        <div className="space-y-1">
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
                        </div>
                      </div>
                      
                      <div>
                        <h4 className="text-white font-medium mb-2">Your Game Files:</h4>
                        <div className="space-y-1">
                          <div>
                            <span className="text-gray-400">Game ID:</span>
                            <span className={`ml-2 font-mono ${
                              md5CheckResult.data!.game_id.toString() === errorInfo.game_id ? 'text-green-400' : 'text-red-400'
                            }`}>
                              {md5CheckResult.data!.game_id}
                            </span>
                          </div>
                          <div>
                            <span className="text-gray-400">Version:</span>
                            <span className={`ml-2 font-mono ${
                              md5CheckResult.data!.version === errorInfo.version ? 'text-green-400' : 'text-red-400'
                            }`}>
                              {md5CheckResult.data!.version}
                            </span>
                          </div>
                          <div>
                            <span className="text-gray-400">Channel:</span>
                            <span className={`ml-2 font-mono ${
                              md5CheckResult.data!.channel.toString() === errorInfo.channel ? 'text-green-400' : 'text-red-400'
                            }`}>
                              {md5CheckResult.data!.channel}
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    <div className="mt-3 p-2 bg-blue-900/20 border border-blue-500/30 rounded">
                      <p className="text-blue-200 text-sm">
                        <strong>Solution:</strong> Try selecting Game ID {md5CheckResult.data!.game_id}, 
                        Version {md5CheckResult.data!.version}, Channel {md5CheckResult.data!.channel} 
                        in your game settings to match the server configuration.
                      </p>
                    </div>
                  </div>
                ) : (
                  <div className="bg-red-900/20 border border-red-500/30 rounded-lg p-3">
                    <div className="flex items-center gap-2 mb-2">
                      <XCircle className="w-5 h-5 text-red-400" />
                      <span className="text-red-200 font-medium">MD5 Not Found on Server</span>
                    </div>
                    <p className="text-red-200 text-sm">
                      {md5CheckResult.error || 'The MD5 hash was not found on the patch server.'}
                    </p>
                  </div>
                )}
              </div>
            )}
          </div>

          <div className="bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-4">
            <h3 className="text-yellow-200 font-medium mb-2">What to do next:</h3>
            <ul className="text-yellow-200 text-sm space-y-1 list-disc list-inside">
              {md5CheckResult?.found ? (
                <>
                  <li>Update your game settings to match the server configuration shown above</li>
                  <li>Make sure you have the correct game version installed</li>
                  <li>Verify that you're selecting the right channel for your game region</li>
                </>
              ) : (
                <>
                  <li>Please notify the administrator about this missing patch</li>
                  <li>Include the error details above in your report</li>
                  <li>Try again later as the patch may be added soon</li>
                  <li>Check if you're using the correct game version and channel</li>
                </>
              )}
            </ul>
          </div>
        </div>
        
      </div>
    </div>
  );
};