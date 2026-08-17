import { listProducts } from "@lib/data/products"
import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import ProductPreview from "@modules/products/components/product-preview"

export default async function ProductRail({
  collection,
  region,
}: {
  collection: HttpTypes.StoreCollection
  region: HttpTypes.StoreRegion
}) {
  const countryCode = region?.countries?.[0]?.iso_2 || "us"

  const {
    response: { products: pricedProducts },
  } = await listProducts({
    countryCode,
    regionId: region.id,
    queryParams: {
      collection_id: collection.id,
      fields: "*variants.calculated_price",
    },
  }).catch(() => ({
    response: { products: [], count: 0 },
    nextPage: null,
  }))

  if (!pricedProducts || pricedProducts.length === 0) {
    return null
  }

  return (
    <div className="content-container py-12 sm:py-20">
      {/* Section Header */}
      <div className="flex flex-col sm:flex-row sm:items-end justify-between mb-8 pb-4 border-b border-zinc-200 gap-4">
        <div>
          <span className="text-xs uppercase tracking-[0.2em] text-zinc-500 font-semibold block mb-1">
            Featured Capsule
          </span>
          <h2 className="text-2xl sm:text-3xl font-serif font-bold text-zinc-950">
            {collection.title}
          </h2>
        </div>
        <LocalizedClientLink
          href={`/collections/${collection.handle}`}
          className="text-xs font-semibold uppercase tracking-widest text-zinc-900 hover:text-zinc-600 transition-colors flex items-center gap-1.5 group self-start sm:self-auto"
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
