"use client"

import { Dialog, DialogPanel, Transition, TransitionChild } from "@headlessui/react"
import { Fragment, useState } from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import CountrySelect from "../country-select"
import LanguageSelect from "../language-select"
import { HttpTypes } from "@medusajs/types"
import { useToggleState, type Locale } from "@stheno/storefront-core"
import { X } from "@phosphor-icons/react/dist/ssr"

const SideMenuItems = [
  { name: "Home", href: "/" },
  { name: "Catalog", href: "/store" },
  { name: "Latest Drops", href: "/store?sortBy=created_at" },
  { name: "Account", href: "/account" },
  { name: "Shopping Bag", href: "/cart" },
]

export type SideMenuProps = {
  regions: HttpTypes.StoreRegion[] | null
  locales: Locale[] | null
  currentLocale: string | null
}

/**
 * Lustre Mobile Navigation (FRONTEND_LUSTRE_PLAN §4.1).
 * Full-height Headless UI dialog for mobile viewports (hidden on desktop).
 */
const SideMenu: React.FC<SideMenuProps> = ({
  regions,
  locales,
  currentLocale,
}) => {
  const [isOpen, setIsOpen] = useState(false)
  const countryToggle = useToggleState()
  const languageToggle = useToggleState()

  return (
    <div className="flex items-center">
      {/* Mobile-only menu toggle */}
      <button
        type="button"
        onClick={() => setIsOpen(true)}
        data-testid="nav-menu-button"
        aria-label="Open Navigation Menu"
        className="flex items-center gap-1.5 p-2 rounded-full text-xs font-medium text-[var(--text-primary)] hover:bg-[var(--bg-raised)] transition-colors focus:outline-none lustre-btn-press"
      >
        <span className="flex flex-col gap-1 w-4" aria-hidden="true">
          <span className="h-0.5 w-4 bg-[var(--text-primary)] rounded-full" />
          <span className="h-0.5 w-3 bg-[var(--text-primary)] rounded-full" />
          <span className="h-0.5 w-4 bg-[var(--text-primary)] rounded-full" />
        </span>
        <span className="sr-only">Menu</span>
      </button>

      {/* Full-height mobile dialog */}
      <Transition show={isOpen} as={Fragment}>
        <Dialog
          as="div"
          className="relative z-50 md:hidden"
          onClose={() => setIsOpen(false)}
        >
          <TransitionChild
            as={Fragment}
            enter="transition-opacity ease-out duration-250"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="transition-opacity ease-in duration-200"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div
              className="fixed inset-0 bg-black/60 backdrop-blur-md"
              data-testid="side-menu-backdrop"
            />
          </TransitionChild>

          <div className="fixed inset-0 overflow-hidden">
            <TransitionChild
              as={Fragment}
              enter="transform transition ease-out duration-300"
              enterFrom="translate-y-full opacity-0"
              enterTo="translate-y-0 opacity-100"
              leave="transform transition ease-in duration-200"
              leaveFrom="translate-y-0 opacity-100"
              leaveTo="translate-y-full opacity-0"
            >
              <DialogPanel
                className="fixed inset-0 w-full h-full bg-[var(--bg-canvas)] text-[var(--text-primary)] flex flex-col justify-between p-6 sm:p-8 overflow-y-auto"
                data-testid="nav-menu-popup"
              >
                {/* Header: Wordmark and Close Button */}
                <div className="flex items-center justify-between pb-6 border-b border-[var(--hairline)]">
                  <div className="flex flex-col">
                    <span className="text-base font-semibold tracking-[0.2em] uppercase text-[var(--text-primary)]">
                      Store
                    </span>
                    <span className="text-[9px] tracking-[0.25em] text-[var(--text-muted)] uppercase -mt-0.5">
                      Watches &amp; Objects
                    </span>
                  </div>
                  <button
                    type="button"
                    data-testid="close-menu-button"
                    onClick={() => setIsOpen(false)}
                    aria-label="Close navigation"
                    className="p-2 rounded-full text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-raised)] transition-colors lustre-btn-press"
                  >
                    <X size={20} />
                  </button>
                </div>

                {/* Editorial Navigation Links */}
                <nav className="flex-1 py-10 flex flex-col justify-center">
                  <ul className="flex flex-col gap-6">
                    {SideMenuItems.map((item) => (
                      <li key={item.name}>
                        <LocalizedClientLink
                          href={item.href}
                          onClick={() => setIsOpen(false)}
                          data-testid={`${item.name.toLowerCase().replace(/\s+/g, "-")}-link`}
                          className="font-medium text-3xl sm:text-4xl tracking-tight text-[var(--text-primary)] hover:text-[var(--accent-sapphire)] transition-colors block"
                        >
                          {item.name}
                        </LocalizedClientLink>
                      </li>
                    ))}
                  </ul>
                </nav>

                {/* Footer: Preferences & Regional selection */}
                <div className="pt-6 border-t border-[var(--hairline)] flex flex-col gap-3 text-xs">
                  {Boolean(locales?.length) && (
                    <div
                      className="p-3 rounded-[12px] bg-[var(--bg-surface)] border border-[var(--hairline)]"
                      onMouseEnter={languageToggle.open}
                      onMouseLeave={languageToggle.close}
                    >
                      <LanguageSelect
                        toggleState={languageToggle}
                        locales={locales!}
                        currentLocale={currentLocale}
                      />
                    </div>
                  )}

                  {Boolean(regions?.length) && (
                    <div
                      className="p-3 rounded-[12px] bg-[var(--bg-surface)] border border-[var(--hairline)]"
                      onMouseEnter={countryToggle.open}
                      onMouseLeave={countryToggle.close}
                    >
                      <CountrySelect
                        toggleState={countryToggle}
                        regions={regions!}
                      />
                    </div>
                  )}
                </div>
              </DialogPanel>
            </TransitionChild>
          </div>
        </Dialog>
      </Transition>
    </div>
  )
}

export default SideMenu
