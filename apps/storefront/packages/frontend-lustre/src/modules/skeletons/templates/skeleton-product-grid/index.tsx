import React from "react"
import { repeat } from "@stheno/storefront-core"
import SkeletonProductPreview from "@modules/skeletons/components/skeleton-product-preview"

export type SkeletonProductGridProps = {
  numberOfProducts?: number
}

/**
 * Lustre SkeletonProductGrid.
 * Responsive product grid placeholder during initial SSR / query execution.
 */
const SkeletonProductGrid = ({
  numberOfProducts = 8,
}: SkeletonProductGridProps) => {
  return (
    <ul
      className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-x-5 gap-y-10 flex-1"
      data-testid="products-list-loader"
    >
      {repeat(numberOfProducts).map((index) => (
        <li key={index}>
          <SkeletonProductPreview />
        </li>
      ))}
    </ul>
  )
}

export default SkeletonProductGrid
