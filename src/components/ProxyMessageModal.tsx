import React, { useState, useEffect } from 'react';
import { X, AlertCircle, Clock } from 'lucide-react';

interface ProxyMessageModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSetAndContinue: () => void;
  onContinueWithoutSetting: () => void;
  message: string;
  gameTitle: string;
  recommendedServer: string;
}

const IGNORE_STORAGE_KEY = 'proxy-message-ignore';

export const ProxyMessageModal: React.FC<ProxyMessageModalProps> = ({
  isOpen,
  onClose,
  onSetAndContinue,
  onContinueWithoutSetting,
  message,
  gameTitle,
  recommendedServer
}) => {
  const [ignoreForDay, setIgnoreForDay] = useState(false);
  const [countdown, setCountdown] = useState(30);
  const [isCountdownActive, setIsCountdownActive] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setCountdown(30);
      setIsCountdownActive(true);
      const timer = setInterval(() => {
        setCountdown(prev => {
          if (prev <= 1) {
            setIsCountdownActive(false);
            handleContinueWithoutSetting();
            return 0;
          }
          return prev - 1;
        });
      }, 1000);
      return () => clearInterval(timer);
    }
  }, [isOpen]);

  const persistIgnoreIfNeeded = () => {
    if (ignoreForDay) {
      const tomorrow = new Date();
      tomorrow.setDate(tomorrow.getDate() + 1);
      tomorrow.setHours(0, 0, 0, 0);
      const ignoreData = {
        message,
        recommendedServer,
        expiresAt: tomorrow.getTime()
      };
      localStorage.setItem(IGNORE_STORAGE_KEY, JSON.stringify(ignoreData));
    }
  };

  const handleContinueWithSetting = () => {
    persistIgnoreIfNeeded();
    onSetAndContinue();
  };

  const handleContinueWithoutSetting = () => {
    persistIgnoreIfNeeded();
    onContinueWithoutSetting();
  };

  const handleClose = () => {
    setIsCountdownActive(false);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md mx-4">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-2">
            <AlertCircle className="w-6 h-6 text-yellow-500" />
            <h2 className="text-xl font-bold text-white">Proxy Recommendation</h2>
          </div>
          <button onClick={handleClose} className="text-gray-400 hover:text-white transition-colors">
            <X className="w-6 h-6" />
          </button>
        </div>

        <div className="mb-4">
          <p className="text-gray-300 text-sm">Game: <span className="text-white font-medium">{gameTitle}</span></p>
        </div>

        <div className="mb-3">
          <div className="bg-gray-700 rounded-lg p-4">
            <p className="text-white whitespace-pre-wrap">{message}</p>
          </div>
        </div>

        <div className="mb-4">
          <div className="bg-gray-700 rounded-lg p-3">
            <p className="text-gray-300 text-sm">Recommended server: <span className="text-white font-mono">{recommendedServer}</span></p>
          </div>
        </div>

        <div className="mb-6">
          <label className="flex items-center space-x-2 cursor-pointer">
            <input
              type="checkbox"
              checked={ignoreForDay}
              onChange={(e) => setIgnoreForDay(e.target.checked)}
              className="w-4 h-4 text-yellow-500 bg-gray-700 border-gray-600 rounded focus:ring-yellow-500 focus:ring-2"
            />
            <span className="text-gray-300 text-sm">Ignore this recommendation for a day</span>
          </label>
        </div>

        {isCountdownActive && (
          <div className="mb-4 flex items-center justify-center space-x-2 text-gray-400">
            <Clock className="w-4 h-4" />
            <span className="text-sm">Auto-continue without setting in {countdown} seconds...</span>
          </div>
        )}

        <div className="grid grid-cols-1 gap-2">
          <button onClick={handleClose} className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg transition-colors">
            Cancel
          </button>
          <button onClick={handleContinueWithSetting} className="bg-green-600 hover:bg-green-700 text-white font-medium px-4 py-2 rounded-lg transition-colors">
            Continue with Recommendation
          </button>
          <button onClick={handleContinueWithoutSetting} className="bg-red-600 hover:bg-red-700 text-white font-medium px-4 py-2 rounded-lg transition-colors">
            Ignore Recommendation
          </button>
        </div>
      </div>
    </div>
  );
};

export const shouldIgnoreProxyMessage = (message: string): boolean => {
  try {
    const stored = localStorage.getItem(IGNORE_STORAGE_KEY);
    if (!stored) return false;
    const ignoreData = JSON.parse(stored);
    const now = Date.now();
    if (now >= ignoreData.expiresAt) {
      localStorage.removeItem(IGNORE_STORAGE_KEY);
      return false;
    }
    return ignoreData.message === message;
  } catch {
    localStorage.removeItem(IGNORE_STORAGE_KEY);
    return false;
  }
};
