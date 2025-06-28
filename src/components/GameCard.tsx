import React from 'react';
import { Play, Clock, Calendar } from 'lucide-react';
import { Game } from '../types';

interface GameCardProps {
  game: Game;
}

export const GameCard: React.FC<GameCardProps> = ({ game }) => {
  const handlePlay = () => {
    alert(`This is a web demo. In the desktop version, this would launch ${game.title}.`);
  };

  return (
    <div className="bg-gray-800 rounded-xl overflow-hidden shadow-xl hover:shadow-2xl transition-all duration-300 hover:transform hover:scale-105 group">
      <div className="relative h-48 overflow-hidden">
        <img
          src={game.image}
          alt={game.title}
          className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
        />
      </div>
      
      <div className="p-6">
        <div className="mb-4">
          <h3 className="text-xl font-bold text-white mb-1">{game.title}</h3>
          {game.subtitle && <p className="text-purple-400 text-sm font-medium">{game.subtitle}</p>}
          <p className="text-gray-400 text-sm mt-2 line-clamp-2">{game.description}</p>
        </div>

        <div className="flex items-center justify-between text-xs text-gray-400 mb-4">
          {game.version && <span>Version {game.version}</span>}
          {game.lastUpdate && !game.version && (
            <span>Updated {new Date(game.lastUpdate * 1000).toLocaleDateString()}</span>
          )}
          {game.lastPlayed && (
            <div className="flex items-center space-x-1">
              <Calendar className="w-3 h-3" />
              <span>{game.lastPlayed}</span>
            </div>
          )}
        </div>

        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4 text-xs text-gray-400">
            {game.playTime && (
              <div className="flex items-center space-x-1">
                <Clock className="w-3 h-3" />
                <span>{game.playTime}</span>
              </div>
            )}
          </div>
          
          <button
            onClick={handlePlay}
            disabled={game.status !== 'available'}
            className="flex items-center space-x-2 bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 disabled:from-gray-600 disabled:to-gray-700 disabled:cursor-not-allowed text-white px-6 py-2 rounded-lg font-medium transition-all duration-200 hover:shadow-lg hover:shadow-purple-500/25"
          >
            <Play className="w-4 h-4" />
            <span>Play</span>
          </button>
        </div>
      </div>
    </div>
  );
};