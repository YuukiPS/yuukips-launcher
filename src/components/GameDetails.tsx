import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { Game, GameEngine } from '../types';
import { Play, Settings, Download, Clock, Square } from 'lucide-react';
import { GameSettingsModal } from './GameSettingsModal';
import { EngineSelectionModal } from './EngineSelectionModal';
import { SSLCertificateModal } from './SSLCertificateModal';
import { invoke } from '@tauri-apps/api/core';
import { startProxyWithSSLCheck } from '../services/gameApi';

interface GameDetailsProps {
  game: Game;
  onGameUpdate: (updatedGame: Game) => void;
}

// Constants
const MONITOR_INTERVAL_MS = 2000;
const STORAGE_KEY_PREFIX = 'game-';
const STORAGE_KEY_SUFFIX = '-directories-v2';

export const GameDetails: React.FC<GameDetailsProps> = ({ game, onGameUpdate }) => {
  const [showSettings, setShowSettings] = useState(false);
  const [showEngineSelection, setShowEngineSelection] = useState(false);
  const [showSSLModal, setShowSSLModal] = useState(false);
  const [isLaunching, setIsLaunching] = useState(false);
  const [isGameRunning, setIsGameRunning] = useState(false);
  const [isInstalled, setIsInstalled] = useState(false);
  const [pendingLaunch, setPendingLaunch] = useState<{ engine?: GameEngine; version?: string; channel?: number } | null>(null);
  const [gameProcessId, setGameProcessId] = useState<number | null>(null);
  const [monitorInterval, setMonitorInterval] = useState<NodeJS.Timeout | null>(null);

  useEffect(() => {
    // Check if game is installed when component mounts or game changes
    checkGameInstallation();
  }, [game.id]);

  useEffect(() => {
    // Cleanup monitor interval on unmount
    return () => {
      if (monitorInterval) {
        clearInterval(monitorInterval);
      }
    };
  }, [monitorInterval]);

  const checkGameInstallation = useCallback(async () => {
    try {
      // const installed = await invoke('check_game_installed', { gameId: game.id.toString() });
      setIsInstalled(true); // TODO
    } catch (error) {
      console.error('Error checking game installation:', error);
      setIsInstalled(false);
    }
  }, [game.id]);

  const handlePlay = useCallback(async () => {
    if (isGameRunning) {
      // Stop the game if it's running
      await handleStopGame();
    } else {
      // Check if game has engines (API games) or is a legacy game
      if (game.engine && game.engine.length > 0) {
        // Show engine selection modal for API games
        setShowEngineSelection(true);
      } else {
        // Legacy (no more)
        alert(`No Game?`);
      }
    }
  }, [isGameRunning, game.engine]);

  const handleEngineLaunch = useCallback(async (engine: GameEngine, version: string, channel: number) => {
    setIsLaunching(true);
    try {
      // Check proxy and SSL status before launching
      const proxyStatus = await startProxyWithSSLCheck();
      
      if (proxyStatus.needsSSL) {
        // Store the pending launch details and show SSL modal
        setPendingLaunch({ engine, version, channel });
        setShowSSLModal(true);
        setIsLaunching(false);
        return;
      }
      
      // Proceed with game launch
      await launchGameWithEngine(engine, version, channel);
    } catch (error) {
      console.error('Error in pre-launch checks:', error);
      // Continue with launch even if proxy check fails
      await launchGameWithEngine(engine, version, channel);
    }
  }, []);

  const getGameFolderPath = useCallback((version: string, channel: number): string => {
    const storageKey = `${STORAGE_KEY_PREFIX}${game.id}${STORAGE_KEY_SUFFIX}`;
    const savedDirectories = localStorage.getItem(storageKey);
    
    if (!savedDirectories) return '';
    
    try {
      const directories = JSON.parse(savedDirectories);
      // Handle both old format (version -> path) and new format (version -> channel -> path)
      if (typeof directories[version] === 'string') {
        // Old format: migrate to new format
        return directories[version] || '';
      } else if (typeof directories[version] === 'object' && directories[version]) {
        // New format: get path for specific channel
        return directories[version][channel] || '';
      }
    } catch (error) {
      console.error('Failed to parse saved directories:', error);
    }
    
    return '';
  }, [game.id]);

  const launchGameWithEngine = useCallback(async (engine: GameEngine, version: string, channel: number) => {
    try {
      const gameFolderPath = getGameFolderPath(version, channel);
      
      if (!gameFolderPath) {
        alert(`Game folder path not configured for ${game.title} version ${version} (Channel ${channel}). Please set it in game settings.`);
        setIsLaunching(false);
        return;
      }
      
      const result = await invoke('launch_game_with_engine', {
        gameId: game.id,
        gameTitle: game.title,
        engineId: engine.id,
        engineName: engine.name,
        version: version,
        gameFolderPath: gameFolderPath
      });
      
      console.log('Game launch result:', result);
      
      // Extract process ID from result if available
      if (result && typeof result === 'string') {
        try {
          const parsedResult = JSON.parse(result);
          if (parsedResult.processId) {
            setGameProcessId(parsedResult.processId);
          }
        } catch (error) {
          console.error('Failed to parse game launch result:', error, result);
        }
      } else if (result && typeof result === 'object' && 'processId' in result) {
        setGameProcessId((result as { processId: number }).processId);
      }
      
      // Start monitoring the game process
      startGameMonitoring();
      
      // Update last played time
      const updatedGame = { ...game, lastPlayed: 'Just now', version: version };
      onGameUpdate(updatedGame);
      
      setIsGameRunning(true);
    } catch (error) {
      console.error('Error launching game:', error);
      alert(`Failed to launch ${game.title} with ${engine.name}: ${error}`);
    } finally {
      setIsLaunching(false);
    }
  }, [game, getGameFolderPath, onGameUpdate]);

  const stopGameMonitoring = useCallback(() => {
    if (monitorInterval) {
      clearInterval(monitorInterval);
      setMonitorInterval(null);
    }
  }, [monitorInterval]);

  const handleGameStopped = useCallback(async () => {
    setIsGameRunning(false);
    setGameProcessId(null);
    stopGameMonitoring();
    
    // Stop proxy when game closes
    try {
      await invoke('stop_proxy');
    } catch (error) {
      console.error('Error stopping proxy:', error);
    }
  }, [stopGameMonitoring]);

  const startGameMonitoring = useCallback(() => {
    // Clear any existing interval
    stopGameMonitoring();
    
    // Start monitoring every 2 seconds
    const interval = setInterval(async () => {
      try {
        const isRunning = await invoke('check_game_running', { gameId: game.id });
        
        if (!isRunning && isGameRunning) {
          // Game has stopped
          await handleGameStopped();
        }
      } catch (error) {
        console.error('Error checking game status:', error);
        // If we can't check status, assume game stopped
        await handleGameStopped();
      }
    }, MONITOR_INTERVAL_MS);
    
    setMonitorInterval(interval);
  }, [game.id, isGameRunning, stopGameMonitoring, handleGameStopped]);

  const handleStopGame = useCallback(async () => {
    try {
      if (gameProcessId) {
        await invoke('stop_game_process', { processId: gameProcessId });
      } else {
        await invoke('stop_game', { gameId: game.id });
      }
      
      await handleGameStopped();
    } catch (error) {
      console.error('Error stopping game:', error);
      alert(`Failed to stop ${game.title}: ${error}`);
    }
  }, [gameProcessId, game.id, game.title, handleGameStopped]);

  const handleSSLInstallComplete = useCallback(() => {
    // Resume the pending launch after SSL installation
    if (pendingLaunch) {
      const { engine, version, channel } = pendingLaunch;
      setPendingLaunch(null);
      if (engine && version && channel) {
        setIsLaunching(true);
        launchGameWithEngine(engine, version, channel);
      }
    }
  }, [pendingLaunch, launchGameWithEngine]);

  const handleSSLModalClose = useCallback(() => {
    setShowSSLModal(false);
    // If user closes modal without installing, still allow launch but warn
    if (pendingLaunch) {
      const { engine, version, channel } = pendingLaunch;
      setPendingLaunch(null);
      if (engine && version && channel) {
        const proceed = confirm(
          'SSL certificate is not installed. HTTPS game traffic may not work properly. Do you want to continue anyway?'
        );
        if (proceed) {
          setIsLaunching(true);
          launchGameWithEngine(engine, version, channel);
        }
      }
    }
  }, [pendingLaunch, launchGameWithEngine]);



  const handleSettings = useCallback(() => {
    setShowSettings(true);
  }, []);

  const handleVersionChange = useCallback((gameId: number, newVersion: string) => {
    const updatedGame = { ...game, version: newVersion };
    onGameUpdate(updatedGame);
  }, [game, onGameUpdate]);

  // Memoized button styles
  const buttonStyles = useMemo(() => {
    const baseStyles = 'flex items-center space-x-3 font-bold px-8 py-4 rounded-xl transition-all duration-200 text-lg';
    const disabledStyles = 'disabled:from-gray-600 disabled:to-gray-700 disabled:cursor-not-allowed';
    
    if (isGameRunning) {
      return `${baseStyles} bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 hover:shadow-lg hover:shadow-red-500/25 text-white ${disabledStyles}`;
    }
    
    return `${baseStyles} bg-gradient-to-r from-yellow-500 to-yellow-600 hover:from-yellow-600 hover:to-yellow-700 hover:shadow-lg hover:shadow-yellow-500/25 text-black ${disabledStyles}`;
  }, [isGameRunning]);

  // Memoized button content
  const buttonContent = useMemo(() => {
    if (isLaunching) return { icon: <Play className="w-6 h-6 animate-spin" />, text: 'Launching...' };
    if (isGameRunning) return { icon: <Square className="w-6 h-6" />, text: 'Stop Game' };
    return { icon: <Play className="w-6 h-6" />, text: 'Start Game' };
  }, [isLaunching, isGameRunning]);

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
                disabled={isLaunching}
                className={buttonStyles}
              >
                {buttonContent.icon}
                <span>{buttonContent.text}</span>
              </button>
              
              <button
                onClick={handleSettings}
                className="flex items-center space-x-2 bg-gray-800/80 hover:bg-gray-700/80 text-white px-6 py-4 rounded-xl transition-all duration-200 backdrop-blur-sm border border-gray-600/50 hover:border-gray-500/50"
              >
                <Settings className="w-5 h-5" />
                <span>Game Settings</span>
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
      
      <SSLCertificateModal
        isOpen={showSSLModal}
        onClose={handleSSLModalClose}
        onInstallComplete={handleSSLInstallComplete}
      />
    </>
  );
};