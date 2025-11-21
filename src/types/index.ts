export interface GameEngine {
  id: number;
  name: string;
  short: string;
  description: string;
  version: string;
  versionSupport: Record<string, Record<string, number[]>>;
  link: string;
  command: number;
  features?: string[];
}

export interface Game {
  id: number;
  slug: string;
  title: string;
  description: string;
  keyword: string;
  lastUpdate: number;
  image: string;
  thumbnail: string;
  icon: string;
  engine: GameEngine[];
  subtitle?: string;
  //version?: string;
  backgroundUrl?: string;
  status?: 'available' | 'updating' | 'installing';
  lastPlayed?: string;
  playTime?: string;
  developer?: string;
  genre?: string;
  rating?: number;
  size?: string;
}

export interface NewsItem {
  id: string;
  title: string;
  summary: string;
  date: string;
  category: 'update' | 'event' | 'announcement';
  imageUrl?: string;
}

export interface SocialLink {
  platform: string;
  url: string;
  icon: string;
}

export enum TypeGame {
	None = 0,
	GenshinImpact = 1,
	StarRail = 2,
	BlueArchive = 3,
  StellaSora = 4
}

export enum GameEngineType {
	None = 0,
	GC = 1,
	GIO = 2,
	CP = 3, // outdate ts ps
	VIA = 4, // emulator gio
	LC = 5,
	BP = 6 // blue archive aka BaPs
}

export enum PlatformType {
	PC = 1,
	Android = 2,
	iOS = 3
}

export enum channelType {
  None = 0,
  OS = 1,
  CN = 2,
  JP = 3 // Ba/Hi3
}

// Download Manager Types
export interface DownloadItem {
  id: string;
  fileName: string;
  fileExtension: string;
  totalSize: number;
  downloadedSize: number;
  progress: number;
  speed: number; // bytes per second
  status: 'downloading' | 'paused' | 'completed' | 'error' | 'cancelled' | 'queued';
  timeRemaining: number; // seconds
  url: string;
  filePath: string;
  startTime: number;
  endTime?: number;
  errorMessage?: string;
  userPaused?: boolean; // Track if pause was initiated by user
  resumeSupported?: boolean; // Whether the download supports resuming
}



export interface DownloadStats {
  total_downloads: number;
  active_downloads: number;
  completed_downloads: number;
  total_downloaded_size: number;
  average_speed: number;
}

export interface ActivityEntry {
  id: string;
  timestamp: string;
  actionType: string;
  fileName?: string;
  identifier?: string;
  status?: string;
  details?: string;
}

export interface FileExistenceInfo {
  exists: boolean;
  size: number;
  can_resume: boolean;
  is_complete: boolean;
}