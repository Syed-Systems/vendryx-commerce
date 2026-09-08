import React from "react"
import SkeletonButton from "../skeleton-button"
import SkeletonCartTotals from "../skeleton-cart-totals"

const SkeletonOrderSummary: React.FC = () => {
  return (
    <div className="rounded-[var(--radius-lg)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 space-y-6">
      <SkeletonCartTotals header={false} />
      <SkeletonButton />
    </div>
  )
}

export default SkeletonOrderSummary
