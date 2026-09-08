import React from "react"
import { IconProps } from "types/icon"

const Ideal: React.FC<IconProps> = ({
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
      d="M16 6C10.477 6 6 10.477 6 16C6 21.523 10.477 26 16 26C21.523 26 26 21.523 26 16C26 10.477 21.523 6 16 6Z"
      fill="#CC0066"
    />
    <path
      d="M14 10H16V22H14V10ZM17.5 13H19C20.657 13 22 14.343 22 16C22 17.657 20.657 19 19 19H17.5V13Z"
      fill="#FFFFFF"
    />
  </svg>
)

export default Ideal
