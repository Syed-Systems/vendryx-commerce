import React from "react"
import { HttpTypes } from "@medusajs/types"
import ProductPreview from "@modules/products/components/product-preview"
import { Pagination } from "@modules/store/components/pagination"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"

const PRODUCT_LIMIT = 12

export type ProductListRenderer = (args: {
  sortBy?: SortOptions
  page: number
  collectionId?: string
  categoryId?: string
  countryCode: string
}) => React.ReactNode

export type PaginatedProductsProps = {
  page: number
  region: HttpTypes.StoreRegion
  products: HttpTypes.StoreProduct[]
  count: number
}

/**
 * Lustre PaginatedProducts.
 * Asymmetric / balanced grid presentation of products.
 * Responsive: 2 cols mobile, 3 cols tablet, 4 cols desktop.
 */
export default async function PaginatedProducts({
  page,
  region,
  products,
  count,
}: PaginatedProductsProps) {
  const totalPages = Math.ceil(count / PRODUCT_LIMIT)

  if (!products || products.length === 0) {
    return (
      <div className="py-20 text-center">
        <p className="text-sm text-[var(--text-secondary)]">
          No items found in this section.
        </p>
      </div>
    )
  }

  return (
    <div className="w-full">
      <ul
        className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-x-5 gap-y-10"
        data-testid="products-list"
      >
        {products.map((p) => (
          <li key={p.id}>
            <ProductPreview product={p} region={region} />
          </li>
        ))}
      </ul>
      {totalPages > 1 && (
        <Pagination
          data-testid="product-pagination"
          page={page}
          totalPages={totalPages}
        />
      )}
    </div>
  )
}
