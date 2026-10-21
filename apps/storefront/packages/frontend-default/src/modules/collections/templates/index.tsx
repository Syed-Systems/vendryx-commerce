import { Suspense } from "react"
import SkeletonProductGrid from "@modules/skeletons/templates/skeleton-product-grid"
import RefinementList from "@modules/store/components/refinement-list"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import type { ProductListRenderer } from "@modules/store/templates/paginated-products"
import { HttpTypes } from "@medusajs/types"

export default function CollectionTemplate({
  productList,
  sortBy,
  collection,
  page,
  countryCode,
}: {
  sortBy?: SortOptions
  collection: HttpTypes.StoreCollection
  page?: string
  countryCode: string
  productList: ProductListRenderer
}) {
  const pageNumber = page ? parseInt(page) : 1
  const sort = sortBy || "created_at"

  return (
    <div className="w-full">
      {/* Header Banner */}
      <div className="bg-[color:var(--band-dark)] text-[color:var(--on-dark)] py-12 sm:py-16 mb-8">
        <div className="content-container">
          <span className="text-xs uppercase tracking-[0.25em] text-[color:var(--ink-faint)] font-semibold block mb-2">
            Curated Capsule
          </span>
          <h1
            className="text-3xl sm:text-4xl lg:text-5xl font-heading font-bold tracking-tight text-[color:var(--on-dark)]"
          >
            {collection.title}
          </h1>
          <p className="text-sm text-[color:var(--on-dark-muted)] font-light max-w-xl mt-3 leading-relaxed">
            Meticulously edited silhouettes designed to pair harmoniously across seasons.
          </p>
        </div>
      </div>

      {/* Body */}
      <div className="flex flex-col small:flex-row small:items-start py-4 content-container gap-8">
        <div className="w-full small:w-64 shrink-0">
          <RefinementList sortBy={sort} />
        </div>
        <div className="w-full flex-1">
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
