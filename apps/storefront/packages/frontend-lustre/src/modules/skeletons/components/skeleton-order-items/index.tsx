import React from "react"

const SkeletonOrderItems: React.FC = () => {
  return (
    <div className="space-y-4 py-8 border-y border-[var(--hairline)]">
      {[1, 2].map((i) => (
        <div key={i} className="flex gap-4 items-center">
          <div className="h-16 w-16 shrink-0 animate-pulse rounded-[var(--radius-sm)] bg-[var(--hairline)]" />
          <div className="space-y-2 flex-1">
            <div className="h-4 w-40 animate-pulse rounded bg-[var(--hairline)]" />
            <div className="h-3 w-20 animate-pulse rounded bg-[var(--hairline)]" />
          </div>
          <div className="h-4 w-16 animate-pulse rounded bg-[var(--hairline)]" />
        </div>
      ))}
    </div>
  )
}

export default SkeletonOrderItems
