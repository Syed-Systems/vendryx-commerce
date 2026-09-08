import { Suspense } from "react"
import SkeletonProductGrid from "@modules/skeletons/templates/skeleton-product-grid"
import RefinementList from "@modules/store/components/refinement-list"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import type { ProductListRenderer } from "@modules/store/templates/paginated-products"

export type StoreTemplateProps = {
  sortBy?: SortOptions
  page?: string
  countryCode: string
  productList: ProductListRenderer
}

/**
 * Lustre StoreTemplate.
 * Editorial catalog presentation with refined header, sorting, and responsive product grid.
 * Conforms to contract slot 52.
 */
const StoreTemplate = ({
  productList,
  sortBy,
  page,
  countryCode,
}: StoreTemplateProps) => {
  const pageNumber = page ? parseInt(page) : 1
  const sort = sortBy || "created_at"

  return (
    <div className="w-full max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 py-8 md:py-12">
      {/* Editorial Header */}
      <div className="pb-8 mb-8 border-b border-[var(--hairline)]">
        <span className="text-xs uppercase tracking-[0.2em] font-semibold text-[var(--accent-sapphire)] block mb-2">
          Catalog
        </span>
        <h1
          data-testid="store-page-title"
          className="text-3xl sm:text-4xl md:text-5xl font-semibold tracking-tight text-[var(--text-primary)]"
        >
          Watches &amp; Objects
        </h1>
        <p className="text-sm text-[var(--text-secondary)] max-w-2xl mt-2 leading-relaxed">
          Browse the products currently available from this store.
        </p>
      </div>

      {/* Catalog Container */}
      <div data-testid="category-container" className="w-full">
        <RefinementList sortBy={sort} />

        <div className="w-full">
          <Suspense fallback={<SkeletonProductGrid />}>
            {productList({
              sortBy: sort,
              page: pageNumber,
              countryCode,
            })}
          </Suspense>
        </div>
      </div>
    </div>
  )
}

export default StoreTemplate
