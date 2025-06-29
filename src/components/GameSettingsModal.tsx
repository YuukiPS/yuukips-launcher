import React, { useState, useEffect, useCallback } from 'react';
import { X, Folder, RotateCcw, HardDrive, Calendar, Clock, Check, Trash2, Plus, RefreshCw, Trash } from 'lucide-react';
import { Game } from '../types';
import { GameApiService } from '../services/gameApi';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

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

  // Save proxy domains to localStorage whenever they change
  const saveProxyDomains = useCallback((domains: string[]) => {
    setProxyDomains(domains);
    localStorage.setItem('saved-proxy-domains', JSON.stringify(domains));
  }, []);

  // Fetch proxy domains from backend
  const fetchProxyDomains = useCallback(async () => {
    try {
      const domains = await invoke('get_proxy_domains');
      if (Array.isArray(domains)) {
        saveProxyDomains(domains);
      }
    } catch (error) {
      console.error('Failed to fetch proxy domains:', error);
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

    try {
      const result = await invoke('add_proxy_domain', { domain: trimmedDomain });
      if (typeof result === 'string') {
        showNotification(result);
        setNewDomainInput('');
        // Update localStorage and state with the new domain
        const updatedDomains = [...proxyDomains, trimmedDomain];
        saveProxyDomains(updatedDomains);
        fetchProxyDomains(); // Sync with backend
      }
    } catch (error) {
      console.error('Failed to add domain:', error);
      showNotification(typeof error === 'string' ? error : 'Failed to add domain', 'error');
    }
  };

  // Remove domain
  const handleRemoveDomain = async (domain: string) => {
    try {
      const result = await invoke('remove_proxy_domain', { domain });
      if (typeof result === 'string') {
        showNotification(result);
        // Update localStorage and state by removing the domain
        const updatedDomains = proxyDomains.filter(d => d !== domain);
        saveProxyDomains(updatedDomains);
        fetchProxyDomains(); // Sync with backend
      }
    } catch (error) {
      console.error('Failed to remove domain:', error);
      showNotification(typeof error === 'string' ? error : 'Failed to remove domain', 'error');
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

  const handleRelocate = async () => {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        defaultPath: getCurrentDirectory() || undefined,
        title: `Select directory for ${selectedVersion} (Channel ${getChannelName(selectedChannel)})`
      });

      if (selectedPath && typeof selectedPath === 'string') {
        const updatedDirectories = {
          ...versionDirectories,
          [selectedVersion]: {
            ...versionDirectories[selectedVersion],
            [selectedChannel]: selectedPath
          }
        };
        saveDirectories(updatedDirectories);
        showNotification(`Directory for ${selectedVersion} (Channel ${getChannelName(selectedChannel)}) updated successfully!`);
      }
    } catch (error) {
      console.error('Failed to open directory dialog:', error);
      showNotification('Failed to open directory selection dialog', 'error');
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
                <div className={`bg-gray-800/50 rounded-lg p-4 ${isGameRunning ? 'opacity-60' : ''}`}>
                  <h4 className="text-white font-semibold mb-3">Game Version</h4>
                  <div className="space-y-2">
                    {availableVersions.map((version) => (
                      <label
                        key={version}
                        className={`flex items-center space-x-3 p-3 bg-gray-700/50 rounded-lg transition-colors ${
                          isGameRunning 
                            ? 'cursor-not-allowed' 
                            : 'hover:bg-gray-700/70 cursor-pointer'
                        }`}
                      >
                        <input
                          type="radio"
                          name="version"
                          value={version}
                          checked={selectedVersion === version}
                          onChange={() => !isGameRunning && handleVersionChange(version)}
                          disabled={isGameRunning}
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
                  <div className={`bg-gray-800/50 rounded-lg p-4 ${isGameRunning ? 'opacity-60' : ''}`}>
                    <h4 className="text-white font-semibold mb-3">Select Channel</h4>
                    <div className="space-y-2">
                      {availableChannels.map((channel) => (
                        <label
                          key={channel}
                          className={`flex items-center space-x-3 p-3 bg-gray-700/50 rounded-lg transition-colors ${
                            isGameRunning 
                              ? 'cursor-not-allowed' 
                              : 'hover:bg-gray-700/70 cursor-pointer'
                          }`}
                        >
                          <input
                            type="radio"
                            name="channel"
                            value={channel}
                            checked={selectedChannel === channel}
                            onChange={() => !isGameRunning && setSelectedChannel(channel)}
                            disabled={isGameRunning}
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
                <div className={`bg-gray-800/50 rounded-lg p-4 ${isGameRunning ? 'opacity-60' : ''}`}>
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-white font-semibold">Game Directory</h4>
                    <button
                      onClick={handleOpenDirectory}
                      disabled={isGameRunning}
                      className={`flex items-center space-x-2 px-3 py-1 rounded transition-colors ${
                        isGameRunning
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
                        ? `Update the directory path for ${selectedVersion}. Select the folder where "${game.title}.exe" is located.`
                        : `Set the directory path for ${selectedVersion}. Select the folder where "${game.title}.exe" is located.`
                      }
                    </p>
                    <button
                      onClick={handleRelocate}
                      disabled={isGameRunning}
                      className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-colors ${
                        isGameRunning
                          ? 'bg-gray-600 text-gray-500 cursor-not-allowed'
                          : 'bg-purple-600 text-white hover:bg-purple-700'
                      }`}
                    >
                      <RotateCcw className="w-4 h-4" />
                      <span>{getCurrentDirectory() ? 'Relocate' : 'Set Directory'}</span>
                    </button>
                  </div>
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
                          {isProxyRunning ? `Proxy server is active on port ${proxyPort}` : 'Proxy server is not running'}
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
                        className={`flex items-center justify-center space-x-2 px-4 py-2 rounded-lg font-medium transition-colors ${
                          isProxyRunning
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
                          className={`flex-1 bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:border-purple-500 focus:outline-none ${
                            isProxyRunning ? 'opacity-50 cursor-not-allowed' : ''
                          }`}
                          placeholder="Custom port (1024-65535)"
                          min="1024"
                          max="65535"
                        />
                        <button
                          onClick={handleSetCustomPort}
                          disabled={isProxyRunning || !customPortInput.trim()}
                          className={`px-3 py-2 rounded-lg font-medium transition-colors ${
                            isProxyRunning || !customPortInput.trim()
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

                {/* Proxy Address Configuration */}
                <div className="bg-gray-800/50 rounded-lg p-4">
                  <h4 className="text-white font-semibold mb-3">Proxy Server Address</h4>
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
                      Manage your proxy servers. Click 'Set' to switch between saved servers or add new ones.
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
                      <label className="block text-gray-300 text-sm mb-2">Current Domains ({proxyDomains.length})</label>
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