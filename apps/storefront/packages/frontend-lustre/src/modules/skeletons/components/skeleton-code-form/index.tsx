import React from "react"

const SkeletonCodeForm: React.FC = () => {
  return (
    <div className="space-y-3">
      <div className="h-4 w-28 animate-pulse rounded bg-[var(--hairline)]" />
      <div className="flex gap-2">
        <div className="h-10 flex-1 animate-pulse rounded-full bg-[var(--hairline)]" />
        <div className="h-10 w-20 animate-pulse rounded-full bg-[var(--hairline)]" />
      </div>
    </div>
  )
}

export default SkeletonCodeForm
