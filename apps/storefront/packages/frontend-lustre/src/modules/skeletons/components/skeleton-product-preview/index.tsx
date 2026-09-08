import React from "react"

/**
 * Lustre SkeletonProductPreview.
 * Shape-matched skeleton tile for loading products with 16px radius and low-contrast shimmer.
 */
const SkeletonProductPreview = () => {
  return (
    <div className="animate-pulse flex flex-col space-y-3">
      <div className="aspect-[4/5] w-full rounded-[16px] bg-[var(--bg-raised)] border border-[var(--hairline)]/50" />
      <div className="flex items-center justify-between gap-3">
        <div className="h-4 w-3/5 rounded-[8px] bg-[var(--bg-raised)]" />
        <div className="h-4 w-1/5 rounded-[8px] bg-[var(--bg-raised)]" />
      </div>
      <div className="h-3 w-2/5 rounded-[6px] bg-[var(--bg-raised)]/60" />
    </div>
  )
}

export default SkeletonProductPreview
