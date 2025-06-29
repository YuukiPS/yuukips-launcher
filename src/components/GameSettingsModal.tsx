import React, { useState, useEffect } from 'react';
import { X, Folder, RotateCcw, HardDrive, Calendar, Clock, Check } from 'lucide-react';
import { Game } from '../types';
import { GameApiService } from '../services/gameApi';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

interface GameSettingsModalProps {
  game: Game;
  isOpen: boolean;
  onClose: () => void;
  onVersionChange: (gameId: number, newVersion: string) => void;
}

export const GameSettingsModal: React.FC<GameSettingsModalProps> = ({
  game,
  isOpen,
  onClose,
  onVersionChange
}) => {
  const [activeTab, setActiveTab] = useState('basic');
  const [selectedVersion, setSelectedVersion] = useState("");
  const [versionDirectories, setVersionDirectories] = useState<Record<string, string>>({});
  const [notification, setNotification] = useState<{ message: string; type: 'success' | 'error' } | null>(null);

  // Get available versions dynamically from game engine data
  const availableVersions = GameApiService.getAvailableVersionsForPlatform(game, 1);

  // Initialize selectedVersion with the first available version
  useEffect(() => {
    if (availableVersions.length > 0 && !selectedVersion) {
      setSelectedVersion(availableVersions[0]);
    }
  }, [availableVersions, selectedVersion]);

  // Load saved directories from localStorage on component mount
  useEffect(() => {
    const savedDirectories = localStorage.getItem(`game-${game.id}-directories`);
    if (savedDirectories) {
      try {
        setVersionDirectories(JSON.parse(savedDirectories));
      } catch (error) {
        console.error('Failed to parse saved directories:', error);
      }
    }
  }, [game.id]);

  // Save directories to localStorage whenever they change
  const saveDirectories = (newDirectories: Record<string, string>) => {
    setVersionDirectories(newDirectories);
    localStorage.setItem(`game-${game.id}-directories`, JSON.stringify(newDirectories));
  };

  // Show notification
  const showNotification = (message: string, type: 'success' | 'error' = 'success') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 3000);
  };

  const handleVersionChange = (version: string) => {
    setSelectedVersion(version);
    onVersionChange(game.id, version);
  };

  const handleOpenDirectory = async () => {
    const currentDir = getCurrentDirectory();
    if (currentDir) {
      try {
        await invoke('open_directory', { path: currentDir });
      } catch (error) {
        console.error('Failed to open directory:', error);
        showNotification('Failed to open directory', 'error');
      }
    } else {
      showNotification(`No directory set for ${selectedVersion}. Please set a directory first.`, 'error');
    }
  };

  const handleRelocate = async () => {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        defaultPath: versionDirectories[selectedVersion] || undefined,
        title: `Select directory for ${selectedVersion}`
      });

      if (selectedPath && typeof selectedPath === 'string') {
        const updatedDirectories = {
          ...versionDirectories,
          [selectedVersion]: selectedPath
        };
        saveDirectories(updatedDirectories);
        showNotification(`Directory for ${selectedVersion} updated successfully!`);
      }
    } catch (error) {
      console.error('Failed to open directory dialog:', error);
      showNotification('Failed to open directory selection dialog', 'error');
    }
  };

  // Get current directory for selected version
  const getCurrentDirectory = () => {
    return versionDirectories[selectedVersion] || '';
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      {/* Notification */}
      {notification && (
        <div className={`fixed top-4 right-4 z-60 px-4 py-3 rounded-lg shadow-lg flex items-center space-x-2 ${notification.type === 'success'
            ? 'bg-green-600 text-white'
            : 'bg-red-600 text-white'
          }`}>
          {notification.type === 'success' && <Check className="w-4 h-4" />}
          <span>{notification.message}</span>
        </div>
      )}

      <div className="bg-gray-900 rounded-xl border border-gray-700 shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <h2 className="text-xl font-bold text-white">Game Settings</h2>
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
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${activeTab === 'basic'
                    ? 'bg-purple-600/30 text-purple-400 border border-purple-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                  }`}
              >
                Basic Information
              </button>
              <button
                onClick={() => setActiveTab('advanced')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${activeTab === 'advanced'
                    ? 'bg-purple-600/30 text-purple-400 border border-purple-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                  }`}
              >
                Advanced Settings
              </button>
              <button
                onClick={() => setActiveTab('logs')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${activeTab === 'logs'
                    ? 'bg-purple-600/30 text-purple-400 border border-purple-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                  }`}
              >
                Log Info
              </button>
            </nav>
          </div>

          {/* Content */}
          <div className="flex-1 p-6 overflow-y-auto">
            {activeTab === 'basic' && (
              <div className="space-y-6">
                <h3 className="text-lg font-semibold text-white mb-4">Basic Information: {game.title}</h3>

                {/* Version Selection */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Game Version</h4>
                  <div className="space-y-2">
                    {availableVersions.map((version) => (
                      <label
                        key={version}
                        className="flex items-center space-x-3 p-3 bg-gray-700/50 rounded-lg hover:bg-gray-700/70 cursor-pointer transition-colors"
                      >
                        <input
                          type="radio"
                          name="version"
                          value={version}
                          checked={selectedVersion === version}
                          onChange={() => handleVersionChange(version)}
                          className="text-purple-600 focus:ring-purple-500"
                        />
                        <span className="text-white font-medium">{version}</span>
                        <div className="flex items-center space-x-2 ml-auto">
                          {versionDirectories[version] ? (
                            <div className="flex items-center space-x-1">
                              <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                              <span className="text-green-400 text-xs">Configured</span>
                            </div>
                          ) : (
                            <div className="flex items-center space-x-1">
                              <div className="w-2 h-2 bg-yellow-400 rounded-full"></div>
                              <span className="text-yellow-400 text-xs">Not Set</span>
                            </div>
                          )}
                        </div>
                      </label>
                    ))}
                  </div>
                </div>

                {/* Game Directory */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-white font-semibold">Game Directory</h4>
                    <button
                      onClick={handleOpenDirectory}
                      className="flex items-center space-x-2 px-3 py-1 bg-gray-700 text-gray-300 rounded hover:bg-gray-600 transition-colors"
                    >
                      <Folder className="w-4 h-4" />
                      <span>Open Directory</span>
                    </button>
                  </div>

                  <div className="bg-gray-700/50 rounded p-3 mb-3">
                    <p className="text-gray-300 font-mono text-sm">
                      {getCurrentDirectory() || 'No directory set for this version'}
                    </p>
                    {!getCurrentDirectory() && (
                      <p className="text-yellow-400 text-xs mt-1">
                        ⚠️ Directory not configured for {selectedVersion}
                      </p>
                    )}
                  </div>

                  <div>
                    <h5 className="text-white font-medium mb-2">Relocate Game</h5>
                    <p className="text-gray-400 text-sm mb-3">
                      {getCurrentDirectory()
                        ? `Update the directory path for ${selectedVersion}. Select the folder where "${game.title}.exe" is located.`
                        : `Set the directory path for ${selectedVersion}. Select the folder where "${game.title}.exe" is located.`
                      }
                    </p>
                    <button
                      onClick={handleRelocate}
                      className="flex items-center space-x-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
                    >
                      <RotateCcw className="w-4 h-4" />
                      <span>{getCurrentDirectory() ? 'Relocate' : 'Set Directory'}</span>
                    </button>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'advanced' && (
              <div className="space-y-6">
                <h3 className="text-lg font-semibold text-white mb-4">Advanced Settings</h3>

                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Launch Options</h4>
                  <div className="space-y-4">
                    <div>
                      <label className="block text-gray-300 text-sm mb-2">Command Line Arguments</label>
                      <input
                        type="text"
                        className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:border-purple-500 focus:outline-none"
                        placeholder="--windowed --fps-limit=60"
                      />
                    </div>
                    <div className="flex items-center space-x-3">
                      <input type="checkbox" id="admin" className="text-purple-600 focus:ring-purple-500" />
                      <label htmlFor="admin" className="text-gray-300">Run as Administrator</label>
                    </div>
                    <div className="flex items-center space-x-3">
                      <input type="checkbox" id="compatibility" className="text-purple-600 focus:ring-purple-500" />
                      <label htmlFor="compatibility" className="text-gray-300">Compatibility Mode</label>
                    </div>
                  </div>
                </div>

                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Performance</h4>
                  <div className="space-y-4">
                    <div>
                      <label className="block text-gray-300 text-sm mb-2">CPU Priority</label>
                      <select className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:border-purple-500 focus:outline-none">
                        <option>Normal</option>
                        <option>High</option>
                        <option>Real-time</option>
                      </select>
                    </div>
                    <div className="flex items-center space-x-3">
                      <input type="checkbox" id="overlay" className="text-purple-600 focus:ring-purple-500" />
                      <label htmlFor="overlay" className="text-gray-300">Enable Game Overlay</label>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'logs' && (
              <div className="space-y-6">
                <h3 className="text-lg font-semibold text-white mb-4">Log Information</h3>

                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Recent Activity</h4>
                  <div className="space-y-3">
                    <div className="flex items-center space-x-3 p-3 bg-gray-700/50 rounded">
                      <Calendar className="w-4 h-4 text-green-400" />
                      <div>
                        <p className="text-white text-sm">Game launched successfully</p>
                        <p className="text-gray-400 text-xs">2024-01-15 14:30:22</p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-3 p-3 bg-gray-700/50 rounded">
                      <Clock className="w-4 h-4 text-blue-400" />
                      <div>
                        <p className="text-white text-sm">Update completed</p>
                        <p className="text-gray-400 text-xs">2024-01-14 09:15:45</p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-3 p-3 bg-gray-700/50 rounded">
                      <HardDrive className="w-4 h-4 text-yellow-400" />
                      <div>
                        <p className="text-white text-sm">Cache cleared</p>
                        <p className="text-gray-400 text-xs">2024-01-13 16:45:12</p>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Error Logs</h4>
                  <div className="bg-gray-900/50 rounded p-3 font-mono text-sm text-gray-300 max-h-40 overflow-y-auto">
                    <p>[2024-01-15 14:30:22] INFO: Game started</p>
                    <p>[2024-01-15 14:30:23] INFO: Loading assets...</p>
                    <p>[2024-01-15 14:30:25] INFO: Assets loaded successfully</p>
                    <p>[2024-01-15 14:30:26] INFO: Connecting to server...</p>
                    <p>[2024-01-15 14:30:27] INFO: Connected successfully</p>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};