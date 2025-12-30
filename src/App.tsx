import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Header } from "./components/Header";
import { Sidebar } from "./components/Sidebar";
import { GameDetails } from "./components/GameDetails";
import { NewsPanel } from "./components/NewsPanel";
import { UpdateModal } from "./components/UpdateModal";
import { DiscordIcon } from "./components/DiscordIcon";
import { TelegramIcon } from "./components/TelegramIcon";

import { UpdateFailureModal } from "./components/UpdateFailureModal";
import ErrorBoundary from "./components/ErrorBoundary";
import { newsItems } from "./data/news";
import { socialLinks } from "./data/socialLinks";
import { Game } from "./types";
import { GameApiService } from "./services/gameApi";
import { UpdateService, UpdateInfo } from "./services/updateService";
import { DownloadService } from "./services/downloadService";
import {
  Megaphone,
  MessageCircle,
  Twitter,
  Youtube,
  Tv,
  Loader,
} from "lucide-react";

function App() {
  const [games, setGames] = useState<Game[]>([]);
  const [selectedGameId, setSelectedGameId] = useState<number | null>(null);
  const [runningGameId, setRunningGameId] = useState<number | null>(null);
  const [gameLoadError, setGameLoadError] = useState<string | null>(null);
  const gamesLoadedRef = useRef(false);

  // Update-related state
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [updateCheckCompleted, setUpdateCheckCompleted] = useState(false);
  const [updateCheckError, setUpdateCheckError] = useState<string | null>(null);
  const [showUpdateFailure, setShowUpdateFailure] = useState(false);

  const handleGameSelect = (gameId: string | number) => {
    // Prevent game selection if any game is currently running
    if (runningGameId !== null) {
      return;
    }
    const numericGameId =
      typeof gameId === "string" ? parseInt(gameId) : gameId;
    setSelectedGameId(numericGameId);

    // Save the selected game ID to localStorage
    localStorage.setItem("selectedGameId", numericGameId.toString());
  };
  const [showNews, setShowNews] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  const selectedGame = games.find((game) => game.id === selectedGameId) || null;

  const loadGames = useCallback(async () => {
    if (gamesLoadedRef.current) {
      return;
    }

    setIsLoading(true);

    try {
      const apiGames = await GameApiService.fetchGames();

      // Fetch local playtime settings and merge
      try {
        const settings: any = await invoke("get_all_app_settings");
        const playtimes = settings.game_playtime || {};

        const gamesWithPlaytime = apiGames.map((game) => {
          const seconds = playtimes[game.id] || 0;
          if (seconds > 0) {
            const hours = Math.floor(seconds / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            // Format: "1h 20m" or "45m"
            const timeString =
              hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
            return { ...game, playTime: timeString };
          }
          return game;
        });

        setGames(gamesWithPlaytime);
      } catch (e) {
        console.error("Failed to load playtime settings:", e);
        setGames(apiGames);
      }
      gamesLoadedRef.current = true;
      setGameLoadError(null); // Clear any previous errors

      // Try to restore previously selected game from localStorage
      const savedGameId = localStorage.getItem("selectedGameId");
      if (savedGameId) {
        const parsedGameId = parseInt(savedGameId);
        // Check if the saved game ID exists in the loaded games
        const gameExists = apiGames.some((game) => game.id === parsedGameId);
        if (gameExists) {
          setSelectedGameId(parsedGameId);
        } else {
          // If saved game doesn't exist, auto-select first game and update localStorage
          const firstGameId = apiGames.length > 0 ? apiGames[0].id : null;
          setSelectedGameId(firstGameId);
          if (firstGameId) {
            localStorage.setItem("selectedGameId", firstGameId.toString());
          }
        }
      } else {
        // Auto-select first game if none selected and save to localStorage
        const firstGameId = apiGames.length > 0 ? apiGames[0].id : null;
        setSelectedGameId(firstGameId);
        if (firstGameId) {
          localStorage.setItem("selectedGameId", firstGameId.toString());
        }
      }
    } catch (error) {
      console.error("Failed to load games:", error);
      const errorMessage =
        error instanceof Error ? error.message : "Unknown error occurred";
      setGameLoadError(errorMessage);
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
  const checkForUpdates = useCallback(
    async (showErrorToUser: boolean = false) => {
      if (updateCheckCompleted && !showErrorToUser) return;

      try {
        const updateInfo = await UpdateService.checkForUpdates();

        if (updateInfo.available) {
          setUpdateInfo(updateInfo);
          setShowUpdateModal(true);
          setUpdateCheckError(null);
          setShowUpdateFailure(false);
        } else {
          setUpdateCheckError(null);
          setShowUpdateFailure(false);
        }
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : "Unknown error occurred";
        setUpdateCheckError(errorMessage);
        setShowUpdateFailure(true);
      } finally {
        if (!showErrorToUser) {
          setUpdateCheckCompleted(true);
        }
      }
    },
    [updateCheckCompleted]
  );

  // Function to manually refresh games list, check for updates, and resume interrupted downloads on startup
  useEffect(() => {
    const initializeApp = async () => {
      // Resume interrupted downloads first
      try {
        await DownloadService.resumeInterruptedDownloads();
        console.log("Interrupted downloads resumed on app startup");
      } catch (error) {
        console.error(
          "Failed to resume interrupted downloads on startup:",
          error
        );
      }

      // Then load games and check for updates
      loadGames();
      checkForUpdates();
    };

    initializeApp();
  }, [loadGames, checkForUpdates]);

  // Disable right-click context menu globally
  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      return false;
    };

    // Add event listener to disable right-click context menu
    document.addEventListener("contextmenu", handleContextMenu);

    // Cleanup function to remove event listener
    return () => {
      document.removeEventListener("contextmenu", handleContextMenu);
    };
  }, []);

  const handleGameUpdate = (updatedGame: Game) => {
    setGames((prevGames) =>
      prevGames.map((game) => (game.id === updatedGame.id ? updatedGame : game))
    );
  };

  const handleGameRunningStatusChange = (
    gameId: number,
    isRunning: boolean
  ) => {
    setRunningGameId(isRunning ? gameId : null);

    // Refresh playtime when game stops
    if (!isRunning) {
      setTimeout(async () => {
        try {
          const settings: any = await invoke("get_all_app_settings");
          const playtimes = settings.game_playtime || {};
          const seconds = playtimes[gameId] || 0;

          if (seconds > 0) {
            setGames((prevGames) =>
              prevGames.map((game) => {
                if (game.id === gameId) {
                  const hours = Math.floor(seconds / 3600);
                  const minutes = Math.floor((seconds % 3600) / 60);
                  const timeString =
                    hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
                  return { ...game, playTime: timeString };
                }
                return game;
              })
            );
          }
        } catch (error) {
          console.error("Failed to refresh playtime:", error);
        }
      }, 2000); // Wait for backend to save settings
    }
  };

  // Force update function for debug purposes
  const handleForceUpdate = useCallback(async () => {
    console.log("🔧 Force update triggered from debug menu");

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
      setShowUpdateFailure(false);
    } catch (error) {
      console.error("❌ Force update check failed:", error);
      // Fallback to fake data if API call fails
      const fallbackUpdateInfo: UpdateInfo = {
        available: true,
        currentVersion: "0.0.7",
        latestVersion: "0.0.8",
        releaseNotes:
          "This is a simulated update for testing purposes.\n\n**Debug Features:**\n- Force update functionality\n- Update error notifications\n- Enhanced update system\n\n**Note:** This is not a real update. The API call failed, so fallback debug data is being used.",
        downloadUrl: "",
        assetSize: undefined,
      };

      setUpdateInfo(fallbackUpdateInfo);
      setShowUpdateModal(true);
      setUpdateCheckError(null);
      setShowUpdateFailure(false);
    }
  }, []);

  // Retry update check function
  const handleRetryUpdateCheck = useCallback(async () => {
    setShowUpdateFailure(false);
    setUpdateCheckError(null);
    await checkForUpdates(true); // Show errors to user on manual retry
  }, [checkForUpdates]);

  const getSocialIcon = (iconName: string) => {
    switch (iconName) {
      case "MessageCircle":
        return <MessageCircle className="w-5 h-5" />;
      case "Discord":
        return <DiscordIcon className="w-5 h-5" />;
      case "Telegram":
        return <TelegramIcon className="w-5 h-5" />;
      case "Twitter":
        return <Twitter className="w-5 h-5" />;
      case "Youtube":
        return <Youtube className="w-5 h-5" />;
      case "Tv":
        return <Tv className="w-5 h-5" />;
      default:
        return <MessageCircle className="w-5 h-5" />;
    }
  };

  const getSocialColor = (platform: string) => {
    switch (platform.toLowerCase()) {
      case "discord":
        return "bg-indigo-600 hover:bg-indigo-700 hover:shadow-indigo-500/25";
      case "twitter":
        return "bg-blue-500 hover:bg-blue-600 hover:shadow-blue-500/25";
      case "youtube":
        return "bg-red-600 hover:bg-red-700 hover:shadow-red-500/25";
      case "twitch":
        return "bg-purple-600 hover:bg-purple-700 hover:shadow-purple-500/25";
      default:
        return "bg-gray-600 hover:bg-gray-700 hover:shadow-gray-500/25";
    }
  };

  const handleSocialClick = async (url: string, platform: string) => {
    try {
      await openUrl(url);
    } catch (error) {
      console.error(`Failed to open ${platform}:`, error);
      // Fallback for web or if Tauri is not available
      window.open(url, "_blank");
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
        {/* Sidebar - Hidden when there's a game load error */}
        {!gameLoadError && (
          <Sidebar
            games={games}
            selectedGameId={selectedGameId}
            onGameSelect={handleGameSelect}
            runningGameId={runningGameId}
          />
        )}

        {selectedGame ? (
          <GameDetails
            key={selectedGame.id}
            game={selectedGame}
            onGameUpdate={handleGameUpdate}
            onGameRunningStatusChange={handleGameRunningStatusChange}
          />
        ) : (
          <div className="flex-1 flex items-center justify-center bg-gray-800">
            <div className="text-center max-w-md mx-auto px-6">
              {gameLoadError ? (
                <>
                  <div className="mb-4">
                    <div className="w-16 h-16 bg-red-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
                      <svg
                        className="w-8 h-8 text-red-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"
                        />
                      </svg>
                    </div>
                  </div>
                  <h3 className="text-white text-xl font-semibold mb-3">
                    Failed to Load Games
                  </h3>
                  <div className="bg-gray-700/50 rounded-lg p-4 mb-4 text-left">
                    <p className="text-red-400 text-sm font-medium mb-2">
                      Error Details:
                    </p>
                    <p className="text-gray-300 text-sm break-words">
                      {gameLoadError}
                    </p>
                  </div>
                  <div className="space-y-2">
                    <button
                      onClick={() => {
                        setGameLoadError(null);
                        gamesLoadedRef.current = false;
                        loadGames();
                      }}
                      className="w-full px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition-colors duration-200"
                    >
                      Retry Loading Games
                    </button>
                    <p className="text-gray-400 text-xs">
                      Check your internet connection and try again. If the
                      problem persists, the game server may be temporarily
                      unavailable.
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <p className="text-white text-lg mb-2">No game selected</p>
                  <p className="text-gray-400">
                    Please select a game from the sidebar
                  </p>
                </>
              )}
            </div>
          </div>
        )}

        {/* Floating Social Media Icons - Hidden when there's a game load error */}
        {!gameLoadError && (
          <div className="absolute top-6 right-6 flex space-x-3 z-40">
            {socialLinks.map((link) => (
              <button
                key={link.platform}
                onClick={() => handleSocialClick(link.url, link.platform)}
                className={`p-3 text-white rounded-full shadow-lg transition-all duration-200 hover:scale-110 ${getSocialColor(
                  link.platform
                )}`}
                title={link.platform}
              >
                {getSocialIcon(link.icon)}
              </button>
            ))}
          </div>
        )}

        {/* Floating News Button - Hidden when there's a game load error */}
        {!gameLoadError && (
          <div className="absolute bottom-6 right-6 z-40">
            <button
              onClick={() => setShowNews(!showNews)}
              className="p-3 bg-blue-600 hover:bg-blue-700 text-white rounded-full shadow-lg hover:shadow-blue-500/25 transition-all duration-200 hover:scale-110"
              title="Latest News"
            >
              <Megaphone className="w-6 h-6" />
            </button>
          </div>
        )}

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

        {/* Update Failure Modal */}
        <UpdateFailureModal
          isOpen={showUpdateFailure}
          onClose={() => setShowUpdateFailure(false)}
          onRetry={handleRetryUpdateCheck}
          errorMessage={updateCheckError || ""}
        />
      </div>
    </div>
  );
}

export default App;
