import React from "react"
import { getPercentageDiff, convertToLocale } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"

export type LineItemPriceProps = {
  item: HttpTypes.StoreCartLineItem | HttpTypes.StoreOrderLineItem
  style?: "default" | "tight"
  currencyCode: string
}

/**
 * Lustre LineItemPrice.
 * Displays item total price with compare-at strikethrough and percentage discount.
 */
const LineItemPrice: React.FC<LineItemPriceProps> = ({
  item,
  style = "default",
  currencyCode,
}) => {
  const { total, original_total } = item
  const originalPrice = original_total ?? 0
  const currentPrice = total ?? 0
  const hasReducedPrice = currentPrice < originalPrice && originalPrice > 0

  return (
    <div className="flex flex-col items-end text-sm">
      {hasReducedPrice && (
        <div className="flex items-center gap-1.5 text-xs text-[var(--text-muted)]">
          <span
            className="line-through tabular-nums"
            data-testid="product-original-price"
          >
            {convertToLocale({
              amount: originalPrice,
              currency_code: currencyCode,
            })}
          </span>
          {style === "default" && (
            <span className="font-semibold text-[var(--accent-sapphire)]">
              -{getPercentageDiff(originalPrice, currentPrice)}%
            </span>
          )}
        </div>
      )}
      <span
        data-testid="product-price"
        className={`tabular-nums font-medium ${
          hasReducedPrice ? "text-[var(--accent-sapphire)]" : "text-[var(--text-primary)]"
        }`}
      >
        {convertToLocale({
          amount: currentPrice,
          currency_code: currencyCode,
        })}
      </span>
    </div>
  )
}

export default LineItemPrice
