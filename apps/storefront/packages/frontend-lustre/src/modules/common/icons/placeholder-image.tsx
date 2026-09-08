import React from "react"
import { Image as PhosphorImage } from "@phosphor-icons/react/dist/ssr"
import { IconProps } from "types/icon"

const PlaceholderImage: React.FC<IconProps> = ({
  size = "24",
  color = "currentColor",
  className = "",
  ...attributes
}) => (
  <PhosphorImage
    size={Number(size) || 24}
    color={color}
    className={className}
    aria-hidden="true"
    {...attributes}
  />
)

export default PlaceholderImage
