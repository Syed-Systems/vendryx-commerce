import React from "react"
import { convertToLocale } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"

export type LineItemUnitPriceProps = {
  item: HttpTypes.StoreCartLineItem | HttpTypes.StoreOrderLineItem
  style?: "default" | "tight"
  currencyCode: string
}

/**
 * Lustre LineItemUnitPrice.
 * Unit price breakdown with tabular numerals.
 */
const LineItemUnitPrice: React.FC<LineItemUnitPriceProps> = ({
  item,
  style = "default",
  currencyCode,
}) => {
  const currentTotal = item.total ?? 0
  const origTotal = item.original_total ?? 0
  const qty = item.quantity || 1
  const hasReducedPrice = currentTotal < origTotal && origTotal > 0

  const percentage_diff =
    origTotal > 0
      ? Math.round(((origTotal - currentTotal) / origTotal) * 100)
      : 0

  return (
    <div className="flex flex-col text-xs text-[var(--text-secondary)]">
      {hasReducedPrice && (
        <div className="flex items-center gap-1 text-[var(--text-muted)]">
          {style === "default" && <span>Was: </span>}
          <span
            className="line-through tabular-nums"
            data-testid="product-unit-original-price"
          >
            {convertToLocale({
              amount: origTotal / qty,
              currency_code: currencyCode,
            })}
          </span>
          {style === "default" && (
            <span className="font-semibold text-[var(--accent-sapphire)]">
              -{percentage_diff}%
            </span>
          )}
        </div>
      )}
      <span
        data-testid="product-unit-price"
        className={`tabular-nums font-medium ${
          hasReducedPrice ? "text-[var(--accent-sapphire)]" : "text-[var(--text-primary)]"
        }`}
      >
        {convertToLocale({
          amount: currentTotal / qty,
          currency_code: currencyCode,
        })}
      </span>
    </div>
  )
}

export default LineItemUnitPrice
