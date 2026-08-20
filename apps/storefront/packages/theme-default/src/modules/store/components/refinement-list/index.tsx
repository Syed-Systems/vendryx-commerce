"use client"

import { usePathname, useRouter, useSearchParams } from "next/navigation"
import { useCallback } from "react"

import SortProducts, { SortOptions } from "./sort-products"

type RefinementListProps = {
  sortBy: SortOptions
  search?: boolean
  'data-testid'?: string
}

const RefinementList = ({ sortBy, 'data-testid': dataTestId }: RefinementListProps) => {
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

  const setQueryParams = (name: string, value: string) => {
    const query = createQueryString(name, value)
    router.push(`${pathname}?${query}`)
  }

  // No left offset of its own. `pl-6` and `small:ml-[1.675rem]` came from the upstream
  // starter, whose page had no shared container; here the parent already applies the
  // `content-container` gutter, so those pushed this column 24px (mobile) / 27px (desktop)
  // to the right of the hero, the nav and the product grid. The parent also sets the column
  // width (`small:w-64`), so `min-w-[250px]` only fought it.
  return (
    <div className="flex small:flex-col gap-12 py-4 mb-8 w-full">
      <SortProducts sortBy={sortBy} setQueryParams={setQueryParams} data-testid={dataTestId} />
    </div>
  )
}

export default RefinementList
