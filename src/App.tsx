import { useState, useEffect, useCallback, useRef } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { GameDetails } from './components/GameDetails';
import { NewsPanel } from './components/NewsPanel';
import { UpdateModal } from './components/UpdateModal';
import { UpdateErrorModal } from './components/UpdateErrorModal';
import ErrorBoundary from './components/ErrorBoundary';
import { newsItems } from './data/news';
import { socialLinks } from './data/socialLinks';
import { Game } from './types';
import { GameApiService } from './services/gameApi';
import { UpdateService, UpdateInfo } from './services/updateService';
import { Megaphone, MessageCircle, Twitter, Youtube, Tv, Loader } from 'lucide-react';

function App() {
  const [games, setGames] = useState<Game[]>([]);
  const [selectedGameId, setSelectedGameId] = useState<number | null>(null);
  const [runningGameId, setRunningGameId] = useState<number | null>(null);
  const gamesLoadedRef = useRef(false);
  
  // Update-related state
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [updateCheckCompleted, setUpdateCheckCompleted] = useState(false);
  const [updateCheckError, setUpdateCheckError] = useState<string | null>(null);
  const [showUpdateError, setShowUpdateError] = useState(false);
  
  const handleGameSelect = (gameId: string | number) => {
    // Prevent game selection if any game is currently running
    if (runningGameId !== null) {
      return;
    }
    setSelectedGameId(typeof gameId === 'string' ? parseInt(gameId) : gameId);
  };
  const [showNews, setShowNews] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  const selectedGame = games.find(game => game.id === selectedGameId) || null;

  const loadGames = useCallback(async () => {
    if (gamesLoadedRef.current) {
      return;
    }
    
    setIsLoading(true);
    
    try {
      const apiGames = await GameApiService.fetchGames();
      setGames(apiGames);
      gamesLoadedRef.current = true;
      
      // Auto-select first game if none selected
      setSelectedGameId(prev => prev || (apiGames.length > 0 ? apiGames[0].id : null));
    } catch (error) {
      console.error('Failed to load games:', error);
      setGames([]);
    } finally {
      setIsLoading(false);
      
      // Hide the initial loading screen once the app data is loaded
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if ((window as any).hideInitialLoading) {
        setTimeout(() => {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          (window as any).hideInitialLoading();
        }, 100);
      }
    }
  }, []);

  // Check for updates on startup
  const checkForUpdates = useCallback(async (showErrorToUser: boolean = false) => {
    if (updateCheckCompleted && !showErrorToUser) return;
    
    try {
      const updateInfo = await UpdateService.checkForUpdates();
      
      if (updateInfo.available) {
        setUpdateInfo(updateInfo);
        setShowUpdateModal(true);
        setUpdateCheckError(null);
        setShowUpdateError(false);
      } else {
        setUpdateCheckError(null);
        setShowUpdateError(false);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      setUpdateCheckError(errorMessage);
      
      if (showErrorToUser) {
        setShowUpdateError(true);
      }
    } finally {
      if (!showErrorToUser) {
        setUpdateCheckCompleted(true);
      }
    }
  }, [updateCheckCompleted]);
  
  // Function to manually refresh games list and check for updates on startup
  useEffect(() => {
    loadGames();
    checkForUpdates();
  }, [loadGames, checkForUpdates]);

  const handleGameUpdate = (updatedGame: Game) => {
    setGames(prevGames => 
      prevGames.map(game => 
        game.id === updatedGame.id ? updatedGame : game
      )
    );
  };

  const handleGameRunningStatusChange = (gameId: number, isRunning: boolean) => {
    setRunningGameId(isRunning ? gameId : null);
  };

  // Force update function for debug purposes
  const handleForceUpdate = useCallback(async () => {
    console.log('🔧 Force update triggered from debug menu');
    
    try {
      // Make a real API call with force=true to get authentic update information
      const realUpdateInfo = await UpdateService.checkForUpdates(true);
      
      // Force the update to appear available even if versions are the same
      const forcedUpdateInfo: UpdateInfo = {
        ...realUpdateInfo,
        available: true, // Always show as available in debug mode
      };
      
      setUpdateInfo(forcedUpdateInfo);
      setShowUpdateModal(true);
      setUpdateCheckError(null);
      setShowUpdateError(false);
    } catch (error) {
       console.error('❌ Force update check failed:', error);
       // Fallback to fake data if API call fails
       const fallbackUpdateInfo: UpdateInfo = {
         available: true,
         currentVersion: '0.0.7',
         latestVersion: '0.0.8',
         releaseNotes: 'This is a simulated update for testing purposes.\n\n**Debug Features:**\n- Force update functionality\n- Update error notifications\n- Enhanced update system\n\n**Note:** This is not a real update. The API call failed, so fallback debug data is being used.',
         downloadUrl: '',
         assetSize: undefined,
       };
      
      setUpdateInfo(fallbackUpdateInfo);
      setShowUpdateModal(true);
      setUpdateCheckError(null);
      setShowUpdateError(false);
    }
  }, []);

  // Retry update check function
  const handleRetryUpdateCheck = useCallback(async () => {
    setShowUpdateError(false);
    setUpdateCheckError(null);
    await checkForUpdates(true); // Show errors to user on manual retry
  }, [checkForUpdates]);

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
        <ErrorBoundary>
          <Header onForceUpdate={handleForceUpdate} />
        </ErrorBoundary>
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
      <ErrorBoundary>
        <Header onForceUpdate={handleForceUpdate} />
      </ErrorBoundary>
      
      <div className="flex-1 flex overflow-hidden relative">
        <Sidebar 
          games={games} 
          selectedGameId={selectedGameId} 
          onGameSelect={handleGameSelect}
          runningGameId={runningGameId}
        />
        
        {selectedGame ? (
          <GameDetails 
            key={selectedGame.id}
            game={selectedGame} 
            onGameUpdate={handleGameUpdate}
            onGameRunningStatusChange={handleGameRunningStatusChange}
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
        
        {/* Update Modal */}
        {updateInfo && (
          <UpdateModal
            isOpen={showUpdateModal}
            onClose={() => setShowUpdateModal(false)}
            updateInfo={updateInfo}
          />
        )}
        
        {/* Update Error Modal */}
        <UpdateErrorModal
          isOpen={showUpdateError}
          onClose={() => setShowUpdateError(false)}
          onRetry={handleRetryUpdateCheck}
          errorMessage={updateCheckError || 'Unknown error'}
        />
      </div>
    </div>
  );
}

export default App;