import React from "react"
import { IconProps } from "types/icon"

const Bancontact: React.FC<IconProps> = ({
  size = "24",
  className,
  ...attributes
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    className={className}
    aria-hidden="true"
    xmlns="http://www.w3.org/2000/svg"
    {...attributes}
  >
    <rect width="32" height="32" rx="6" fill="#F4F6F7" />
    <path
      d="M10 11L18 21H13L7 13.5L10 11Z"
      fill="#00529B"
    />
    <path
      d="M22 21L14 11H19L25 18.5L22 21Z"
      fill="#FFD200"
    />
  </svg>
)

export default Bancontact
