import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { SettingsService } from '../services/settingsService';

interface DownloadSettings {
  speedLimit: number;
  divideSpeedEnabled: boolean;
  maxSimultaneousDownloads: number;
}

interface DownloadSettingsContextType {
  settings: DownloadSettings;
  updateSettings: (newSettings: Partial<DownloadSettings>) => Promise<void>;
  reloadSettings: () => Promise<void>;
  isLoading: boolean;
}

const DownloadSettingsContext = createContext<DownloadSettingsContextType | undefined>(undefined);

interface DownloadSettingsProviderProps {
  children: ReactNode;
}

export const DownloadSettingsProvider: React.FC<DownloadSettingsProviderProps> = ({ children }) => {
  const [settings, setSettings] = useState<DownloadSettings>({
    speedLimit: 0,
    divideSpeedEnabled: false,
    maxSimultaneousDownloads: 3
  });
  const [isLoading, setIsLoading] = useState(true);

  // Load settings from backend
  const loadSettings = async () => {
    console.log('🔄 Loading download settings from backend...');
    setIsLoading(true);
    try {
      const loadedSettings = await SettingsService.getAllSettings();
      
      console.log('✅ Download settings loaded successfully:', loadedSettings);
      setSettings(loadedSettings);
    } catch (error) {
      console.error('❌ Failed to load download settings:', error);
      // Keep default values if loading fails
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  // Load settings from backend on component mount
  useEffect(() => {
    console.log('🚀 DownloadSettingsProvider mounted, starting to load settings...');
    loadSettings().catch(error => {
      console.error('💥 Critical error during initial settings load:', error);
    });
  }, []);

  const updateSettings = async (newSettings: Partial<DownloadSettings>) => {
    console.log('💾 Saving download settings:', newSettings);
    try {
      // Update backend settings
      const promises = [];
      
      if (newSettings.speedLimit !== undefined) {
        console.log('Setting speed limit:', newSettings.speedLimit);
        promises.push(SettingsService.setSpeedLimit(newSettings.speedLimit));
      }
      
      if (newSettings.divideSpeedEnabled !== undefined) {
        console.log('Setting divide speed enabled:', newSettings.divideSpeedEnabled);
        promises.push(SettingsService.setDivideSpeedEnabled(newSettings.divideSpeedEnabled));
      }
      
      if (newSettings.maxSimultaneousDownloads !== undefined) {
        console.log('Setting max simultaneous downloads:', newSettings.maxSimultaneousDownloads);
        promises.push(SettingsService.setMaxSimultaneousDownloads(newSettings.maxSimultaneousDownloads));
      }
      
      await Promise.all(promises);
      
      // Reload settings from backend to ensure we have the latest state
      console.log('🔄 Reloading settings after save to ensure consistency...');
      await loadSettings();
      console.log('✅ Download settings saved and reloaded successfully');
    } catch (error) {
      console.error('❌ Failed to update download settings:', error);
      throw error;
    }
  };

  const value: DownloadSettingsContextType = {
    settings,
    updateSettings,
    reloadSettings: loadSettings,
    isLoading
  };

  return (
    <DownloadSettingsContext.Provider value={value}>
      {children}
    </DownloadSettingsContext.Provider>
  );
};

export const useDownloadSettingsContext = (): DownloadSettingsContextType => {
  const context = useContext(DownloadSettingsContext);
  if (context === undefined) {
    throw new Error('useDownloadSettingsContext must be used within a DownloadSettingsProvider');
  }
  return context;
};