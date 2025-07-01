import React, { useState } from 'react';
import { X, Bug, RefreshCw, TestTube } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface DebugSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onForceUpdate: () => void;
}

export const DebugSettingsModal: React.FC<DebugSettingsModalProps> = ({
  isOpen,
  onClose,
  onForceUpdate
}) => {
  const [isForcing, setIsForcing] = useState(false);
  const [testResult, setTestResult] = useState<string>('');
  const [isProxyTesting, setIsProxyTesting] = useState(false);

  if (!isOpen) return null;

  const handleForceUpdate = async () => {
    setIsForcing(true);
    try {
      // Simulate an update being available by calling the force update callback
      onForceUpdate();
    } finally {
      setIsForcing(false);
    }
  };

  const runProxyBypassTest = async () => {
    setIsProxyTesting(true);
    setTestResult('');
    
    try {
      const result = await invoke('test_proxy_bypass', {
        url: 'https://httpbin.org/get'
      }) as string;
      setTestResult(`✅ Success: ${result}`);
    } catch (error) {
      setTestResult(`❌ Error: ${error}`);
    } finally {
      setIsProxyTesting(false);
    }
  };

  const testGameApiCall = async () => {
    setIsProxyTesting(true);
    setTestResult('');
    
    try {
      const result = await invoke('fetch_api_data', { 
        url: 'https://ps.yuuki.me/json/game_all.json' 
      }) as string;
      const games = JSON.parse(result);
      setTestResult(`✅ Game API Success: Fetched ${games.length} games`);
    } catch (error) {
      setTestResult(`❌ Game API Error: ${error}`);
    } finally {
      setIsProxyTesting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4 border border-gray-700">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-orange-600 rounded-lg">
              <Bug className="w-5 h-5 text-white" />
            </div>
            <h2 className="text-xl font-semibold text-white">Debug Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          <div className="space-y-4">
            <h3 className="text-lg font-medium text-white">Update System</h3>
            
            <div className="space-y-3">
              <p className="text-gray-300 text-sm">
                Force an update notification to appear, even if no new version is available.
              </p>
              
              <button
                onClick={handleForceUpdate}
                disabled={isForcing}
                className="w-full flex items-center justify-center space-x-2 px-4 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-600/50 text-white rounded-lg transition-colors font-medium"
              >
                {isForcing ? (
                  <>
                    <RefreshCw className="w-4 h-4 animate-spin" />
                    <span>Forcing Update...</span>
                  </>
                ) : (
                  <>
                    <RefreshCw className="w-4 h-4" />
                    <span>Force Update Check</span>
                  </>
                )}
              </button>
            </div>
          </div>

          <div className="space-y-4">
            <h3 className="text-lg font-medium text-white">Proxy Testing</h3>
            
            <div className="space-y-3">
              <p className="text-gray-300 text-sm">
                Test proxy bypass functionality and game API connectivity.
              </p>
              
              <div className="space-y-2">
                <button
                  onClick={runProxyBypassTest}
                  disabled={isProxyTesting}
                  className="w-full flex items-center justify-center space-x-2 px-4 py-3 bg-purple-600 hover:bg-purple-700 disabled:bg-purple-600/50 text-white rounded-lg transition-colors font-medium"
                >
                  {isProxyTesting ? (
                    <>
                      <TestTube className="w-4 h-4 animate-pulse" />
                      <span>Testing...</span>
                    </>
                  ) : (
                    <>
                      <TestTube className="w-4 h-4" />
                      <span>Test Basic Proxy Bypass</span>
                    </>
                  )}
                </button>
                
                <button
                  onClick={testGameApiCall}
                  disabled={isProxyTesting}
                  className="w-full flex items-center justify-center space-x-2 px-4 py-3 bg-green-600 hover:bg-green-700 disabled:bg-green-600/50 text-white rounded-lg transition-colors font-medium"
                >
                  {isProxyTesting ? (
                    <>
                      <TestTube className="w-4 h-4 animate-pulse" />
                      <span>Testing...</span>
                    </>
                  ) : (
                    <>
                      <TestTube className="w-4 h-4" />
                      <span>Test Game API Call</span>
                    </>
                  )}
                </button>
              </div>
              
              {testResult && (
                <div className="mt-3 p-3 bg-gray-700 rounded-lg text-sm text-white whitespace-pre-wrap">
                  {testResult}
                </div>
              )}
            </div>
          </div>

          <div className="pt-4 border-t border-gray-700">
            <p className="text-gray-400 text-xs">
              Debug features are intended for development and testing purposes only.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};