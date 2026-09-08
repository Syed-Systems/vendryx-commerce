import React from "react"

export type SkeletonCartTotalsProps = {
  header?: boolean
}

const SkeletonCartTotals: React.FC<SkeletonCartTotalsProps> = ({ header = true }) => {
  return (
    <div className="space-y-4">
      {header && <div className="h-4 w-28 animate-pulse rounded bg-[var(--hairline)]" />}
      <div className="flex justify-between">
        <div className="h-3 w-20 animate-pulse rounded bg-[var(--hairline)]" />
        <div className="h-3 w-16 animate-pulse rounded bg-[var(--hairline)]" />
      </div>
      <div className="flex justify-between">
        <div className="h-3 w-24 animate-pulse rounded bg-[var(--hairline)]" />
        <div className="h-3 w-14 animate-pulse rounded bg-[var(--hairline)]" />
      </div>
      <div className="flex justify-between">
        <div className="h-3 w-16 animate-pulse rounded bg-[var(--hairline)]" />
        <div className="h-3 w-12 animate-pulse rounded bg-[var(--hairline)]" />
      </div>
      <div className="border-t border-[var(--hairline)] pt-3 flex justify-between">
        <div className="h-4 w-20 animate-pulse rounded bg-[var(--hairline)]" />
        <div className="h-4 w-24 animate-pulse rounded bg-[var(--hairline)]" />
      </div>
    </div>
  )
}

export default SkeletonCartTotals
