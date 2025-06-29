import React from 'react';
import { Game } from '../types';

interface SidebarProps {
  games: Game[];
  selectedGameId: number | null;
  onGameSelect: (gameId: string | number) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ games, selectedGameId, onGameSelect }) => {
  return (
    <div className="w-20 bg-gray-900/95 backdrop-blur-sm border-r border-gray-700 flex flex-col">
      <div className="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-gray-600 scrollbar-track-gray-800">
        <div className="p-3 space-y-3">
          {games.map((game) => (
            <div
              key={game.id}
              onClick={() => onGameSelect(game.id)}
              className={`relative w-14 h-14 rounded-xl cursor-pointer transition-all duration-300 group overflow-hidden ${
                selectedGameId === game.id
                  ? 'ring-2 ring-purple-500 ring-offset-2 ring-offset-gray-900'
                  : 'hover:ring-2 hover:ring-gray-500 hover:ring-offset-2 hover:ring-offset-gray-900'
              }`}
            >
              <img
                src={game.icon}
                alt={game.title}
                className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-110"
              />
              
              {selectedGameId === game.id && (
                <div className="absolute left-0 top-0 bottom-0 w-1 bg-gradient-to-b from-purple-400 to-blue-400 rounded-r-full" />
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};