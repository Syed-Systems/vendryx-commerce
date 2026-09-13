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
      {/* Header Banner - Apple HIG */}
      <div className="bg-zinc-950 text-white py-12 sm:py-16 mb-8 border-b border-zinc-850">
        <div className="content-container">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-zinc-900 border border-zinc-800 text-[11px] font-medium tracking-wide text-zinc-400 mb-3">
            <span className="w-1.5 h-1.5 rounded-full bg-indigo-500" />
            Hardware Ecosystem
          </div>
          <h1
            className="text-3xl sm:text-4xl lg:text-5xl font-semibold tracking-tight text-white"
          >
            {collection.title}
          </h1>
          <p className="text-sm text-zinc-400 font-normal max-w-xl mt-3 leading-relaxed">
            Meticulously engineered devices and precision peripherals designed to operate together in complete harmony.
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
