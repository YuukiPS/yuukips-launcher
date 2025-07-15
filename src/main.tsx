import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App.tsx';
import { DownloadSettingsProvider } from './contexts/DownloadSettingsContext.tsx';
import './index.css';

// Hide the initial loading screen once React is ready
const hideInitialLoading = () => {
  const loadingElement = document.getElementById('initial-loading');
  if (loadingElement) {
    loadingElement.style.opacity = '0';
    loadingElement.style.transition = 'opacity 0.3s ease-out';
    setTimeout(() => {
      loadingElement.remove();
    }, 300);
  }
};

// Create root and render app
const root = createRoot(document.getElementById('root')!);
root.render(
  <StrictMode>
    <DownloadSettingsProvider>
      <App />
    </DownloadSettingsProvider>
  </StrictMode>
);

// Hide loading screen after React has mounted and DOM is ready
// Use requestAnimationFrame to ensure the React app has rendered
requestAnimationFrame(() => {
  // Add a small delay to ensure the app content is visible
  setTimeout(hideInitialLoading, 200);
});

// Also expose the function globally so the App component can call it when fully loaded
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(window as any).hideInitialLoading = hideInitialLoading;
