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
      {/* Section Header. Was `zinc-*` while nav, footer and the rest of the home
          page used `stone-*` — two neutral families on one screen. */}
      <div className="flex flex-col sm:flex-row sm:items-end justify-between mb-8 pb-4 border-b border-[color:var(--brand-hairline)] gap-4">
        <div>
          <span className="label-eyebrow text-[color:var(--ink-faint)] block mb-1">
            Featured Capsule
          </span>
          <h2 className="display-section font-heading font-bold text-[color:var(--ink)]">
            {collection.title}
          </h2>
        </div>
        <LocalizedClientLink
          href={`/collections/${collection.handle}`}
          className="tap label-eyebrow text-[color:var(--ink)] hover:text-[color:var(--ink-muted)] transition-colors flex items-center gap-1.5 group self-start sm:self-auto"
        >
          <span>Explore Capsule</span>
          <span className="group-hover:translate-x-1 transition-transform">→</span>
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
