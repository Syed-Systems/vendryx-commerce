import React from "react"
import SkeletonCartTotals from "../skeleton-cart-totals"

const SkeletonOrderInformation: React.FC = () => {
  return (
    <div className="space-y-6 pt-8">
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
        <div className="space-y-2">
          <div className="h-4 w-32 animate-pulse rounded bg-[var(--hairline)]" />
          <div className="h-3 w-48 animate-pulse rounded bg-[var(--hairline)]" />
          <div className="h-3 w-40 animate-pulse rounded bg-[var(--hairline)]" />
        </div>
        <div className="space-y-2">
          <div className="h-4 w-32 animate-pulse rounded bg-[var(--hairline)]" />
          <div className="h-3 w-48 animate-pulse rounded bg-[var(--hairline)]" />
          <div className="h-3 w-40 animate-pulse rounded bg-[var(--hairline)]" />
        </div>
      </div>

      <div className="pt-4 border-t border-[var(--hairline)]">
        <SkeletonCartTotals header />
      </div>
    </div>
  )
}

export default SkeletonOrderInformation
