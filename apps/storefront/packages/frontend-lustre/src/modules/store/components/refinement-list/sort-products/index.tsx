"use client"

import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
  Transition,
} from "@headlessui/react"
import { Fragment } from "react"
import { CaretDown } from "@phosphor-icons/react/dist/ssr"

export type SortOptions = "price_asc" | "price_desc" | "created_at"

export type SortProductsProps = {
  sortBy: SortOptions
  setQueryParams: (name: string, value: SortOptions) => void
  "data-testid"?: string
}

const sortOptions: { value: SortOptions; label: string }[] = [
  { value: "created_at", label: "Newest Arrivals" },
  { value: "price_asc", label: "Price: Ascending" },
  { value: "price_desc", label: "Price: Descending" },
]

/**
 * Lustre SortProducts.
 * Refined Headless UI listbox for sort preference selection.
 */
const SortProducts = ({
  sortBy,
  setQueryParams,
  "data-testid": dataTestId,
}: SortProductsProps) => {
  const selectedOption =
    sortOptions.find((opt) => opt.value === sortBy) || sortOptions[0]

  return (
    <div className="relative inline-block text-left" data-testid={dataTestId}>
      <Listbox
        value={sortBy}
        onChange={(val: SortOptions) => setQueryParams("sortBy", val)}
      >
        <ListboxButton className="inline-flex items-center gap-2 h-9 px-3.5 rounded-full text-xs font-medium bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--hairline)] hover:border-[var(--text-secondary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent-sapphire)] transition-colors lustre-btn-press">
          <span className="text-[var(--text-muted)]">Sort:</span>
          <span>{selectedOption.label}</span>
          <CaretDown size={14} className="text-[var(--text-muted)] ml-1" />
        </ListboxButton>

        <Transition
          as={Fragment}
          enter="transition ease-out duration-100"
          enterFrom="opacity-0 scale-95"
          enterTo="opacity-100 scale-100"
          leave="transition ease-in duration-75"
          leaveFrom="opacity-100 scale-100"
          leaveTo="opacity-0 scale-95"
        >
          <ListboxOptions className="absolute right-0 top-full mt-2 w-48 rounded-[12px] bg-[var(--bg-surface)] border border-[var(--hairline)] shadow-[var(--elevation-dropdown)] p-1.5 z-30 focus:outline-none text-xs">
            {sortOptions.map((opt) => (
              <ListboxOption
                key={opt.value}
                value={opt.value}
                className={({ active, selected }) =>
                  `py-2 px-3 rounded-[8px] cursor-pointer flex items-center justify-between transition-colors ${
                    selected
                      ? "font-semibold text-[var(--accent-sapphire)] bg-[var(--accent-sapphire-subtle)]"
                      : active
                      ? "bg-[var(--bg-raised)] text-[var(--text-primary)]"
                      : "text-[var(--text-secondary)]"
                  }`
                }
              >
                {({ selected }) => (
                  <>
                    <span>{opt.label}</span>
                    {selected && (
                      <span className="w-1.5 h-1.5 rounded-full bg-[var(--accent-sapphire)]" />
                    )}
                  </>
                )}
              </ListboxOption>
            ))}
          </ListboxOptions>
        </Transition>
      </Listbox>
    </div>
  )
}

export default SortProducts
