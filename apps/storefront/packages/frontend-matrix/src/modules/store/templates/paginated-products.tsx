import React from "react"
import { HttpTypes } from "@medusajs/types"
import ProductPreview from "@modules/products/components/product-preview"
import { Pagination } from "@modules/store/components/pagination"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import styles from "./product-grid.module.css"

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
/**
 * The grid itself is the source design's: `product-grid.module.css` owns the
 * column counts and gutters, so the breakpoints are The Matrix Hours', not the
 * Medusa preset's `small:`/`medium:` scale — FRONTEND_CONTRACT_INPUTS §6.4 asked
 * who owns breakpoints, and for this theme the answer is the theme.
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
      <ul className={styles.grid} data-testid="products-list">
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
