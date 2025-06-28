import React from 'react';
import { Calendar, Zap, Megaphone, AlertCircle } from 'lucide-react';
import { NewsItem } from '../types';

interface NewsSectionProps {
  news: NewsItem[];
}

export const NewsSection: React.FC<NewsSectionProps> = ({ news }) => {
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

  return (
    <div className="bg-gray-800 rounded-xl p-6 shadow-xl">
      <h2 className="text-2xl font-bold text-white mb-6 flex items-center space-x-2">
        <Megaphone className="w-6 h-6 text-purple-400" />
        <span>Latest News</span>
      </h2>
      
      <div className="space-y-4 max-h-96 overflow-y-auto scrollbar-thin scrollbar-thumb-gray-600 scrollbar-track-gray-800">
        {news.map((item) => (
          <div
            key={item.id}
            className="bg-gray-700/50 rounded-lg p-4 hover:bg-gray-700/70 transition-colors duration-200 cursor-pointer group"
          >
            <div className="flex items-start space-x-4">
              {item.imageUrl && (
                <img
                  src={item.imageUrl}
                  alt={item.title}
                  className="w-16 h-16 rounded-lg object-cover flex-shrink-0"
                />
              )}
              
              <div className="flex-1 min-w-0">
                <div className="flex items-center space-x-2 mb-2">
                  <span className={`inline-flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium ${getCategoryColor(item.category)}`}>
                    {getCategoryIcon(item.category)}
                    <span>{item.category}</span>
                  </span>
                  <span className="text-gray-400 text-xs">{formatDate(item.date)}</span>
                </div>
                
                <h3 className="text-white font-semibold mb-1 group-hover:text-purple-400 transition-colors">
                  {item.title}
                </h3>
                <p className="text-gray-400 text-sm line-clamp-2">{item.summary}</p>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};