import React from "react";

interface TelegramIconProps {
  className?: string;
}

export const TelegramIcon: React.FC<TelegramIconProps> = ({ className }) => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 256 256"
      className={className}
      fill="none"
      xmlnsXlink="http://www.w3.org/1999/xlink"
    >
      <g transform="translate(1.4065934065934016 1.4065934065934016) scale(2.81 2.81)">
        <path
          d="M 36.75 65.625 c -1.458 0 -1.21 -0.55 -1.713 -1.939 L 30.75 49.578 L 63.75 30"
          fill="rgb(200,218,234)"
        />
        <path
          d="M 36.75 65.625 c 1.125 0 1.622 -0.514 2.25 -1.125 l 6 -5.834 l -7.484 -4.513"
          fill="rgb(169,201,221)"
        />
        <path
          d="M 37.515 54.154 L 55.65 67.552 c 2.07 1.142 3.563 0.551 4.079 -1.921 l 7.382 -34.786 c 0.756 -3.03 -1.155 -4.405 -3.135 -3.506 L 20.629 44.053 c -2.959 1.187 -2.941 2.838 -0.539 3.573 l 11.124 3.472 l 25.752 -16.247 c 1.216 -0.737 2.332 -0.341 1.416 0.472 L 37.515 54.154 z"
          fill="rgb(252,253,255)"
        />
      </g>
    </svg>
  );
};
