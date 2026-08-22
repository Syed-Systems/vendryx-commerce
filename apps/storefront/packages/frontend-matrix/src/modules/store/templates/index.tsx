import { Suspense } from "react"
import SkeletonProductGrid from "@modules/skeletons/templates/skeleton-product-grid"
import RefinementList from "@modules/store/components/refinement-list"
import { SortOptions } from "@modules/store/components/refinement-list/sort-products"
import type { ProductListRenderer } from "@modules/store/templates/paginated-products"
import styles from "./product-grid.module.css"

/**
 * PORT of The Matrix Hours `ProductGrid` section — its stylesheet, its centred
 * header, its chip bar above the grid rather than a sidebar beside it.
 *
 * The Medusa template put refinement in a 256px left column and the products in
 * a flex sibling. The source design does not have a sidebar at all: filters are
 * a horizontal rail directly above a full-bleed grid. That layout difference is
 * the whole reason this file is not a restyle of the old one.
 *
 * The GSAP `ScrollTrigger.batch` stagger that revealed cards is dropped — no
 * `gsap` dependency for an entrance.
 */
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
    <section className={styles.section} data-testid="category-container">
      <div className={styles.container}>
        <div className={styles.header}>
          <span className={styles.eyebrow}>The Collection</span>
          <h1 className={styles.heading} data-testid="store-page-title">
            All Watches
          </h1>
        </div>

        <RefinementList sortBy={sort} />

        <Suspense fallback={<SkeletonProductGrid />}>
          {productList({
            sortBy: sort,
            page: pageNumber,
            countryCode,
          })}
        </Suspense>
      </div>
    </section>
  )
}

export default StoreTemplate
