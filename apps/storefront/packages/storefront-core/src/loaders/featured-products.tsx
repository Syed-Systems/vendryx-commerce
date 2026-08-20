import { listProducts } from "@lib/data/products"
import { HttpTypes } from "@medusajs/types"
import ProductRail from "@theme/home/components/featured-products/product-rail"

/**
 * Was theme-default's featured-products. The per-collection read belongs in
 * core (ADR-013 decision 4); the rail markup is still the theme's.
 */
export default async function FeaturedProductsLoader({
  collections,
  region,
}: {
  collections: HttpTypes.StoreCollection[]
  region: HttpTypes.StoreRegion
}) {
  const countryCode = region?.countries?.[0]?.iso_2 || "us"

  return Promise.all(
    collections.map(async (collection) => {
      const {
        response: { products },
      } = await listProducts({
        countryCode,
        regionId: region.id,
        queryParams: {
          collection_id: collection.id,
          fields: "*variants.calculated_price",
        },
      }).catch(() => ({ response: { products: [], count: 0 } }))

      return (
        <li key={collection.id}>
          <ProductRail
            collection={collection}
            region={region}
            products={products as HttpTypes.StoreProduct[]}
          />
        </li>
      )
    })
  )
}
