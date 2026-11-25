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
  const src = markOnly
    ? "/brand/vendryx/vendryx-mark.svg"
    : isDark
    ? "/brand/vendryx/vendryx-primary-dark.svg"
    : "/brand/vendryx/vendryx-primary-light.svg";
  const alt = markOnly ? "Vendryx Mark" : "Vendryx by Syed Systems";

  return (
    <div className={`inline-flex items-center justify-center shrink-0 ${className}`} {...props}>
      <img
        src={src}
        alt={alt}
        style={{ height, width: markOnly ? height : "auto" }}
        className="shrink-0 object-contain"
        loading="eager"
      />
    </div>
  );
};

export default VendryxLogo;
