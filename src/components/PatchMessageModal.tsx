import React, { useState, useEffect } from 'react';
import { X, AlertCircle, Clock } from 'lucide-react';

interface PatchMessageModalProps {
  isOpen: boolean;
  onClose: () => void;
  onContinue: () => void;
  message: string;
  gameTitle: string;
}

const IGNORE_STORAGE_KEY = 'patch-message-ignore';

export const PatchMessageModal: React.FC<PatchMessageModalProps> = ({
  isOpen,
  onClose,
  onContinue,
  message,
  gameTitle
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
            handleContinue();
            return 0;
          }
          return prev - 1;
        });
      }, 1000);

      return () => clearInterval(timer);
    }
  }, [isOpen]);

  const handleContinue = () => {
    if (ignoreForDay) {
      const tomorrow = new Date();
      tomorrow.setDate(tomorrow.getDate() + 1);
      tomorrow.setHours(0, 0, 0, 0);
      
      const ignoreData = {
        message: message,
        expiresAt: tomorrow.getTime()
      };
      
      localStorage.setItem(IGNORE_STORAGE_KEY, JSON.stringify(ignoreData));
    }
    
    setIsCountdownActive(false);
    onContinue();
  };

  const handleClose = () => {
    setIsCountdownActive(false);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md mx-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-2">
            <AlertCircle className="w-6 h-6 text-yellow-500" />
            <h2 className="text-xl font-bold text-white">Important Message</h2>
          </div>
          <button
            onClick={handleClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Game Title */}
        <div className="mb-4">
          <p className="text-gray-300 text-sm">Game: <span className="text-white font-medium">{gameTitle}</span></p>
        </div>

        {/* Message */}
        <div className="mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <p className="text-white whitespace-pre-wrap">{message}</p>
          </div>
        </div>

        {/* Ignore for day checkbox */}
        <div className="mb-6">
          <label className="flex items-center space-x-2 cursor-pointer">
            <input
              type="checkbox"
              checked={ignoreForDay}
              onChange={(e) => setIgnoreForDay(e.target.checked)}
              className="w-4 h-4 text-yellow-500 bg-gray-700 border-gray-600 rounded focus:ring-yellow-500 focus:ring-2"
            />
            <span className="text-gray-300 text-sm">Ignore this message for a day</span>
          </label>
        </div>

        {/* Auto-continue countdown */}
        {isCountdownActive && (
          <div className="mb-4 flex items-center justify-center space-x-2 text-gray-400">
            <Clock className="w-4 h-4" />
            <span className="text-sm">Auto-continuing in {countdown} seconds...</span>
          </div>
        )}

        {/* Action Buttons */}
        <div className="flex space-x-3">
          <button
            onClick={handleClose}
            className="flex-1 bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleContinue}
            className="flex-1 bg-yellow-500 hover:bg-yellow-600 text-black font-medium px-4 py-2 rounded-lg transition-colors"
          >
            Continue
          </button>
        </div>
      </div>
    </div>
  );
};

// Utility function to check if a message should be ignored
export const shouldIgnoreMessage = (message: string): boolean => {
  try {
    const stored = localStorage.getItem(IGNORE_STORAGE_KEY);
    if (!stored) return false;
    
    const ignoreData = JSON.parse(stored);
    const now = Date.now();
    
    // Check if the ignore period has expired
    if (now >= ignoreData.expiresAt) {
      localStorage.removeItem(IGNORE_STORAGE_KEY);
      return false;
    }
    
    // Check if it's the same message
    return ignoreData.message === message;
  } catch (error) {
    console.error('Error checking ignore status:', error);
    localStorage.removeItem(IGNORE_STORAGE_KEY);
    return false;
  }
};