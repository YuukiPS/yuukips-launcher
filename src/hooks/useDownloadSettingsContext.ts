import { useContext } from 'react';
import { DownloadSettingsContext, DownloadSettingsContextType } from '../contexts/DownloadSettingsContext';

export const useDownloadSettingsContext = (): DownloadSettingsContextType => {
  const context = useContext(DownloadSettingsContext);
  if (context === undefined) {
    throw new Error('useDownloadSettingsContext must be used within a DownloadSettingsProvider');
  }
  return context;
};