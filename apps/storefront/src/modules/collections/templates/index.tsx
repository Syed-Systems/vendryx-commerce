import { Suspense } from "react"
import SkeletonProductGrid from "@modules/skeletons/templates/skeleton-product-grid"
import RefinementList from "@modules/store/components/refinement-list"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import PaginatedProducts from "@modules/store/templates/paginated-products"
import { HttpTypes } from "@medusajs/types"

export default function CollectionTemplate({
  sortBy,
  collection,
  page,
  countryCode,
}: {
  sortBy?: SortOptions
  collection: HttpTypes.StoreCollection
  page?: string
  countryCode: string
}) {
  const pageNumber = page ? parseInt(page) : 1
  const sort = sortBy || "created_at"

  return (
    <div className="w-full">
      {/* Header Banner */}
      <div className="bg-zinc-900 text-white py-12 sm:py-16 mb-8">
        <div className="content-container">
          <span className="text-xs uppercase tracking-[0.25em] text-zinc-400 font-semibold block mb-2">
            Curated Capsule
          </span>
          <h1
            className="text-3xl sm:text-4xl lg:text-5xl font-serif font-bold tracking-tight text-white"
          >
            {collection.title}
          </h1>
          <p className="text-sm text-zinc-300 font-light max-w-xl mt-3 leading-relaxed">
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
            <PaginatedProducts
              sortBy={sort}
              page={pageNumber}
              collectionId={collection.id}
              countryCode={countryCode}
            />
          </Suspense>
        </div>
      </div>
    </div>
  )
}
