"use client"

import styles from "../filter-bar.module.css"

export type SortOptions = "price_asc" | "price_desc" | "created_at"

type SortProductsProps = {
  sortBy: SortOptions
  setQueryParams: (name: string, value: SortOptions) => void
  "data-testid"?: string
}

const sortOptions: { value: SortOptions; label: string }[] = [
  { value: "created_at", label: "Latest" },
  { value: "price_asc", label: "Price ↑" },
  { value: "price_desc", label: "Price ↓" },
]

/**
 * PORT of The Matrix Hours `FilterBar` — its chip row, its stylesheet, its
 * active state.
 *
 * The source bar filtered client-side out of `ShopContext` (category and brand
 * chips over an in-memory list). Core owns the query params and runs the query
 * server-side, so the chips write `sortBy` to the URL instead. Same control,
 * same look; the filtering is core's.
 *
 * Category and brand chips are not here because core exposes no facet for them
 * yet — see docs/FRONTEND_MATRIX_SLOTS.md. Rendering dead chips would be worse
 * than rendering none.
 */
const SortProducts = ({
  "data-testid": dataTestId,
  sortBy,
  setQueryParams,
}: SortProductsProps) => (
  <div className={styles.filterBarWrapper}>
    <div className={styles.filterBar} role="group" aria-label="Sort products">
      {sortOptions.map((option, i) => (
        <div key={option.value} style={{ display: "contents" }}>
          {i > 0 && <div className={styles.divider} />}
          <button
            type="button"
            className={`${styles.chip} ${
              sortBy === option.value ? styles.active : ""
            }`}
            aria-pressed={sortBy === option.value}
            onClick={() => setQueryParams("sortBy", option.value)}
            data-testid={i === 0 ? dataTestId : undefined}
          >
            {option.label}
          </button>
        </div>
      ))}
    </div>
  </div>
)

export default SortProducts
