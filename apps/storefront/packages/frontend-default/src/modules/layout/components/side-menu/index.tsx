"use client"

import { Dialog, DialogPanel, Transition, TransitionChild } from "@headlessui/react"
import { ArrowRightMini, XMark } from "@medusajs/icons"
import { clx, useToggleState } from "@medusajs/ui"
import { Fragment, useState } from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import CountrySelect from "../country-select"
import LanguageSelect from "../language-select"
import { HttpTypes } from "@medusajs/types"
import type { Locale } from "@stheno/storefront-core"

type SideMenuProps = {
  regions: HttpTypes.StoreRegion[] | null
  locales: Locale[] | null
  currentLocale: string | null
  categories?: Array<{ id: string; name: string; handle: string }> | null
  enableCustomerAccounts?: boolean
}

const getCategoryIcon = (handle: string) => {
  const h = handle.toLowerCase()
  if (h.includes("phone")) {
    return (
      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <rect x="5" y="2" width="14" height="20" rx="3" />
        <line x1="12" y1="18" x2="12.01" y2="18" strokeWidth={2.5} strokeLinecap="round" />
      </svg>
    )
  }
  if (h.includes("laptop") || h.includes("tablet") || h.includes("computer")) {
    return (
      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <rect x="3" y="4" width="18" height="12" rx="2" />
        <path d="M2 18h20a1 1 0 011 1v1H1v-1a1 1 0 011-1z" />
      </svg>
    )
  }
  if (h.includes("audio") || h.includes("sound") || h.includes("headphone")) {
    return (
      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path d="M3 14h3a2 2 0 012 2v3a2 2 0 01-2 2H5a2 2 0 01-2-2v-7a9 9 0 0118 0v7a2 2 0 01-2 2h-1a2 2 0 01-2-2v-3a2 2 0 012-2h3" />
      </svg>
    )
  }
  if (h.includes("wearable") || h.includes("watch")) {
    return (
      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <rect x="6" y="6" width="12" height="12" rx="3" />
        <path d="M9 6V2h6v4M9 18v4h6v-4" />
      </svg>
    )
  }
  return (
    <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
    </svg>
  )
}

const SideMenu = ({
  regions,
  locales,
  currentLocale,
  categories,
  enableCustomerAccounts = false,
}: SideMenuProps) => {
  const [isOpen, setIsOpen] = useState(false)
  const openMenu = () => setIsOpen(true)
  const closeMenu = () => setIsOpen(false)

  const countryToggleState = useToggleState()
  const languageToggleState = useToggleState()

  const primaryItems = [
    {
      name: "Home",
      href: "/",
      icon: (
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
        </svg>
      ),
      badge: null,
    },
    {
      name: "All Tech Catalog",
      href: "/store",
      icon: (
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
        </svg>
      ),
      badge: "Full Lineup",
    },
    {
      name: "Latest Releases",
      href: "/store?sortBy=created_at",
      icon: (
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
        </svg>
      ),
      badge: "2026",
    },
    {
      name: "Shopping Bag",
      href: "/cart",
      icon: (
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M16 11V7a4 4 0 00-8 0v4M5 9h14l1 12H4L5 9z" />
        </svg>
      ),
      badge: "Checkout",
    },
    ...(enableCustomerAccounts
      ? [
          {
            name: "My Account",
            href: "/account",
            icon: (
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            ),
            badge: null,
          },
        ]
      : []),
  ]

  return (
    <div className="h-full flex items-center">
      {/* Menu Trigger Button */}
      <button
        type="button"
        onClick={openMenu}
        data-testid="nav-menu-button"
        className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-zinc-800 dark:text-zinc-200 hover:text-zinc-950 dark:hover:text-white transition-colors focus:outline-none py-2 px-1 cursor-pointer"
      >
        <span className="flex flex-col gap-1 w-4">
          <span className="h-0.5 w-4 bg-zinc-900 dark:bg-zinc-100 rounded-full" />
          <span className="h-0.5 w-3 bg-zinc-900 dark:bg-zinc-100 rounded-full" />
          <span className="h-0.5 w-4 bg-zinc-900 dark:bg-zinc-100 rounded-full" />
        </span>
        <span className="hidden sm:inline">Menu</span>
      </button>

      {/* Slide-Over Drawer using Dialog (Portaled to Body) */}
      <Transition show={isOpen} as={Fragment}>
        <Dialog as="div" className="relative z-[1000]" onClose={closeMenu}>
          {/* Backdrop with Apple blur */}
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
              className="fixed inset-0 bg-black/70 backdrop-blur-md transition-all"
              data-testid="side-menu-backdrop"
            />
          </TransitionChild>

          <div className="fixed inset-0 overflow-hidden">
            <div className="absolute inset-0 overflow-hidden">
              <div className="pointer-events-none fixed inset-y-0 left-0 flex max-w-full">
                <TransitionChild
                  as={Fragment}
                  enter="transform transition ease-out duration-300"
                  enterFrom="-translate-x-full"
                  enterTo="translate-x-0"
                  leave="transform transition ease-in duration-200"
                  leaveFrom="translate-x-0"
                  leaveTo="-translate-x-full"
                >
                  <DialogPanel
                    className="pointer-events-auto w-screen max-w-sm sm:max-w-md bg-[#0C0C0E]/95 text-white shadow-2xl flex flex-col justify-between p-6 sm:p-7 overflow-y-auto no-scrollbar border-r border-white/10 backdrop-blur-2xl relative"
                    data-testid="nav-menu-popup"
                  >
                    {/* Ambient Glow */}
                    <div className="absolute top-0 left-0 w-64 h-64 bg-blue-600/10 rounded-full blur-3xl pointer-events-none" />

                    {/* Top Content Area */}
                    <div className="flex flex-col gap-6 relative z-10">
                      {/* Header */}
                      <div className="flex items-center justify-between pb-5 border-b border-white/10">
                        <div className="flex flex-col">
                          <div className="flex items-center gap-2">
                            <span className="text-xl font-bold tracking-[0.18em] uppercase text-white leading-none">
                              VENDRYX
                            </span>
                            <span className="px-2 py-0.5 rounded-full bg-blue-500/15 border border-blue-500/30 text-[9px] font-medium text-blue-400 tracking-wide uppercase">
                              Pro Tech
                            </span>
                          </div>
                          <span className="text-[10px] tracking-wider text-zinc-400 font-mono mt-1">
                            Flagship Hardware Store
                          </span>
                        </div>
                        <button
                          type="button"
                          data-testid="close-menu-button"
                          onClick={closeMenu}
                          aria-label="Close navigation menu"
                          className="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 active:scale-95 text-zinc-300 hover:text-white transition-all flex items-center justify-center border border-white/10 cursor-pointer"
                        >
                          <XMark className="w-4 h-4" />
                        </button>
                      </div>

                      {/* Quick Search Bar */}
                      <LocalizedClientLink
                        href="/store"
                        onClick={closeMenu}
                        className="flex items-center gap-3 px-3.5 py-2.5 rounded-xl bg-white/[0.05] hover:bg-white/[0.09] border border-white/[0.08] text-xs text-zinc-400 hover:text-zinc-200 transition-all group"
                      >
                        <svg
                          className="w-4 h-4 text-zinc-400 group-hover:text-blue-400 transition-colors"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke="currentColor"
                          strokeWidth={2}
                        >
                          <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                        <span>Search hardware & devices...</span>
                        <span className="ml-auto text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/10 text-zinc-400 group-hover:text-zinc-200">
                          Catalog
                        </span>
                      </LocalizedClientLink>

                      {/* Primary Navigation List (Apple HIG Inset Grouped) */}
                      <div className="space-y-1">
                        <span className="text-[10px] font-semibold uppercase tracking-[0.2em] text-zinc-400 px-1 block mb-2">
                          Store Navigation
                        </span>
                        <div className="flex flex-col gap-1">
                          {primaryItems.map((item) => (
                            <LocalizedClientLink
                              key={item.name}
                              href={item.href}
                              onClick={closeMenu}
                              className="flex items-center justify-between px-3.5 py-2.5 rounded-xl hover:bg-white/[0.08] active:bg-white/[0.12] text-zinc-200 hover:text-white transition-all group"
                              data-testid={`${item.name.toLowerCase().replace(/\s+/g, "-")}-link`}
                            >
                              <div className="flex items-center gap-3">
                                <span className="text-zinc-400 group-hover:text-blue-400 transition-colors">
                                  {item.icon}
                                </span>
                                <span className="text-sm font-medium tracking-tight">
                                  {item.name}
                                </span>
                              </div>
                              <div className="flex items-center gap-2">
                                {item.badge && (
                                  <span className="text-[10px] font-medium px-2 py-0.5 rounded-full bg-white/10 text-zinc-300">
                                    {item.badge}
                                  </span>
                                )}
                                <svg
                                  className="w-3.5 h-3.5 text-zinc-600 group-hover:text-zinc-300 group-hover:translate-x-0.5 transition-all"
                                  fill="none"
                                  viewBox="0 0 24 24"
                                  stroke="currentColor"
                                  strokeWidth={2}
                                >
                                  <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
                                </svg>
                              </div>
                            </LocalizedClientLink>
                          ))}
                        </div>
                      </div>

                      {/* Hardware Categories Section (Apple 2-Column Grid) */}
                      {categories && categories.length > 0 && (
                        <div className="space-y-3 pt-2">
                          <div className="flex items-center justify-between px-1">
                            <span className="text-[10px] font-semibold uppercase tracking-[0.2em] text-zinc-400">
                              Device Families
                            </span>
                            <LocalizedClientLink
                              href="/store"
                              onClick={closeMenu}
                              className="text-[11px] text-blue-400 hover:text-blue-300 font-medium transition-colors"
                            >
                              View All →
                            </LocalizedClientLink>
                          </div>
                          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                            {categories.map((cat) => (
                              <LocalizedClientLink
                                key={cat.id}
                                href={`/categories/${cat.handle}`}
                                onClick={closeMenu}
                                className="flex items-center gap-2.5 p-2.5 rounded-xl bg-white/[0.04] hover:bg-white/[0.09] active:bg-white/[0.12] border border-white/[0.06] hover:border-white/[0.15] text-zinc-300 hover:text-white transition-all group"
                              >
                                <span className="w-7 h-7 rounded-lg bg-white/5 border border-white/10 flex items-center justify-center text-zinc-400 group-hover:text-blue-400 group-hover:bg-blue-500/10 transition-colors shrink-0">
                                  {getCategoryIcon(cat.handle)}
                                </span>
                                <span className="text-xs font-medium truncate">{cat.name}</span>
                              </LocalizedClientLink>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Hardware Guarantees Strip */}
                      <div className="grid grid-cols-3 gap-2 py-3 px-3.5 rounded-xl bg-white/[0.03] border border-white/[0.06] text-[10px] text-zinc-400">
                        <div className="flex flex-col items-center text-center gap-0.5">
                          <span className="text-blue-400 text-xs">🛡️</span>
                          <span className="font-medium text-zinc-200">2-Year</span>
                          <span className="text-[9px] text-zinc-400">Warranty</span>
                        </div>
                        <div className="flex flex-col items-center text-center gap-0.5 border-x border-white/10 px-1">
                          <span className="text-blue-400 text-xs">⚡</span>
                          <span className="font-medium text-zinc-200">Express</span>
                          <span className="text-[9px] text-zinc-400">Tracked</span>
                        </div>
                        <div className="flex flex-col items-center text-center gap-0.5">
                          <span className="text-blue-400 text-xs">💬</span>
                          <span className="font-medium text-zinc-200">Direct</span>
                          <span className="text-[9px] text-zinc-400">Support</span>
                        </div>
                      </div>
                    </div>

                    {/* Bottom Area: Localization & Copyright */}
                    <div className="flex flex-col gap-y-3 pt-5 border-t border-white/10 text-xs mt-6 relative z-10">
                      <div className="space-y-2">
                        {!!locales?.length && (
                          <div
                            className="flex justify-between items-center py-2 px-3 rounded-xl bg-white/[0.04] hover:bg-white/[0.07] border border-white/[0.06] transition-colors"
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
                          className="flex justify-between items-center py-2 px-3 rounded-xl bg-white/[0.04] hover:bg-white/[0.07] border border-white/[0.06] transition-colors"
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
                      </div>

                      <div className="pt-2 text-zinc-400 text-[10px] flex items-center justify-between font-mono">
                        <span>© {new Date().getFullYear()} VENDRYX</span>
                        <span>Pro Hardware Store</span>
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
