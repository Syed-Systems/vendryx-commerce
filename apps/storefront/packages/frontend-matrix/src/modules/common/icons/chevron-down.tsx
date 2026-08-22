import React from "react"
import { IconProps } from "types/icon"

/** Square-capped chevron — MASTER.md §1: nothing in this theme is rounded. */
const ChevronDown: React.FC<IconProps> = ({
  size = "16",
  color = "currentColor",
  ...attributes
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke={color}
    strokeWidth="2"
    strokeLinecap="square"
    strokeLinejoin="miter"
    xmlns="http://www.w3.org/2000/svg"
    {...attributes}
  >
    <path d="m5 9 7 7 7-7" />
  </svg>
)

export default ChevronDown
