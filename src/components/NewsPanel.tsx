import React from 'react';
import { Calendar, Zap, Megaphone, AlertCircle, X } from 'lucide-react';
import { NewsItem } from '../types';

interface NewsPanelProps {
  news: NewsItem[];
  isOpen: boolean;
  onClose: () => void;
}

export const NewsPanel: React.FC<NewsPanelProps> = ({ news, isOpen, onClose }) => {
  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'update':
        return <Zap className="w-4 h-4" />;
      case 'event':
        return <Calendar className="w-4 h-4" />;
      case 'announcement':
        return <Megaphone className="w-4 h-4" />;
      default:
        return <AlertCircle className="w-4 h-4" />;
    }
  };

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'update':
        return 'text-blue-400 bg-blue-500/20';
      case 'event':
        return 'text-green-400 bg-green-500/20';
      case 'announcement':
        return 'text-yellow-400 bg-yellow-500/20';
      default:
        return 'text-gray-400 bg-gray-500/20';
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric',
      year: 'numeric'
    });
  };

  if (!isOpen) return null;

  return (
    <div className="absolute bottom-4 left-4 right-4 bg-gray-900/95 backdrop-blur-sm rounded-xl border border-gray-700 shadow-2xl z-50 max-h-96">
      <div className="p-4 border-b border-gray-700 flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <Megaphone className="w-5 h-5 text-purple-400" />
          <h3 className="text-lg font-bold text-white">Latest News</h3>
        </div>
        <button
          onClick={onClose}
          className="p-1 text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
      
      <div className="p-4 space-y-3 max-h-80 overflow-y-auto scrollbar-thin scrollbar-thumb-gray-600 scrollbar-track-gray-800">
        {news.slice(0, 3).map((item) => (
          <div
            key={item.id}
            className="bg-gray-800/50 rounded-lg p-3 hover:bg-gray-700/50 transition-colors duration-200 cursor-pointer"
          >
            <div className="flex items-center space-x-2 mb-2">
              <span className={`inline-flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium ${getCategoryColor(item.category)}`}>
                {getCategoryIcon(item.category)}
                <span>{item.category}</span>
              </span>
              <span className="text-gray-400 text-xs">{formatDate(item.date)}</span>
            </div>
            
            <h4 className="text-white font-medium mb-1 text-sm">
              {item.title}
            </h4>
            <p className="text-gray-400 text-xs line-clamp-2">{item.summary}</p>
          </div>
        ))}
      </div>
    </div>
  );
};