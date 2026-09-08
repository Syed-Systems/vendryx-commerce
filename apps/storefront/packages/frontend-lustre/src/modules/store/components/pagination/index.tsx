"use client"

import { usePathname, useRouter, useSearchParams } from "next/navigation"

export type PaginationProps = {
  page: number
  totalPages: number
  "data-testid"?: string
}

/**
 * Lustre Pagination.
 * Minimalist page navigator with circular indicators.
 */
export function Pagination({
  page,
  totalPages,
  "data-testid": dataTestId,
}: PaginationProps) {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()

  const handlePageChange = (newPage: number) => {
    const params = new URLSearchParams(searchParams)
    params.set("page", newPage.toString())
    router.push(`${pathname}?${params.toString()}`)
  }

  if (totalPages <= 1) {
    return null
  }

  const renderPageButton = (p: number, isCurrent: boolean) => (
    <button
      key={p}
      type="button"
      disabled={isCurrent}
      onClick={() => handlePageChange(p)}
      className={`w-8 h-8 rounded-full text-xs font-medium tabular-nums transition-colors ${
        isCurrent
          ? "bg-[var(--accent-sapphire)] text-white font-semibold cursor-default"
          : "text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-raised)]"
      }`}
    >
      {p}
    </button>
  )

  const pages = []
  for (let i = 1; i <= totalPages; i++) {
    pages.push(renderPageButton(i, i === page))
  }

  return (
    <nav
      aria-label="Pagination"
      className="flex justify-center w-full mt-16"
      data-testid={dataTestId}
    >
      <div className="flex items-center gap-1.5 p-1.5 rounded-full bg-[var(--bg-surface)] border border-[var(--hairline)]">
        {pages}
      </div>
    </nav>
  )
}

export default Pagination
