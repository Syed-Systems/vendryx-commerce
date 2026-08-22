import { listProducts } from "@lib/data/products"
import { HttpTypes } from "@medusajs/types"
import ProductActions from "@frontend/products/components/product-actions"

/**
 * Was frontend-default's product-actions-wrapper. It was only ever a read wrapped
 * around a themed component, which is core's job under ADR-013 decision 4.
 *
 * Fetches real time pricing for a product and renders the product actions.
 */
export default async function ProductActionsLoader({
  id,
  region,
}: {
  id: string
  region: HttpTypes.StoreRegion
}) {
  const product = await listProducts({
    queryParams: { id: [id] },
    regionId: region.id,
  }).then(({ response }) => response.products[0])

  if (!product) {
    return null
  }

  return <ProductActions product={product} region={region} />
}
