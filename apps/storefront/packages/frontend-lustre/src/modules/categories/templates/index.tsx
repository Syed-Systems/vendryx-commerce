import { notFound } from "next/navigation"
import { Suspense } from "react"
import SkeletonProductGrid from "@modules/skeletons/templates/skeleton-product-grid"
import RefinementList from "@modules/store/components/refinement-list"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import type { ProductListRenderer } from "@modules/store/templates/paginated-products"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { HttpTypes } from "@medusajs/types"

export type CategoryTemplateProps = {
  category: HttpTypes.StoreProductCategory
  sortBy?: SortOptions
  page?: string
  countryCode: string
  productList: ProductListRenderer
}

/**
 * Lustre CategoryTemplate.
 * Editorial category page displaying parent breadcrumbs, description, child pills, and products.
 * Conforms to contract slot 21.
 */
export default function CategoryTemplate({
  productList,
  category,
  sortBy,
  page,
  countryCode,
}: CategoryTemplateProps) {
  const pageNumber = page ? parseInt(page) : 1
  const sort = sortBy || "created_at"

  if (!category || !countryCode) {
    notFound()
  }

  const parents = [] as HttpTypes.StoreProductCategory[]
  const getParents = (cat: HttpTypes.StoreProductCategory) => {
    if (cat.parent_category) {
      parents.push(cat.parent_category)
      getParents(cat.parent_category)
    }
  }
  getParents(category)

  return (
    <div className="w-full max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 py-8 md:py-12">
      {/* Editorial Header */}
      <div className="pb-8 mb-8 border-b border-[var(--hairline)]">
        {/* Breadcrumb path */}
        <div className="flex items-center gap-2 text-xs text-[var(--text-muted)] font-medium mb-3">
          <LocalizedClientLink
            href="/store"
            className="hover:text-[var(--text-primary)] transition-colors"
          >
            Catalog
          </LocalizedClientLink>
          {parents.map((parent) => (
            <span key={parent.id} className="flex items-center gap-2">
              <span>/</span>
              <LocalizedClientLink
                href={`/categories/${parent.handle}`}
                className="hover:text-[var(--text-primary)] transition-colors"
              >
                {parent.name}
              </LocalizedClientLink>
            </span>
          ))}
          <span>/</span>
          <span className="text-[var(--text-primary)]">{category.name}</span>
        </div>

        <h1
          data-testid="category-page-title"
          className="text-3xl sm:text-4xl md:text-5xl font-semibold tracking-tight text-[var(--text-primary)]"
        >
          {category.name}
        </h1>

        {category.description && (
          <p className="text-sm text-[var(--text-secondary)] max-w-2xl mt-2 leading-relaxed">
            {category.description}
          </p>
        )}

        {category.category_children && category.category_children.length > 0 && (
          <div className="flex flex-wrap gap-2 mt-4 pt-2">
            {category.category_children.map((child) => (
              <LocalizedClientLink
                key={child.id}
                href={`/categories/${child.handle}`}
                className="px-3.5 py-1.5 rounded-full text-xs font-medium bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--hairline)] hover:border-[var(--accent-sapphire)] transition-colors"
              >
                {child.name}
              </LocalizedClientLink>
            ))}
          </div>
        )}
      </div>

      {/* Product List */}
      <div data-testid="category-container" className="w-full">
        <RefinementList sortBy={sort} data-testid="sort-by-container" />

        <div className="w-full">
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
