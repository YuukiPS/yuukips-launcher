export interface Game {
  id: string;
  title: string;
  subtitle: string;
  description: string;
  version: string;
  imageUrl: string;
  backgroundUrl: string;
  status: 'available' | 'updating' | 'installing';
  lastPlayed?: string;
  playTime?: string;
  developer: string;
  genre: string;
  rating: number;
  size: string;
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