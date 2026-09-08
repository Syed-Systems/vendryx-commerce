"use client"

import { Popover, PopoverButton, PopoverPanel, Transition } from "@headlessui/react"
import { Fragment, useMemo } from "react"
import ReactCountryFlag from "react-country-flag"
import { updateRegion } from "@stheno/storefront-core"
import { useParams, usePathname } from "next/navigation"
import { HttpTypes } from "@medusajs/types"
import { CaretDown } from "@phosphor-icons/react/dist/ssr"

export type HeaderRegionSelectProps = {
  regions: HttpTypes.StoreRegion[]
}

type CountryOption = {
  country: string
  region: string
  label: string
}

export default function HeaderRegionSelect({ regions }: HeaderRegionSelectProps) {
  const { countryCode } = useParams() as { countryCode?: string }
  const currentPath = usePathname().split(`/${countryCode}`)[1] || ""

  const options = useMemo(() => {
    return (
      regions
        ?.map((r) =>
          r.countries?.map((c) => ({
            country: c.iso_2 ?? "",
            region: r.id,
            label: c.display_name ?? "",
          }))
        )
        .flat()
        .filter((c): c is CountryOption => Boolean(c && c.country))
        .sort((a, b) => a.label.localeCompare(b.label)) || []
    )
  }, [regions])

  const activeOption = useMemo(() => {
    return options.find((o) => o.country.toLowerCase() === countryCode?.toLowerCase())
  }, [options, countryCode])

  const handleSelect = (country: string, close: () => void) => {
    updateRegion(country, currentPath)
    close()
  }

  if (!regions || regions.length === 0) {
    return null
  }

  return (
    <Popover className="relative">
      {({ close }) => (
        <>
          <PopoverButton
            aria-label="Select destination country"
            className="flex items-center gap-2 px-2.5 py-1.5 rounded-full text-xs font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-raised)] transition-colors focus:outline-none lustre-btn-press"
          >
            {activeOption ? (
              <>
                {/* @ts-ignore */}
                <ReactCountryFlag
                  svg
                  style={{ width: "16px", height: "12px", borderRadius: "2px" }}
                  countryCode={activeOption.country}
                />
                <span className="uppercase text-[11px] tracking-wider font-mono">{activeOption.country}</span>
              </>
            ) : (
              <span className="text-[11px] uppercase tracking-wider">Region</span>
            )}
            <CaretDown size={12} className="text-[var(--text-muted)]" />
          </PopoverButton>

          <Transition
            as={Fragment}
            enter="transition ease-out duration-150"
            enterFrom="opacity-0 translate-y-1 scale-95"
            enterTo="opacity-100 translate-y-0 scale-100"
            leave="transition ease-in duration-100"
            leaveFrom="opacity-100 translate-y-0 scale-100"
            leaveTo="opacity-0 translate-y-1 scale-95"
          >
            <PopoverPanel className="absolute right-0 top-full mt-2 w-56 rounded-[12px] bg-[var(--bg-surface)] border border-[var(--hairline)] shadow-[var(--elevation-dropdown)] p-2 z-50 max-h-72 overflow-y-auto no-scrollbar">
              <div className="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)] border-b border-[var(--hairline)] mb-1">
                Destination
              </div>
              <div className="space-y-0.5">
                {options.map((opt) => (
                  <button
                    key={opt.country}
                    type="button"
                    onClick={() => handleSelect(opt.country, close)}
                    className={`flex items-center justify-between w-full px-2.5 py-1.5 text-xs rounded-[8px] transition-colors text-left ${
                      opt.country.toLowerCase() === countryCode?.toLowerCase()
                        ? "bg-[var(--accent-sapphire-subtle)] text-[var(--accent-sapphire)] font-medium"
                        : "text-[var(--text-primary)] hover:bg-[var(--bg-raised)]"
                    }`}
                  >
                    <span className="flex items-center gap-2 truncate">
                      {/* @ts-ignore */}
                      <ReactCountryFlag
                        svg
                        style={{ width: "16px", height: "12px", borderRadius: "2px" }}
                        countryCode={opt.country}
                      />
                      <span className="truncate">{opt.label}</span>
                    </span>
                    <span className="text-[10px] uppercase text-[var(--text-muted)] ml-2">
                      {opt.country}
                    </span>
                  </button>
                ))}
              </div>
            </PopoverPanel>
          </Transition>
        </>
      )}
    </Popover>
  )
}
