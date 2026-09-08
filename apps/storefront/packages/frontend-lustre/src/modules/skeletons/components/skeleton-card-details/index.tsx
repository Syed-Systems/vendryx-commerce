import React from "react"

/**
 * Lustre SkeletonCardDetails (Slot 49).
 */
const SkeletonCardDetails: React.FC = () => {
  return (
    <div className="space-y-2 my-4">
      <div className="h-3 w-1/4 animate-pulse rounded bg-[var(--hairline)]" />
      <div className="h-11 w-full animate-pulse rounded-[var(--radius-sm)] border border-[var(--hairline)] bg-[var(--bg-surface)]" />
    </div>
  )
}

export default SkeletonCardDetails
