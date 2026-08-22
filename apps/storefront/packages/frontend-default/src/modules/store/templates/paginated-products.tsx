import React from "react"
import { HttpTypes } from "@medusajs/types"
import ProductPreview from "@modules/products/components/product-preview"
import { Pagination } from "@modules/store/components/pagination"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"

const PRODUCT_LIMIT = 12

/**
 * Core supplies this. A template knows its own sort and page defaults, so it
 * asks core for the matching product list rather than being handed one.
 */
export type ProductListRenderer = (args: {
  sortBy?: SortOptions
  page: number
  collectionId?: string
  categoryId?: string
  countryCode: string
}) => React.ReactNode

/**
 * Presentation only. Core resolves the region and runs the paged, sorted query,
 * per ADR-013 decision 4.
 */
export default async function PaginatedProducts({
  page,
  region,
  products,
  count,
}: {
  page: number
  region: HttpTypes.StoreRegion
  products: HttpTypes.StoreProduct[]
  count: number
}) {
  const totalPages = Math.ceil(count / PRODUCT_LIMIT)

  return (
    <>
      <ul
        className="grid grid-cols-2 w-full small:grid-cols-3 medium:grid-cols-4 gap-x-6 gap-y-8"
        data-testid="products-list"
      >
        {products.map((p) => {
          return (
            <li key={p.id}>
              <ProductPreview product={p} region={region} />
            </li>
          )
        })}
      </ul>
      {totalPages > 1 && (
        <Pagination
          data-testid="product-pagination"
          page={page}
          totalPages={totalPages}
        />
      )}
    </>
  )
}
