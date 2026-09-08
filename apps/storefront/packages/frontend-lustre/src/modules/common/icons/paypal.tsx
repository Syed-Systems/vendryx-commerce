import React from "react"
import { IconProps } from "types/icon"

const PayPal: React.FC<IconProps> = ({
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
      d="M10 22L12 8H17C19.5 8 21 9.2 21 11.5C21 13.8 19.5 15 17 15H13.8L12.8 22H10Z"
      fill="#003087"
    />
    <path
      d="M13 24L14.5 14H18.5C21 14 22.5 15.2 22.5 17.5C22.5 19.8 21 21 18.5 21H15.8L14.8 24H13Z"
      fill="#0079C1"
    />
  </svg>
)

export default PayPal
