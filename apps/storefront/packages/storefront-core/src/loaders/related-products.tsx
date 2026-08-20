import { listProducts } from "@lib/data/products"
import { getRegion } from "@lib/data/regions"
import { HttpTypes } from "@medusajs/types"
import RelatedProducts from "@theme/products/components/related-products"

/**
 * Was the read half of theme-default's related-products. The selection rules
 * are commerce logic and belong in core (ADR-013 decision 4); the grid markup
 * is still the theme's.
 */
export default async function RelatedProductsLoader({
  product,
  countryCode,
}: {
  product: HttpTypes.StoreProduct
  countryCode: string
}) {
  const region = await getRegion(countryCode)

  if (!region) {
    return null
  }

  // edit this function to define your related products logic
  const queryParams: HttpTypes.StoreProductListParams = {}
  if (region?.id) {
    queryParams.region_id = region.id
  }
  if (product.collection_id) {
    queryParams.collection_id = [product.collection_id]
  }
  if (product.tags) {
    queryParams.tag_id = product.tags
      .map((t) => t.id)
      .filter(Boolean) as string[]
  }
  queryParams.is_giftcard = false

  const products = await listProducts({
    queryParams,
    countryCode,
  }).then(({ response }) =>
    response.products.filter(
      (responseProduct) => responseProduct.id !== product.id
    )
  )

  return (
    <RelatedProducts
      region={region}
      products={products as HttpTypes.StoreProduct[]}
    />
  )
}
