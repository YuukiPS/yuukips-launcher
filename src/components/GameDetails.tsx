import React, { useState, useEffect } from 'react';
import { Game, GameEngine } from '../types';
import { Play, Settings, Download, Clock, Folder } from 'lucide-react';
import { GameSettingsModal } from './GameSettingsModal';
import { EngineSelectionModal } from './EngineSelectionModal';
import { invoke } from '@tauri-apps/api/core';

interface GameDetailsProps {
  game: Game;
  onGameUpdate: (updatedGame: Game) => void;
}

export const GameDetails: React.FC<GameDetailsProps> = ({ game, onGameUpdate }) => {
  const [showSettings, setShowSettings] = useState(false);
  const [showEngineSelection, setShowEngineSelection] = useState(false);
  const [isLaunching, setIsLaunching] = useState(false);
  const [isInstalled, setIsInstalled] = useState(false);

  useEffect(() => {
    // Check if game is installed when component mounts or game changes
    checkGameInstallation();
  }, [game.id]);

  const checkGameInstallation = async () => {
    try {
      const installed = await invoke('check_game_installed', { gameId: game.id });
      setIsInstalled(installed as boolean);
    } catch (error) {
      console.error('Error checking game installation:', error);
      setIsInstalled(false);
    }
  };

  const handlePlay = async () => {
    // Check if game has engines (API games) or is a legacy game
    if (game.engine && game.engine.length > 0) {
      // Show engine selection modal for API games
      setShowEngineSelection(true);
    } else {
      // Legacy behavior for fallback games
      if (!isInstalled) {
        alert(`${game.title} is not installed. Please install the game first.`);
        return;
      }

      setIsLaunching(true);
      try {
        const result = await invoke('launch_game', {
          gameId: game.id,
          gameTitle: game.title
        });
        console.log('Game launch result:', result);
        // Update last played time
        const updatedGame = { ...game, lastPlayed: 'Just now' };
        onGameUpdate(updatedGame);
      } catch (error) {
        console.error('Error launching game:', error);
        alert(`Failed to launch ${game.title}: ${error}`);
      } finally {
        setIsLaunching(false);
      }
    }
  };

  const handleEngineLaunch = async (engine: GameEngine, version: string) => {
    setIsLaunching(true);
    try {
      const result = await invoke('launch_game_with_engine', {
        gameId: game.id,
        gameTitle: game.title,
        engineId: engine.id,
        engineName: engine.name,
        version: version
      });
      console.log('Game launch result:', result);
      // Update last played time
      const updatedGame = { ...game, lastPlayed: 'Just now', version: version };
      onGameUpdate(updatedGame);
    } catch (error) {
      console.error('Error launching game:', error);
      alert(`Failed to launch ${game.title} with ${engine.name}: ${error}`);
    } finally {
      setIsLaunching(false);
    }
  };

  const handleShowFolder = async () => {
    try {
      const result = await invoke('show_game_folder', { gameId: game.id });
      console.log('Folder open result:', result);
    } catch (error) {
      console.error('Error opening game folder:', error);
      alert(`Failed to open game folder: ${error}`);
    }
  };

  const handleSettings = () => {
    setShowSettings(true);
  };

  const handleVersionChange = (gameId: number, newVersion: string) => {
    const updatedGame = { ...game, version: newVersion };
    onGameUpdate(updatedGame);
  };

  return (
    <>
      <div className="flex-1 relative overflow-hidden">
        {/* Background Image */}
        <div 
          className="absolute inset-0 bg-cover bg-center bg-no-repeat"
          style={{ backgroundImage: `url(${game.backgroundUrl})` }}
        >
          <div className="absolute inset-0 bg-gradient-to-r from-gray-900/95 via-gray-900/80 to-gray-900/60" />
          <div className="absolute inset-0 bg-gradient-to-t from-gray-900/90 via-transparent to-transparent" />
        </div>

        {/* Content */}
        <div className="relative z-10 h-full flex flex-col">
          {/* Game Info */}
          <div className="flex-1 p-8">
            <div className="max-w-2xl">


              {/* Title */}
              <h1 className="text-5xl font-bold text-white mb-2">
                {game.title}
              </h1>


              {/* Description */}
              <p className="text-gray-300 text-lg leading-relaxed mb-8 max-w-xl">
                {game.description}
              </p>



              {/* Play Time Info */}
              {game.playTime && (
                <div className="flex items-center space-x-6 mb-8">
                  <div className="flex items-center space-x-2 text-gray-300">
                    <Clock className="w-4 h-4" />
                    <span>Played: {game.playTime}</span>
                  </div>
                  {game.lastPlayed && (
                    <div className="text-gray-400">
                      Last session: {game.lastPlayed}
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>

          {/* Action Buttons */}
          <div className="p-8 bg-gradient-to-t from-gray-900/95 to-transparent">
            <div className="flex items-center space-x-4">
              <button
                onClick={handlePlay}
                disabled={(game.status !== 'available' && game.status !== undefined) || (!isInstalled && !game.engine) || isLaunching}
                className="flex items-center space-x-3 bg-gradient-to-r from-yellow-500 to-yellow-600 hover:from-yellow-600 hover:to-yellow-700 disabled:from-gray-600 disabled:to-gray-700 disabled:cursor-not-allowed text-black font-bold px-8 py-4 rounded-xl transition-all duration-200 hover:shadow-lg hover:shadow-yellow-500/25 text-lg"
              >
                <Play className={`w-6 h-6 ${isLaunching ? 'animate-spin' : ''}`} />
                <span>
                  {isLaunching ? 'Launching...' : 
                   (!isInstalled && !game.engine) ? 'Not Installed' : 
                   'Start Game'}
                </span>
              </button>
              
              <button
                onClick={handleSettings}
                className="flex items-center space-x-2 bg-gray-800/80 hover:bg-gray-700/80 text-white px-6 py-4 rounded-xl transition-all duration-200 backdrop-blur-sm border border-gray-600/50 hover:border-gray-500/50"
              >
                <Settings className="w-5 h-5" />
                <span>Game Settings</span>
              </button>

              <button
                onClick={handleShowFolder}
                className="flex items-center space-x-2 bg-blue-800/80 hover:bg-blue-700/80 text-white px-6 py-4 rounded-xl transition-all duration-200 backdrop-blur-sm border border-blue-600/50 hover:border-blue-500/50"
              >
                <Folder className="w-5 h-5" />
                <span>Open Folder</span>
              </button>

              {game.status === 'updating' && (
                <div className="flex items-center space-x-2 text-yellow-400">
                  <Download className="w-5 h-5 animate-bounce" />
                  <span>Updating...</span>
                </div>
              )}

              {!isInstalled && (
                <div className="flex items-center space-x-2 text-red-400">
                  <Download className="w-5 h-5" />
                  <span>Game not installed</span>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      <GameSettingsModal
        game={game}
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
        onVersionChange={handleVersionChange}
      />
      
      <EngineSelectionModal
        game={game}
        isOpen={showEngineSelection}
        onClose={() => setShowEngineSelection(false)}
        onLaunch={handleEngineLaunch}
      />
    </>
  );
};