import { useState, useEffect, useCallback } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { GameDetails } from './components/GameDetails';
import { NewsPanel } from './components/NewsPanel';
import { newsItems } from './data/news';
import { socialLinks } from './data/socialLinks';
import { Game } from './types';
import { GameApiService } from './services/gameApi';
import { Megaphone, MessageCircle, Twitter, Youtube, Tv, Loader } from 'lucide-react';

function App() {
  const [games, setGames] = useState<Game[]>([]);
  const [selectedGameId, setSelectedGameId] = useState<number | null>(null);
  
  const handleGameSelect = (gameId: string | number) => {
    setSelectedGameId(typeof gameId === 'string' ? parseInt(gameId) : gameId);
  };
  const [showNews, setShowNews] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  const selectedGame = games.find(game => game.id === selectedGameId) || null;

  const loadGames = useCallback(async () => {
    setIsLoading(true);
    
    try {
      const apiGames = await GameApiService.fetchGames();
      setGames(apiGames);
      
      // Set the first game as selected if no game is currently selected
      if (selectedGameId === null && apiGames.length > 0) {
        setSelectedGameId(apiGames[0].id);
      }
    } catch (err) {
      console.error('Failed to load games from API:', err);
      setGames([]);
    } finally {
      setIsLoading(false);
    }
  }, [selectedGameId]);

  useEffect(() => {
    loadGames();
  }, [loadGames]);

  const handleGameUpdate = (updatedGame: Game) => {
    setGames(prevGames => 
      prevGames.map(game => 
        game.id === updatedGame.id ? updatedGame : game
      )
    );
  };

  const getSocialIcon = (iconName: string) => {
    switch (iconName) {
      case 'MessageCircle':
        return <MessageCircle className="w-5 h-5" />;
      case 'Twitter':
        return <Twitter className="w-5 h-5" />;
      case 'Youtube':
        return <Youtube className="w-5 h-5" />;
      case 'Tv':
        return <Tv className="w-5 h-5" />;
      default:
        return <MessageCircle className="w-5 h-5" />;
    }
  };

  const getSocialColor = (platform: string) => {
    switch (platform.toLowerCase()) {
      case 'discord':
        return 'bg-indigo-600 hover:bg-indigo-700 hover:shadow-indigo-500/25';
      case 'twitter':
        return 'bg-blue-500 hover:bg-blue-600 hover:shadow-blue-500/25';
      case 'youtube':
        return 'bg-red-600 hover:bg-red-700 hover:shadow-red-500/25';
      case 'twitch':
        return 'bg-purple-600 hover:bg-purple-700 hover:shadow-purple-500/25';
      default:
        return 'bg-gray-600 hover:bg-gray-700 hover:shadow-gray-500/25';
    }
  };

  const handleSocialClick = async (url: string, platform: string) => {
    try {
      await openUrl(url);
    } catch (error) {
      console.error(`Failed to open ${platform}:`, error);
      // Fallback for web or if Tauri is not available
      window.open(url, '_blank');
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="h-screen bg-gray-900 flex flex-col overflow-hidden">
        <Header />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <Loader className="w-12 h-12 text-purple-500 animate-spin mx-auto mb-4" />
            <p className="text-white text-lg">Loading games...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen bg-gray-900 flex flex-col overflow-hidden">
      <Header />
      

      
      <div className="flex-1 flex overflow-hidden relative">
        <Sidebar 
          games={games} 
          selectedGameId={selectedGameId} 
          onGameSelect={handleGameSelect} 
        />
        
        {selectedGame ? (
          <GameDetails 
            key={selectedGame.id}
            game={selectedGame} 
            onGameUpdate={handleGameUpdate}
          />
        ) : (
          <div className="flex-1 flex items-center justify-center bg-gray-800">
            <div className="text-center">
              <p className="text-white text-lg mb-2">No game selected</p>
              <p className="text-gray-400">Please select a game from the sidebar</p>
            </div>
          </div>
        )}

        {/* Floating Social Media Icons */}
        <div className="absolute top-6 right-6 flex space-x-3 z-40">
          {socialLinks.map((link) => (
            <button
              key={link.platform}
              onClick={() => handleSocialClick(link.url, link.platform)}
              className={`p-3 text-white rounded-full shadow-lg transition-all duration-200 hover:scale-110 ${getSocialColor(link.platform)}`}
              title={link.platform}
            >
              {getSocialIcon(link.icon)}
            </button>
          ))}
        </div>

        {/* Floating News Button */}
        <div className="absolute bottom-6 right-6 z-40">
          <button
            onClick={() => setShowNews(!showNews)}
            className="p-3 bg-blue-600 hover:bg-blue-700 text-white rounded-full shadow-lg hover:shadow-blue-500/25 transition-all duration-200 hover:scale-110"
            title="Latest News"
          >
            <Megaphone className="w-6 h-6" />
          </button>
        </div>

        <NewsPanel 
          news={newsItems}
          isOpen={showNews}
          onClose={() => setShowNews(false)}
        />
      </div>
    </div>
  );
}

export default App;