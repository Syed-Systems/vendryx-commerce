import React from "react";

export interface VendryxLogoProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "light" | "dark";
  markOnly?: boolean;
  height?: number | string;
}

export const VendryxLogo: React.FC<VendryxLogoProps> = ({
  variant = "light",
  markOnly = false,
  height = 32,
  className = "",
  ...props
}) => {
  const isDark = variant === "dark";
  const logoColor = "var(--accent)";

  if (markOnly) {
    return (
      <div className={`inline-flex items-center justify-center ${className}`} {...props}>
        <svg
          viewBox="0 0 120 120"
          style={{ height, width: height }}
          className="shrink-0"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <title>Vendryx Mark</title>
          <rect width="120" height="120" rx="28" fill={isDark ? "#27272A" : "#18181B"} />
          <path d="M26 28 H46 L60 68 L44 68 Z" fill={logoColor} />
          <path d="M74 28 H94 L68 96 H50 Z" fill={logoColor} />
          <path d="M46 28 L60 68 L74 28 Z" fill={logoColor} opacity="0.9" />
          <circle cx="60" cy="80" r="3.5" fill={logoColor} />
        </svg>
      </div>
    );
  }

  return (
    <div className={`inline-flex items-center gap-2.5 ${className}`} {...props}>
      <svg
        viewBox="0 0 320 80"
        style={{ height, width: "auto" }}
        className="shrink-0"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <title>Vendryx Logo</title>
        <g transform="translate(10, 10) scale(0.5)">
          <rect
            width="120"
            height="120"
            rx="28"
            fill={isDark ? "#27272A" : "#18181B"}
            stroke={isDark ? "#3F3F46" : "none"}
            strokeWidth="2"
          />
          <path d="M26 28 H46 L60 68 L44 68 Z" fill={logoColor} />
          <path d="M74 28 H94 L68 96 H50 Z" fill={logoColor} />
          <path d="M46 28 L60 68 L74 28 Z" fill={logoColor} opacity="0.9" />
          <circle cx="60" cy="80" r="3.5" fill={logoColor} />
        </g>

        <text
          x="82"
          y="47"
          fontFamily="Geist, Inter, system-ui, sans-serif"
          fontSize="33"
          fontWeight="700"
          letterSpacing="-0.5"
          fill={isDark ? "#FFFFFF" : "#18181B"}
        >
          Vendryx
        </text>
        <text
          x="83"
          y="65"
          fontFamily="Geist, Inter, system-ui, sans-serif"
          fontSize="10.5"
          fontWeight="600"
          letterSpacing="1.2"
          fill={isDark ? "#A1A1AA" : "#7C818A"}
        >
          BY SYED SYSTEMS
        </text>
      </svg>
    </div>
  );
};

export default VendryxLogo;
