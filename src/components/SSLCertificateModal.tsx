import React, { useState } from 'react';
import { X, Shield, AlertTriangle, CheckCircle, Download } from 'lucide-react';
import { installSSLCertificate, checkSSLCertificateInstalled } from '../services/gameApi';

interface SSLCertificateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onInstallComplete?: () => void;
}

export const SSLCertificateModal: React.FC<SSLCertificateModalProps> = ({
  isOpen,
  onClose,
  onInstallComplete
}) => {
  const [isInstalling, setIsInstalling] = useState(false);
  const [installationResult, setInstallationResult] = useState<string | null>(null);
  const [isInstalled, setIsInstalled] = useState(false);

  if (!isOpen) return null;

  const handleInstallCertificate = async () => {
    setIsInstalling(true);
    setInstallationResult(null);
    
    try {
      const result = await installSSLCertificate();
      setInstallationResult(result);
      
      // Check if installation was successful
      if (result.includes('installed automatically')) {
        setIsInstalled(true);
        setTimeout(() => {
          onInstallComplete?.();
          onClose();
        }, 2000);
      }
    } catch (error) {
      setInstallationResult(`Installation failed: ${error}`);
    } finally {
      setIsInstalling(false);
    }
  };

  const handleCheckInstallation = async () => {
    try {
      const installed = await checkSSLCertificateInstalled();
      if (installed) {
        setIsInstalled(true);
        setInstallationResult('SSL Certificate is now installed and active!');
        setTimeout(() => {
          onInstallComplete?.();
          onClose();
        }, 2000);
      } else {
        setInstallationResult('SSL Certificate is not yet installed. Please follow the manual installation steps.');
      }
    } catch (error) {
      setInstallationResult(`Failed to check installation: ${error}`);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 border border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-2">
            <Shield className="w-6 h-6 text-yellow-500" />
            <h2 className="text-xl font-bold text-white">SSL Certificate Required</h2>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        <div className="mb-6">
          <div className="flex items-start space-x-3 mb-4">
            <AlertTriangle className="w-5 h-5 text-yellow-500 mt-0.5 flex-shrink-0" />
            <div className="text-gray-300">
              <p className="mb-2">
                To enable HTTPS interception for game domains, you need to install our SSL certificate as a trusted root certificate.
              </p>
              <p className="text-sm text-gray-400">
                This allows the proxy to decrypt and redirect HTTPS traffic from game servers to the private server.
              </p>
            </div>
          </div>

          {installationResult && (
            <div className={`p-3 rounded-lg mb-4 ${
              isInstalled || installationResult.includes('automatically') 
                ? 'bg-green-900 border border-green-700' 
                : installationResult.includes('failed') || installationResult.includes('Failed')
                ? 'bg-red-900 border border-red-700'
                : 'bg-blue-900 border border-blue-700'
            }`}>
              <div className="flex items-start space-x-2">
                {isInstalled || installationResult.includes('automatically') ? (
                  <CheckCircle className="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" />
                ) : (
                  <AlertTriangle className="w-5 h-5 text-yellow-400 mt-0.5 flex-shrink-0" />
                )}
                <p className="text-sm whitespace-pre-line">{installationResult}</p>
              </div>
            </div>
          )}
        </div>

        <div className="space-y-3">
          <button
            onClick={handleInstallCertificate}
            disabled={isInstalling || isInstalled}
            className={`w-full py-2 px-4 rounded-lg font-medium transition-colors flex items-center justify-center space-x-2 ${
              isInstalled
                ? 'bg-green-600 text-white cursor-not-allowed'
                : isInstalling
                ? 'bg-gray-600 text-gray-300 cursor-not-allowed'
                : 'bg-blue-600 hover:bg-blue-700 text-white'
            }`}
          >
            {isInstalling ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                <span>Installing...</span>
              </>
            ) : isInstalled ? (
              <>
                <CheckCircle className="w-4 h-4" />
                <span>Certificate Installed</span>
              </>
            ) : (
              <>
                <Download className="w-4 h-4" />
                <span>Install SSL Certificate</span>
              </>
            )}
          </button>

          {installationResult && !isInstalled && !installationResult.includes('automatically') && (
            <button
              onClick={handleCheckInstallation}
              className="w-full py-2 px-4 rounded-lg font-medium bg-gray-600 hover:bg-gray-700 text-white transition-colors flex items-center justify-center space-x-2"
            >
              <Shield className="w-4 h-4" />
              <span>Check Installation Status</span>
            </button>
          )}

          <button
            onClick={onClose}
            className="w-full py-2 px-4 rounded-lg font-medium bg-gray-600 hover:bg-gray-700 text-white transition-colors"
          >
            {isInstalled ? 'Continue' : 'Skip for Now'}
          </button>
        </div>

        <div className="mt-4 p-3 bg-gray-900 rounded-lg">
          <p className="text-xs text-gray-400">
            <strong>Note:</strong> This certificate is only used for redirecting game traffic to the private server. 
            It does not affect your system security or other applications.
          </p>
        </div>
      </div>
    </div>
  );
};