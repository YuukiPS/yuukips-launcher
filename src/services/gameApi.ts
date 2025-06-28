import { Game } from '../types';

const API_BASE_URL = 'https://ps.yuuki.me/json';

export class GameApiService {
  static async fetchGames(): Promise<Game[]> {
    try {
      const randomTime = Date.now();
      const response = await fetch(`${API_BASE_URL}/game_all.json?time=${randomTime}`);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
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
      throw new Error('Sorry, your internet is having problems or our server is having problems, please try again later.');
    }
  }
  
  static getAvailableVersionsForPlatform(game: Game, platformType: number = 1): string[] {
    const versions: string[] = [];
    
    game.engine.forEach(engine => {
      Object.entries(engine.versionSupport).forEach(([version, platforms]) => {
        if (platforms.includes(platformType)) {
          versions.push(version);
        }
      });
    });
    
    return [...new Set(versions)]; // Remove duplicates
  }
  
  static getEnginesForVersion(game: Game, version: string, platformType: number = 1) {
    return game.engine.filter(engine => {
      const platforms = engine.versionSupport[version];
      return platforms && platforms.includes(platformType);
    });
  }
  
  static gameSupportsPC(game: Game): boolean {
    return game.engine.some(engine => {
      return Object.values(engine.versionSupport).some(platforms => 
        platforms.includes(1) // Platform 1 = PC
      );
    });
  }
}