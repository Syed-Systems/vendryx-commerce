import React from "react"
import { CaretDown } from "@phosphor-icons/react/dist/ssr"
import { IconProps } from "types/icon"

/**
 * Lustre ChevronDown slot: Phosphor caret with rotation state (Plan Table 8).
 * Conforms to contract slot without @medusajs/icons.
 */
const ChevronDown: React.FC<IconProps> = ({
  size = 16,
  color = "currentColor",
  className = "",
  ...attributes
}) => {
  return (
    <CaretDown
      size={Number(size) || 16}
      color={color}
      className={className}
      {...(attributes as any)}
    />
  )
}

export default ChevronDown
