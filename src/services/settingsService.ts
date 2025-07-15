import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  speedLimit: number;
  divideSpeedEnabled: boolean;
  maxSimultaneousDownloads: number;
}

export class SettingsService {
  /**
   * Get all application settings
   */
  static async getAllSettings(): Promise<AppSettings> {
    try {
      console.log('[SettingsService] Getting all settings from backend...');
      const settings = await invoke<{
        speed_limit_mbps: number;
        divide_speed_enabled: boolean;
        max_simultaneous_downloads: number;
      }>('get_all_app_settings');
      
      const result = {
        speedLimit: settings.speed_limit_mbps,
        divideSpeedEnabled: settings.divide_speed_enabled,
        maxSimultaneousDownloads: settings.max_simultaneous_downloads
      };
      
      console.log('[SettingsService] Retrieved settings:', result);
      return result;
    } catch (error) {
      console.error('[SettingsService] Failed to get all settings:', error);
      throw error;
    }
  }

  /**
   * Get speed limit setting
   */
  static async getSpeedLimit(): Promise<number> {
    try {
      console.log('[SettingsService] Getting speed limit from backend...');
      const speedLimit = await invoke<number>('get_app_speed_limit');
      console.log('[SettingsService] Speed limit:', speedLimit);
      return speedLimit;
    } catch (error) {
      console.error('[SettingsService] Failed to get speed limit:', error);
      throw error;
    }
  }

  /**
   * Set speed limit setting
   */
  static async setSpeedLimit(speedLimit: number): Promise<void> {
    try {
      console.log('[SettingsService] Setting speed limit to:', speedLimit);
      await invoke('set_app_speed_limit', { speedLimitMbps: speedLimit });
      console.log('[SettingsService] Speed limit set successfully');
    } catch (error) {
      console.error('[SettingsService] Failed to set speed limit:', error);
      throw error;
    }
  }

  /**
   * Get divide speed enabled setting
   */
  static async getDivideSpeedEnabled(): Promise<boolean> {
    try {
      console.log('[SettingsService] Getting divide speed enabled from backend...');
      const enabled = await invoke<boolean>('get_app_divide_speed_enabled');
      console.log('[SettingsService] Divide speed enabled:', enabled);
      return enabled;
    } catch (error) {
      console.error('[SettingsService] Failed to get divide speed enabled:', error);
      throw error;
    }
  }

  /**
   * Set divide speed enabled setting
   */
  static async setDivideSpeedEnabled(enabled: boolean): Promise<void> {
    try {
      console.log('[SettingsService] Setting divide speed enabled to:', enabled);
      await invoke('set_app_divide_speed_enabled', { enabled });
      console.log('[SettingsService] Divide speed enabled set successfully');
    } catch (error) {
      console.error('[SettingsService] Failed to set divide speed enabled:', error);
      throw error;
    }
  }

  /**
   * Get max simultaneous downloads setting
   */
  static async getMaxSimultaneousDownloads(): Promise<number> {
    try {
      console.log('[SettingsService] Getting max simultaneous downloads from backend...');
      const maxDownloads = await invoke<number>('get_app_max_simultaneous_downloads');
      console.log('[SettingsService] Max simultaneous downloads:', maxDownloads);
      return maxDownloads;
    } catch (error) {
      console.error('[SettingsService] Failed to get max simultaneous downloads:', error);
      throw error;
    }
  }

  /**
   * Set max simultaneous downloads setting
   */
  static async setMaxSimultaneousDownloads(maxDownloads: number): Promise<void> {
    try {
      console.log('[SettingsService] Setting max simultaneous downloads to:', maxDownloads);
      await invoke('set_app_max_simultaneous_downloads', { maxDownloads });
      console.log('[SettingsService] Max simultaneous downloads set successfully');
    } catch (error) {
      console.error('[SettingsService] Failed to set max simultaneous downloads:', error);
      throw error;
    }
  }
}