import { listProductsWithSort } from "@lib/data/products"
import { getRegion } from "@lib/data/regions"
import { HttpTypes } from "@medusajs/types"
import PaginatedProducts from "@theme/store/templates/paginated-products"
import { SortOptions } from "@theme/store/components/refinement-list/sort-products"

type PaginatedProductsParams = {
  limit: number
  collection_id?: string[]
  category_id?: string[]
  id?: string[]
  order?: string
}

/**
 * Was the read half of theme-default's paginated-products. Paging, sorting and
 * region resolution are commerce logic, not presentation - ADR-013 decision 4.
 */
export default async function PaginatedProductsLoader({
  sortBy,
  page,
  collectionId,
  categoryId,
  productsIds,
  countryCode,
}: {
  sortBy?: SortOptions
  page: number
  collectionId?: string
  categoryId?: string
  productsIds?: string[]
  countryCode: string
}) {
  const queryParams: PaginatedProductsParams = {
    limit: 12,
  }

  if (collectionId) {
    queryParams["collection_id"] = [collectionId]
  }

  if (categoryId) {
    queryParams["category_id"] = [categoryId]
  }

  if (productsIds) {
    queryParams["id"] = productsIds
  }

  if (sortBy === "created_at") {
    queryParams["order"] = "created_at"
  }

  const region = await getRegion(countryCode)

  if (!region) {
    return null
  }

  const {
    response: { products, count },
  } = await listProductsWithSort({
    page,
    queryParams,
    sortBy,
    countryCode,
  })

  return (
    <PaginatedProducts
      page={page}
      region={region}
      products={products as HttpTypes.StoreProduct[]}
      count={count}
    />
  )
}
