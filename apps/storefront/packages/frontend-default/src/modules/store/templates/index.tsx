import { Suspense } from "react"
import SkeletonProductGrid from "@modules/skeletons/templates/skeleton-product-grid"
import RefinementList from "@modules/store/components/refinement-list"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import type { ProductListRenderer } from "@modules/store/templates/paginated-products"

const StoreTemplate = ({
  productList,
  sortBy,
  page,
  countryCode,
}: {
  sortBy?: SortOptions
  page?: string
  countryCode: string
  productList: ProductListRenderer
}) => {
  const pageNumber = page ? parseInt(page) : 1
  const sort = sortBy || "created_at"

  return (
    <div className="w-full">
      {/* Header Banner */}
      <div className="bg-[color:var(--band-dark)] text-[color:var(--on-dark)] py-12 sm:py-16 mb-8">
        <div className="content-container">
          <span className="text-xs uppercase tracking-[0.25em] text-[color:var(--ink-faint)] font-semibold block mb-2">
            The Complete Atelier Collection
          </span>
          <h1
            className="text-3xl sm:text-4xl lg:text-5xl font-heading font-bold tracking-tight text-[color:var(--on-dark)]"
            data-testid="store-page-title"
          >
            All Apparel & Essentials
          </h1>
          <p className="text-sm text-[color:var(--on-dark-muted)] font-light max-w-xl mt-3 leading-relaxed">
            Discover modern tailoring, organic textiles, and timeless silhouettes engineered for enduring elegance.
          </p>
        </div>
      </div>

      {/* Catalog Body */}
      <div
        className="flex flex-col small:flex-row small:items-start py-4 content-container gap-8"
        data-testid="category-container"
      >
        <div className="w-full small:w-64 shrink-0">
          <RefinementList sortBy={sort} />
        </div>

        <div className="w-full flex-1">
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
