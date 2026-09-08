import React from "react"
import { IconProps } from "types/icon"

const Spinner: React.FC<IconProps> = ({
  size = "18",
  color = "currentColor",
  className = "",
  ...attributes
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    className={`animate-spin ${className}`}
    aria-hidden="true"
    {...attributes}
  >
    <circle
      cx="12"
      cy="12"
      r="10"
      stroke={color}
      strokeWidth="3"
      className="opacity-20"
    />
    <path
      d="M12 2C6.47715 2 2 6.47715 2 12"
      stroke={color}
      strokeWidth="3"
      strokeLinecap="round"
      className="opacity-80"
    />
  </svg>
)

export default Spinner
