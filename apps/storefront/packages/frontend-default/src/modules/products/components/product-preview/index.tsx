import { getProductPrice } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import Thumbnail from "../thumbnail"
import PreviewPrice from "./price"

export default async function ProductPreview({
  product,
  isFeatured,
  region,
}: {
  product: HttpTypes.StoreProduct
  isFeatured?: boolean
  region: HttpTypes.StoreRegion
}) {
  const { cheapestPrice } = getProductPrice({
    product,
  })

  const isSale = cheapestPrice?.price_type === "sale"
  const variantCount = product.variants?.length || 0

  return (
    <LocalizedClientLink href={`/products/${product.handle}`} className="group block">
      <div data-testid="product-wrapper" className="flex flex-col">
        {/* Image Container with Badges */}
        <div className="relative">
          <Thumbnail
            thumbnail={product.thumbnail}
            images={product.images}
            size="full"
            isFeatured={isFeatured}
          />

          {/* Floating Badges */}
          <div className="absolute top-3 left-3 flex flex-col gap-1 pointer-events-none">
            {isSale && (
              <span className="badge-tag bg-rose-600 text-white shadow-sm">
                Sale
              </span>
            )}
            {isFeatured && !isSale && (
              <span className="badge-tag bg-zinc-950/80 text-white shadow-sm">
                Atelier Edit
              </span>
            )}
          </div>

          {/* Quick View Pill on Hover */}
          <div className="absolute bottom-3 inset-x-3 opacity-0 group-hover:opacity-100 transition-all duration-200 transform translate-y-1 group-hover:translate-y-0 hidden sm:block">
            <div className="w-full py-2 bg-white/95 backdrop-blur-md text-zinc-950 text-center text-xs font-semibold uppercase tracking-wider rounded-xl shadow-md">
              View Garment
            </div>
          </div>
        </div>

        {/* Product Details */}
        <div className="mt-3.5 flex flex-col space-y-1">
          <div className="flex items-baseline justify-between gap-2">
            <h3
              className="text-sm font-medium text-zinc-900 group-hover:text-zinc-600 transition-colors line-clamp-1"
              data-testid="product-title"
            >
              {product.title}
            </h3>
            {cheapestPrice && <PreviewPrice price={cheapestPrice} />}
          </div>

          {/* Subtitle / Department / Variant indicator */}
          <div className="flex items-center justify-between text-[11px] text-zinc-400 font-light">
            <span>
              {product.collection?.title || (product.categories && product.categories[0]?.name) || "Ready to Wear"}
            </span>
            {variantCount > 1 && (
              <span>{variantCount} options</span>
            )}
          </div>
        </div>
      </div>
    </LocalizedClientLink>
  )
}
