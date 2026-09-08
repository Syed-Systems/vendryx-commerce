import React from "react"

const SkeletonCartItem: React.FC = () => {
  return (
    <tr className="border-b border-[var(--hairline)]">
      <td className="py-4 pl-0 pr-4 w-20">
        <div className="h-16 w-16 animate-pulse rounded-[var(--radius-sm)] bg-[var(--hairline)]" />
      </td>
      <td className="py-4 px-2 text-left">
        <div className="space-y-2">
          <div className="h-4 w-40 animate-pulse rounded bg-[var(--hairline)]" />
          <div className="h-3 w-24 animate-pulse rounded bg-[var(--hairline)]" />
        </div>
      </td>
      <td className="py-4 px-3">
        <div className="h-9 w-14 animate-pulse rounded-full bg-[var(--hairline)]" />
      </td>
      <td className="py-4 pl-4 pr-0 text-right">
        <div className="ml-auto h-4 w-16 animate-pulse rounded bg-[var(--hairline)]" />
      </td>
    </tr>
  )
}

export default SkeletonCartItem
