import { getProductPrice } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import Thumbnail from "../thumbnail"
import PreviewPrice from "./price"

export type ProductPreviewProps = {
  product: HttpTypes.StoreProduct
  isFeatured?: boolean
  region: HttpTypes.StoreRegion
}

/**
 * Lustre ProductPreview.
 * Refined product card with 16px surface radius, tabular price, and sourced category.
 */
export default async function ProductPreview({
  product,
  isFeatured,
  region,
}: ProductPreviewProps) {
  const { cheapestPrice } = getProductPrice({
    product,
  })

  const isSale = cheapestPrice?.price_type === "sale"
  const categoryLabel =
    product.collection?.title ||
    (product.categories && product.categories[0]?.name) ||
    ""

  return (
    <LocalizedClientLink
      href={`/products/${product.handle}`}
      className="group block text-left"
    >
      <div data-testid="product-wrapper" className="flex flex-col">
        <div className="relative">
          <Thumbnail
            thumbnail={product.thumbnail}
            images={product.images}
            size="full"
            isFeatured={isFeatured}
          />

          {isSale && (
            <div className="absolute top-3 left-3 pointer-events-none">
              <span className="px-2.5 py-1 rounded-full text-[10px] font-semibold uppercase tracking-wider bg-[var(--accent-sapphire)] text-white shadow-sm">
                Special Pricing
              </span>
            </div>
          )}
        </div>

        <div className="mt-3 flex flex-col space-y-1">
          <div className="flex items-baseline justify-between gap-2">
            <h3
              data-testid="product-title"
              className="text-sm font-medium text-[var(--text-primary)] group-hover:text-[var(--accent-sapphire)] transition-colors truncate"
            >
              {product.title}
            </h3>
            {cheapestPrice && <PreviewPrice price={cheapestPrice} />}
          </div>

          {categoryLabel && (
            <span className="text-xs text-[var(--text-muted)] truncate block">
              {categoryLabel}
            </span>
          )}
        </div>
      </div>
    </LocalizedClientLink>
  )
}
