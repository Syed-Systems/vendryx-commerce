"use client"

import { usePathname, useRouter, useSearchParams } from "next/navigation"
import { useCallback } from "react"
import SortProducts, { SortOptions } from "./sort-products"

export type RefinementListProps = {
  sortBy: SortOptions
  search?: boolean
  "data-testid"?: string
}

/**
 * Lustre RefinementList.
 * Controls catalog sorting through URL query parameters.
 */
const RefinementList = ({ sortBy, "data-testid": dataTestId }: RefinementListProps) => {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()

  const createQueryString = useCallback(
    (name: string, value: string) => {
      const params = new URLSearchParams(searchParams)
      params.set(name, value)
      return params.toString()
    },
    [searchParams]
  )

  const setQueryParams = (name: string, value: SortOptions) => {
    const query = createQueryString("sortBy", value)
    router.push(`${pathname}?${query}`)
  }

  return (
    <div className="flex items-center justify-between py-3 mb-6 w-full border-b border-[var(--hairline)]">
      <span className="text-xs text-[var(--text-muted)] font-medium uppercase tracking-wider">
        Refine
      </span>
      <SortProducts
        sortBy={sortBy}
        setQueryParams={setQueryParams}
        data-testid={dataTestId}
      />
    </div>
  )
}

export default RefinementList
