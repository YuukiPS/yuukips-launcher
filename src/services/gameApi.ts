import { Game, GameEngine } from '../types';
import { invoke } from '@tauri-apps/api/core';

// Proxy management functions
export const startProxy = async (): Promise<string> => {
  try {
    return await invoke('start_proxy') as string;
  } catch (error) {
    throw new Error(`Failed to start proxy: ${error}`);
  }
};

export const stopProxy = async (): Promise<string> => {
  try {
    return await invoke('stop_proxy') as string;
  } catch (error) {
    throw new Error(`Failed to stop proxy: ${error}`);
  }
};

export const installSSLCertificate = async (): Promise<string> => {
  try {
    return await invoke('install_ssl_certificate') as string;
  } catch (error) {
    throw new Error(`Failed to install SSL certificate: ${error}`);
  }
};

export const checkSSLCertificateInstalled = async (): Promise<boolean> => {
  try {
    return await invoke('check_ssl_certificate_installed') as boolean;
  } catch (error) {
    console.error('Failed to check SSL certificate status:', error);
    return false;
  }
};

// Enhanced proxy management with SSL certificate checking
export const startProxyWithSSLCheck = async (): Promise<{ success: boolean; message: string; needsSSL: boolean }> => {
  try {
    // Start the proxy first
    const proxyResult = await startProxy();
    
    // Check if SSL certificate is installed
    const sslInstalled = await checkSSLCertificateInstalled();
    
    return {
      success: true,
      message: proxyResult,
      needsSSL: !sslInstalled
    };
  } catch (error) {
    throw new Error(`Failed to start proxy with SSL check: ${error}`);
  }
};

export class GameApiService {
  private static fetchPromise: Promise<Game[]> | null = null;
  
  static async fetchGames(): Promise<Game[]> {
    // If there's already a pending request, return it
    if (this.fetchPromise) {
      return this.fetchPromise;
    }
    
    // Create and store the promise
    this.fetchPromise = this.performFetch();
    
    try {
      const result = await this.fetchPromise;
      return result;
    } finally {
      // Clear the promise after completion (success or failure)
      this.fetchPromise = null;
    }
  }
  
  private static async performFetch(): Promise<Game[]> {
    try {
      const timestamp = Date.now();
      const response = await fetch(`https://ps.yuuki.me/json/game_all.json?time=${timestamp}`);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const games: Game[] = await response.json();
      
      // Filter games to only include those that support Platform 1 (PC)
      const pcSupportedGames = games.filter(game => this.gameSupportsPC(game));
      
      // Transform API data to include legacy fields for compatibility
      return pcSupportedGames.map(game => ({
        ...game,
  
        backgroundUrl: game.image,
        subtitle: game.keyword.split(',')[0].trim(),
        version: game.engine[0]?.version || '?.?.?',
        status: 'available' as const,
        developer: 'Private Server',
        genre: 'Game',
        rating: 4.5,
        size: 'Unknown'
      }));
    } catch (error) {
      console.error('Failed to fetch games:', error);
      
      const errorMessage = error instanceof Error ? error.message : String(error);
      
      throw new Error(
        `🚫 Game List Loading Failed:\n` +
        `• Error: ${errorMessage}\n` +
        `• URL: https://ps.yuuki.me/json/game_all.json\n\n` +
        `Troubleshooting Steps:\n` +
        `1. Try using a VPN like Cloudflare Warp (1.1.1.1)\n` +
        `2. Turn off other proxy software or VPN services\n` +
        `3. Contact your ISP to ask why ps.yuuki.me cannot be accessed\n` +
        `4. As a last resort, consider reinstalling your operating system`
      );
    }
  }
  
  static getAvailableVersionsForPlatform(game: Game, platformType: number = 1): string[] {
    const versions: string[] = [];
    
    game.engine.forEach(engine => {
      Object.entries(engine.versionSupport).forEach(([version, platformData]) => {
        if (platformData[platformType.toString()]) {
          versions.push(version);
        }
      });
    });
    
    return [...new Set(versions)]; // Remove duplicates
  }
  
  static getEnginesForVersion(game: Game, version: string, platformType: number = 1) {
    return game.engine.filter(engine => {
      const platformData = engine.versionSupport[version];
      return platformData && platformData[platformType.toString()];
    });
  }
  
  static gameSupportsPC(game: Game): boolean {
    return game.engine.some(engine => {
      return Object.values(engine.versionSupport).some(platformData => 
        platformData['1'] // Platform 1 = PC
      );
    });
  }
  
  static getAvailableChannelsForEngineVersion(engine: GameEngine, version: string, platformType: number = 1): number[] {
    const platformData = engine.versionSupport[version];
    return platformData?.[platformType.toString()] || [];
  }
}