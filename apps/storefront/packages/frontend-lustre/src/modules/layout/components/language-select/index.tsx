"use client"

import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
  Transition,
} from "@headlessui/react"
import { Fragment, useEffect, useMemo, useState, useTransition } from "react"
import { useRouter } from "next/navigation"
import ReactCountryFlag from "react-country-flag"
import { StateType, updateLocale, type Locale } from "@stheno/storefront-core"

type LanguageOption = {
  code: string
  name: string
  localizedName: string
  countryCode: string
}

const getCountryCodeFromLocale = (localeCode: string): string => {
  try {
    const locale = new Intl.Locale(localeCode)
    return locale.region?.toUpperCase() ?? localeCode.toUpperCase()
  } catch {
    const parts = localeCode.split(/[-_]/)
    return parts.length > 1 ? parts[1].toUpperCase() : parts[0].toUpperCase()
  }
}

const getLocalizedLanguageName = (
  code: string,
  fallbackName: string,
  displayLocale: string = "en-US"
): string => {
  try {
    const displayNames = new Intl.DisplayNames([displayLocale], {
      type: "language",
    })
    return displayNames.of(code) ?? fallbackName
  } catch {
    return fallbackName
  }
}

const DEFAULT_OPTION: LanguageOption = {
  code: "",
  name: "Default",
  localizedName: "English (Default)",
  countryCode: "",
}

type LanguageSelectProps = {
  toggleState: StateType
  locales: Locale[]
  currentLocale: string | null
}

const LanguageSelect = ({
  toggleState,
  locales,
  currentLocale,
}: LanguageSelectProps) => {
  const [current, setCurrent] = useState<LanguageOption | undefined>(undefined)
  const [isPending, startTransition] = useTransition()
  const router = useRouter()
  const { state, close } = toggleState

  const options = useMemo(() => {
    const localeOptions = locales.map((locale) => ({
      code: locale.code,
      name: locale.name,
      localizedName: getLocalizedLanguageName(
        locale.code,
        locale.name,
        currentLocale ?? "en-US"
      ),
      countryCode: getCountryCodeFromLocale(locale.code),
    }))
    return [DEFAULT_OPTION, ...localeOptions]
  }, [locales, currentLocale])

  useEffect(() => {
    if (currentLocale) {
      const option = options.find(
        (o) => o.code.toLowerCase() === currentLocale.toLowerCase()
      )
      setCurrent(option ?? DEFAULT_OPTION)
    } else {
      setCurrent(DEFAULT_OPTION)
    }
  }, [options, currentLocale])

  const handleChange = (option: LanguageOption) => {
    startTransition(async () => {
      await updateLocale(option.code)
      close()
      router.refresh()
    })
  }

  return (
    <div className="w-full">
      <Listbox
        as="div"
        onChange={handleChange}
        defaultValue={current}
        disabled={isPending}
      >
        <ListboxButton className="w-full text-left py-1 text-xs">
          <div className="flex items-center justify-between text-[var(--text-secondary)]">
            <span className="font-medium text-[var(--text-primary)]">Language:</span>
            {current && (
              <span className="flex items-center gap-1.5 font-medium text-[var(--text-primary)]">
                {current.countryCode && (
                  /* @ts-ignore */
                  <ReactCountryFlag
                    svg
                    style={{ width: "16px", height: "12px", borderRadius: "2px" }}
                    countryCode={current.countryCode}
                  />
                )}
                <span className="truncate max-w-[140px]">{isPending ? "Updating..." : current.localizedName}</span>
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
            {options.map((o) => (
              <ListboxOption
                key={o.code || "default"}
                value={o}
                className="py-1.5 px-2.5 rounded-[8px] hover:bg-[var(--bg-raised)] cursor-pointer flex items-center gap-2 text-[var(--text-primary)] transition-colors"
              >
                {o.countryCode ? (
                  /* @ts-ignore */
                  <ReactCountryFlag
                    svg
                    style={{ width: "16px", height: "12px", borderRadius: "2px" }}
                    countryCode={o.countryCode}
                  />
                ) : (
                  <span className="w-4 h-3" />
                )}
                <span className="truncate">{o.localizedName}</span>
              </ListboxOption>
            ))}
          </ListboxOptions>
        </Transition>
      </Listbox>
    </div>
  )
}

export default LanguageSelect
