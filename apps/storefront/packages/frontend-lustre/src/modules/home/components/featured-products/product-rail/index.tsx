import React from "react"
import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import ProductPreview from "@modules/products/components/product-preview"
import SectionReveal from "@modules/common/components/section-reveal"

export type ProductRailProps = {
  collection: HttpTypes.StoreCollection
  region: HttpTypes.StoreRegion
  products: HttpTypes.StoreProduct[]
}

/**
 * Lustre ProductRail component.
 * Horizontal capsule exhibition highlighting curations from specific collections.
 */
export default function ProductRail({
  collection,
  region,
  products,
}: ProductRailProps) {
  if (!products || products.length === 0) {
    return null
  }

  return (
    <SectionReveal>
      <section aria-label={`Featured: ${collection.title}`} className="content-container py-14 sm:py-20">
      <div className="flex flex-col sm:flex-row sm:items-end justify-between mb-8 pb-4 border-b border-[var(--hairline)] gap-4">
        <div>
          <span className="text-xs font-semibold uppercase tracking-[0.25em] text-[var(--accent-sapphire)] block mb-1">
            Collection Showcase
          </span>
          <h2 className="lustre-section-title text-[var(--text-primary)]">
            {collection.title}
          </h2>
        </div>

        <LocalizedClientLink
          href={`/collections/${collection.handle}`}
          className="text-xs font-semibold uppercase tracking-widest text-[var(--text-primary)] hover:text-[var(--accent-sapphire)] transition flex items-center gap-1.5"
        >
          <span>View Collection</span>
          <span>→</span>
        </LocalizedClientLink>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-6 sm:gap-8">
        {products.map((product) => (
          <ProductPreview key={product.id} product={product} region={region} isFeatured />
        ))}
      </div>
    </section>
    </SectionReveal>
  )
}
