import React from "react";
import {
  MessageCircle,
  Twitter,
  Youtube,
  Tv,
  ExternalLink,
} from "lucide-react";
import { SocialLink } from "../types";
import { DiscordIcon } from "./DiscordIcon";
import { TelegramIcon } from "./TelegramIcon";

interface SocialLinksProps {
  links: SocialLink[];
}

export const SocialLinks: React.FC<SocialLinksProps> = ({ links }) => {
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
        return "hover:bg-indigo-600 hover:shadow-indigo-500/25";
      case "telegram":
        return "hover:bg-blue-500 hover:shadow-blue-500/25";
      case "youtube":
        return "hover:bg-red-600 hover:shadow-red-500/25";
      default:
        return "hover:bg-gray-600 hover:shadow-gray-500/25";
    }
  };

  const handleClick = (url: string, platform: string) => {
    alert(
      `This is a web demo. In the desktop version, this would open ${platform}.`
    );
  };

  return (
    <div className="bg-gray-800 rounded-xl p-6 shadow-xl">
      <h2 className="text-xl font-bold text-white mb-4 flex items-center space-x-2">
        <ExternalLink className="w-5 h-5 text-purple-400" />
        <span>Connect With Us</span>
      </h2>

      <div className="grid grid-cols-2 gap-3">
        {links.map((link) => (
          <button
            key={link.platform}
            onClick={() => handleClick(link.url, link.platform)}
            className={`flex items-center space-x-3 p-3 bg-gray-700 rounded-lg transition-all duration-200 hover:transform hover:scale-105 ${getPlatformColor(
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

      <div className="mt-4 p-3 bg-gray-700/50 rounded-lg">
        <p className="text-gray-400 text-xs text-center">
          Join our community for updates, events, and exclusive content!
        </p>
      </div>
    </div>
  );
};
