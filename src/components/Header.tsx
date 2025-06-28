import React, { useState, useEffect } from 'react';
import { Gamepad2, Settings, User, Bell } from 'lucide-react';
import { WindowControls } from './WindowControls';

export const Header: React.FC = () => {
  const [isTauri, setIsTauri] = useState(false);

  useEffect(() => {
    setIsTauri(window.__TAURI__ !== undefined);
  }, []);

  return (
    <header className="bg-gray-900/95 backdrop-blur-sm border-b border-gray-700 p-4 flex-shrink-0">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3" data-tauri-drag-region={isTauri ? "" : undefined}>
          <div className="p-2 bg-gradient-to-r from-purple-600 to-blue-600 rounded-lg">
            <Gamepad2 className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white">YuukiPS Launcher</h1>
            <p className="text-gray-400 text-sm">{isTauri ? 'Desktop Version' : 'Web Demo Version'}</p>
          </div>
        </div>

        <div className="flex items-center space-x-4">
          <button className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors duration-200">
            <Bell className="w-5 h-5" />
          </button>
          <button className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors duration-200">
            <Settings className="w-5 h-5" />
          </button>
          <button className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors duration-200">
            <User className="w-5 h-5" />
          </button>
          <WindowControls />
        </div>
      </div>
    </header>
  );
};