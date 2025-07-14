import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { Game, GameEngine } from '../types';
import { Play, Settings, Download, Clock, Square } from 'lucide-react';
import { GameSettingsModal } from './GameSettingsModal';
import { EngineSelectionModal } from './EngineSelectionModal';
import { SSLCertificateModal } from './SSLCertificateModal';
import { PatchErrorInfo, PatchErrorModal } from './PatchErrorModal';
import { PatchMessageModal, shouldIgnoreMessage } from './PatchMessageModal';
import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';
// Removed startProxyWithSSLCheck import - proxy is now managed by backend after patching

interface GameDetailsProps {
  game: Game;
  onGameUpdate: (updatedGame: Game) => void;
  onGameRunningStatusChange: (gameId: number, isRunning: boolean) => void;
}

// Constants
const STORAGE_KEY_PREFIX = 'game-';
const STORAGE_KEY_SUFFIX = '-directories-v2';

export const GameDetails: React.FC<GameDetailsProps> = ({ game, onGameUpdate, onGameRunningStatusChange }) => {
  const [showSettings, setShowSettings] = useState(false);
  const [showEngineSelection, setShowEngineSelection] = useState(false);
  const [showSSLModal, setShowSSLModal] = useState(false);
  const [showPatchError, setShowPatchError] = useState(false);
  const [showPatchMessage, setShowPatchMessage] = useState(false);
  const [patchErrorInfo, setPatchErrorInfo] = useState<PatchErrorInfo | null>(null);
  const [patchMessage, setPatchMessage] = useState<string>('');
  const [isLaunching, setIsLaunching] = useState(false);
  const [isGameRunning, setIsGameRunning] = useState(false);
  const [isInstalled, setIsInstalled] = useState(false);
  const [pendingLaunch, setPendingLaunch] = useState<{ engine?: GameEngine; version?: string; channel?: number } | null>(null);
  const [pendingPatchLaunch, setPendingPatchLaunch] = useState<{ engine?: GameEngine; version?: string; channel?: number } | null>(null);
  const [gameProcessId, setGameProcessId] = useState<number | null>(null);
  const [monitorInterval, setMonitorInterval] = useState<NodeJS.Timeout | null>(null);

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

  const stopGameMonitoring = useCallback(() => {
    if (monitorInterval) {
      clearInterval(monitorInterval);
      setMonitorInterval(null);
    }
  }, [monitorInterval]);

  // Unified function to validate game folder path
  const validateGameFolderPath = useCallback((version: string, channel: number): string | null => {
    const gameFolderPath = getGameFolderPath(version, channel);
    if (!gameFolderPath) {
      alert(`Game folder path not configured for ${game.title} version ${version} (Channel ${channel}). Opening game settings to configure the path.`);
      setIsLaunching(false);
      setShowSettings(true);
      return null;
    }
    return gameFolderPath;
  }, [getGameFolderPath, game.title]);

  const handleGameStopped = useCallback(async () => {
    setIsGameRunning(false);
    onGameRunningStatusChange(game.id, false);
    setGameProcessId(null);
    stopGameMonitoring();
    
    // Stop proxy when game closes
    try {
      await invoke('stop_proxy');
    } catch (error) {
      console.error('Error stopping proxy:', error);
    }
  }, [stopGameMonitoring, onGameRunningStatusChange, game.id]);

  const startGameMonitoring = useCallback(() => {
    // Clear any existing interval
    stopGameMonitoring();
    
    // Start lightweight monitoring - only check if backend monitor is active
    // Backend monitor handles all game state detection and proxy management
    const interval = setInterval(async () => {
      try {
        // Only check if backend game monitor is still active
        const isMonitorActive = await invoke('is_game_monitor_active');
        
        setIsGameRunning(currentIsGameRunning => {
          if (!isMonitorActive && currentIsGameRunning) {
            // Backend monitor stopped, which means game stopped
            handleGameStopped();
            return false;
          }
          
          return currentIsGameRunning;
        });
      } catch (error) {
        console.error('Error checking monitor status:', error);
        // If we can't check status, assume game stopped
        handleGameStopped();
      }
    }, 5000); // Check every 5 seconds (reduced frequency)
    
    setMonitorInterval(interval);
  }, [stopGameMonitoring, handleGameStopped]);

  const launchGameWithEngine = useCallback(async (engine: GameEngine, version: string, channel: number) => {
    try {
      const gameFolderPath = validateGameFolderPath(version, channel);
      if (!gameFolderPath) {
        return;
      }

      console.log(`Launching game ${game.id}/${version}/${channel} from folder: ${gameFolderPath}`);
      
      // Get delete hoyo pass setting from localStorage (default: true)
      const deleteHoyoPassSetting = localStorage.getItem('delete-hoyo-pass-setting');
      const deleteHoyoPass = deleteHoyoPassSetting ? JSON.parse(deleteHoyoPassSetting) : true;
      
      const result = await invoke('launch_game', {
        gameId: game.id,
        version: version,
        channel: channel,
        gameFolderPath: gameFolderPath,
        deleteHoyoPass: deleteHoyoPass
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
      onGameRunningStatusChange(game.id, true);
    } catch (error) {
      console.error('Error launching game:', error);
      
      // Check if this is a patch 404 error
      const errorString = String(error);
      if (errorString.includes('PATCH_ERROR_404:')) {
        try {
          const errorJson = errorString.split('PATCH_ERROR_404:')[1];
          const patchError = JSON.parse(errorJson);
          setPatchErrorInfo(patchError);
          setShowPatchError(true);
        } catch (parseError) {
          console.error('Failed to parse patch error info:', parseError);
          alert(`Failed to launch ${game.title} with ${engine.name}: ${error}`);
        }
      } else {
        alert(`Failed to launch ${game.title} with ${engine.name}: ${error}`);
      }
    } finally {
      setIsLaunching(false);
    }
  }, [game, validateGameFolderPath, onGameUpdate, startGameMonitoring, onGameRunningStatusChange]);

  const checkGameInstallation = useCallback(async () => {
    try {
      // const installed = await invoke('check_game_installed', { gameId: game.id.toString() });
      setIsInstalled(true); // TODO
    } catch (error) {
      console.error('Error checking game installation:', error);
      setIsInstalled(false);
    }
  }, []);

  useEffect(() => {
    // Check if game is installed when component mounts or game changes
    checkGameInstallation();
  }, [checkGameInstallation]);

  useEffect(() => {
    // Cleanup monitor interval on unmount
    return () => {
      if (monitorInterval) {
        clearInterval(monitorInterval);
      }
    };
  }, [monitorInterval]);

  const stopGame = useCallback(async () => {
    try {
      // Force stop the game monitor for clean shutdown
      await invoke('force_stop_game_monitor');
      
      if (gameProcessId) {
        await invoke('stop_game_process', { processId: gameProcessId });
      } else {
        await invoke('stop_game', { gameId: game.id });
      }
      
      // Immediately update UI state
      await handleGameStopped();
      
      // Also check monitor status to ensure backend state is synced
      setTimeout(async () => {
        try {
          const isMonitorActive = await invoke('is_game_monitor_active');
          if (!isMonitorActive) {
            await handleGameStopped();
          }
        } catch (error) {
          console.error('Error checking monitor status after stop:', error);
        }
      }, 100);
    } catch (error) {
      console.error('Error stopping game:', error);
      alert(`Failed to stop ${game.title}: ${error}`);
    }
  }, [gameProcessId, game.id, game.title, handleGameStopped]);

  const onPlayButtonClick = useCallback(async () => {
    if (isGameRunning) {
      // Stop the game if it's running
      await stopGame();
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
  }, [isGameRunning, game.engine, stopGame]);

  const handleEngineLaunch = useCallback(async (engine: GameEngine, version: string, channel: number) => {
    setIsLaunching(true);
    try {
      // First validate game folder path
      console.log("channel_id: "+channel)
      const gameFolderPath = validateGameFolderPath(version, channel);
      if (!gameFolderPath) {
        return;
      }

      // Check for patch messages
      const patchCheckResult = await invoke('check_patch_message', {
        gameId: game.id,
        version: version,
        channel: channel,
        gameFolderPath: gameFolderPath
      });

      if (typeof patchCheckResult === 'string') {
        const checkResult = JSON.parse(patchCheckResult);
        
        if (checkResult.has_message && checkResult.message) {
          // Check if this message should be ignored
          if (!shouldIgnoreMessage(checkResult.message)) {
            // Store the pending launch details and show patch message modal
            setPendingPatchLaunch({ engine, version, channel });
            setPatchMessage(checkResult.message);
            setShowPatchMessage(true);
            setIsLaunching(false);
            return;
          }
        }
      }
      
      // Check SSL certificate status before launching
      const sslInstalled = await invoke('check_ssl_certificate_installed');
      
      if (!sslInstalled) {
        // Store the pending launch details and show SSL modal
        setPendingLaunch({ engine, version, channel });
        setShowSSLModal(true);
        setIsLaunching(false);
        return;
      }
      
      // Proceed with actual game launch
      await launchGameWithEngine(engine, version, channel);
    } catch (error) {
      console.error('Error in pre-launch checks:', error);
      // Continue with launch even if SSL check fails
      await launchGameWithEngine(engine, version, channel);
    }
  }, [launchGameWithEngine, validateGameFolderPath, game.id]);

  // Unified function to resume launch with SSL check
  const resumeLaunchWithSSLCheck = useCallback(async (engine: GameEngine, version: string, channel: number) => {
    setIsLaunching(true);
    try {
      const sslInstalled = await invoke('check_ssl_certificate_installed');
      
      if (!sslInstalled) {
        setPendingLaunch({ engine, version, channel });
        setShowSSLModal(true);
        setIsLaunching(false);
        return;
      }
      
      await launchGameWithEngine(engine, version, channel);
    } catch (error) {
      console.error('Error in SSL check during launch:', error);
      await launchGameWithEngine(engine, version, channel);
    }
  }, [launchGameWithEngine]);

  // Unified function to cancel any pending launch
  const cancelPendingLaunch = useCallback((type: 'ssl' | 'patch') => {
    if (type === 'ssl') {
      setShowSSLModal(false);
      setPendingLaunch(null);
    } else {
      setShowPatchMessage(false);
      setPendingPatchLaunch(null);
    }
  }, []);

  // SSL Modal Handlers
  const onSSLInstallComplete = useCallback(() => {
    if (pendingLaunch) {
      const { engine, version, channel } = pendingLaunch;
      setPendingLaunch(null);
      setShowSSLModal(false);
      if (engine && version && channel) {
        setIsLaunching(true);
        launchGameWithEngine(engine, version, channel);
      }
    }
  }, [pendingLaunch, launchGameWithEngine]);

  const onSSLModalClose = useCallback(async () => {
    if (pendingLaunch) {
      const { engine, version, channel } = pendingLaunch;
      const proceed = await confirm(
        'SSL certificate is not installed. HTTPS game traffic may not work properly. Do you want to continue anyway?',
        {
          title: 'SSL Certificate Warning',
          kind: 'warning'
        }
      );
      if (proceed && engine && version && channel) {
        setPendingLaunch(null);
        setShowSSLModal(false);
        setIsLaunching(true);
        launchGameWithEngine(engine, version, channel);
      } else {
        cancelPendingLaunch('ssl');
      }
    } else {
      cancelPendingLaunch('ssl');
    }
  }, [pendingLaunch, launchGameWithEngine, cancelPendingLaunch]);

  const onSSLModalCancel = useCallback(() => {
    cancelPendingLaunch('ssl');
  }, [cancelPendingLaunch]);

  // Patch Message Modal Handlers
  const onPatchMessageContinue = useCallback(async () => {
    if (pendingPatchLaunch) {
      const { engine, version, channel } = pendingPatchLaunch;
      setPendingPatchLaunch(null);
      setShowPatchMessage(false);
      if (engine && version && channel) {
        await resumeLaunchWithSSLCheck(engine, version, channel);
      }
    }
  }, [pendingPatchLaunch, resumeLaunchWithSSLCheck]);

  const onPatchMessageClose = useCallback(() => {
    cancelPendingLaunch('patch');
  }, [cancelPendingLaunch]);

  const openSettings = useCallback(() => {
    setShowSettings(true);
  }, []);

  const onVersionChange = useCallback((_gameId: number, newVersion: string) => {
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
    if (isLaunching) {
      return { icon: <Play className="w-6 h-6 animate-spin" />, text: 'Launching...' };
    }
    if (isGameRunning) {
      return { icon: <Square className="w-6 h-6" />, text: 'Stop Game' };
    }
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
                onClick={onPlayButtonClick}
                disabled={isLaunching}
                className={buttonStyles}
              >
                {buttonContent.icon}
                <span>{buttonContent.text}</span>
              </button>
              
              <button
                onClick={openSettings}
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
        onVersionChange={onVersionChange}
        isGameRunning={isGameRunning}
      />
      
      <EngineSelectionModal
        game={game}
        isOpen={showEngineSelection}
        onClose={() => setShowEngineSelection(false)}
        onLaunch={handleEngineLaunch}
      />
      
      <SSLCertificateModal
        isOpen={showSSLModal}
        onClose={onSSLModalClose}
        onCancel={onSSLModalCancel}
        onInstallComplete={onSSLInstallComplete}
      />
      
      <PatchErrorModal
        isOpen={showPatchError}
        onClose={() => setShowPatchError(false)}
        errorInfo={patchErrorInfo}
        gameTitle={game.title}
      />
      
      <PatchMessageModal
        isOpen={showPatchMessage}
        message={patchMessage}
        gameTitle={game.title}
        onContinue={onPatchMessageContinue}
        onClose={onPatchMessageClose}
      />
    </>
  );
};