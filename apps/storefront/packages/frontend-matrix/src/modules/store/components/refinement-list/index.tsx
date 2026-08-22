"use client"

import { usePathname, useRouter, useSearchParams } from "next/navigation"
import { useCallback } from "react"
import SortProducts, { SortOptions } from "./sort-products"

type RefinementListProps = {
  sortBy: SortOptions
  search?: boolean
  "data-testid"?: string
}

/**
 * The Matrix bar is a single horizontal chip row above the grid, not a sidebar
 * column — so this is a plain wrapper. Core owns the query params; the chips
 * only write to them.
 */
const RefinementList = ({
  sortBy,
  "data-testid": dataTestId,
}: RefinementListProps) => {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()

  const setQueryParams = useCallback(
    (name: string, value: string) => {
      const params = new URLSearchParams(searchParams)
      params.set(name, value)
      router.push(`${pathname}?${params.toString()}`)
    },
    [searchParams, pathname, router]
  )

  return (
    <SortProducts
      sortBy={sortBy}
      setQueryParams={setQueryParams}
      data-testid={dataTestId}
    />
  )
}

export default RefinementList
