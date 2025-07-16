/* eslint-disable @typescript-eslint/no-explicit-any */
import React, { useState, useEffect, useCallback } from 'react';
import { X, Folder, RotateCcw, HardDrive, Calendar, Clock, Check, Trash2, Plus, RefreshCw, Trash, Search, Download, FolderOpen, Loader2, CheckCircle, AlertTriangle } from 'lucide-react';
import { Game } from '../types';
import { GameApiService } from '../services/gameApi';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { PatchErrorModal, PatchErrorInfo } from './PatchErrorModal';
import { DiskScanModal } from './DiskScanModal';

interface GameSettingsModalProps {
  game: Game;
  isOpen: boolean;
  onClose: () => void;
  onVersionChange: (gameId: number, newVersion: string) => void;
  isGameRunning?: boolean;
}

export const GameSettingsModal: React.FC<GameSettingsModalProps> = ({
  game,
  isOpen,
  onClose,
  onVersionChange,
  isGameRunning = false
}) => {
  const [activeTab, setActiveTab] = useState('basic');
  const [selectedVersion, setSelectedVersion] = useState("");
  const [selectedChannel, setSelectedChannel] = useState<number>(0);
  const [availableChannels, setAvailableChannels] = useState<number[]>([]);
  const [versionDirectories, setVersionDirectories] = useState<Record<string, Record<number, string>>>({});
  const [notification, setNotification] = useState<{ message: string; type: 'success' | 'error' } | null>(null);
  const [proxyAddress, setProxyAddress] = useState('https://ps.yuuki.me');
  const [savedProxyServers, setSavedProxyServers] = useState<string[]>(['https://ps.yuuki.me']);
  const [newServerInput, setNewServerInput] = useState('');
  const [proxyLogs, setProxyLogs] = useState<Array<{ timestamp: string, original_url: string, redirected_url: string }>>([]);
  const [autoRefreshLogs, setAutoRefreshLogs] = useState(false);
  const [isProxyRunning, setIsProxyRunning] = useState(false);
  const [proxyStatusLoading, setProxyStatusLoading] = useState(false);
  const [proxyDomains, setProxyDomains] = useState<string[]>([]);
  const [newDomainInput, setNewDomainInput] = useState('');
  const [proxyPort, setProxyPort] = useState<number>(8080);
  const [customPortInput, setCustomPortInput] = useState<string>('');
  const [isValidating, setIsValidating] = useState(false);
  const [patchErrorModalOpen, setPatchErrorModalOpen] = useState(false);
  const [patchErrorInfo, setPatchErrorInfo] = useState<PatchErrorInfo | null>(null);
  const [deleteHoyoPass, setDeleteHoyoPass] = useState<boolean>(true);
  const [diskScanModalOpen, setDiskScanModalOpen] = useState(false);
  const [downloadModalOpen, setDownloadModalOpen] = useState(false);
  const [downloadData, setDownloadData] = useState<any>(null);
  const [isCheckingDownload, setIsCheckingDownload] = useState(false);
  const [selectedDownloadFolder, setSelectedDownloadFolder] = useState<string>('');
  const [isCheckingFiles, setIsCheckingFiles] = useState(false);
  const [fileCheckProgress, setFileCheckProgress] = useState({ current: 0, total: 0 });
  const [fileCheckResults, setFileCheckResults] = useState<{[key: string]: {exists: boolean, md5Match: boolean}}>({});

  // Get available versions dynamically from game engine data
  const availableVersions = GameApiService.getAvailableVersionsForPlatform(game, 1);

  // Reset selectedVersion when game changes or modal opens
  useEffect(() => {
    if (isOpen && availableVersions.length > 0 && !selectedVersion) {
      setSelectedVersion(availableVersions[0]);
    } else if (!isOpen) {
      // Reset state when modal closes
      setSelectedVersion("");
      setSelectedChannel(0);
    }
  }, [isOpen, game.id, availableVersions, selectedVersion]); // Added selectedVersion back to check if already set

  // Load available channels when version changes
  useEffect(() => {
    if (selectedVersion && game.engine && game.engine.length > 0) {
      // Get the first engine for the selected version to determine available channels
      const engineForVersion = game.engine.find(() =>
        GameApiService.getAvailableVersionsForPlatform(game, 1).includes(selectedVersion)
      );

      if (engineForVersion) {
        const channels = GameApiService.getAvailableChannelsForEngineVersion(engineForVersion, selectedVersion, 1);
        setAvailableChannels(channels);

        // Set default channel when version changes
        if (channels.length > 0) {
          setSelectedChannel(channels[0]);
        }
      }
    }
  }, [selectedVersion, game]); // Removed selectedChannel to prevent infinite loop

  // Load saved directories from localStorage on component mount
  useEffect(() => {
    const savedDirectories = localStorage.getItem(`game-${game.id}-directories-v2`);
    if (savedDirectories) {
      try {
        setVersionDirectories(JSON.parse(savedDirectories));
      } catch (error) {
        console.error('Failed to parse saved directories:', error);
      }
    }
  }, [game.id]);

  // Load current proxy address on component mount
  useEffect(() => {
    const loadProxyAddress = async () => {
      try {
        const currentProxy = await invoke('get_proxy_addr');
        if (currentProxy && typeof currentProxy === 'string') {
          setProxyAddress(currentProxy.replace(':443', ''));
        }
      } catch (error) {
        console.error('Failed to load proxy address:', error);
      }
    };
    loadProxyAddress();
  }, []);

  // Load saved proxy servers from localStorage
  useEffect(() => {
    const savedServers = localStorage.getItem('saved-proxy-servers');
    if (savedServers) {
      try {
        const servers = JSON.parse(savedServers);
        if (Array.isArray(servers) && servers.length > 0) {
          setSavedProxyServers(servers);
        }
      } catch (error) {
        console.error('Failed to parse saved proxy servers:', error);
      }
    }
  }, []);

  // Load saved proxy domains from localStorage
  useEffect(() => {
    const savedDomains = localStorage.getItem('saved-proxy-domains');
    if (savedDomains) {
      try {
        const domains = JSON.parse(savedDomains);
        if (Array.isArray(domains) && domains.length > 0) {
          setProxyDomains(domains);
        }
      } catch (error) {
        console.error('Failed to parse saved proxy domains:', error);
      }
    }
  }, []);

  // Load saved delete hoyo pass setting from localStorage
  useEffect(() => {
    const savedDeleteHoyoPass = localStorage.getItem('delete-hoyo-pass-setting');
    if (savedDeleteHoyoPass !== null) {
      try {
        setDeleteHoyoPass(JSON.parse(savedDeleteHoyoPass));
      } catch (error) {
        console.error('Failed to parse saved delete hoyo pass setting:', error);
      }
    }
  }, []);

  // Save proxy domains to localStorage whenever they change
  const saveProxyDomains = useCallback((domains: string[]) => {
    setProxyDomains(domains);
    localStorage.setItem('saved-proxy-domains', JSON.stringify(domains));
  }, []);

  // Fetch user proxy domains from backend (initialize with defaults if empty)
  const fetchProxyDomains = useCallback(async () => {
    try {
      const domains = await invoke('initialize_user_domains_if_empty');
      if (Array.isArray(domains)) {
        saveProxyDomains(domains);
      }
    } catch (error) {
      console.error('Failed to fetch user proxy domains:', error);
    }
  }, [saveProxyDomains]);

  // Load current proxy port
  const loadProxyPort = async () => {
    try {
      const currentPort = await invoke('get_proxy_port');
      if (typeof currentPort === 'number') {
        setProxyPort(currentPort);
      }
    } catch (error) {
      console.error('Failed to load proxy port:', error);
    }
  };

  // Load proxy logs and check proxy status when modal opens
  useEffect(() => {
    if (isOpen) {
      fetchProxyLogs();
      checkProxyStatus();
      fetchProxyDomains();
      loadProxyPort();
    }
  }, [isOpen, fetchProxyDomains]);

  // Auto-refresh logs every 2 seconds when enabled
  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (autoRefreshLogs && isOpen) {
      interval = setInterval(fetchProxyLogs, 2000);
    }
    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, [autoRefreshLogs, isOpen]);

  // Save proxy servers to localStorage whenever they change
  const saveProxyServers = (servers: string[]) => {
    setSavedProxyServers(servers);
    localStorage.setItem('saved-proxy-servers', JSON.stringify(servers));
  };

  // Save directories to localStorage whenever they change
  const saveDirectories = (newDirectories: Record<string, Record<number, string>>) => {
    setVersionDirectories(newDirectories);
    localStorage.setItem(`game-${game.id}-directories-v2`, JSON.stringify(newDirectories));
  };

  // Save delete hoyo pass setting to localStorage whenever it changes
  const saveDeleteHoyoPassSetting = (enabled: boolean) => {
    setDeleteHoyoPass(enabled);
    localStorage.setItem('delete-hoyo-pass-setting', JSON.stringify(enabled));
  };

  // Show notification
  const showNotification = (message: string, type: 'success' | 'error' = 'success') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 3000);
  };

  // Fetch proxy logs from backend
  const fetchProxyLogs = async () => {
    try {
      const logs = await invoke('get_proxy_logs');
      if (Array.isArray(logs)) {
        setProxyLogs(logs);
      }
    } catch (error) {
      console.error('Failed to fetch proxy logs:', error);
    }
  };

  // Clear proxy logs
  const handleClearProxyLogs = async () => {
    try {
      await invoke('clear_proxy_logs');
      setProxyLogs([]);
      showNotification('Proxy logs cleared successfully!');
    } catch (error) {
      console.error('Failed to clear proxy logs:', error);
      showNotification('Failed to clear proxy logs', 'error');
    }
  };

  // Check proxy status
  const checkProxyStatus = async () => {
    try {
      const status = await invoke('check_proxy_status');
      setIsProxyRunning(status as boolean);
    } catch (error) {
      console.error('Failed to check proxy status:', error);
      setIsProxyRunning(false);
    }
  };

  // Start proxy
  const handleStartProxy = async () => {
    setProxyStatusLoading(true);
    try {
      const result = await invoke('start_proxy_with_port', { port: proxyPort });
      setIsProxyRunning(true);
      showNotification(typeof result === 'string' ? result : 'Proxy started successfully!');
    } catch (error) {
      console.error('Failed to start proxy:', error);
      showNotification('Failed to start proxy', 'error');
    } finally {
      setProxyStatusLoading(false);
    }
  };

  // Find available port
  const handleFindAvailablePort = async () => {
    try {
      const availablePort = await invoke('find_available_port');
      if (typeof availablePort === 'number') {
        setProxyPort(availablePort);
        await invoke('set_proxy_port', { port: availablePort });
        showNotification(`Found available port: ${availablePort}`);
      }
    } catch (error) {
      console.error('Failed to find available port:', error);
      showNotification('Failed to find available port', 'error');
    }
  };

  // Set custom port
  const handleSetCustomPort = async () => {
    const port = parseInt(customPortInput);
    if (isNaN(port) || port < 1024 || port > 65535) {
      showNotification('Please enter a valid port number (1024-65535)', 'error');
      return;
    }

    try {
      await invoke('set_proxy_port', { port });
      setProxyPort(port);
      setCustomPortInput('');
      showNotification(`Proxy port set to ${port}`);
    } catch (error) {
      console.error('Failed to set proxy port:', error);
      showNotification(typeof error === 'string' ? error : 'Failed to set proxy port', 'error');
    }
  };

  // Stop proxy
  const handleStopProxy = async () => {
    setProxyStatusLoading(true);
    try {
      await invoke('stop_proxy');
      setIsProxyRunning(false);
      showNotification('Proxy stopped successfully!');
    } catch (error) {
      console.error('Failed to stop proxy:', error);
      showNotification('Failed to stop proxy', 'error');
    } finally {
      setProxyStatusLoading(false);
    }
  };

  // Add new domain
  const handleAddDomain = async () => {
    const trimmedDomain = newDomainInput.trim();
    if (!trimmedDomain) {
      showNotification('Please enter a valid domain', 'error');
      return;
    }

    // Basic validation for domain format (allows domain:port)
    const domainRegex = /^[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(:[0-9]{1,5})?$/;
    if (!domainRegex.test(trimmedDomain)) {
      showNotification('Please enter a valid domain format (e.g., example.com or example.com:8080)', 'error');
      return;
    }

    if (proxyDomains.includes(trimmedDomain)) {
      showNotification(`Domain '${trimmedDomain}' already exists`, 'error');
      return;
    }

    try {
      // Add to frontend state and localStorage
      const updatedDomains = [...proxyDomains, trimmedDomain];
      saveProxyDomains(updatedDomains);

      // Sync with backend
      await invoke('add_proxy_domain', { domain: trimmedDomain });

      setNewDomainInput('');
      showNotification(`Domain '${trimmedDomain}' added successfully`);
    } catch (error) {
      console.error('Failed to add domain:', error);
      showNotification(typeof error === 'string' ? error : 'Failed to add domain', 'error');
    }
  };

  // Remove domain
  const handleRemoveDomain = async (domain: string) => {
    try {
      // Update frontend state and localStorage
      const updatedDomains = proxyDomains.filter(d => d !== domain);
      saveProxyDomains(updatedDomains);

      // Sync with backend
      await invoke('remove_proxy_domain', { domain });

      showNotification(`Domain '${domain}' removed successfully`);
    } catch (error) {
      console.error('Failed to remove domain:', error);
      showNotification(typeof error === 'string' ? error : 'Failed to remove domain', 'error');
    }
  };

  // Delete all domains
  const handleDeleteAllDomains = async () => {
    try {
      // Clear frontend state and localStorage
      saveProxyDomains([]);

      // Sync with backend - remove all current domains
      for (const domain of proxyDomains) {
        await invoke('remove_proxy_domain', { domain });
      }

      showNotification('All domains deleted successfully');
    } catch (error) {
      console.error('Failed to delete all domains:', error);
      showNotification(typeof error === 'string' ? error : 'Failed to delete all domains', 'error');
    }
  };

  // Reset to default domains
  const handleResetToDefaults = async () => {
    try {
      // First clear all current domains
      for (const domain of proxyDomains) {
        await invoke('remove_proxy_domain', { domain });
      }

      // Get default domains and set them as user domains
      const defaultDomains = await invoke('get_proxy_domains');
      if (Array.isArray(defaultDomains)) {
        // Add each default domain to backend
        for (const domain of defaultDomains) {
          await invoke('add_proxy_domain', { domain });
        }

        // Update frontend state and localStorage
        saveProxyDomains(defaultDomains);
        showNotification('Domains reset to defaults successfully');
      }
    } catch (error) {
      console.error('Failed to reset to defaults:', error);
      showNotification(typeof error === 'string' ? error : 'Failed to reset to defaults', 'error');
    }
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
      showNotification(`No directory set for ${selectedVersion} (Channel ${getChannelName(selectedChannel)}). Please set a directory first.`, 'error');
    }
  };

  // Helper function to validate game directory with patch status checking
  const validateGameDirectory = async (path: string): Promise<boolean> => {
    setIsValidating(true);
    try {
      // First validate the basic directory structure and get MD5
      const validationResult = await invoke('validate_game_directory', {
        gameId: game.id,
        channel: selectedChannel,
        gameFolderPath: path
      }) as string;

      // Extract MD5 from validation result
      const md5Match = validationResult.match(/MD5: ([a-fA-F0-9]{32})/);
      if (!md5Match) {
        throw new Error('Could not extract MD5 hash from validation result');
      }
      const md5 = md5Match[1];

      // Then fetch patch info (this can take a long time)
      showNotification('Validating directory and checking patch status...', 'success');
      await invoke('fetch_patch_info_command', {
        gameId: game.id,
        version: selectedVersion,
        channel: selectedChannel,
        md5: md5
      });

      return true;
    } catch (error) {
      console.error('Directory validation failed:', error);

      // Check if this is a patch error with detailed info
      const errorString = String(error);
      if (errorString.includes('PATCH_ERROR_404:')) {
        try {
          const errorJson = errorString.split('PATCH_ERROR_404:')[1];
          const errorInfo: PatchErrorInfo = JSON.parse(errorJson);
          setPatchErrorInfo(errorInfo);
          setPatchErrorModalOpen(true);
          return false;
        } catch (parseError) {
          console.error('Failed to parse patch error info:', parseError);
        }
      }

      showNotification(`Invalid game directory: ${error}`, 'error');
      return false;
    } finally {
      setIsValidating(false);
    }
  };

  const handleDiskScanPathSelected = async (selectedPath: string) => {
    try {
      // Validate the selected directory
      const isValid = await validateGameDirectory(selectedPath);
      if (!isValid) {
        return; // Validation failed, error already shown
      }

      // Save the selected directory
      const updatedDirectories = {
        ...versionDirectories,
        [selectedVersion]: {
          ...versionDirectories[selectedVersion],
          [selectedChannel]: selectedPath
        }
      };
      saveDirectories(updatedDirectories);
      showNotification(`Game directory for ${selectedVersion} (Channel ${getChannelName(selectedChannel)}) set successfully!\n\nPath: ${selectedPath}`);
    } catch (error) {
      console.error('Failed to set selected path:', error);
      showNotification('Failed to set selected path', 'error');
    }
  };

  const handleRelocate = async () => {
    // Directly open disk scan modal for automatic game detection
    setDiskScanModalOpen(true);
  };

  const handleAutoDetect = async () => {
    try {
      setIsValidating(true);

      // First, try to get games from HoyoPlay registry
      try {
        const installedGamesRaw = await invoke('get_hoyoplay_list_game') as Array<[string, string]>;

        // Convert array of arrays to object for easier lookup
        const installedGames: Record<string, string> = {};
        if (installedGamesRaw && Array.isArray(installedGamesRaw)) {
          installedGamesRaw.forEach(([nameCode, path]) => {
            installedGames[nameCode] = path;
          });
        }

        if (installedGames && Object.keys(installedGames).length > 0) {
          // Get all supported game name codes
          const allGameCodes = await invoke('get_all_game_name_codes') as Array<[number, number, string]>;

          // Find matching game installation for current game and channel
          const matchingCode = allGameCodes.find(([gameId, channelId]) =>
            gameId === game.id && channelId === selectedChannel
          );

          if (matchingCode) {
            const [, , nameCode] = matchingCode;
            const installPath = installedGames[nameCode];

            if (installPath) {
              // Automatically set the detected path without confirmation
              const isValid = await validateGameDirectory(installPath);
              if (!isValid) {
                showNotification(`Auto-detected directory for ${selectedVersion} (Channel ${getChannelName(selectedChannel)}) is invalid`, 'error');
                return;
              }

              const updatedDirectories = {
                ...versionDirectories,
                [selectedVersion]: {
                  ...versionDirectories[selectedVersion],
                  [selectedChannel]: installPath
                }
              };
              saveDirectories(updatedDirectories);
              showNotification(`Auto-detected directory for ${selectedVersion} (Channel ${getChannelName(selectedChannel)}) set successfully!`);
            } else {
              showNotification(`Installations of ${game.title} found on this computer, but not detected by Hoyoplay`, 'error');
            }
          } else {
            showNotification(`No matching installations of ${game.title} found on this computer`, 'error');
          }
        } else {
          showNotification(`No ${game.title} installations found on this computer`, 'error');
        }
      } catch (hoyoplayError) {
        showNotification(`Failed to auto-detect ${game.title} directory: ${hoyoplayError}`, 'error');
      }

    } catch (error) {
      console.error('Failed to auto-detect game directory:', error);
      showNotification('Failed to auto-detect game directory', 'error');
    } finally {
      setIsValidating(false);
    }
  };

  // Handle download game data check
  const handleDownloadGameCheck = async () => {
    if (!selectedVersion || selectedChannel === undefined) {
      showNotification('Please select a version and channel first', 'error');
      return;
    }

    // Reset all download modal state to ensure fresh start
    setSelectedDownloadFolder('');
    setFileCheckResults({});
    setDownloadData(null);
    setFileCheckProgress({ current: 0, total: 0 });

    setIsCheckingDownload(true);
    try {
      const apiUrl = `https://ps.yuuki.me/game/download/pc/${game.id}/${selectedChannel}/${selectedVersion}.json`;
      const response = await fetch(apiUrl);
      const data = await response.json();

      if (data.retcode === -1) {
        showNotification(`Download not available: ${data.message}`, 'error');
        return;
      }

      setDownloadData(data);
      setDownloadModalOpen(true);
    } catch (error) {
      console.error('Failed to check download data:', error);
      showNotification('Failed to check download data', 'error');
    } finally {
      setIsCheckingDownload(false);
    }
  };

  const handleSelectDownloadFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: selectedDownloadFolder || undefined
      });
      
      if (selected && typeof selected === 'string') {
        setSelectedDownloadFolder(selected);
        // Check files in the selected folder
        await checkFilesInFolder(selected);
      }
    } catch (error) {
      console.error('Failed to select folder:', error);
      showNotification('Failed to select folder', 'error');
    }
  };

  const checkFilesInFolder = async (folderPath: string) => {
    if (!downloadData?.file) {
      return;
    }

    setIsCheckingFiles(true);
    setFileCheckProgress({ current: 0, total: downloadData.file.length });
    const results: {[key: string]: {exists: boolean, md5Match: boolean}} = {};

    try {
        for (let i = 0; i < downloadData.file.length; i++) {
          const file = downloadData.file[i];
          const filePath = `${folderPath}\\${file.file}`;
          
          // Update progress
          setFileCheckProgress({ current: i + 1, total: downloadData.file.length });
        
        try {
          // Check if file exists
          const exists = await invoke<boolean>('check_file_exists', { filePath });
          
          if (exists) {
            // Check MD5 if file exists
            try {
               const fileMd5 = await invoke<string>('get_file_md5', { filePath: filePath });
               console.log(`MD5 for ${file.file}: ${fileMd5} > ${file.md5}`);
               const md5Match = fileMd5.toLowerCase() === file.md5.toLowerCase();
               results[file.file] = { exists: true, md5Match };
             } catch (md5Error) {
               console.error(`Failed to get MD5 for ${file.file}:`, md5Error);
               results[file.file] = { exists: true, md5Match: false };
             }
           } else {
             results[file.file] = { exists: false, md5Match: false };
           }
         } catch (error) {
           console.error(`Failed to check file ${file.file}:`, error);
           results[file.file] = { exists: false, md5Match: false };
        }
        
        // Add small delay to prevent UI blocking
        await new Promise(resolve => setTimeout(resolve, 10));
      }
      
      setFileCheckResults(results);
    } catch (error) {
      console.error('Failed to check files:', error);
      showNotification('Failed to check files in folder', 'error');
    } finally {
      setIsCheckingFiles(false);
    }
  };

  const handleProxyAddressChange = async (newAddress: string) => {
    setProxyAddress(newAddress);
    try {
      // Add :443 port if not specified
      const addressWithPort = newAddress.includes(':') ? newAddress : `${newAddress}:443`;
      await invoke('set_proxy_addr', { addr: addressWithPort });
      showNotification('Proxy address updated successfully!');
    } catch (error) {
      console.error('Failed to set proxy address:', error);
      showNotification('Failed to update proxy address', 'error');
    }
  };

  const handleAddNewServer = () => {
    const trimmedServer = newServerInput.trim();
    if (trimmedServer && !savedProxyServers.includes(trimmedServer)) {
      const updatedServers = [...savedProxyServers, trimmedServer];
      saveProxyServers(updatedServers);
      setNewServerInput('');
      showNotification('Server added to list successfully!');
    } else if (savedProxyServers.includes(trimmedServer)) {
      showNotification('Server already exists in the list', 'error');
    } else {
      showNotification('Please enter a valid server address', 'error');
    }
  };

  const handleSelectServer = async (serverAddress: string) => {
    setProxyAddress(serverAddress);
    await handleProxyAddressChange(serverAddress);
  };

  const handleRemoveServer = (serverToRemove: string) => {
    if (savedProxyServers.length > 1) {
      const updatedServers = savedProxyServers.filter(server => server !== serverToRemove);
      saveProxyServers(updatedServers);
      showNotification('Server removed from list');

      // If the removed server was the current one, switch to the first available
      if (proxyAddress === serverToRemove && updatedServers.length > 0) {
        handleSelectServer(updatedServers[0]);
      }
    } else {
      showNotification('Cannot remove the last server from the list', 'error');
    }
  };

  // Get current directory for selected version and channel
  const getCurrentDirectory = () => {
    return getDirectoryForVersionChannel(selectedVersion, selectedChannel);
  };

  // Helper function to get directory for any version/channel combination
  const getDirectoryForVersionChannel = (version: string, channel: number): string => {
    return versionDirectories[version]?.[channel] || '';
  };

  // Helper function to get channel name
  const getChannelName = (channelId: number): string => {
    switch (channelId) {
      case 0: return 'None';
      case 1: return 'Global';
      case 2: return 'China';
      case 3: return 'Japan';
      default: return `Channel ${channelId}`;
    }
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
                onClick={() => setActiveTab('proxy')}
                className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${activeTab === 'proxy'
                  ? 'bg-purple-600/30 text-purple-400 border border-purple-500/50'
                  : 'text-gray-300 hover:bg-gray-700/50'
                  }`}
              >
                Proxy Settings
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
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-semibold text-white">Basic Information: {game.title}</h3>
                  {isGameRunning && (
                    <div className="flex items-center space-x-2 px-3 py-1 bg-red-600/20 border border-red-500/50 rounded-lg">
                      <div className="w-2 h-2 bg-red-400 rounded-full animate-pulse"></div>
                      <span className="text-red-400 text-sm font-medium">Game Running - Settings Locked</span>
                    </div>
                  )}
                </div>

                {/* Version Selection */}
                <div className={`bg-gray-800/50 rounded-lg p-4 ${isGameRunning || isValidating ? 'opacity-60' : ''}`}>
                  <h4 className="text-white font-semibold mb-3">Game Version</h4>
                  <div className="space-y-2">
                    {availableVersions.map((version) => (
                      <label
                        key={version}
                        className={`flex items-center space-x-3 p-3 bg-gray-700/50 rounded-lg transition-colors ${isGameRunning || isValidating
                          ? 'cursor-not-allowed'
                          : 'hover:bg-gray-700/70 cursor-pointer'
                          }`}
                      >
                        <input
                          type="radio"
                          name="version"
                          value={version}
                          checked={selectedVersion === version}
                          onChange={() => !(isGameRunning || isValidating) && handleVersionChange(version)}
                          disabled={isGameRunning || isValidating}
                          className="text-purple-600 focus:ring-purple-500 disabled:opacity-50 disabled:cursor-not-allowed"
                        />
                        <span className="text-white font-medium">{version}</span>
                        <div className="flex items-center space-x-2 ml-auto">
                          {versionDirectories[version] && Object.keys(versionDirectories[version] || {}).length > 0 ? (
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

                {/* Channel Selection */}
                {selectedVersion && availableChannels.length > 0 && (
                  <div className={`bg-gray-800/50 rounded-lg p-4 ${isGameRunning || isValidating ? 'opacity-60' : ''}`}>
                    <h4 className="text-white font-semibold mb-3">Select Channel</h4>
                    <div className="space-y-2">
                      {availableChannels.map((channel) => (
                        <label
                          key={channel}
                          className={`flex items-center space-x-3 p-3 bg-gray-700/50 rounded-lg transition-colors ${isGameRunning || isValidating
                            ? 'cursor-not-allowed'
                            : 'hover:bg-gray-700/70 cursor-pointer'
                            }`}
                        >
                          <input
                            type="radio"
                            name="channel"
                            value={channel}
                            checked={selectedChannel === channel}
                            onChange={() => !(isGameRunning || isValidating) && setSelectedChannel(channel)}
                            disabled={isGameRunning || isValidating}
                            className="text-purple-600 focus:ring-purple-500 disabled:opacity-50 disabled:cursor-not-allowed"
                          />
                          <span className="text-white font-medium">{getChannelName(channel)}</span>
                          <div className="flex items-center space-x-2 ml-auto">
                            {getDirectoryForVersionChannel(selectedVersion, channel) ? (
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
                )}

                {/* Game Directory */}
                <div className={`bg-gray-800/50 rounded-lg p-4 ${isGameRunning || isValidating ? 'opacity-60' : ''}`}>
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-white font-semibold">Game Directory</h4>
                    <button
                      onClick={handleOpenDirectory}
                      disabled={isGameRunning || isValidating}
                      className={`flex items-center space-x-2 px-3 py-1 rounded transition-colors ${isGameRunning || isValidating
                        ? 'bg-gray-600 text-gray-500 cursor-not-allowed'
                        : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                        }`}
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
                        ⚠️ Directory not configured for {selectedVersion} ({getChannelName(selectedChannel)})
                      </p>
                    )}
                  </div>

                  <div>
                    <h5 className="text-white font-medium mb-2">Relocate Game</h5>
                    <p className="text-gray-400 text-sm mb-3">
                      {getCurrentDirectory()
                        ? `Update the directory path for ${selectedVersion} (${getChannelName(selectedChannel)}). Select the folder where "${game.title}.exe" is located.`
                        : `Set the directory path for ${selectedVersion} (${getChannelName(selectedChannel)}). Select the folder where "${game.title}.exe" is located.`
                      }
                    </p>
                    <div className="flex space-x-3">
                      <button
                        onClick={handleAutoDetect}
                        disabled={isGameRunning || isValidating}
                        className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-colors ${isGameRunning || isValidating
                          ? 'bg-gray-600 text-gray-500 cursor-not-allowed'
                          : 'bg-blue-600 text-white hover:bg-blue-700'
                          }`}
                        title="Auto detect game installation from HoyoPlay registry or scan drives"
                      >
                        {isValidating ? (
                          <>
                            <div className="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                            <span>Validating...</span>
                          </>
                        ) : (
                          <>
                            <Search className="w-4 h-4" />
                            <span>Auto Detect</span>
                          </>
                        )}
                      </button>
                      <button
                        onClick={handleRelocate}
                        disabled={isGameRunning || isValidating}
                        className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-colors ${isGameRunning || isValidating
                          ? 'bg-gray-600 text-gray-500 cursor-not-allowed'
                          : 'bg-purple-600 text-white hover:bg-purple-700'
                          }`}
                      >
                        {isValidating ? (
                          <>
                            <div className="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                            <span>Validating...</span>
                          </>
                        ) : (
                          <>
                            <RotateCcw className="w-4 h-4" />
                            <span>{getCurrentDirectory() ? 'Relocate' : 'Set Directory'}</span>
                          </>
                        )}
                      </button>
                    </div>
                  </div>
                </div>

                {/* Download Game Data */}
                <div className={`bg-gray-800/50 rounded-lg p-4 ${isGameRunning || isValidating ? 'opacity-60' : ''}`}>
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-white font-semibold">Download Game Data</h4>
                  </div>
                  
                  <p className="text-gray-400 text-sm mb-3">
                    Check and download game data files for {selectedVersion} ({getChannelName(selectedChannel)}). This will fetch the latest game files from the server.
                  </p>
                  
                  <button
                    onClick={handleDownloadGameCheck}
                    disabled={isGameRunning || isValidating || isCheckingDownload || !selectedVersion}
                    className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-colors ${
                      isGameRunning || isValidating || isCheckingDownload || !selectedVersion
                        ? 'bg-gray-600 text-gray-500 cursor-not-allowed'
                        : 'bg-green-600 text-white hover:bg-green-700'
                    }`}
                  >
                    {isCheckingDownload ? (
                      <>
                        <div className="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                        <span>Checking...</span>
                      </>
                    ) : (
                      <>
                        <Download className="w-4 h-4" />
                        <span>Download Game</span>
                      </>
                    )}
                  </button>
                </div>

              </div>
            )}

            {activeTab === 'proxy' && (
              <div className="space-y-6">
                <h3 className="text-lg font-semibold text-white mb-4">Proxy Settings</h3>

                {/* Proxy Status and Control */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Proxy Status</h4>
                  <div className="space-y-4">
                    {/* Status Display */}
                    <div className="flex items-center justify-between p-3 bg-gray-700/50 rounded-lg">
                      <div className="flex items-center space-x-3">
                        <div className={`w-3 h-3 rounded-full ${isProxyRunning ? 'bg-green-400' : 'bg-red-400'
                          }`}></div>
                        <span className={`font-medium ${isProxyRunning ? 'text-green-400' : 'text-red-400'
                          }`}>
                          {isProxyRunning ? 'Running' : 'Stopped'}
                        </span>
                        <span className="text-gray-300 text-sm">
                          {isProxyRunning ? `Proxy server is active on port ${proxyPort} with ${proxyDomains.length} domains` : 'Proxy server is not running'}
                        </span>
                      </div>

                      {/* Control Button */}
                      <button
                        onClick={isProxyRunning ? handleStopProxy : handleStartProxy}
                        disabled={proxyStatusLoading}
                        className={`flex items-center space-x-2 px-4 py-2 rounded-lg font-medium transition-colors ${proxyStatusLoading
                          ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                          : isProxyRunning
                            ? 'bg-red-600 text-white hover:bg-red-700'
                            : 'bg-green-600 text-white hover:bg-green-700'
                          }`}
                      >
                        {proxyStatusLoading ? (
                          <>
                            <div className="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                            <span>Loading...</span>
                          </>
                        ) : (
                          <>
                            <div className={`w-2 h-2 rounded-full ${isProxyRunning ? 'bg-white' : 'bg-white'
                              }`}></div>
                            <span>{isProxyRunning ? 'Stop Proxy' : 'Start Proxy'}</span>
                          </>
                        )}
                      </button>
                    </div>

                    <p className="text-gray-400 text-sm">
                      The proxy server intercepts and redirects game traffic. Games can continue running even when the proxy is stopped.
                    </p>
                  </div>
                </div>

                {/* Proxy Port Configuration */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Proxy Port Configuration</h4>
                  <div className="space-y-4">
                    {/* Current Port Display */}
                    <div className="flex items-center space-x-2 p-3 bg-gray-700/50 rounded-lg">
                      <div className="w-2 h-2 bg-blue-400 rounded-full"></div>
                      <span className="text-blue-400 text-sm font-medium">Current Port:</span>
                      <span className="text-white text-sm font-mono">{proxyPort}</span>
                    </div>

                    {/* Port Actions */}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                      {/* Find Available Port */}
                      <button
                        onClick={handleFindAvailablePort}
                        disabled={isProxyRunning}
                        className={`flex items-center justify-center space-x-2 px-4 py-2 rounded-lg font-medium transition-colors ${isProxyRunning
                          ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                          : 'bg-blue-600 text-white hover:bg-blue-700'
                          }`}
                      >
                        <RefreshCw className="w-4 h-4" />
                        <span>Find Available Port</span>
                      </button>

                      {/* Custom Port Input */}
                      <div className="flex space-x-2">
                        <input
                          type="number"
                          value={customPortInput}
                          onChange={(e) => setCustomPortInput(e.target.value)}
                          onKeyPress={(e) => e.key === 'Enter' && handleSetCustomPort()}
                          disabled={isProxyRunning}
                          className={`flex-1 bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:border-purple-500 focus:outline-none ${isProxyRunning ? 'opacity-50 cursor-not-allowed' : ''
                            }`}
                          placeholder="Custom port (1024-65535)"
                          min="1024"
                          max="65535"
                        />
                        <button
                          onClick={handleSetCustomPort}
                          disabled={isProxyRunning || !customPortInput.trim()}
                          className={`px-3 py-2 rounded-lg font-medium transition-colors ${isProxyRunning || !customPortInput.trim()
                            ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                            : 'bg-purple-600 text-white hover:bg-purple-700'
                            }`}
                        >
                          Set
                        </button>
                      </div>
                    </div>

                    <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3">
                      <p className="text-yellow-400 text-sm">
                        ⚠️ Port changes require stopping and restarting the proxy server. Make sure the new port is not in use by other applications.
                      </p>
                    </div>
                  </div>
                </div>

                {/* Proxy Address (Private Server) Configuration */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Private Server Address</h4>
                  <div className="space-y-4">
                    {/* Current Server Status */}
                    <div className="flex items-center space-x-2 p-3 bg-gray-700/50 rounded-lg">
                      <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                      <span className="text-green-400 text-sm font-medium">Active:</span>
                      <span className="text-white text-sm">{proxyAddress}</span>
                    </div>

                    {/* Saved Servers List */}
                    <div>
                      <label className="block text-gray-300 text-sm mb-2">Saved Servers</label>
                      <div className="space-y-2 max-h-32 overflow-y-auto">
                        {savedProxyServers.map((server, index) => (
                          <div
                            key={index}
                            className={`flex items-center justify-between p-2 rounded-lg transition-colors ${server === proxyAddress
                              ? 'bg-purple-600/30 border border-purple-500/50'
                              : 'bg-gray-700/50 hover:bg-gray-700/70'
                              }`}
                          >
                            <span className="text-white text-sm font-mono flex-1">{server}</span>
                            <div className="flex items-center space-x-2">
                              {server !== proxyAddress && (
                                <button
                                  onClick={() => handleSelectServer(server)}
                                  className="px-2 py-1 bg-purple-600 text-white text-xs rounded hover:bg-purple-700 transition-colors"
                                >
                                  Set
                                </button>
                              )}
                              {savedProxyServers.length > 1 && (
                                <button
                                  onClick={() => handleRemoveServer(server)}
                                  className="p-1 text-red-400 hover:text-red-300 hover:bg-red-400/20 rounded transition-colors"
                                >
                                  <Trash2 className="w-3 h-3" />
                                </button>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>

                    {/* Add New Server */}
                    <div>
                      <label className="block text-gray-300 text-sm mb-2">Add New Server</label>
                      <div className="flex space-x-2">
                        <input
                          type="text"
                          value={newServerInput}
                          onChange={(e) => setNewServerInput(e.target.value)}
                          onKeyPress={(e) => e.key === 'Enter' && handleAddNewServer()}
                          className="flex-1 bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:border-purple-500 focus:outline-none"
                          placeholder="https://example.com"
                        />
                        <button
                          onClick={handleAddNewServer}
                          className="flex items-center space-x-1 px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                        >
                          <Plus className="w-4 h-4" />
                          <span>Add</span>
                        </button>
                      </div>
                    </div>

                    <p className="text-gray-400 text-sm">
                      Manage your private server. Click 'Set' to switch between saved servers or add new ones.
                    </p>
                  </div>
                </div>

                {/* Domain Management */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Intercepted Domains</h4>
                  <div className="space-y-4">
                    <p className="text-gray-400 text-sm">
                      Manage domains that will be intercepted and redirected by the proxy. These domains will have their traffic routed through your selected proxy server.
                    </p>

                    {/* Domain List */}
                    <div>
                      <label className="block text-gray-300 text-sm mb-2">
                        {proxyDomains.length > 0 ? `Current Domains (${proxyDomains.length})` : `No domains configured`}
                      </label>
                      <div className="space-y-2 max-h-40 overflow-y-auto bg-gray-700/30 rounded-lg p-3">
                        {proxyDomains.length > 0 ? (
                          proxyDomains.map((domain, index) => (
                            <div
                              key={index}
                              className="flex items-center justify-between p-2 bg-gray-700/50 rounded-lg hover:bg-gray-700/70 transition-colors"
                            >
                              <span className="text-white text-sm font-mono flex-1">{domain}</span>
                              <button
                                onClick={() => handleRemoveDomain(domain)}
                                className="p-1 text-red-400 hover:text-red-300 hover:bg-red-400/20 rounded transition-colors"
                                title={`Remove ${domain}`}
                              >
                                <Trash2 className="w-3 h-3" />
                              </button>
                            </div>
                          ))
                        ) : (
                          <div className="text-gray-400 text-sm text-center py-4">
                            No domains configured
                          </div>
                        )}
                      </div>
                    </div>

                    {/* Add New Domain */}
                    <div>
                      <label className="block text-gray-300 text-sm mb-2">Add New Domain</label>
                      <div className="flex space-x-2">
                        <input
                          type="text"
                          value={newDomainInput}
                          onChange={(e) => setNewDomainInput(e.target.value)}
                          onKeyPress={(e) => e.key === 'Enter' && handleAddDomain()}
                          className="flex-1 bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:border-purple-500 focus:outline-none"
                          placeholder="example.com or example.com:8080"
                        />
                        <button
                          onClick={handleAddDomain}
                          className="flex items-center space-x-1 px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                        >
                          <Plus className="w-4 h-4" />
                          <span>Add</span>
                        </button>
                      </div>
                      <p className="text-gray-400 text-xs mt-1">
                        Enter domain names without protocol. Ports are supported (e.g., "mihoyo.com" or "yuanshen.com:12401")
                      </p>
                    </div>

                    {/* Domain Management Actions */}
                    <div className="flex space-x-2 pt-2 border-t border-gray-700/50">
                      {proxyDomains.length > 0 && (
                        <button
                          onClick={handleDeleteAllDomains}
                          className="flex items-center space-x-1 px-3 py-2 bg-red-600 text-white text-sm rounded-lg hover:bg-red-700 transition-colors"
                        >
                          <Trash2 className="w-4 h-4" />
                          <span>Delete All</span>
                        </button>
                      )}
                      <button
                        onClick={handleResetToDefaults}
                        className="flex items-center space-x-1 px-3 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 transition-colors"
                      >
                        <RefreshCw className="w-4 h-4" />
                        <span>Reset to Defaults</span>
                      </button>
                    </div>

                  </div>
                </div>

                {/* Proxy Logs */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-white font-semibold">Proxy Logs</h4>
                    <div className="flex items-center space-x-2">
                      <label className="flex items-center space-x-2 text-sm text-gray-300">
                        <input
                          type="checkbox"
                          checked={autoRefreshLogs}
                          onChange={(e) => setAutoRefreshLogs(e.target.checked)}
                          className="text-purple-600 focus:ring-purple-500"
                        />
                        <span>Auto-refresh</span>
                      </label>
                      <button
                        onClick={fetchProxyLogs}
                        className="flex items-center space-x-1 px-2 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 transition-colors"
                      >
                        <RefreshCw className="w-3 h-3" />
                        <span>Refresh</span>
                      </button>
                      <button
                        onClick={handleClearProxyLogs}
                        className="flex items-center space-x-1 px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 transition-colors"
                      >
                        <Trash className="w-3 h-3" />
                        <span>Clear</span>
                      </button>
                    </div>
                  </div>

                  <div className="bg-gray-900/50 rounded-lg p-3 max-h-64 overflow-y-auto">
                    {proxyLogs.length === 0 ? (
                      <div className="text-center text-gray-400 py-4">
                        <p>No proxy logs available</p>
                        <p className="text-xs mt-1">Logs will appear here when proxy redirections occur</p>
                      </div>
                    ) : (
                      <div className="space-y-2">
                        {proxyLogs.slice().reverse().map((log, index) => (
                          <div key={index} className="flex items-center space-x-3 p-2 bg-gray-800/50 rounded text-xs font-mono">
                            <span className="text-blue-400 font-medium min-w-[60px]">{log.timestamp}</span>
                            <span className="text-gray-300">-</span>
                            <span className="text-yellow-400 flex-1 truncate" title={log.original_url}>
                              {log.original_url}
                            </span>
                            <span className="text-gray-300">to</span>
                            <span className="text-green-400 flex-1 truncate" title={log.redirected_url}>
                              {log.redirected_url}
                            </span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>

                  <p className="text-gray-400 text-xs mt-2">
                    Shows real-time proxy redirections. Latest entries appear at the top.
                  </p>
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
                    <div className="flex items-center space-x-3">
                      <input
                        type="checkbox"
                        id="deleteHoyoPass"
                        checked={deleteHoyoPass}
                        onChange={(e) => saveDeleteHoyoPassSetting(e.target.checked)}
                        className="text-purple-600 focus:ring-purple-500"
                      />
                      <label htmlFor="deleteHoyoPass" className="text-gray-300">Delete hoyo pass</label>
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

      {/* Patch Error Modal */}
      {patchErrorModalOpen && patchErrorInfo && (
        <PatchErrorModal
          isOpen={patchErrorModalOpen}
          onClose={() => {
            setPatchErrorModalOpen(false);
            setPatchErrorInfo(null);
          }}
          errorInfo={patchErrorInfo} gameTitle={''} />
      )}

      {/* Disk Scan Modal */}
      <DiskScanModal
        isOpen={diskScanModalOpen}
        onClose={() => setDiskScanModalOpen(false)}
        onPathSelected={handleDiskScanPathSelected}
        gameId={game.id}
        channel={selectedChannel}
      />

      {/* Download Game Modal */}
      {downloadModalOpen && downloadData && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-gray-800 rounded-lg max-w-4xl w-full max-h-[90vh] overflow-hidden">
            <div className="flex items-center justify-between p-6 border-b border-gray-700">
              <h2 className="text-xl font-semibold text-white">Download Game Data</h2>
              <button
                onClick={() => {
                  setDownloadModalOpen(false);
                  setDownloadData(null);
                }}
                className="text-gray-400 hover:text-white transition-colors"
              >
                <X className="w-6 h-6" />
              </button>
            </div>
            
            <div className="p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
              <div className="space-y-6">
                {/* Download Info */}
                <div className="bg-gray-700/50 rounded-lg p-4">
                  <h3 className="text-white font-semibold mb-2">Download Information</h3>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-gray-400">Game:</span>
                      <span className="text-white ml-2">{game.title}</span>
                    </div>
                    <div>
                      <span className="text-gray-400">Version:</span>
                      <span className="text-white ml-2">{selectedVersion}</span>
                    </div>
                    <div>
                      <span className="text-gray-400">Channel:</span>
                      <span className="text-white ml-2">{getChannelName(selectedChannel)}</span>
                    </div>
                    <div>
                      <span className="text-gray-400">Method:</span>
                      <span className="text-white ml-2">{downloadData.metode}</span>
                    </div>
                  </div>
                </div>

                {/* File List - Only show after folder selection and file checking */}
                {selectedDownloadFolder && Object.keys(fileCheckResults).length > 0 && (
                  <div className="bg-gray-700/50 rounded-lg p-4">
                    <h3 className="text-white font-semibold mb-4">Files to Download</h3>
                    
                    {/* Summary */}
                    <div className="bg-gray-800/50 rounded-lg p-3 mb-4">
                      <div className="grid grid-cols-3 gap-4 text-sm">
                        <div>
                          <span className="text-gray-400">Total Files:</span>
                          <span className="text-white ml-2">{downloadData.file?.length || 0}</span>
                        </div>
                        <div>
                          <span className="text-gray-400">Total Size:</span>
                          <span className="text-white ml-2">
                            {downloadData.file ? 
                              (downloadData.file.reduce((sum: number, file: any) => sum + file.package_size, 0) / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
                              : '0 GB'
                            }
                          </span>
                        </div>
                        <div>
                          <span className="text-gray-400">Estimated Unzipped:</span>
                          <span className="text-white ml-2">
                            {downloadData.file ? 
                              (downloadData.file.reduce((sum: number, file: any) => sum + file.package_size, 0) * 2 / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
                              : '0 GB'
                            }
                          </span>
                        </div>
                      </div>
                    </div>

                    {/* File List Table */}
                    <div className="bg-gray-800/50 rounded-lg overflow-hidden">
                      <div className="max-h-64 overflow-y-auto">
                        <table className="w-full text-sm">
                          <thead className="bg-gray-700/50 sticky top-0">
                            <tr>
                              <th className="text-left p-3 text-gray-300 font-medium">File Name</th>
                              <th className="text-left p-3 text-gray-300 font-medium">Size</th>
                              <th className="text-left p-3 text-gray-300 font-medium">MD5</th>
                              <th className="text-left p-3 text-gray-300 font-medium">Status</th>
                            </tr>
                          </thead>
                          <tbody>
                            {downloadData.file?.map((file: any, index: number) => {
                              const result = fileCheckResults[file.file];
                              let rowClassName = "border-t border-gray-700/50 hover:bg-gray-700/30";
                              
                              if (selectedDownloadFolder && result) {
                                if (result.exists && result.md5Match) {
                                  rowClassName = "border-t border-gray-700/50 bg-green-900/20 hover:bg-green-900/30";
                                } else if (result.exists && !result.md5Match) {
                                  rowClassName = "border-t border-gray-700/50 bg-yellow-900/20 hover:bg-yellow-900/30";
                                } else {
                                  rowClassName = "border-t border-gray-700/50 bg-red-900/20 hover:bg-red-900/30";
                                }
                              }
                              
                              return (
                                <tr key={index} className={rowClassName}>
                                  <td className="p-3 text-white font-mono text-xs">{file.file}</td>
                                  <td className="p-3 text-gray-300">
                                    {(file.package_size / (1024 * 1024)).toFixed(2)} MB
                                  </td>
                                  <td className="p-3 text-gray-400 font-mono text-xs">{file.md5}</td>
                                  <td className="p-3">
                                    {selectedDownloadFolder && result ? (
                                      result.exists ? (
                                        result.md5Match ? (
                                          <div className="flex items-center space-x-1 text-green-400">
                                            <CheckCircle className="w-3 h-3" />
                                            <span className="text-xs">Valid</span>
                                          </div>
                                        ) : (
                                          <div className="flex items-center space-x-1 text-yellow-400">
                                            <AlertTriangle className="w-3 h-3" />
                                            <span className="text-xs">MD5 Mismatch</span>
                                          </div>
                                        )
                                      ) : (
                                        <div className="flex items-center space-x-1 text-red-400">
                                          <X className="w-3 h-3" />
                                          <span className="text-xs">Missing</span>
                                        </div>
                                      )
                                    ) : (
                                      <span className="text-gray-500 text-xs">-</span>
                                    )}
                                  </td>
                                </tr>
                              );
                            })}
                          </tbody>
                        </table>
                      </div>
                    </div>
                  </div>
                )}

                {/* Download Location */}
                <div className="bg-gray-700/50 rounded-lg p-4">
                  <h3 className="text-white font-semibold mb-3">Download Location</h3>
                  <p className="text-gray-400 text-sm mb-3">
                    Select the folder where you want to save the downloaded files. The system will check for existing files and verify their MD5 checksums.
                  </p>
                  
                  {/* Selected Folder Display */}
                  {selectedDownloadFolder && (
                    <div className="bg-gray-800/50 rounded-lg p-3 mb-3">
                      <div className="flex items-center space-x-2">
                        <FolderOpen className="w-4 h-4 text-purple-400" />
                        <span className="text-white text-sm font-mono">{selectedDownloadFolder}</span>
                      </div>
                    </div>
                  )}
                  
                  <div className="flex space-x-3 mb-4">
                    <button
                      onClick={handleSelectDownloadFolder}
                      disabled={isCheckingFiles}
                      className="flex items-center space-x-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {isCheckingFiles ? (
                        <>
                          <Loader2 className="w-4 h-4 animate-spin" />
                          <span>Check {fileCheckProgress.current}/{fileCheckProgress.total}</span>
                        </>
                      ) : (
                        <>
                          <Folder className="w-4 h-4" />
                          <span>Select Download Folder</span>
                        </>
                      )}
                    </button>
                    
                    <button
                      onClick={() => {
                        setDownloadModalOpen(false);
                        setDownloadData(null);
                        setSelectedDownloadFolder('');
                        setFileCheckResults({});
                        showNotification('Download cancelled');
                      }}
                      className="flex items-center space-x-2 px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
                    >
                      <X className="w-4 h-4" />
                      <span>Cancel</span>
                    </button>
                  </div>


                </div>
              </div>
            </div>
          </div>
        </div>
      )}

    </div>
  );
};