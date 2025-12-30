import React from "react";
import {
  MessageCircle,
  Twitter,
  Youtube,
  Tv,
  ExternalLink,
  X,
} from "lucide-react";
import { SocialLink } from "../types";
import { DiscordIcon } from "./DiscordIcon";
import { TelegramIcon } from "./TelegramIcon";

interface SocialPanelProps {
  links: SocialLink[];
  isOpen: boolean;
  onClose: () => void;
}

export const SocialPanel: React.FC<SocialPanelProps> = ({
  links,
  isOpen,
  onClose,
}) => {
  const getIcon = (iconName: string) => {
    switch (iconName) {
      case "MessageCircle":
        return <MessageCircle className="w-5 h-5" />;
      case "Discord":
        return <DiscordIcon className="w-5 h-5" />;
      case "Telegram":
        return <TelegramIcon className="w-5 h-5" />;
      case "Twitter":
        return <Twitter className="w-5 h-5" />;
      case "Youtube":
        return <Youtube className="w-5 h-5" />;
      case "Tv":
        return <Tv className="w-5 h-5" />;
      default:
        return <ExternalLink className="w-5 h-5" />;
    }
  };

  const getPlatformColor = (platform: string) => {
    switch (platform.toLowerCase()) {
      case "discord":
        return "hover:bg-indigo-600 hover:shadow-indigo-500/25 bg-indigo-600/20";
      case "twitter":
        return "hover:bg-blue-500 hover:shadow-blue-500/25 bg-blue-500/20";
      case "youtube":
        return "hover:bg-red-600 hover:shadow-red-500/25 bg-red-600/20";
      case "twitch":
        return "hover:bg-purple-600 hover:shadow-purple-500/25 bg-purple-600/20";
      default:
        return "hover:bg-gray-600 hover:shadow-gray-500/25 bg-gray-600/20";
    }
  };

  const handleClick = (url: string, platform: string) => {
    alert(
      `This is a web demo. In the desktop version, this would open ${platform}.`
    );
  };

  if (!isOpen) return null;

  return (
    <div className="absolute top-4 right-4 bg-gray-900/95 backdrop-blur-sm rounded-xl border border-gray-700 shadow-2xl z-50 w-80">
      <div className="p-4 border-b border-gray-700 flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <ExternalLink className="w-5 h-5 text-purple-400" />
          <h3 className="text-lg font-bold text-white">Connect With Us</h3>
        </div>
        <button
          onClick={onClose}
          className="p-1 text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="p-4">
        <div className="grid grid-cols-2 gap-3 mb-4">
          {links.map((link) => (
            <button
              key={link.platform}
              onClick={() => handleClick(link.url, link.platform)}
              className={`flex items-center space-x-3 p-3 rounded-lg transition-all duration-200 hover:transform hover:scale-105 ${getPlatformColor(
                link.platform
              )}`}
            >
              <div className="text-white">{getIcon(link.icon)}</div>
              <span className="text-white font-medium text-sm">
                {link.platform}
              </span>
            </button>
          ))}
        </div>

        <div className="p-3 bg-gray-800/50 rounded-lg">
          <p className="text-gray-400 text-xs text-center">
            Join our community for updates, events, and exclusive content!
          </p>
        </div>
      </div>
    </div>
  );
};
