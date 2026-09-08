import React from "react"
import { VariantPrice } from "types/global"

export type PreviewPriceProps = {
  price?: VariantPrice | null
}

/**
 * Lustre PreviewPrice.
 * Tabular product listing price with sale strikethrough.
 */
export default async function PreviewPrice({ price }: PreviewPriceProps) {
  if (!price) {
    return null
  }

  const isSale = price.price_type === "sale"

  return (
    <div className="flex items-center gap-1.5 text-xs tabular-nums">
      {isSale && (
        <span
          className="line-through text-[var(--text-muted)] text-[11px]"
          data-testid="original-price"
        >
          {price.original_price}
        </span>
      )}
      <span
        className={`font-semibold ${
          isSale ? "text-[var(--accent-sapphire)]" : "text-[var(--text-primary)]"
        }`}
        data-testid="price"
      >
        {price.calculated_price}
      </span>
    </div>
  )
}
