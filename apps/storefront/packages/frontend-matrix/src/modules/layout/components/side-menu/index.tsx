"use client"

import {
  Dialog,
  DialogPanel,
  Transition,
  TransitionChild,
} from "@headlessui/react"
import { ArrowRightMini, XMark } from "@medusajs/icons"
import { clx, useToggleState } from "@medusajs/ui"
import { Fragment, useState } from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import CountrySelect from "../country-select"
import LanguageSelect from "../language-select"
import { HttpTypes } from "@medusajs/types"
import type { Locale } from "@stheno/storefront-core"

const SideMenuItems = [
  { name: "Home", href: "/" },
  { name: "Shop All", href: "/store" },
  { name: "New Arrivals", href: "/store?sortBy=created_at" },
  { name: "Collections", href: "/collections/summer-collection" },
  { name: "Account", href: "/account" },
  { name: "Shopping Bag", href: "/cart" },
]

type SideMenuProps = {
  regions: HttpTypes.StoreRegion[] | null
  locales: Locale[] | null
  currentLocale: string | null
}

const SideMenu = ({ regions, locales, currentLocale }: SideMenuProps) => {
  const [isOpen, setIsOpen] = useState(false)
  const openMenu = () => setIsOpen(true)
  const closeMenu = () => setIsOpen(false)

  const countryToggleState = useToggleState()
  const languageToggleState = useToggleState()

  return (
    <div className="h-full flex items-center">
      {/* Menu Trigger Button */}
      <button
        type="button"
        onClick={openMenu}
        data-testid="nav-menu-button"
        className="flex items-center gap-2 text-xs font-semibold uppercase tracking-widest text-[#f5f5f0] hover:text-[#c9a96e] transition-colors focus:outline-none py-2 px-1 cursor-pointer"
      >
        <span className="flex flex-col gap-1 w-4">
          <span className="h-0.5 w-4 bg-zinc-900 rounded-full" />
          <span className="h-0.5 w-3 bg-zinc-900 rounded-full" />
          <span className="h-0.5 w-4 bg-zinc-900 rounded-full" />
        </span>
        <span>Menu</span>
      </button>

      {/* Slide-Over Drawer using Dialog (Portaled to Body) */}
      <Transition show={isOpen} as={Fragment}>
        <Dialog as="div" className="relative z-[1000]" onClose={closeMenu}>
          {/* Backdrop */}
          <TransitionChild
            as={Fragment}
            enter="transition-opacity ease-linear duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="transition-opacity ease-linear duration-200"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div
              className="fixed inset-0 bg-black/75 backdrop-blur-sm"
              data-testid="side-menu-backdrop"
            />
          </TransitionChild>

          <div className="fixed inset-0 overflow-hidden">
            <div className="absolute inset-0 overflow-hidden">
              <div className="pointer-events-none fixed inset-y-0 left-0 flex max-w-full">
                <TransitionChild
                  as={Fragment}
                  enter="transform transition ease-in-out duration-300"
                  enterFrom="-translate-x-full"
                  enterTo="translate-x-0"
                  leave="transform transition ease-in-out duration-300"
                  leaveFrom="translate-x-0"
                  leaveTo="-translate-x-full"
                >
                  <DialogPanel
                    className="pointer-events-auto w-screen max-w-sm sm:max-w-md bg-zinc-950 text-white flex flex-col justify-between p-6 sm:p-10 overflow-y-auto no-scrollbar"
                    data-testid="nav-menu-popup"
                  >
                    {/* Header */}
                    <div className="flex items-center justify-between pb-6 border-b border-zinc-800">
                      <div className="flex flex-col">
                        <span className="text-xl sm:text-2xl font-[family-name:var(--font-plus-jakarta)] tracking-[0.25em] uppercase font-bold text-white leading-tight">
                          STHENO
                        </span>
                        <span className="text-[10px] tracking-[0.35em] text-zinc-400 uppercase font-sans mt-1">
                          Atelier • Edition
                        </span>
                      </div>
                      <button
                        type="button"
                        data-testid="close-menu-button"
                        onClick={closeMenu}
                        className="p-2.5 rounded-full bg-zinc-900 hover:bg-zinc-800 text-zinc-300 hover:text-white transition-colors focus:outline-none cursor-pointer"
                      >
                        <XMark className="w-5 h-5" />
                      </button>
                    </div>

                    {/* Navigation Links */}
                    <nav className="flex-1 py-8 sm:py-12 flex flex-col justify-center">
                      <ul className="flex flex-col gap-5 sm:gap-6">
                        {SideMenuItems.map((item) => (
                          <li key={item.name}>
                            <LocalizedClientLink
                              href={item.href}
                              className="text-2xl sm:text-3xl font-[family-name:var(--font-plus-jakarta)] font-bold text-zinc-200 hover:text-white hover:translate-x-2 transition-all block"
                              onClick={closeMenu}
                              data-testid={`${item.name
                                .toLowerCase()
                                .replace(/\s+/g, "-")}-link`}
                            >
                              {item.name}
                            </LocalizedClientLink>
                          </li>
                        ))}
                      </ul>
                    </nav>

                    {/* Footer / Locale / Currency */}
                    <div className="flex flex-col gap-y-4 pt-6 border-t border-zinc-800 text-xs">
                      {!!locales?.length && (
                        <div
                          className="flex justify-between items-center py-2.5 px-3.5 rounded-none bg-zinc-900/80 border border-zinc-800"
                          onMouseEnter={languageToggleState.open}
                          onMouseLeave={languageToggleState.close}
                        >
                          <LanguageSelect
                            toggleState={languageToggleState}
                            locales={locales}
                            currentLocale={currentLocale}
                          />
                          <ArrowRightMini
                            className={clx(
                              "transition-transform duration-150 text-zinc-400",
                              languageToggleState.state ? "-rotate-90" : ""
                            )}
                          />
                        </div>
                      )}

                      <div
                        className="flex justify-between items-center py-2.5 px-3.5 rounded-none bg-zinc-900/80 border border-zinc-800"
                        onMouseEnter={countryToggleState.open}
                        onMouseLeave={countryToggleState.close}
                      >
                        {regions && (
                          <CountrySelect
                            toggleState={countryToggleState}
                            regions={regions}
                          />
                        )}
                        <ArrowRightMini
                          className={clx(
                            "transition-transform duration-150 text-zinc-400",
                            countryToggleState.state ? "-rotate-90" : ""
                          )}
                        />
                      </div>

                      <div className="pt-2 text-[#a0a0a0] text-[11px] flex items-center justify-between font-light">
                        <span>
                          © {new Date().getFullYear()} STHENO Atelier.
                        </span>
                        <span>All rights reserved.</span>
                      </div>
                    </div>
                  </DialogPanel>
                </TransitionChild>
              </div>
            </div>
          </div>
        </Dialog>
      </Transition>
    </div>
  )
}

export default SideMenu
