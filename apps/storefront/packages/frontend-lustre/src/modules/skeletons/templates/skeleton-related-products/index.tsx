import React from "react"
import { repeat } from "@stheno/storefront-core"
import SkeletonProductPreview from "@modules/skeletons/components/skeleton-product-preview"

const SkeletonRelatedProducts: React.FC = () => {
  return (
    <div className="w-full">
      <div className="mb-8 space-y-2">
        <div className="h-4 w-32 animate-pulse rounded-full bg-[var(--hairline)]" />
        <div className="h-7 w-64 animate-pulse rounded-[var(--radius-sm)] bg-[var(--hairline)]" />
      </div>
      <div className="grid grid-cols-2 gap-6 sm:grid-cols-3 lg:grid-cols-4">
        {repeat(4).map((index) => (
          <div key={index}>
            <SkeletonProductPreview />
          </div>
        ))}
      </div>
    </div>
  )
}

export default SkeletonRelatedProducts
