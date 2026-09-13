"use client"

import React, { useState, useRef, useEffect } from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { HttpTypes } from "@medusajs/types"

interface CategoryItem {
  id: string
  name: string
  handle: string
  category_children?: HttpTypes.StoreProductCategory[]
}

interface AppleDepartment {
  key: string
  label: string
  href: string
  exploreTitle: string
  exploreLinks: { name: string; href: string; badge?: string }[]
  shopLinks: { name: string; href: string }[]
  serviceTitle: string
  serviceLinks: { name: string; href: string; desc: string }[]
}

const APPLE_DEPARTMENTS: AppleDepartment[] = [
  {
    key: "store",
    label: "Store",
    href: "/store",
    exploreTitle: "Explore All Appliances",
    exploreLinks: [
      { name: "Shop the Latest Flagships", href: "/store?sortBy=created_at", badge: "New" },
      { name: "All Major Kitchen", href: "/categories/major-kitchen" },
      { name: "All Climate & Air", href: "/categories/climate-air" },
      { name: "All Floorcare & Steam", href: "/categories/floor-garment" },
      { name: "Power Systems & Energy", href: "/categories/electrical-power" },
    ],
    shopLinks: [
      { name: "Certified Factory Reconditioned", href: "/store" },
      { name: "Energy Star Rebate Eligible", href: "/store" },
      { name: "Commercial Inverter Series", href: "/store" },
    ],
    serviceTitle: "Special Services",
    serviceLinks: [
      { name: "White-Glove Delivery", href: "/store", desc: "Room-of-choice placement & unboxing" },
      { name: "Licensed Installation", href: "/store", desc: "Certified gas & 240V electrical hookup" },
      { name: "Haul-Away & Recycling", href: "/store", desc: "Safe decommission of old units" },
    ],
  },
  {
    key: "kitchen",
    label: "Kitchen",
    href: "/categories/major-kitchen",
    exploreTitle: "Explore Kitchen",
    exploreLinks: [
      { name: "Induction & Gas Ranges", href: "/categories/major-kitchen", badge: "Precision" },
      { name: "Built-in Convection Ovens", href: "/categories/major-kitchen" },
      { name: "Smart Inverter Refrigerators", href: "/categories/major-kitchen" },
      { name: "Silent Cycle Dishwashers", href: "/categories/major-kitchen" },
      { name: "Small Appliances & Espresso", href: "/categories/small-appliances" },
    ],
    shopLinks: [
      { name: "Custom Panel-Ready Models", href: "/categories/major-kitchen" },
      { name: "Dual-Fuel Commercial Cookers", href: "/categories/major-kitchen" },
      { name: "Under-Counter Wine Coolers", href: "/categories/major-kitchen" },
    ],
    serviceTitle: "Kitchen Engineering",
    serviceLinks: [
      { name: "Custom Cabinet Cutout Specs", href: "/store", desc: "Dimensional templates for contractors" },
      { name: "Water Filtration Systems", href: "/store", desc: "In-line reverse osmosis integration" },
    ],
  },
  {
    key: "climate",
    label: "Climate & Air",
    href: "/categories/climate-air",
    exploreTitle: "Explore Climate & Air",
    exploreLinks: [
      { name: "Multi-Stage HEPA Purifiers", href: "/categories/climate-air", badge: "QuietMark" },
      { name: "Inverter Heat Pump Systems", href: "/categories/climate-air" },
      { name: "Dual-Inverter Dehumidifiers", href: "/categories/climate-air" },
      { name: "Humidification & Aroma", href: "/categories/climate-air" },
    ],
    shopLinks: [
      { name: "Whole-Home Air Monitors", href: "/categories/climate-air" },
      { name: "Smart WiFi Thermostats", href: "/categories/climate-air" },
      { name: "Replacement Composite Filters", href: "/categories/climate-air" },
    ],
    serviceTitle: "Air Purity Advisory",
    serviceLinks: [
      { name: "Room Cubic-Volume Sizing", href: "/store", desc: "Calculated CADR performance match" },
      { name: "Filter Subscription Service", href: "/store", desc: "Auto-replenishment on schedule" },
    ],
  },
  {
    key: "floorcare",
    label: "Floorcare",
    href: "/categories/floor-garment",
    exploreTitle: "Explore Floorcare",
    exploreLinks: [
      { name: "Cordless Digital Motor Sticks", href: "/categories/floor-garment", badge: "Laser Detect" },
      { name: "LiDAR Robotic Mop & Vacuum", href: "/categories/floor-garment" },
      { name: "High-Pressure Steam Systems", href: "/categories/floor-garment" },
      { name: "Upright Deep Carpet Cleaners", href: "/categories/floor-garment" },
    ],
    shopLinks: [
      { name: "Self-Emptying Auto Docks", href: "/categories/floor-garment" },
      { name: "Specialized Pet Hair Toolkits", href: "/categories/floor-garment" },
    ],
    serviceTitle: "Machine Care",
    serviceLinks: [
      { name: "Extended Motor Coverage", href: "/store", desc: "Factory repair & replacement support" },
      { name: "Genuine Battery Replacements", href: "/store", desc: "Preserve peak runtime cycles" },
    ],
  },
  {
    key: "power",
    label: "Electrical & Power",
    href: "/categories/electrical-power",
    exploreTitle: "Explore Power & Living",
    exploreLinks: [
      { name: "Home Backup Inverters", href: "/categories/electrical-power", badge: "ESS" },
      { name: "Smart Circuit Breaker Panels", href: "/categories/electrical-power" },
      { name: "EV Charging Hardware", href: "/categories/electrical-power" },
      { name: "Surge & Power Conditioning", href: "/categories/electrical-power" },
    ],
    shopLinks: [
      { name: "Portable Solar Generators", href: "/categories/electrical-power" },
      { name: "High-Amperage Wall Receptacles", href: "/categories/electrical-power" },
    ],
    serviceTitle: "Certified Installation",
    serviceLinks: [
      { name: "Permit Filing Support", href: "/store", desc: "Local municipal compliance support" },
      { name: "Energy Load Calculation", href: "/store", desc: "Assess peak household amperage" },
    ],
  },
  {
    key: "support",
    label: "Support",
    href: "/store",
    exploreTitle: "Owner Support & Services",
    exploreLinks: [
      { name: "Equipment Registration", href: "/account" },
      { name: "User Manuals & Cut-Sheets", href: "/store" },
      { name: "Schedule In-Home Service", href: "/store", badge: "Expert" },
      { name: "Warranty Verification", href: "/account" },
    ],
    shopLinks: [
      { name: "Authorized Spare Parts", href: "/store" },
      { name: "Maintenance Lubricants & Cleaners", href: "/store" },
    ],
    serviceTitle: "Concierge Network",
    serviceLinks: [
      { name: "Virtual Technician Video Call", href: "/store", desc: "Instant remote diagnostic help" },
      { name: "Commercial Fleet Support", href: "/store", desc: "Dedicated account manager support" },
    ],
  },
]

export default function AppleNavbar({
  cartButton,
  categories,
  enableCustomerAccounts = true,
  customer,
}: {
  cartButton: React.ReactNode
  categories?: HttpTypes.StoreProductCategory[] | null
  enableCustomerAccounts?: boolean
  customer?: HttpTypes.StoreCustomer | null
}) {
  const [activeFlyout, setActiveFlyout] = useState<string | null>(null)
  const [searchOpen, setSearchOpen] = useState(false)
  const [searchQuery, setSearchQuery] = useState("")
  const [activeMobileCategory, setActiveMobileCategory] = useState<string>("all")
  const flyoutTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const searchInputRef = useRef<HTMLInputElement>(null)

  const handleMouseEnter = (key: string) => {
    if (flyoutTimeoutRef.current) {
      clearTimeout(flyoutTimeoutRef.current)
    }
    setActiveFlyout(key)
    setSearchOpen(false)
  }

  const handleMouseLeave = () => {
    flyoutTimeoutRef.current = setTimeout(() => {
      setActiveFlyout(null)
    }, 180)
  }

  const toggleSearch = () => {
    setSearchOpen((prev) => {
      const next = !prev
      if (next) {
        setActiveFlyout(null)
        setTimeout(() => searchInputRef.current?.focus(), 100)
      }
      return next
    })
  }

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setActiveFlyout(null)
        setSearchOpen(false)
      }
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [])

  const currentDepartment = APPLE_DEPARTMENTS.find((d) => d.key === activeFlyout)

  // Mobile horizontal categories list
  const mobilePills = [
    { id: "all", name: "All Store", href: "/store" },
    ...(categories && categories.length > 0
      ? categories
          .filter((c) => !c.parent_category_id)
          .map((c) => ({ id: c.id, name: c.name, href: `/categories/${c.handle}` }))
      : [
          { id: "kitchen", name: "Kitchen", href: "/categories/major-kitchen" },
          { id: "small", name: "Small Appliances", href: "/categories/small-appliances" },
          { id: "climate", name: "Climate & Air", href: "/categories/climate-air" },
          { id: "floorcare", name: "Floorcare", href: "/categories/floor-garment" },
          { id: "power", name: "Power Systems", href: "/categories/electrical-power" },
        ]),
  ]

  return (
    <div
      className="relative z-50 font-sans"
      onMouseLeave={handleMouseLeave}
    >
      {/* ------------------------------------------------------------- */}
      {/* 1. Global Apple Bar (48px exact height, ultra frosted glass) */}
      {/* ------------------------------------------------------------- */}
      <header className="relative h-12 w-full bg-[#161617]/92 text-[#F5F5F7] backdrop-blur-2xl border-b border-white/[0.08] transition-colors duration-300">
        <div className="max-w-[1024px] mx-auto h-full px-4 sm:px-6 flex items-center justify-between text-[12px] tracking-[-0.01em]">
          {/* Brand Mark (Apple Minimalist Aesthetic) */}
          <LocalizedClientLink
            href="/"
            className="flex items-center gap-2 group py-1.5 focus-visible:outline-none shrink-0"
            aria-label="Vendryx Appliances Homepage"
          >
            {/* Apple-grade geometric precision mark */}
            <div className="w-4 h-4 rounded-[4px] bg-white/15 border border-white/30 flex items-center justify-center text-[9px] font-mono font-bold tracking-tighter text-white group-hover:bg-white group-hover:text-black transition-all">
              V
            </div>
            <span className="text-[13px] font-semibold tracking-[-0.02em] text-[#F5F5F7] group-hover:text-white transition-colors">
              VENDRYX
            </span>
          </LocalizedClientLink>

          {/* Desktop Apple Category Row (Zero Hamburger Menu) */}
          <nav
            className="hidden md:flex items-center space-x-7 lg:space-x-9 text-[12px] font-normal text-[#C7C7CC] h-full"
            aria-label="Global Store Navigation"
          >
            {APPLE_DEPARTMENTS.map((dept) => {
              const isActive = activeFlyout === dept.key
              return (
                <div
                  key={dept.key}
                  onMouseEnter={() => handleMouseEnter(dept.key)}
                  className="h-full flex items-center"
                >
                  <LocalizedClientLink
                    href={dept.href}
                    className={`transition-colors duration-150 py-1 ${
                      isActive
                        ? "text-white font-medium"
                        : "hover:text-white text-[#C7C7CC]"
                    }`}
                  >
                    {dept.label}
                  </LocalizedClientLink>
                </div>
              )
            })}
          </nav>

          {/* Action Icons (Search, Account, Bag) */}
          <div className="flex items-center gap-x-5 text-[#C7C7CC] shrink-0">
            {/* Search Trigger */}
            <button
              type="button"
              onClick={toggleSearch}
              aria-label="Search Appliances"
              className="p-1 hover:text-white transition-colors focus-visible:outline-none"
            >
              <svg
                width="15"
                height="15"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="7" cy="7" r="5" />
                <path d="M11 11L14.5 14.5" />
              </svg>
            </button>

            {/* Account / User Portal */}
            {enableCustomerAccounts && (
              <LocalizedClientLink
                href="/account"
                aria-label="Account and Orders"
                className="p-1 hover:text-white transition-colors hidden sm:inline-flex"
              >
                <svg
                  width="15"
                  height="15"
                  viewBox="0 0 16 16"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <circle cx="8" cy="5" r="3" />
                  <path d="M2.5 14.5C2.5 11.5 5 9.5 8 9.5C11 9.5 13.5 11.5 13.5 14.5" />
                </svg>
              </LocalizedClientLink>
            )}

            {/* Bag / Cart with live badge */}
            <div className="flex items-center h-full">
              {cartButton}
            </div>
          </div>
        </div>

        {/* ----------------------------------------------------------- */}
        {/* Apple In-Header Search Overlay                              */}
        {/* ----------------------------------------------------------- */}
        {searchOpen && (
          <div className="absolute top-12 inset-x-0 bg-[#161617]/98 backdrop-blur-2xl border-b border-white/[0.1] shadow-2xl transition-all duration-200 py-6 px-4">
            <div className="max-w-[680px] mx-auto flex flex-col gap-4">
              <div className="relative flex items-center">
                <svg
                  className="absolute left-3.5 text-zinc-400"
                  width="16"
                  height="16"
                  viewBox="0 0 16 16"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                >
                  <circle cx="7" cy="7" r="5" />
                  <path d="M11 11L14.5 14.5" />
                </svg>
                <input
                  ref={searchInputRef}
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search appliances, models, specifications..."
                  className="w-full bg-[#242426] text-white pl-10 pr-10 py-2.5 rounded-xl text-[14px] placeholder-zinc-500 border border-white/[0.08] focus:outline-none focus:border-white/30"
                />
                <button
                  type="button"
                  onClick={() => setSearchOpen(false)}
                  className="absolute right-3 text-xs text-zinc-400 hover:text-white px-1.5 py-0.5"
                >
                  ESC
                </button>
              </div>

              {/* Quick Jump Suggestions */}
              <div className="flex items-center gap-2 overflow-x-auto text-[11px] text-zinc-400 pt-1">
                <span className="text-zinc-500 shrink-0">Quick Links:</span>
                <LocalizedClientLink
                  href="/categories/major-kitchen"
                  onClick={() => setSearchOpen(false)}
                  className="px-2.5 py-1 rounded-full bg-white/5 hover:bg-white/10 hover:text-white shrink-0"
                >
                  Induction Cooktops
                </LocalizedClientLink>
                <LocalizedClientLink
                  href="/categories/climate-air"
                  onClick={() => setSearchOpen(false)}
                  className="px-2.5 py-1 rounded-full bg-white/5 hover:bg-white/10 hover:text-white shrink-0"
                >
                  HEPA Air Purifiers
                </LocalizedClientLink>
                <LocalizedClientLink
                  href="/categories/floor-garment"
                  onClick={() => setSearchOpen(false)}
                  className="px-2.5 py-1 rounded-full bg-white/5 hover:bg-white/10 hover:text-white shrink-0"
                >
                  Cordless Stick Vacuums
                </LocalizedClientLink>
                <LocalizedClientLink
                  href="/categories/electrical-power"
                  onClick={() => setSearchOpen(false)}
                  className="px-2.5 py-1 rounded-full bg-white/5 hover:bg-white/10 hover:text-white shrink-0"
                >
                  Home Backup Power
                </LocalizedClientLink>
              </div>
            </div>
          </div>
        )}
      </header>

      {/* ------------------------------------------------------------- */}
      {/* 2. Apple Full-Width Flyout Curtain (Desktop Hover/Focus)     */}
      {/* ------------------------------------------------------------- */}
      {activeFlyout && currentDepartment && (
        <div
          onMouseEnter={() => handleMouseEnter(activeFlyout)}
          onMouseLeave={handleMouseLeave}
          className="hidden md:block absolute top-12 inset-x-0 bg-[#161617]/98 backdrop-blur-3xl border-b border-white/[0.1] shadow-2xl transition-all duration-200 z-40"
        >
          <div className="max-w-[1024px] mx-auto py-10 px-8">
            <div className="grid grid-cols-12 gap-10">
              {/* Column 1: Explore Categories (Large Apple typography) */}
              <div className="col-span-5 flex flex-col gap-3">
                <span className="text-[11px] font-semibold tracking-wider uppercase text-[#86868B]">
                  {currentDepartment.exploreTitle}
                </span>
                <div className="flex flex-col gap-2.5 pt-1">
                  {currentDepartment.exploreLinks.map((item, idx) => (
                    <LocalizedClientLink
                      key={idx}
                      href={item.href}
                      onClick={() => setActiveFlyout(null)}
                      className="group flex items-center justify-between text-[16px] font-semibold text-[#F5F5F7] hover:text-white tracking-[-0.02em] transition-colors"
                    >
                      <span>{item.name}</span>
                      {item.badge && (
                        <span className="text-[9px] font-medium tracking-wide uppercase px-1.5 py-0.5 rounded-full bg-amber-400/15 text-amber-400 border border-amber-400/30">
                          {item.badge}
                        </span>
                      )}
                    </LocalizedClientLink>
                  ))}
                </div>
              </div>

              {/* Column 2: Shop Department */}
              <div className="col-span-4 flex flex-col gap-3 border-l border-white/[0.08] pl-8">
                <span className="text-[11px] font-semibold tracking-wider uppercase text-[#86868B]">
                  Specialized Hardware
                </span>
                <div className="flex flex-col gap-2 pt-1 text-[13px] text-[#A1A1A6]">
                  {currentDepartment.shopLinks.map((shop, idx) => (
                    <LocalizedClientLink
                      key={idx}
                      href={shop.href}
                      onClick={() => setActiveFlyout(null)}
                      className="hover:text-white transition-colors py-1"
                    >
                      {shop.name}
                    </LocalizedClientLink>
                  ))}
                </div>
              </div>

              {/* Column 3: White-Glove Concierge / Services */}
              <div className="col-span-3 flex flex-col gap-3 border-l border-white/[0.08] pl-8">
                <span className="text-[11px] font-semibold tracking-wider uppercase text-[#86868B]">
                  {currentDepartment.serviceTitle}
                </span>
                <div className="flex flex-col gap-3.5 pt-1">
                  {currentDepartment.serviceLinks.map((srv, idx) => (
                    <LocalizedClientLink
                      key={idx}
                      href={srv.href}
                      onClick={() => setActiveFlyout(null)}
                      className="group flex flex-col text-start"
                    >
                      <span className="text-[13px] font-medium text-[#F5F5F7] group-hover:text-white transition-colors">
                        {srv.name}
                      </span>
                      <span className="text-[11px] text-[#86868B] leading-snug">
                        {srv.desc}
                      </span>
                    </LocalizedClientLink>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Dimming Apple Backdrop Scrim when flyout is open */}
      {activeFlyout && (
        <div
          className="fixed inset-0 top-12 bg-black/40 backdrop-blur-xs z-30 pointer-events-auto"
          onClick={() => setActiveFlyout(null)}
        />
      )}

      {/* ------------------------------------------------------------- */}
      {/* 3. Apple Mobile Ribbon (NO HAMBURGER MENU!)                   */}
      {/* ------------------------------------------------------------- */}
      <div className="md:hidden bg-[#1D1D1F] border-b border-white/[0.08] px-3 py-2">
        <div className="flex items-center gap-1.5 overflow-x-auto no-scrollbar scroll-smooth">
          {mobilePills.map((pill) => {
            const isCurrent = activeMobileCategory === pill.id
            return (
              <LocalizedClientLink
                key={pill.id}
                href={pill.href}
                onClick={() => setActiveMobileCategory(pill.id)}
                className={`px-3 py-1 rounded-full text-[12px] whitespace-nowrap transition-all shrink-0 ${
                  isCurrent
                    ? "bg-white text-black font-semibold shadow-xs"
                    : "bg-[#2D2D2F] text-[#D1D1D6] hover:text-white"
                }`}
              >
                {pill.name}
              </LocalizedClientLink>
            )
          })}
        </div>
      </div>

      {/* ------------------------------------------------------------- */}
      {/* 4. Apple Secondary Local Navigation Ribbon (Flagship Specs)   */}
      {/* ------------------------------------------------------------- */}
      <div className="hidden lg:block bg-white/95 dark:bg-[#121316]/95 border-b border-zinc-200/80 dark:border-white/[0.06] backdrop-blur-md">
        <div className="max-w-[1024px] mx-auto h-11 px-6 flex items-center justify-between text-xs">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-zinc-900 dark:text-zinc-100 tracking-[-0.01em]">
              Electrical &amp; Living Flagship
            </span>
            <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400">
              Commercial Grade
            </span>
          </div>

          <div className="flex items-center gap-6 text-[12px] text-zinc-600 dark:text-zinc-400">
            <LocalizedClientLink
              href="/store"
              className="hover:text-zinc-950 dark:hover:text-white transition-colors"
            >
              Overview
            </LocalizedClientLink>
            <LocalizedClientLink
              href="/store?sortBy=created_at"
              className="hover:text-zinc-950 dark:hover:text-white transition-colors"
            >
              Tech Specs
            </LocalizedClientLink>
            <LocalizedClientLink
              href="/store"
              className="hover:text-zinc-950 dark:hover:text-white transition-colors"
            >
              Compare
            </LocalizedClientLink>
            <LocalizedClientLink
              href="/store"
              className="px-3 py-1 rounded-full bg-zinc-900 dark:bg-white text-white dark:text-zinc-950 font-medium text-[11px] hover:opacity-90 transition-opacity"
            >
              Shop Catalog
            </LocalizedClientLink>
          </div>
        </div>
      </div>
    </div>
  )
}
