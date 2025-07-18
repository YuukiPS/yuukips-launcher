import React, { useState, useEffect } from 'react';
import { X, Play, ExternalLink, ChevronDown, ChevronUp } from 'lucide-react';
import { Game, GameEngine, channelType } from '../types';
import { GameApiService } from '../services/gameApi';

interface EngineSelectionModalProps {
  game: Game;
  isOpen: boolean;
  onClose: () => void;
  onLaunch: (engine: GameEngine, version: string, channel: number) => void;
}

export const EngineSelectionModal: React.FC<EngineSelectionModalProps> = ({
  game,
  isOpen,
  onClose,
  onLaunch
}) => {
  const [selectedVersion, setSelectedVersion] = useState<string>('');
  const [selectedEngine, setSelectedEngine] = useState<GameEngine | null>(null);
  const [selectedChannel, setSelectedChannel] = useState<number>(1);
  const [availableVersions, setAvailableVersions] = useState<string[]>([]);
  const [availableEngines, setAvailableEngines] = useState<GameEngine[]>([]);
  const [availableChannels, setAvailableChannels] = useState<number[]>([]);
  const [showFeatures, setShowFeatures] = useState<{[key: string]: boolean}>({});

  useEffect(() => {
    if (isOpen && game) {
      // Get available versions for PC platform (PlatformType 1)
      const versions = GameApiService.getAvailableVersionsForPlatform(game, 1);
      setAvailableVersions(versions);
      
      // Try to load saved preferences
      const savedPreferences = JSON.parse(localStorage.getItem('gamePreferences') || '{}');
      const gamePreference = savedPreferences[game.id];
      
      if (gamePreference && versions.includes(gamePreference.selectedVersion)) {
        // Use saved version if it's still available
        setSelectedVersion(gamePreference.selectedVersion);
      } else if (versions.length > 0) {
        // Select first version by default
        setSelectedVersion(versions[0]);
      }
    }
  }, [isOpen, game]);

  useEffect(() => {
    if (selectedVersion && game) {
      // Get engines available for selected version
      const engines = GameApiService.getEnginesForVersion(game, selectedVersion, 1);
      setAvailableEngines(engines);
      
      // Try to restore saved engine preference
      const savedPreferences = JSON.parse(localStorage.getItem('gamePreferences') || '{}');
      const gamePreference = savedPreferences[game.id];
      
      if (gamePreference && 
          gamePreference.selectedVersion === selectedVersion && 
          gamePreference.selectedEngine) {
        // Find the saved engine in available engines
        const savedEngine = engines.find(engine => engine.id === gamePreference.selectedEngine.id);
        if (savedEngine) {
          setSelectedEngine(savedEngine);
        } else if (engines.length > 0) {
          setSelectedEngine(engines[0]);
        } else {
          setSelectedEngine(null);
        }
      } else if (engines.length > 0) {
        // Select first engine by default
        setSelectedEngine(engines[0]);
      } else {
        setSelectedEngine(null);
      }
    }
  }, [selectedVersion, game]);

  useEffect(() => {
    if (selectedEngine && selectedVersion) {
      // Get available channels for selected engine and version
      const channels = GameApiService.getAvailableChannelsForEngineVersion(selectedEngine, selectedVersion, 1);
      setAvailableChannels(channels);
      
      // Try to restore saved channel preference
      const savedPreferences = JSON.parse(localStorage.getItem('gamePreferences') || '{}');
      const gamePreference = savedPreferences[game.id];
      
      if (gamePreference && 
          gamePreference.selectedVersion === selectedVersion && 
          gamePreference.selectedEngine?.id === selectedEngine.id &&
          gamePreference.selectedChannel &&
          channels.includes(gamePreference.selectedChannel)) {
        setSelectedChannel(gamePreference.selectedChannel);
      } else if (channels.length > 0) {
        // Select first channel by default
        setSelectedChannel(channels[0]);
      }
    }
  }, [selectedEngine, selectedVersion, game]);

  const handleLaunch = () => {
    if (selectedEngine && selectedVersion && selectedChannel) {
      // Save the selected version, engine, and channel to localStorage
      const gamePreferences = {
        gameId: game.id,
        selectedVersion,
        selectedEngine: {
          id: selectedEngine.id,
          name: selectedEngine.name,
          version: selectedEngine.version
        },
        selectedChannel,
        lastSelected: Date.now()
      };
      
      // Get existing preferences or create new object
      const existingPreferences = JSON.parse(localStorage.getItem('gamePreferences') || '{}');
      existingPreferences[game.id] = gamePreferences;
      
      // Save to localStorage
      localStorage.setItem('gamePreferences', JSON.stringify(existingPreferences));
      
      onLaunch(selectedEngine, selectedVersion, selectedChannel);
      onClose();
    }
  };

  const handleEngineInfoClick = (link: string) => {
    window.open(link, '_blank');
  };

  const getChannelName = (channelId: number): string => {
    switch (channelId) {
      case channelType.None:
        return 'None';
      case channelType.OS:
        return 'Global';
      case channelType.CN:
        return 'China';
      case channelType.JP:
        return 'Japan';
      default:
        return `Channel ${channelId}`;
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-xl p-6 max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-white">Launch {game.title}</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Version Selection */}
        <div className="mb-6">
          <label className="block text-white font-medium mb-3">Select Version:</label>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
            {availableVersions.map((version) => (
              <button
                key={version}
                onClick={() => setSelectedVersion(version)}
                className={`p-3 rounded-lg border transition-all ${
                  selectedVersion === version
                    ? 'border-blue-500 bg-blue-500/20 text-blue-400'
                    : 'border-gray-600 bg-gray-700 text-gray-300 hover:border-gray-500'
                }`}
              >
                {version}
              </button>
            ))}
          </div>
        </div>

        {/* Engine Selection */}
        {availableEngines.length > 0 && (
          <div className="mb-6">
            <label className="block text-white font-medium mb-3">Select Engine:</label>
            <div className="space-y-3">
              {availableEngines.map((engine) => (
                <div
                  key={engine.id}
                  className={`p-4 rounded-lg border cursor-pointer transition-all ${
                    selectedEngine?.id === engine.id
                      ? 'border-purple-500 bg-purple-500/20'
                      : 'border-gray-600 bg-gray-700 hover:border-gray-500'
                  }`}
                  onClick={() => setSelectedEngine(engine)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-2 mb-1">
                        <h3 className="text-white font-medium">{engine.name}</h3>
                        <span className="text-xs bg-gray-600 text-gray-300 px-2 py-1 rounded">
                          {engine.short}
                        </span>
                      </div>
                      <p className="text-gray-400 text-sm mb-2">{engine.description}</p>
                      <p className="text-gray-500 text-xs">Version: {engine.version}</p>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleEngineInfoClick(engine.link);
                      }}
                      className="text-gray-400 hover:text-white transition-colors ml-4"
                      title="More Info"
                    >
                      <ExternalLink className="w-4 h-4" />
                    </button>
                  </div>
                  
                  {/* Engine Features - Toggle visibility */}
                  {engine.features && engine.features.length > 0 && selectedEngine?.id === engine.id && (
                    <div className="mt-3 pt-3 border-t border-gray-600">
                      <button
                        onClick={() => setShowFeatures(prev => ({
                          ...prev,
                          [engine.id]: !prev[engine.id]
                        }))}
                        className="flex items-center space-x-2 text-white text-sm font-medium mb-2 hover:text-purple-400 transition-colors"
                      >
                        <span>Features:</span>
                        {showFeatures[engine.id] ? (
                          <ChevronUp className="w-4 h-4" />
                        ) : (
                          <ChevronDown className="w-4 h-4" />
                        )}
                      </button>
                      {showFeatures[engine.id] && (
                        <ul className="text-gray-400 text-xs space-y-1">
                          {engine.features.map((feature, index) => (
                            <li key={index} className="flex items-start space-x-2">
                              <span className="text-purple-400 mt-1">•</span>
                              <span>{feature}</span>
                            </li>
                          ))}
                        </ul>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Channel Selection */}
        {availableChannels.length > 0 && (
          <div className="mb-6">
            <label className="block text-white font-medium mb-3">Select Channel:</label>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
              {availableChannels.map((channel) => (
                <button
                  key={channel}
                  onClick={() => setSelectedChannel(channel)}
                  className={`p-3 rounded-lg border transition-all ${
                    selectedChannel === channel
                      ? 'border-green-500 bg-green-500/20 text-green-400'
                      : 'border-gray-600 bg-gray-700 text-gray-300 hover:border-gray-500'
                  }`}
                >
                  {getChannelName(channel)}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Action Buttons */}
        <div className="flex items-center justify-end space-x-4">
          <button
            onClick={onClose}
            className="px-6 py-2 text-gray-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleLaunch}
            disabled={!selectedEngine || !selectedVersion || !selectedChannel}
            className="flex items-center space-x-2 bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 disabled:from-gray-600 disabled:to-gray-700 disabled:cursor-not-allowed text-white px-6 py-2 rounded-lg font-medium transition-all duration-200"
          >
            <Play className="w-4 h-4" />
            <span>Launch Game</span>
          </button>
        </div>
      </div>
    </div>
  );
};