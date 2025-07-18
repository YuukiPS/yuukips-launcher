import React, { useState } from 'react';
import { X, Bug, RefreshCw, TestTube, Trash2, FolderOpen, Database, HardDrive, Code } from 'lucide-react';
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
  const [activeTab, setActiveTab] = useState('basic');
  const [isForcing, setIsForcing] = useState(false);
  const [testResult, setTestResult] = useState<string>('');
  const [isProxyTesting, setIsProxyTesting] = useState(false);
  const [showClearDataConfirm, setShowClearDataConfirm] = useState(false);

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



  const handleClearData = () => {
    setShowClearDataConfirm(true);
  };

  const confirmClearData = async () => {
    try {
      // Clear all stored data/settings
      await invoke('clear_launcher_data');
      setShowClearDataConfirm(false);
      onClose();
    } catch (error) {
      console.error('Failed to clear launcher data:', error);
    }
  };

  const cancelClearData = () => {
    setShowClearDataConfirm(false);
  };

  const clearBrowserData = () => {
    // Clear localStorage
    localStorage.clear();
    // Clear sessionStorage
    sessionStorage.clear();
    // Clear IndexedDB (if any)
    if ('indexedDB' in window) {
      indexedDB.databases().then(databases => {
        databases.forEach(db => {
          if (db.name) {
            indexedDB.deleteDatabase(db.name);
          }
        });
      }).catch(console.error);
    }
    alert('Browser data cleared successfully!');
  };

  const openDataFolder = async (folderType: 'yuukips' | 'appdata' | 'temp') => {
    try {
      let folderPath: string;
      
      switch (folderType) {
        case 'yuukips':
          folderPath = await invoke('get_yuukips_data_path');
          break;
        case 'appdata':
          folderPath = await invoke('get_app_data_path');
          break;
        case 'temp':
          folderPath = await invoke('get_temp_files_path');
          break;
        default:
          throw new Error('Invalid folder type');
      }
      
      await invoke('open_directory', { path: folderPath });
    } catch (error) {
      console.error(`Failed to open ${folderType} folder:`, error);
      alert(`Failed to open ${folderType} folder: ${error}`);
    }
  };

  const openDevTools = async () => {
    try {
      // Check if we're in a Tauri environment
      if (window.__TAURI__) {
        // Use our custom Tauri command to open devtools
        const result = await invoke('open_devtools') as string;
        console.log('DevTools result:', result);
        // Show success message briefly
        setTestResult(`✅ ${result}`);
        setTimeout(() => setTestResult(''), 3000);
      } else {
        // Fallback for web environment - try to open browser devtools
        // This won't work in production but useful for development
        console.log('DevTools opened (web environment)');
        setTestResult('⚠️ DevTools functionality is only available in the desktop application.');
        setTimeout(() => setTestResult(''), 3000);
      }
    } catch (error) {
      console.error('Failed to open DevTools:', error);
      setTestResult(`❌ Failed to open DevTools: ${error}`);
      setTimeout(() => setTestResult(''), 5000);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gray-900 rounded-xl border border-gray-700 shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-orange-600 rounded-lg">
              <Bug className="w-5 h-5 text-white" />
            </div>
            <h2 className="text-xl font-bold text-white">Debug Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex h-[600px]">
          {/* Sidebar */}
          <div className="w-64 bg-gray-800/50 border-r border-gray-700 p-4">
            <nav className="space-y-2">
              <button
                onClick={() => setActiveTab('basic')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'basic'
                    ? 'bg-orange-600/30 text-orange-400 border border-orange-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                }`}
              >
                Basic
              </button>
              <button
                onClick={() => setActiveTab('update')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'update'
                    ? 'bg-orange-600/30 text-orange-400 border border-orange-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                }`}
              >
                Update System
              </button>
              <button
                onClick={() => setActiveTab('testing')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'testing'
                    ? 'bg-orange-600/30 text-orange-400 border border-orange-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                }`}
              >
                Proxy Testing
              </button>
              <button
                onClick={() => setActiveTab('data')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'data'
                    ? 'bg-orange-600/30 text-orange-400 border border-orange-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                }`}
              >
                Data Management
              </button>
            </nav>
          </div>

          {/* Content */}
          <div className="flex-1 p-6 overflow-y-auto">
            {activeTab === 'basic' && (
              <div className="space-y-6">
                <div className="space-y-4">
                  <h3 className="text-lg font-medium text-white">Basic Developer Tools</h3>
                  
                  <div className="space-y-3">
                    <p className="text-gray-300 text-sm">
                      Access developer tools and basic debugging features.
                    </p>
                    
                    <button
                       onClick={openDevTools}
                       className="w-full flex items-center justify-center space-x-2 px-4 py-3 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors font-medium"
                     >
                       <Code className="w-4 h-4" />
                       <span>Show DevTools</span>
                     </button>
                     
                     {testResult && (
                       <div className="mt-3 p-3 bg-gray-700 rounded-lg text-sm text-white whitespace-pre-wrap">
                         {testResult}
                       </div>
                     )}
                   </div>
                 </div>

                 <div className="pt-4 border-t border-gray-700">
                   <p className="text-gray-400 text-xs">
                     Developer tools provide access to browser debugging features including console, network, and element inspection.
                   </p>
                 </div>
              </div>
            )}

            {activeTab === 'update' && (
              <div className="space-y-6">
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

                <div className="pt-4 border-t border-gray-700">
                  <p className="text-gray-400 text-xs">
                    Debug features are intended for development and testing purposes only.
                  </p>
                </div>
              </div>
            )}

            {activeTab === 'testing' && (
              <div className="space-y-6">

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
            )}

            {activeTab === 'data' && (
              <div className="space-y-6">

                <div className="space-y-4">
                  <h3 className="text-lg font-medium text-white">Data Management</h3>
                  
                  <div className="space-y-3">
                    <p className="text-gray-300 text-sm">
                      Manage launcher data, browser storage, and view data folders.
                    </p>
                    
                    {/* Clear Data Buttons */}
                    <div className="space-y-2">
                      <button
                        onClick={handleClearData}
                        className="w-full flex items-center justify-center space-x-2 px-4 py-3 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors font-medium"
                      >
                        <Trash2 className="w-4 h-4" />
                        <span>Clear Launcher Data</span>
                      </button>
                      
                      <button
                        onClick={clearBrowserData}
                        className="w-full flex items-center justify-center space-x-2 px-4 py-3 bg-orange-600 hover:bg-orange-700 text-white rounded-lg transition-colors font-medium"
                      >
                        <Database className="w-4 h-4" />
                        <span>Clear Browser Data</span>
                      </button>
                    </div>
                    
                    {/* Navigation Buttons */}
                    <div className="pt-3 border-t border-gray-600">
                      <p className="text-gray-400 text-xs mb-3">View Data Folders:</p>
                      <div className="grid grid-cols-1 gap-2">
                        <button
                          onClick={() => openDataFolder('yuukips')}
                          className="flex items-center justify-center space-x-2 px-3 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors text-sm"
                        >
                          <FolderOpen className="w-4 h-4" />
                          <span>YuukiPS Data</span>
                        </button>
                        
                        <button
                          onClick={() => openDataFolder('appdata')}
                          className="flex items-center justify-center space-x-2 px-3 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors text-sm"
                        >
                          <FolderOpen className="w-4 h-4" />
                          <span>App Data</span>
                        </button>
                        
                        <button
                          onClick={() => openDataFolder('temp')}
                          className="flex items-center justify-center space-x-2 px-3 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors text-sm"
                        >
                          <HardDrive className="w-4 h-4" />
                          <span>Temp Files</span>
                        </button>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="pt-4 border-t border-gray-700">
                  <p className="text-gray-400 text-xs">
                    Debug features are intended for development and testing purposes only.
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
      
      {/* Clear Data Confirmation Modal */}
      {showClearDataConfirm && (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-60">
          <div className="bg-gray-800 rounded-lg shadow-xl w-full max-w-sm mx-4 border border-gray-700">
            <div className="p-6">
              <h3 className="text-lg font-semibold text-white mb-4">Confirm Clear Data</h3>
              <p className="text-gray-300 text-sm mb-6">
                Are you sure you want to delete launcher data? This will cause all settings to be reset.
              </p>
              <div className="flex space-x-3">
                <button
                  onClick={cancelClearData}
                  className="flex-1 px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors font-medium"
                >
                  Cancel
                </button>
                <button
                  onClick={confirmClearData}
                  className="flex-1 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors font-medium"
                >
                  OK
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};