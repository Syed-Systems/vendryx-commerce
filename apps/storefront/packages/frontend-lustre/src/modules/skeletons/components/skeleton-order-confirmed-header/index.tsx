import React from "react"

const SkeletonOrderConfirmedHeader: React.FC = () => {
  return (
    <div className="space-y-3 pb-8 text-center sm:text-left">
      <div className="h-4 w-32 animate-pulse rounded-full bg-[var(--hairline)] mx-auto sm:mx-0" />
      <div className="h-8 w-64 animate-pulse rounded bg-[var(--hairline)] mx-auto sm:mx-0" />
      <div className="flex gap-4 justify-center sm:justify-start pt-2">
        <div className="h-4 w-28 animate-pulse rounded bg-[var(--hairline)]" />
        <div className="h-4 w-20 animate-pulse rounded bg-[var(--hairline)]" />
      </div>
    </div>
  )
}

export default SkeletonOrderConfirmedHeader
