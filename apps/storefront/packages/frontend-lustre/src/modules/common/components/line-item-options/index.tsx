import React from "react"
import { HttpTypes } from "@medusajs/types"

export type LineItemOptionsProps = {
  variant: HttpTypes.StoreProductVariant | undefined
  title?: string | null
  "data-testid"?: string
  "data-value"?: HttpTypes.StoreProductVariant
}

/**
 * Lustre LineItemOptions.
 * Renders variant title on line items without @medusajs/ui.
 */
const LineItemOptions: React.FC<LineItemOptionsProps> = ({
  variant,
  title,
  "data-testid": dataTestId,
  "data-value": dataValue,
}) => {
  const label = variant?.title ?? title

  if (!label) {
    return null
  }

  return (
    <span
      data-testid={dataTestId}
      data-value={dataValue}
      className="text-xs text-[var(--text-muted)] truncate block"
    >
      Edition: {label}
    </span>
  )
}

export default LineItemOptions
