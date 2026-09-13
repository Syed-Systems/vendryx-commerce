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
      <div
        data-testid="product-wrapper"
        className="flex flex-col rounded-2xl bg-[#F5F5F7] dark:bg-[#161617] p-3 sm:p-4 border border-zinc-200/60 dark:border-zinc-800/80 transition-all duration-300 ease-out group-hover:shadow-lg group-hover:shadow-zinc-900/5 group-hover:-translate-y-0.5"
      >
        {/* Image Container with Badges */}
        <div className="relative overflow-hidden rounded-xl bg-white dark:bg-zinc-900/60 aspect-square flex items-center justify-center">
          <Thumbnail
            thumbnail={product.thumbnail}
            images={product.images}
            size="full"
            isFeatured={isFeatured}
          />

          {/* Floating Apple-Style Badges */}
          <div className="absolute top-2.5 left-2.5 flex flex-col gap-1 pointer-events-none z-10">
            {isSale && (
              <span className="px-2.5 py-0.5 text-[10px] font-semibold uppercase tracking-wider rounded-full bg-rose-600 text-white shadow-xs">
                Sale
              </span>
            )}
            {isFeatured && !isSale && (
              <span className="px-2.5 py-0.5 text-[10px] font-semibold uppercase tracking-wider rounded-full bg-blue-600 text-white shadow-xs">
                Flagship
              </span>
            )}
          </div>

          {/* Quick View Pill on Desktop Hover */}
          <div className="absolute bottom-2.5 inset-x-2.5 opacity-0 group-hover:opacity-100 transition-all duration-200 transform translate-y-1.5 group-hover:translate-y-0 hidden sm:block z-10">
            <div className="w-full py-2 bg-white/90 dark:bg-zinc-900/90 backdrop-blur-md text-zinc-950 dark:text-white text-center text-xs font-semibold tracking-wide rounded-lg shadow-sm border border-black/5 dark:border-white/10">
              View Specifications
            </div>
          </div>
        </div>

        {/* Product Details (Apple Typography & Tabular Pricing) */}
        <div className="mt-3.5 flex flex-col space-y-1.5">
          <div className="flex items-baseline justify-between gap-2">
            <h3
              className="text-sm font-semibold text-zinc-950 dark:text-zinc-50 group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors line-clamp-1"
              data-testid="product-title"
            >
              {product.title}
            </h3>
            {cheapestPrice && <PreviewPrice price={cheapestPrice} />}
          </div>

          {/* Hardware category / Variant specs */}
          <div className="flex items-center justify-between text-[11px] text-zinc-500 dark:text-zinc-400 font-normal">
            <span>
              {product.collection?.title ||
                (product.categories && product.categories[0]?.name) ||
                "Pro Hardware"}
            </span>
            <span>
              {variantCount > 1 ? `${variantCount} configurations` : "Standard Model"}
            </span>
          </div>
        </div>
      </div>
    </LocalizedClientLink>
  )
}

