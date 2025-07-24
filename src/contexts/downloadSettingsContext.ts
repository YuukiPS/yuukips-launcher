import { createContext } from 'react';

export interface DownloadSettings {
  speedLimit: number;
  divideSpeedEnabled: boolean;
  maxSimultaneousDownloads: number;
}

export interface DownloadSettingsContextType {
  settings: DownloadSettings;
  updateSettings: (newSettings: Partial<DownloadSettings>) => Promise<void>;
  reloadSettings: () => Promise<void>;
  isLoading: boolean;
}

export const DownloadSettingsContext = createContext<DownloadSettingsContextType | undefined>(undefined);