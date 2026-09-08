import { Suspense } from "react"
import SkeletonProductGrid from "@modules/skeletons/templates/skeleton-product-grid"
import RefinementList from "@modules/store/components/refinement-list"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import type { ProductListRenderer } from "@modules/store/templates/paginated-products"
import { HttpTypes } from "@medusajs/types"

export type CollectionTemplateProps = {
  productList: ProductListRenderer
  sortBy?: SortOptions
  collection: HttpTypes.StoreCollection
  page?: string
  countryCode: string
}

/**
 * Lustre CollectionTemplate.
 * Curated collection view with refined heading and product grid.
 * Conforms to contract slot 22.
 */
export default function CollectionTemplate({
  productList,
  sortBy,
  collection,
  page,
  countryCode,
}: CollectionTemplateProps) {
  const pageNumber = page ? parseInt(page) : 1
  const sort = sortBy || "created_at"

  return (
    <div className="w-full max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 py-8 md:py-12">
      <div className="pb-8 mb-8 border-b border-[var(--hairline)]">
        <span className="text-xs uppercase tracking-[0.2em] font-semibold text-[var(--accent-sapphire)] block mb-2">
          Collection
        </span>
        <h1 className="text-3xl sm:text-4xl md:text-5xl font-semibold tracking-tight text-[var(--text-primary)]">
          {collection.title}
        </h1>
      </div>

      <div className="w-full">
        <RefinementList sortBy={sort} />

        <div className="w-full">
          <Suspense
            fallback={
              <SkeletonProductGrid
                numberOfProducts={collection.products?.length}
              />
            }
          >
            {productList({
              sortBy: sort,
              page: pageNumber,
              collectionId: collection.id,
              countryCode,
            })}
          </Suspense>
        </div>
      </div>
    </div>
  )
}
