import React from "react"

const SkeletonButton: React.FC = () => {
  return (
    <div className="h-12 w-full animate-pulse rounded-full bg-[var(--hairline)]" />
  )
}

export default SkeletonButton
