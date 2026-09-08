import React from "react"
import { IconProps } from "types/icon"

const NextJs: React.FC<IconProps> = ({
  size = "20",
  className,
  ...attributes
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 180 180"
    fill="none"
    className={className}
    aria-hidden="true"
    xmlns="http://www.w3.org/2000/svg"
    {...attributes}
  >
    <mask
      id="next-mask"
      maskUnits="userSpaceOnUse"
      x="0"
      y="0"
      width="180"
      height="180"
      style={{ maskType: "alpha" }}
    >
      <circle cx="90" cy="90" r="90" fill="black" />
    </mask>
    <g mask="url(#next-mask)">
      <circle cx="90" cy="90" r="90" fill="black" />
      <path
        d="M149.508 157.438L69.1478 54H54V125.97H66.1136V69.3836L139.999 164.845C143.333 162.614 146.509 160.137 149.508 157.438Z"
        fill="white"
      />
      <rect x="115" y="54" width="12" height="72" fill="white" />
    </g>
  </svg>
)

export default NextJs
