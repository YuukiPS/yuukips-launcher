import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ProxyTestModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const ProxyTestModal: React.FC<ProxyTestModalProps> = ({ isOpen, onClose }) => {
  const [testResult, setTestResult] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);

  const runProxyBypassTest = async () => {
    setIsLoading(true);
    setTestResult('');
    
    try {
      const result = await invoke('test_proxy_bypass') as string;
      setTestResult(`✅ Success: ${result}`);
    } catch (error) {
      setTestResult(`❌ Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const testGameApiCall = async () => {
    setIsLoading(true);
    setTestResult('');
    
    try {
      const result = await invoke('fetch_api_data', { 
        url: 'https://ps.yuuki.me/json/game_all.json' 
      }) as string;
      const games = JSON.parse(result);
      setTestResult(`✅ Game API Success: Fetched ${games.length} games`);
    } catch (error) {
      setTestResult(`❌ Game API Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-96 max-w-md">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold text-white">Proxy Bypass Test</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white"
          >
            ✕
          </button>
        </div>
        
        <div className="space-y-4">
          <div>
            <button
              onClick={runProxyBypassTest}
              disabled={isLoading}
              className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white px-4 py-2 rounded transition-colors"
            >
              {isLoading ? 'Testing...' : 'Test Basic Proxy Bypass'}
            </button>
          </div>
          
          <div>
            <button
              onClick={testGameApiCall}
              disabled={isLoading}
              className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 text-white px-4 py-2 rounded transition-colors"
            >
              {isLoading ? 'Testing...' : 'Test Game API Call'}
            </button>
          </div>
          
          {testResult && (
            <div className="mt-4 p-3 bg-gray-700 rounded text-sm text-white whitespace-pre-wrap">
              {testResult}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ProxyTestModal;