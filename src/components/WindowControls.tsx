import React, { useState, useEffect } from 'react';
import { Minus, Square, X, Maximize2 } from 'lucide-react';

// We'll get the window instance inside the component to avoid import issues

export const WindowControls: React.FC = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const [, setIsTauri] = useState(false);

  useEffect(() => {
    // Simple and reliable Tauri detection
    const detectTauri = () => {
      const isTauriEnv = typeof window !== 'undefined' && '__TAURI__' in window;
      console.log('Tauri detection:', {
        'window.__TAURI__': window.__TAURI__,
        'isTauriEnv': isTauriEnv,
        'userAgent': navigator.userAgent
      });
      setIsTauri(isTauriEnv);
      return isTauriEnv;
    };
    
    const isTauriEnv = detectTauri();
    
    if (isTauriEnv) {
      // Setup window event listeners for Tauri
      import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
        const appWindow = getCurrentWindow();
        
        // Listen for window resize events
        const setupListeners = async () => {
          const unlisten = await appWindow.onResized(() => {
            appWindow.isMaximized().then(setIsMaximized);
          });
          
          // Get initial maximize state
          appWindow.isMaximized().then(setIsMaximized);
          
          return unlisten;
        };
        
        setupListeners().then(unlisten => {
          // Store cleanup function
          return () => unlisten();
        });
      }).catch(error => {
        console.log('Failed to setup Tauri window listeners:', error);
      });
    }
  }, []);

  const handleMinimize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const appWindow = getCurrentWindow();
      await appWindow.minimize();
    } catch (error) {
      console.log('Minimize not available in web environment:', error);
    }
  };

  const handleMaximize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const appWindow = getCurrentWindow();
      if (isMaximized) {
        await appWindow.unmaximize();
      } else {
        await appWindow.maximize();
      }
      setIsMaximized(!isMaximized);
    } catch (error) {
      console.log('Maximize not available in web environment:', error);
    }
  };

  const handleClose = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const appWindow = getCurrentWindow();
      await appWindow.close();
    } catch (error) {
      console.log('Close not available in web environment:', error);
    }
  };

  return (
    <div className="flex items-center space-x-2" data-tauri-drag-region="false">
      <button
        onClick={handleMinimize}
        className="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition-colors duration-200 flex items-center justify-center group"
        title="Minimize"
      >
        <Minus className="w-2 h-2 text-yellow-800 opacity-0 group-hover:opacity-100 transition-opacity" />
      </button>
      <button
        onClick={handleMaximize}
        className="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition-colors duration-200 flex items-center justify-center group"
        title={isMaximized ? "Restore" : "Maximize"}
      >
        {isMaximized ? (
          <Maximize2 className="w-2 h-2 text-green-800 opacity-0 group-hover:opacity-100 transition-opacity" />
        ) : (
          <Square className="w-2 h-2 text-green-800 opacity-0 group-hover:opacity-100 transition-opacity" />
        )}
      </button>
      <button
        onClick={handleClose}
        className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition-colors duration-200 flex items-center justify-center group"
        title="Close"
      >
        <X className="w-2 h-2 text-red-800 opacity-0 group-hover:opacity-100 transition-opacity" />
      </button>
    </div>
  );
};