import { notFound } from "next/navigation"
import { Suspense } from "react"
import InteractiveLink from "@modules/common/components/interactive-link"
import SkeletonProductGrid from "@modules/skeletons/templates/skeleton-product-grid"
import RefinementList from "@modules/store/components/refinement-list"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import type { ProductListRenderer } from "@modules/store/templates/paginated-products"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { HttpTypes } from "@medusajs/types"

export default function CategoryTemplate({
  productList,
  category,
  sortBy,
  page,
  countryCode,
}: {
  category: HttpTypes.StoreProductCategory
  sortBy?: SortOptions
  page?: string
  countryCode: string
  productList: ProductListRenderer
}) {
  const pageNumber = page ? parseInt(page) : 1
  const sort = sortBy || "created_at"

  if (!category || !countryCode) notFound()

  const parents = [] as HttpTypes.StoreProductCategory[]

  const getParents = (category: HttpTypes.StoreProductCategory) => {
    if (category.parent_category) {
      parents.push(category.parent_category)
      getParents(category.parent_category)
    }
  }

  getParents(category)

  return (
    <div className="w-full">
      {/* Header Banner */}
      <div className="bg-[color:var(--band-dark)] text-[color:var(--on-dark)] py-12 sm:py-16 mb-8">
        <div className="content-container">
          <div className="flex items-center gap-2 text-xs uppercase tracking-widest text-[color:var(--ink-faint)] mb-2 font-medium">
            <LocalizedClientLink href="/store" className="hover:text-[color:var(--on-dark)] transition-colors">
              Store
            </LocalizedClientLink>
            {parents.map((parent) => (
              <span key={parent.id} className="flex items-center gap-2">
                <span>/</span>
                <LocalizedClientLink
                  className="hover:text-[color:var(--on-dark)] transition-colors"
                  href={`/categories/${parent.handle}`}
                >
                  {parent.name}
                </LocalizedClientLink>
              </span>
            ))}
          </div>

          <h1
            className="text-3xl sm:text-4xl lg:text-5xl font-heading font-bold tracking-tight text-[color:var(--on-dark)]"
            data-testid="category-page-title"
          >
            {category.name}
          </h1>

          {category.description && (
            <p className="text-sm text-[color:var(--on-dark-muted)] font-light max-w-xl mt-3 leading-relaxed">
              {category.description}
            </p>
          )}

          {category.category_children && category.category_children.length > 0 && (
            <div className="flex flex-wrap gap-2 pt-6">
              {category.category_children.map((c) => (
                <LocalizedClientLink
                  key={c.id}
                  href={`/categories/${c.handle}`}
                  className="px-4 py-1.5 rounded-[var(--brand-radius-button)] bg-[color:var(--brand-surface)]/10 hover:bg-[color:var(--brand-surface)]/20 text-xs font-medium text-[color:var(--on-dark)] transition-colors"
                >
                  {c.name}
                </LocalizedClientLink>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Body */}
      <div
        className="flex flex-col small:flex-row small:items-start py-4 content-container gap-8"
        data-testid="category-container"
      >
        <div className="w-full small:w-64 shrink-0">
          <RefinementList sortBy={sort} data-testid="sort-by-container" />
        </div>
        <div className="w-full flex-1">
          <Suspense
            fallback={
              <SkeletonProductGrid
                numberOfProducts={category.products?.length ?? 8}
              />
            }
          >
            {productList({
              sortBy: sort,
              page: pageNumber,
              categoryId: category.id,
              countryCode,
            })}
          </Suspense>
        </div>
      </div>
    </div>
  )
}
