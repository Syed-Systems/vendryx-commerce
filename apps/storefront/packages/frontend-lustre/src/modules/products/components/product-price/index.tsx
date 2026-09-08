import { getProductPrice } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"

export type ProductPriceProps = {
  product: HttpTypes.StoreProduct
  variant?: HttpTypes.StoreProductVariant
}

/**
 * Lustre ProductPrice.
 * Bold tabular numerals displaying primary acquisition price and compare-at value.
 */
export default function ProductPrice({ product, variant }: ProductPriceProps) {
  const { cheapestPrice, variantPrice } = getProductPrice({
    product,
    variantId: variant?.id,
  })

  const selectedPrice = variant ? variantPrice : cheapestPrice

  if (!selectedPrice) {
    return (
      <div className="w-36 h-8 rounded-[8px] bg-[var(--bg-raised)] animate-pulse" />
    )
  }

  const isSale = selectedPrice.price_type === "sale"

  return (
    <div className="flex flex-col space-y-1 text-left">
      <div className="flex items-baseline gap-2">
        {!variant && (
          <span className="text-xs text-[var(--text-muted)] uppercase tracking-wider font-medium">
            From
          </span>
        )}
        <span
          data-testid="product-price"
          data-value={selectedPrice.calculated_price_number}
          className="text-2xl sm:text-3xl font-semibold tabular-nums text-[var(--text-primary)] tracking-tight"
        >
          {selectedPrice.calculated_price}
        </span>
      </div>

      {isSale && (
        <div className="flex items-center gap-2 text-xs text-[var(--text-muted)]">
          <span>Standard:</span>
          <span
            className="line-through tabular-nums"
            data-testid="original-product-price"
            data-value={selectedPrice.original_price_number}
          >
            {selectedPrice.original_price}
          </span>
          <span className="font-semibold text-[var(--accent-sapphire)]">
            -{selectedPrice.percentage_diff}%
          </span>
        </div>
      )}
    </div>
  )
}
