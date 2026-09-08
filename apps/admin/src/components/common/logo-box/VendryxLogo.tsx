import React from "react";

export interface VendryxLogoProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "light" | "dark";
  markOnly?: boolean;
  height?: number | string;
}

export const VendryxLogo: React.FC<VendryxLogoProps> = ({
  variant = "light",
  markOnly = false,
  height = 28,
  className = "",
  ...props
}) => {
  const isDark = variant === "dark";

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
          <defs>
            <linearGradient id="vxAdminG1" x1="20" y1="20" x2="100" y2="100" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#FB923C" />
              <stop offset="60%" stopColor="#EA580C" />
              <stop offset="100%" stopColor="#C2410C" />
            </linearGradient>
            <linearGradient id="vxAdminG2" x1="90" y1="25" x2="45" y2="95" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#EA580C" />
              <stop offset="100%" stopColor="#9A3412" />
            </linearGradient>
            <linearGradient id="vxAdminG3" x1="45" y1="28" x2="75" y2="72" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#FFF7ED" />
              <stop offset="100%" stopColor="#FED7AA" />
            </linearGradient>
          </defs>
          <rect width="120" height="120" rx="28" fill={isDark ? "#27272A" : "#18181B"} />
          <path d="M26 28 H46 L60 68 L44 68 Z" fill="url(#vxAdminG1)" />
          <path d="M74 28 H94 L68 96 H50 Z" fill="url(#vxAdminG2)" />
          <path d="M46 28 L60 68 L74 28 Z" fill="url(#vxAdminG3)" opacity="0.9" />
          <circle cx="60" cy="80" r="3.5" fill="#FB923C" />
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
        <defs>
          <linearGradient id="vxAdminLogoG1" x1="10" y1="10" x2="70" y2="70" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stopColor="#FB923C" />
            <stop offset="60%" stopColor="#EA580C" />
            <stop offset="100%" stopColor="#C2410C" />
          </linearGradient>
          <linearGradient id="vxAdminLogoG2" x1="65" y1="15" x2="35" y2="65" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stopColor="#EA580C" />
            <stop offset="100%" stopColor="#9A3412" />
          </linearGradient>
          <linearGradient id="vxAdminLogoG3" x1="35" y1="18" x2="55" y2="52" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stopColor="#FFF7ED" />
            <stop offset="100%" stopColor="#FED7AA" />
          </linearGradient>
        </defs>

        <g transform="translate(10, 10) scale(0.5)">
          <rect
            width="120"
            height="120"
            rx="28"
            fill={isDark ? "#27272A" : "#18181B"}
            stroke={isDark ? "#3F3F46" : "none"}
            strokeWidth="2"
          />
          <path d="M26 28 H46 L60 68 L44 68 Z" fill="url(#vxAdminLogoG1)" />
          <path d="M74 28 H94 L68 96 H50 Z" fill="url(#vxAdminLogoG2)" />
          <path d="M46 28 L60 68 L74 28 Z" fill="url(#vxAdminLogoG3)" opacity="0.9" />
          <circle cx="60" cy="80" r="3.5" fill="#FB923C" />
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
