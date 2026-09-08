import React from "react"
import SkeletonOrderConfirmedHeader from "../../components/skeleton-order-confirmed-header"
import SkeletonOrderInformation from "../../components/skeleton-order-information"
import SkeletonOrderItems from "../../components/skeleton-order-items"

/**
 * Lustre SkeletonOrderConfirmed (Slot 51).
 */
const SkeletonOrderConfirmed: React.FC = () => {
  return (
    <main className="min-h-screen bg-[var(--bg-primary)] py-12 sm:py-20">
      <div className="content-container mx-auto max-w-3xl rounded-[var(--radius-lg)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-8 sm:p-12 shadow-sm">
        <SkeletonOrderConfirmedHeader />
        <SkeletonOrderItems />
        <SkeletonOrderInformation />
      </div>
    </main>
  )
}

export default SkeletonOrderConfirmed
