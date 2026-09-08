import React from "react"
import { repeat } from "@stheno/storefront-core"
import SkeletonCartItem from "../../components/skeleton-cart-item"
import SkeletonCodeForm from "../../components/skeleton-code-form"
import SkeletonOrderSummary from "../../components/skeleton-order-summary"

/**
 * Lustre SkeletonCartPage (Slot 50).
 */
const SkeletonCartPage: React.FC = () => {
  return (
    <main className="min-h-screen bg-[var(--bg-primary)] py-12 sm:py-16">
      <div className="content-container">
        <div className="mb-10 space-y-2">
          <div className="h-4 w-32 animate-pulse rounded-full bg-[var(--hairline)]" />
          <div className="h-8 w-48 animate-pulse rounded bg-[var(--hairline)]" />
        </div>

        <div className="grid grid-cols-1 gap-12 lg:grid-cols-12 lg:gap-14 items-start">
          <div className="space-y-6 lg:col-span-8 rounded-[var(--radius-lg)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="border-b border-[var(--hairline)]">
                  <th className="py-3 pl-0 pr-4 w-20">
                    <div className="h-3 w-12 animate-pulse rounded bg-[var(--hairline)]" />
                  </th>
                  <th className="py-3 px-2">
                    <div className="h-3 w-20 animate-pulse rounded bg-[var(--hairline)]" />
                  </th>
                  <th className="py-3 px-3">
                    <div className="h-3 w-14 animate-pulse rounded bg-[var(--hairline)]" />
                  </th>
                  <th className="py-3 pl-4 pr-0 text-right">
                    <div className="ml-auto h-3 w-16 animate-pulse rounded bg-[var(--hairline)]" />
                  </th>
                </tr>
              </thead>
              <tbody>
                {repeat(3).map((index) => (
                  <SkeletonCartItem key={index} />
                ))}
              </tbody>
            </table>
          </div>

          <div className="space-y-6 lg:col-span-4">
            <SkeletonOrderSummary />
            <div className="rounded-[var(--radius-lg)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6">
              <SkeletonCodeForm />
            </div>
          </div>
        </div>
      </div>
    </main>
  )
}

export default SkeletonCartPage
