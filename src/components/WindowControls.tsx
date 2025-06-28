import React from 'react';
import { Minus, Square, X } from 'lucide-react';

export const WindowControls: React.FC = () => {
  const handleMinimize = () => {
    // Web demo - show message instead of actual minimize
    alert('Window controls are disabled in the web demo version');
  };

  const handleMaximize = () => {
    alert('Window controls are disabled in the web demo version');
  };

  const handleClose = () => {
    alert('Window controls are disabled in the web demo version');
  };

  return (
    <div className="flex items-center space-x-2">
      <button
        onClick={handleMinimize}
        className="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition-colors duration-200 flex items-center justify-center group opacity-50 cursor-not-allowed"
        title="Minimize (Disabled in web demo)"
      >
        <Minus className="w-2 h-2 text-yellow-800 opacity-0 group-hover:opacity-100 transition-opacity" />
      </button>
      <button
        onClick={handleMaximize}
        className="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition-colors duration-200 flex items-center justify-center group opacity-50 cursor-not-allowed"
        title="Maximize (Disabled in web demo)"
      >
        <Square className="w-2 h-2 text-green-800 opacity-0 group-hover:opacity-100 transition-opacity" />
      </button>
      <button
        onClick={handleClose}
        className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition-colors duration-200 flex items-center justify-center group opacity-50 cursor-not-allowed"
        title="Close (Disabled in web demo)"
      >
        <X className="w-2 h-2 text-red-800 opacity-0 group-hover:opacity-100 transition-opacity" />
      </button>
    </div>
  );
};