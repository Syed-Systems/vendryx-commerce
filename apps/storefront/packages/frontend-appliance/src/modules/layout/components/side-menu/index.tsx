"use client"

import { Dialog, DialogPanel, Transition, TransitionChild } from "@headlessui/react"
import { Fragment, useState, useMemo } from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import CountrySelect from "../country-select"
import LanguageSelect from "../language-select"
import { HttpTypes } from "@medusajs/types"
import { useToggleState, type Locale, type StateType } from "@stheno/storefront-core"

type SideMenuProps = {
  regions: HttpTypes.StoreRegion[] | null
  locales: Locale[] | null
  currentLocale: string | null
  categories?: Array<{ id: string; name: string; handle: string }> | null
  enableCustomerAccounts?: boolean
  customer?: HttpTypes.StoreCustomer | null
}

const APPLIANCE_DEPARTMENTS = [
  {
    name: "Culinary & Cooking Systems",
    handle: "major-kitchen",
    desc: "Precision Induction, Steam Ovens & Range Cookers",
    tag: "Commercial Grade",
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <rect x="3" y="3" width="18" height="18" rx="2" />
        <circle cx="8" cy="7" r="1.5" />
        <circle cx="16" cy="7" r="1.5" />
        <rect x="6" y="11" width="12" height="7" rx="1.5" />
      </svg>
    ),
  },
  {
    name: "Preservation & Refrigeration",
    handle: "refrigeration",
    desc: "Multi-Zone Climate, Dual-Compressor & Wine Vaults",
    tag: "Zero-Frost",
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <rect x="5" y="2" width="14" height="20" rx="2" />
        <line x1="5" y1="9" x2="19" y2="9" />
        <line x1="8" y1="5" x2="8" y2="7" />
        <line x1="8" y1="12" x2="8" y2="16" />
      </svg>
    ),
  },
  {
    name: "Specialty Domestic & Small Appliances",
    handle: "small-appliances",
    desc: "Dual-Boiler Espresso Stations, Blenders & Thermostatic Kettles",
    tag: "Precision PID",
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M18 8h1a4 4 0 010 8h-1M2 8h16v9a4 4 0 01-4 4H6a4 4 0 01-4-4V8z" />
        <line x1="6" y1="1" x2="6" y2="4" />
        <line x1="10" y1="1" x2="10" y2="4" />
        <line x1="14" y1="1" x2="14" y2="4" />
      </svg>
    ),
  },
  {
    name: "Climate & Air Purification",
    handle: "climate-air",
    desc: "Medical-Grade HEPA Filtration & Inverter Heat Pumps",
    tag: "A+++ Efficiency",
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9.59 4.59A2 2 0 1111 8H2m10.59 11.41A2 2 0 1014 16H2m15.73-8.27A2.5 2.5 0 1119.5 12H2" />
      </svg>
    ),
  },
  {
    name: "Fabric Care & Laundry Systems",
    handle: "floor-garment",
    desc: "Heat Pump Dryers, Direct-Drive Washers & Steam Presses",
    tag: "Quiet-Drive",
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <rect x="4" y="2" width="16" height="20" rx="2" />
        <circle cx="12" cy="13" r="4.5" />
        <circle cx="8" cy="6" r="1" />
        <line x1="12" y1="6" x2="16" y2="6" />
      </svg>
    ),
  },
  {
    name: "Smart Power & Architectural Electrical",
    handle: "electrical-power",
    desc: "Integrated Lighting Controls, Breaker Panels & Automation",
    tag: "Matter Ready",
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
    ),
  },
]

const QUICK_FILTERS = [
  { label: "Eco Inverter A+++", tag: "Efficiency" },
  { label: "Smart Home / Matter", tag: "Connectivity" },
  { label: "Commercial 3-Phase", tag: "Power" },
  { label: "Sub-Zero Preservation", tag: "Cooling" },
  { label: "Induction Boost 3.7kW", tag: "Cooking" },
]

export default function SideMenu({
  regions,
  locales,
  currentLocale,
  categories,
  enableCustomerAccounts = true,
  customer = null,
}: SideMenuProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [activeTab, setActiveTab] = useState<"departments" | "specs" | "concierge">("departments")
  const openMenu = () => setIsOpen(true)
  const closeMenu = () => setIsOpen(false)
  const languageToggleState = useToggleState(false)
  const countryToggleState = useToggleState(false)

  const customerInitials = customer
    ? `${customer.first_name?.[0] || ""}${customer.last_name?.[0] || ""}`.toUpperCase() || "ME"
    : null

  return (
    <div className="h-full flex items-center">
      {/* Menu Trigger Button */}
      <button
        type="button"
        onClick={openMenu}
        data-testid="nav-menu-button"
        className="flex items-center gap-2.5 text-xs font-semibold uppercase tracking-wider text-zinc-800 dark:text-zinc-200 hover:text-zinc-950 dark:hover:text-white transition-colors focus:outline-none py-2 px-1 cursor-pointer group"
        aria-label="Open navigation menu"
      >
        <span className="flex flex-col gap-1 w-4">
          <span className="h-0.5 w-4 bg-zinc-900 dark:bg-zinc-100 rounded-full transition-all group-hover:w-4" />
          <span className="h-0.5 w-3 bg-zinc-900 dark:bg-zinc-100 rounded-full transition-all group-hover:w-4" />
          <span className="h-0.5 w-4 bg-zinc-900 dark:bg-zinc-100 rounded-full transition-all group-hover:w-4" />
        </span>
        <span className="hidden sm:inline text-[11px] tracking-[0.15em] font-medium text-zinc-700 dark:text-zinc-300 group-hover:text-zinc-950 dark:group-hover:text-white transition-colors">
          Menu
        </span>
      </button>

      {/* Slide-Over Drawer using Headless UI Dialog */}
      <Transition show={isOpen} as={Fragment}>
        <Dialog as="div" className="relative z-[1000]" onClose={closeMenu}>
          {/* Backdrop with architectural blur */}
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
              className="fixed inset-0 bg-black/80 backdrop-blur-md transition-all"
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
                    className="pointer-events-auto w-screen max-w-sm sm:max-w-md bg-[#0A0B0E]/98 text-zinc-100 shadow-2xl flex flex-col justify-between p-6 sm:p-7 overflow-y-auto no-scrollbar border-r border-zinc-800/80 backdrop-blur-2xl relative"
                    data-testid="nav-menu-popup"
                  >
                    {/* Ambient subtle warm bronze lighting */}
                    <div className="absolute top-0 left-0 w-80 h-80 bg-gradient-to-br from-amber-500/[0.08] via-transparent to-transparent rounded-full blur-3xl pointer-events-none" />

                    {/* Top Content Area */}
                    <div className="flex flex-col gap-5 relative z-10">
                      {/* Brand Header */}
                      <div className="flex items-center justify-between pb-4 border-b border-zinc-800/70">
                        <div className="flex flex-col">
                          <div className="flex items-center gap-2">
                            <span className="text-lg font-bold tracking-[0.2em] uppercase text-white leading-none">
                              VENDRYX
                            </span>
                            <span className="px-2 py-0.5 rounded-full bg-amber-500/10 border border-amber-500/25 text-[9px] font-semibold text-amber-300 tracking-wider uppercase">
                              Appliance Console
                            </span>
                          </div>
                          <span className="text-[10px] tracking-widest text-zinc-400 uppercase mt-1 font-medium font-mono">
                            Engineering &amp; Infrastructure
                          </span>
                        </div>
                        <button
                          type="button"
                          data-testid="close-menu-button"
                          onClick={closeMenu}
                          aria-label="Close navigation menu"
                          className="w-8 h-8 rounded-full bg-white/[0.06] hover:bg-white/[0.12] active:scale-95 text-zinc-300 hover:text-white transition-all flex items-center justify-center border border-white/10 cursor-pointer"
                        >
                          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </div>

                      {/* Segmented Console Tabs */}
                      <div className="grid grid-cols-3 p-1 rounded-xl bg-white/[0.04] border border-white/[0.06] text-xs">
                        <button
                          type="button"
                          onClick={() => setActiveTab("departments")}
                          className={`py-1.5 px-2 rounded-lg font-medium text-[11px] tracking-wider uppercase transition-all ${
                            activeTab === "departments"
                              ? "bg-white text-zinc-950 font-semibold shadow-sm"
                              : "text-zinc-400 hover:text-zinc-200"
                          }`}
                        >
                          Departments
                        </button>
                        <button
                          type="button"
                          onClick={() => setActiveTab("specs")}
                          className={`py-1.5 px-2 rounded-lg font-medium text-[11px] tracking-wider uppercase transition-all ${
                            activeTab === "specs"
                              ? "bg-white text-zinc-950 font-semibold shadow-sm"
                              : "text-zinc-400 hover:text-zinc-200"
                          }`}
                        >
                          Specs &amp; Tech
                        </button>
                        <button
                          type="button"
                          onClick={() => setActiveTab("concierge")}
                          className={`py-1.5 px-2 rounded-lg font-medium text-[11px] tracking-wider uppercase transition-all ${
                            activeTab === "concierge"
                              ? "bg-white text-zinc-950 font-semibold shadow-sm"
                              : "text-zinc-400 hover:text-zinc-200"
                          }`}
                        >
                          Concierge
                        </button>
                      </div>

                      {/* Tab 1: Departments */}
                      {activeTab === "departments" && (
                        <div className="flex flex-col gap-4">
                          {/* Quick Search */}
                          <LocalizedClientLink
                            href="/store"
                            onClick={closeMenu}
                            className="flex items-center gap-3 px-3.5 py-2.5 rounded-xl bg-white/[0.03] hover:bg-white/[0.06] border border-white/[0.06] text-xs text-zinc-400 hover:text-zinc-200 transition-all group"
                          >
                            <svg className="w-4 h-4 text-zinc-500 group-hover:text-amber-400 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                              <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                            </svg>
                            <span>Search equipment, models, wattage...</span>
                            <span className="ml-auto text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/10 text-zinc-400">
                              Catalog
                            </span>
                          </LocalizedClientLink>

                          {/* Primary Navigation Shortcuts */}
                          <div className="grid grid-cols-2 gap-2">
                            <LocalizedClientLink
                              href="/store"
                              onClick={closeMenu}
                              className="flex items-center gap-2.5 p-2.5 rounded-xl bg-white/[0.03] hover:bg-white/[0.07] border border-white/[0.05] transition-all group"
                              data-testid="all-appliances-link"
                            >
                              <span className="w-7 h-7 rounded-lg bg-amber-500/10 text-amber-400 flex items-center justify-center text-xs">
                                ⚡
                              </span>
                              <div className="flex flex-col">
                                <span className="text-xs font-medium text-zinc-200 group-hover:text-white">Full Catalog</span>
                                <span className="text-[10px] text-zinc-500 font-mono">All Equipment</span>
                              </div>
                            </LocalizedClientLink>
                            <LocalizedClientLink
                              href="/cart"
                              onClick={closeMenu}
                              className="flex items-center gap-2.5 p-2.5 rounded-xl bg-white/[0.03] hover:bg-white/[0.07] border border-white/[0.05] transition-all group"
                              data-testid="bag-checkout-link"
                            >
                              <span className="w-7 h-7 rounded-lg bg-white/5 text-zinc-300 flex items-center justify-center text-xs">
                                🛒
                              </span>
                              <div className="flex flex-col">
                                <span className="text-xs font-medium text-zinc-200 group-hover:text-white">Cart &amp; Dispatch</span>
                                <span className="text-[10px] text-zinc-500 font-mono">Direct Delivery</span>
                              </div>
                            </LocalizedClientLink>
                          </div>

                          {/* Department List */}
                          <div className="space-y-2">
                            <div className="flex items-center justify-between px-1">
                              <span className="text-[10px] font-semibold uppercase tracking-[0.2em] text-zinc-400">
                                Equipment Categories
                              </span>
                              <LocalizedClientLink
                                href="/store"
                                onClick={closeMenu}
                                className="text-[11px] text-amber-400 hover:text-amber-300 font-medium transition-colors"
                              >
                                View all →
                              </LocalizedClientLink>
                            </div>
                            <div className="grid grid-cols-1 gap-2">
                              {categories && categories.length > 0
                                ? categories.map((cat) => (
                                    <LocalizedClientLink
                                      key={cat.id}
                                      href={`/categories/${cat.handle}`}
                                      onClick={closeMenu}
                                      className="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] hover:bg-white/[0.07] border border-white/[0.05] hover:border-white/[0.12] text-zinc-300 hover:text-white transition-all group"
                                    >
                                      <div className="flex items-center gap-3">
                                        <span className="w-8 h-8 rounded-lg bg-white/5 border border-white/10 flex items-center justify-center text-zinc-400 group-hover:text-amber-400 group-hover:bg-amber-500/10 transition-colors shrink-0">
                                          ⚡
                                        </span>
                                        <div className="flex flex-col">
                                          <span className="text-xs font-semibold text-zinc-200 group-hover:text-white">
                                            {cat.name}
                                          </span>
                                          <span className="text-[10px] text-zinc-500 font-mono">
                                            Equipment Line
                                          </span>
                                        </div>
                                      </div>
                                      <span className="text-zinc-500 group-hover:text-amber-400 group-hover:translate-x-0.5 transition-all text-sm">
                                        →
                                      </span>
                                    </LocalizedClientLink>
                                  ))
                                : APPLIANCE_DEPARTMENTS.map((dept) => (
                                    <LocalizedClientLink
                                      key={dept.handle}
                                      href={`/categories/${dept.handle}`}
                                      onClick={closeMenu}
                                      className="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] hover:bg-white/[0.07] border border-white/[0.05] hover:border-white/[0.12] text-zinc-300 hover:text-white transition-all group"
                                    >
                                      <div className="flex items-center gap-3 overflow-hidden">
                                        <span className="w-8 h-8 rounded-lg bg-white/5 border border-white/10 flex items-center justify-center text-zinc-400 group-hover:text-amber-400 group-hover:bg-amber-500/10 transition-colors shrink-0">
                                          {dept.icon}
                                        </span>
                                        <div className="flex flex-col overflow-hidden">
                                          <div className="flex items-center gap-2">
                                            <span className="text-xs font-semibold text-zinc-200 group-hover:text-white truncate">
                                              {dept.name}
                                            </span>
                                          </div>
                                          <span className="text-[10px] text-zinc-400 truncate">
                                            {dept.desc}
                                          </span>
                                        </div>
                                      </div>
                                      <span className="text-[9px] font-mono px-2 py-0.5 rounded bg-white/5 text-amber-300 border border-amber-400/20 shrink-0 ml-2">
                                        {dept.tag}
                                      </span>
                                    </LocalizedClientLink>
                                  ))}
                            </div>
                          </div>
                        </div>
                      )}

                      {/* Tab 2: Specifications & Tech Filters */}
                      {activeTab === "specs" && (
                        <div className="flex flex-col gap-4">
                          <div className="p-3.5 rounded-xl bg-white/[0.03] border border-white/[0.06] flex flex-col gap-2">
                            <span className="text-xs font-semibold text-zinc-200">
                              Direct Engineering Architecture
                            </span>
                            <p className="text-[11px] text-zinc-400 leading-relaxed">
                              Filter inventory by commercial motor types, thermal efficiency certifications, and smart automation standards.
                            </p>
                          </div>

                          <div className="space-y-2">
                            <span className="text-[10px] font-semibold uppercase tracking-[0.2em] text-zinc-400 px-1 block">
                              Technical Standards
                            </span>
                            <div className="grid grid-cols-1 gap-2">
                              {QUICK_FILTERS.map((f) => (
                                <LocalizedClientLink
                                  key={f.label}
                                  href={`/store?q=${encodeURIComponent(f.label)}`}
                                  onClick={closeMenu}
                                  className="flex items-center justify-between p-2.5 rounded-xl bg-white/[0.03] hover:bg-white/[0.07] border border-white/[0.05] transition-all group"
                                >
                                  <span className="text-xs font-medium text-zinc-300 group-hover:text-white">
                                    {f.label}
                                  </span>
                                  <span className="text-[9px] font-mono uppercase px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 group-hover:text-amber-300">
                                    {f.tag}
                                  </span>
                                </LocalizedClientLink>
                              ))}
                            </div>
                          </div>
                        </div>
                      )}

                      {/* Tab 3: Customer Concierge */}
                      {activeTab === "concierge" && enableCustomerAccounts && (
                        <div className="flex flex-col gap-4">
                          {customer ? (
                            <div className="p-4 rounded-2xl bg-white/[0.03] border border-white/[0.08] flex flex-col gap-3">
                              <div className="flex items-center gap-3">
                                <div className="w-10 h-10 rounded-xl bg-amber-500/15 border border-amber-500/30 flex items-center justify-center text-amber-300 font-semibold text-sm">
                                  {customerInitials}
                                </div>
                                <div className="flex flex-col overflow-hidden">
                                  <div className="flex items-center gap-2">
                                    <span className="text-xs font-semibold text-white truncate">
                                      {customer.first_name ? `${customer.first_name} ${customer.last_name || ""}` : customer.email}
                                    </span>
                                    <span className="px-1.5 py-0.5 rounded bg-amber-400/10 text-amber-300 text-[8px] font-mono">
                                      VERIFIED
                                    </span>
                                  </div>
                                  <span className="text-[10px] text-zinc-400 font-mono truncate">
                                    {customer.email}
                                  </span>
                                </div>
                              </div>
                              <div className="grid grid-cols-2 gap-2 pt-2 border-t border-white/[0.06]">
                                <LocalizedClientLink
                                  href="/account/orders"
                                  onClick={closeMenu}
                                  className="py-2 px-3 text-center rounded-lg bg-white/[0.05] hover:bg-white/[0.1] text-xs font-medium text-zinc-200 transition-colors"
                                >
                                  Track Orders
                                </LocalizedClientLink>
                                <LocalizedClientLink
                                  href="/account/addresses"
                                  onClick={closeMenu}
                                  className="py-2 px-3 text-center rounded-lg bg-white/[0.05] hover:bg-white/[0.1] text-xs font-medium text-zinc-200 transition-colors"
                                >
                                  Addresses
                                </LocalizedClientLink>
                              </div>
                            </div>
                          ) : (
                            <div className="p-4 rounded-2xl bg-white/[0.03] border border-white/[0.08] flex flex-col gap-3">
                              <div className="flex items-center gap-2">
                                <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
                                <span className="text-xs font-semibold text-zinc-200">
                                  Appliance Owner Concierge
                                </span>
                              </div>
                              <p className="text-[11px] text-zinc-400 leading-relaxed">
                                Access certified equipment records, schedule installation services, and track delivery logistics.
                              </p>
                              <div className="flex flex-col gap-2 pt-1">
                                <LocalizedClientLink
                                  href="/account"
                                  onClick={closeMenu}
                                  className="w-full py-2.5 text-center rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs tracking-wider transition-all shadow-sm"
                                >
                                  Sign In / Register Hardware
                                </LocalizedClientLink>
                                <LocalizedClientLink
                                  href="/account/orders"
                                  onClick={closeMenu}
                                  className="w-full py-2 text-center rounded-xl bg-white/[0.05] hover:bg-white/[0.1] text-zinc-300 font-medium text-xs border border-white/[0.08] transition-colors"
                                >
                                  Lookup Order by Tracking ID
                                </LocalizedClientLink>
                              </div>
                            </div>
                          )}

                          {/* Certified Promises (AGENTS.md Compliant — zero unsourced figures) */}
                          <div className="grid grid-cols-3 gap-2 py-3 px-3 rounded-xl bg-white/[0.02] border border-white/[0.05] text-[10px] text-zinc-400">
                            <div className="flex flex-col items-center text-center gap-1">
                              <span className="text-amber-400 text-sm">🛡️</span>
                              <span className="font-semibold text-zinc-200">Official</span>
                              <span className="text-[9px] text-zinc-500">Warranty</span>
                            </div>
                            <div className="flex flex-col items-center text-center gap-1 border-x border-white/10 px-1">
                              <span className="text-amber-400 text-sm">⚡</span>
                              <span className="font-semibold text-zinc-200">White-Glove</span>
                              <span className="text-[9px] text-zinc-500">Installation</span>
                            </div>
                            <div className="flex flex-col items-center text-center gap-1">
                              <span className="text-amber-400 text-sm">🔧</span>
                              <span className="font-semibold text-zinc-200">Certified</span>
                              <span className="text-[9px] text-zinc-500">Field Support</span>
                            </div>
                          </div>
                        </div>
                      )}
                    </div>

                    {/* Bottom Area: Localization & Legal */}
                    <div className="flex flex-col gap-y-3 pt-5 border-t border-zinc-800/80 text-xs mt-6 relative z-10">
                      <div className="space-y-2">
                        {!!locales?.length && (
                          <div
                            className="flex justify-between items-center py-2 px-3 rounded-xl bg-white/[0.03] hover:bg-white/[0.06] border border-white/[0.05] transition-colors"
                            onMouseEnter={languageToggleState.open}
                            onMouseLeave={languageToggleState.close}
                          >
                            <LanguageSelect
                              toggleState={languageToggleState}
                              locales={locales}
                              currentLocale={currentLocale}
                            />
                            <span className="text-zinc-500 text-xs">▼</span>
                          </div>
                        )}

                        {regions && (
                          <div
                            className="flex justify-between items-center py-2 px-3 rounded-xl bg-white/[0.03] hover:bg-white/[0.06] border border-white/[0.05] transition-colors"
                            onMouseEnter={countryToggleState.open}
                            onMouseLeave={countryToggleState.close}
                          >
                            <CountrySelect
                              toggleState={countryToggleState}
                              regions={regions}
                            />
                            <span className="text-zinc-500 text-xs">▼</span>
                          </div>
                        )}
                      </div>

                      <div className="pt-2 text-zinc-500 text-[10px] flex items-center justify-between font-mono">
                        <span>© {new Date().getFullYear()} VENDRYX APPLIANCES</span>
                        <span className="text-amber-400/80">Flagship Hardware</span>
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
