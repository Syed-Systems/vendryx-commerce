import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import ProductPreview from "@modules/products/components/product-preview"

/**
 * Presentation only. Core resolves the collection's products, per ADR-013
 * decision 4.
 */
export default async function ProductRail({
  collection,
  region,
  products: pricedProducts,
}: {
  collection: HttpTypes.StoreCollection
  region: HttpTypes.StoreRegion
  products: HttpTypes.StoreProduct[]
}) {
  return (
    <div className="content-container py-12 sm:py-20">
      {/* Section Header */}
      <div className="flex flex-col sm:flex-row sm:items-end justify-between mb-8 pb-4 border-b border-white/10 gap-4">
        <div>
          <span className="text-xs uppercase tracking-[0.2em] text-[#a0a0a0] font-semibold block mb-1">
            Featured Capsule
          </span>
          <h2 className="text-2xl sm:text-3xl font-[family-name:var(--font-plus-jakarta)] font-bold text-[#f5f5f0]">
            {collection.title}
          </h2>
        </div>
        <LocalizedClientLink
          href={`/collections/${collection.handle}`}
          className="text-xs font-semibold uppercase tracking-widest text-[#f5f5f0] hover:text-[#c9a96e] transition-colors flex items-center gap-1.5 group self-start sm:self-auto"
        >
          <span>Explore Capsule</span>
          <span className="group-hover:translate-x-1 transition-transform">
            →
          </span>
        </LocalizedClientLink>
      </div>

      {/* Grid */}
      <ul className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-x-4 sm:gap-x-6 gap-y-8 sm:gap-y-12">
        {pricedProducts.map((product) => (
          <li key={product.id}>
            <ProductPreview product={product} region={region} isFeatured />
          </li>
        ))}
      </ul>
    </div>
  )
}
