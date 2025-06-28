import React, { useState } from 'react';
import { X, Folder, RotateCcw, Trash2, HardDrive, Calendar, Clock } from 'lucide-react';
import { Game } from '../types';

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
  const [selectedVersion, setSelectedVersion] = useState(game.version);

  // Mock available versions for demo
  const availableVersions = [
    'v4.2.0',
    'v4.1.5',
    'v4.1.0',
    'v4.0.8',
    'v3.9.2'
  ];

  const handleVersionChange = (version: string) => {
    setSelectedVersion(version);
    onVersionChange(game.id, version);
  };

  const handleUninstall = () => {
    alert('This is a web demo. In the desktop version, this would uninstall the game.');
  };

  const handleOpenDirectory = () => {
    alert('This is a web demo. In the desktop version, this would open the game directory.');
  };

  const handleRelocate = () => {
    alert('This is a web demo. In the desktop version, this would open a folder selection dialog.');
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
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
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'basic'
                    ? 'bg-purple-600/30 text-purple-400 border border-purple-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                }`}
              >
                Basic Information
              </button>
              <button
                onClick={() => setActiveTab('advanced')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'advanced'
                    ? 'bg-purple-600/30 text-purple-400 border border-purple-500/50'
                    : 'text-gray-300 hover:bg-gray-700/50'
                }`}
              >
                Advanced Settings
              </button>
              <button
                onClick={() => setActiveTab('logs')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'logs'
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
                <h3 className="text-lg font-semibold text-white mb-4">Basic Information</h3>
                
                {/* Game Info */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <div className="flex items-center space-x-4 mb-4">
                    <img
                      src={game.image || game.thumbnail || game.icon}
                      alt={game.title}
                      className="w-16 h-16 rounded-lg object-cover"
                    />
                    <div className="flex-1">
                      <h4 className="text-white font-semibold text-lg">{game.title}</h4>
                      <p className="text-gray-400">{game.subtitle}</p>
                    </div>
                    <button
                      onClick={handleUninstall}
                      className="flex items-center space-x-2 px-4 py-2 bg-red-600/20 text-red-400 rounded-lg hover:bg-red-600/30 transition-colors"
                    >
                      <Trash2 className="w-4 h-4" />
                      <span>Uninstall Game</span>
                    </button>
                  </div>

                  <div className="grid grid-cols-3 gap-4 text-sm">
                    <div>
                      <p className="text-gray-400">Size</p>
                      <p className="text-white font-medium">{game.size}</p>
                    </div>
                    <div>
                      <p className="text-gray-400">Installation Time</p>
                      <p className="text-white font-medium">2024-06-04</p>
                    </div>
                    <div>
                      <p className="text-gray-400">Last Time Played</p>
                      <p className="text-white font-medium">{game.lastPlayed || 'Never'}</p>
                    </div>
                  </div>
                </div>

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
                        {version === game.version && (
                          <span className="text-green-400 text-sm">(Current)</span>
                        )}
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
                    <p className="text-gray-300 font-mono text-sm">F:/Game/GI/{game.title} game</p>
                  </div>

                  <div>
                    <h5 className="text-white font-medium mb-2">Relocate Game</h5>
                    <p className="text-gray-400 text-sm mb-3">
                      To locate again, please select the folder where "{game.title}.exe" is located.
                    </p>
                    <button
                      onClick={handleRelocate}
                      className="flex items-center space-x-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
                    >
                      <RotateCcw className="w-4 h-4" />
                      <span>Locate Again</span>
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