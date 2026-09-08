"use client"

import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
  Transition,
} from "@headlessui/react"
import { Fragment, useEffect, useMemo, useState } from "react"
import ReactCountryFlag from "react-country-flag"
import { StateType, updateRegion } from "@stheno/storefront-core"
import { useParams, usePathname } from "next/navigation"
import { HttpTypes } from "@medusajs/types"

type CountryOption = {
  country: string | undefined
  region: string
  label: string | undefined
}

type CountrySelectProps = {
  toggleState: StateType
  regions: HttpTypes.StoreRegion[]
}

const CountrySelect = ({ toggleState, regions }: CountrySelectProps) => {
  const [current, setCurrent] = useState<CountryOption | undefined>(undefined)

  const { countryCode } = useParams()
  const currentPath = usePathname().split(`/${countryCode}`)[1] || ""
  const { state, close } = toggleState

  const options = useMemo(() => {
    return regions
      ?.map((r) =>
        r.countries?.map((c) => ({
          country: c.iso_2,
          region: r.id,
          label: c.display_name,
        }))
      )
      .flat()
      .filter((c): c is CountryOption => Boolean(c))
      .sort((a, b) => (a.label ?? "").localeCompare(b.label ?? ""))
  }, [regions])

  useEffect(() => {
    if (countryCode) {
      const option = options?.find((o) => o.country === countryCode)
      setCurrent(option)
    }
  }, [options, countryCode])

  const handleChange = (option: CountryOption) => {
    if (option.country) {
      updateRegion(option.country, currentPath)
    }
    close()
  }

  return (
    <div className="w-full">
      <Listbox
        as="div"
        onChange={handleChange}
        defaultValue={
          countryCode
            ? options?.find((o) => o.country === countryCode)
            : undefined
        }
      >
        <ListboxButton className="w-full text-left py-1 text-xs">
          <div className="flex items-center justify-between text-[var(--text-secondary)]">
            <span className="font-medium text-[var(--text-primary)]">Destination:</span>
            {current && (
              <span className="flex items-center gap-1.5 font-medium text-[var(--text-primary)]">
                {/* @ts-ignore */}
                <ReactCountryFlag
                  svg
                  style={{ width: "16px", height: "12px", borderRadius: "2px" }}
                  countryCode={current.country ?? ""}
                />
                <span className="truncate max-w-[140px]">{current.label}</span>
              </span>
            )}
          </div>
        </ListboxButton>

        <Transition
          show={state}
          as={Fragment}
          leave="transition ease-in duration-100"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <ListboxOptions
            static
            className="mt-2 max-h-56 overflow-y-auto rounded-[12px] bg-[var(--bg-surface)] border border-[var(--hairline)] shadow-lg p-1 text-xs no-scrollbar"
          >
            {options?.map((o, idx) => (
              <ListboxOption
                key={idx}
                value={o}
                className="py-1.5 px-2.5 rounded-[8px] hover:bg-[var(--bg-raised)] cursor-pointer flex items-center gap-2 text-[var(--text-primary)] transition-colors"
              >
                {/* @ts-ignore */}
                <ReactCountryFlag
                  svg
                  style={{ width: "16px", height: "12px", borderRadius: "2px" }}
                  countryCode={o.country ?? ""}
                />
                <span className="truncate">{o.label}</span>
              </ListboxOption>
            ))}
          </ListboxOptions>
        </Transition>
      </Listbox>
    </div>
  )
}

export default CountrySelect
