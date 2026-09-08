import { Suspense } from "react"
import type { ReactNode } from "react"
import type { Locale } from "@stheno/storefront-core"
import { HttpTypes, StoreRegion } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import SideMenu from "@modules/layout/components/side-menu"

const DEFAULT_TECH_CATEGORIES = [
  { id: "phones", name: "Phones", handle: "phones" },
  { id: "laptops", name: "Laptops & Tablets", handle: "laptops" },
  { id: "audio", name: "Audio & Sound", handle: "audio" },
  { id: "wearables", name: "Watches & Wearables", handle: "wearables" },
  { id: "accessories", name: "Power & Accessories", handle: "accessories" },
]

/**
 * Apple HIG Navigation Header with dynamic category strip and optional account gating.
 * ADR-013 & ADR-031 compliant.
 */
export default async function Nav({
  regions,
  locales,
  currentLocale,
  cartButton,
  categories,
  enableCustomerAccounts = false,
}: {
  regions: StoreRegion[]
  locales: Locale[] | null
  currentLocale: string | null
  cartButton: ReactNode
  categories?: HttpTypes.StoreProductCategory[] | null
  enableCustomerAccounts?: boolean
}) {
  const resolvedCategories =
    categories && categories.length > 0
      ? categories.filter((c) => !c.parent_category_id)
      : DEFAULT_TECH_CATEGORIES

  return (
    <div className="sticky top-0 inset-x-0 z-50 group">
      {/* Promotional / Announcement Bar */}
      <div className="bg-zinc-950 text-zinc-100 text-[11px] font-medium tracking-wide py-2 px-4 text-center flex items-center justify-center gap-2 border-b border-zinc-800">
        <span className="inline-block w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse" />
        <span>Complimentary Express Delivery & 2-Year Hardware Warranty</span>
        <span className="hidden sm:inline text-zinc-500">•</span>
        <span className="hidden sm:inline text-zinc-400">Trade-in Available on All Devices</span>
      </div>

      {/* Main Navigation Header */}
      <header className="relative h-16 sm:h-18 mx-auto border-b bg-white/80 dark:bg-zinc-950/80 backdrop-blur-xl border-zinc-200/80 dark:border-zinc-800 transition-all duration-300">
        <nav className="content-container flex items-center justify-between w-full h-full text-small-regular">
          {/* Mobile menu trigger & Desktop quick links */}
          <div className="flex-1 basis-0 h-full flex items-center gap-x-6">
            <div className="h-full flex items-center">
              <SideMenu
                regions={regions}
                locales={locales}
                currentLocale={currentLocale}
                categories={resolvedCategories}
                enableCustomerAccounts={enableCustomerAccounts}
              />
            </div>

            {/* Desktop Navigation Links */}
            <div className="hidden lg:flex items-center gap-x-7 text-sm font-medium tracking-tight text-zinc-600 dark:text-zinc-300">
              <LocalizedClientLink
                href="/store"
                className="hover:text-zinc-950 dark:hover:text-white transition-colors py-1"
                data-testid="nav-store-link"
              >
                All Tech
              </LocalizedClientLink>
              <LocalizedClientLink
                href="/store?sortBy=created_at"
                className="hover:text-zinc-950 dark:hover:text-white transition-colors py-1"
              >
                Latest Releases
              </LocalizedClientLink>
            </div>
          </div>

          {/* Brand Logo / Tech Identity */}
          <div className="flex items-center justify-center h-full">
            <LocalizedClientLink
              href="/"
              className="flex flex-col items-center group py-2"
              data-testid="nav-store-link"
            >
              <span className="text-lg sm:text-xl font-bold tracking-[0.2em] uppercase text-zinc-950 dark:text-zinc-50 group-hover:opacity-80 transition-opacity">
                VENDRYX
              </span>
              <span className="text-[9px] tracking-[0.3em] text-zinc-400 uppercase font-mono -mt-0.5">
                Pro Tech Store
              </span>
            </LocalizedClientLink>
          </div>

          {/* Right actions: Search, Account (conditionally hidden), Cart */}
          <div className="flex items-center gap-x-4 sm:gap-x-6 h-full flex-1 basis-0 justify-end">
            <LocalizedClientLink
              className="flex items-center gap-1.5 text-xs font-medium text-zinc-700 dark:text-zinc-300 hover:text-zinc-950 dark:hover:text-white transition-colors p-1.5 rounded-full hover:bg-zinc-100 dark:hover:bg-zinc-900"
              href="/store"
              aria-label="Search tech catalog"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
              <span className="hidden sm:inline">Search</span>
            </LocalizedClientLink>

            {/* Customer Account Button — gated by enableCustomerAccounts */}
            {enableCustomerAccounts && (
              <LocalizedClientLink
                className="hidden small:flex items-center gap-1.5 text-xs font-medium text-zinc-700 dark:text-zinc-300 hover:text-zinc-950 dark:hover:text-white transition-colors p-1.5 rounded-full hover:bg-zinc-100 dark:hover:bg-zinc-900"
                href="/account"
                data-testid="nav-account-link"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-4 w-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  strokeWidth={2}
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                  />
                </svg>
                <span>Account</span>
              </LocalizedClientLink>
            )}

            {/* Cart Button */}
            <Suspense
              fallback={
                <LocalizedClientLink
                  className="hover:text-zinc-950 flex items-center gap-2 text-xs font-medium text-zinc-800 dark:text-zinc-200"
                  href="/cart"
                  data-testid="nav-cart-link"
                >
                  <span className="p-2 rounded-full bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      className="h-4 w-4"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                      strokeWidth={2}
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        d="M16 11V7a4 4 0 00-8 0v4M5 9h14l1 12H4L5 9z"
                      />
                    </svg>
                  </span>
                  <span>Bag (0)</span>
                </LocalizedClientLink>
              }
            >
              {cartButton}
            </Suspense>
          </div>
        </nav>
      </header>

      {/* Dynamic Top Category Navigation Strip (Apple HIG Segmented Pill Bar) */}
      <div className="w-full bg-[#F5F5F7]/95 dark:bg-[#161617]/95 backdrop-blur-md border-b border-zinc-200/70 dark:border-zinc-800/80 py-2.5 overflow-x-auto no-scrollbar">
        <div className="content-container flex items-center gap-2 sm:gap-2.5 text-xs whitespace-nowrap">
          <LocalizedClientLink
            href="/store"
            className="px-4 py-1.5 rounded-full bg-zinc-950 text-white dark:bg-white dark:text-zinc-950 font-medium hover:opacity-90 active:scale-95 transition-all shadow-sm"
          >
            All Tech
          </LocalizedClientLink>
          {resolvedCategories.map((cat) => (
            <LocalizedClientLink
              key={cat.id}
              href={`/categories/${cat.handle}`}
              className="px-3.5 py-1.5 rounded-full bg-white dark:bg-zinc-800/90 text-zinc-700 dark:text-zinc-300 font-medium hover:text-zinc-950 dark:hover:text-white hover:bg-zinc-200/70 dark:hover:bg-zinc-700 active:scale-95 transition-all border border-zinc-200/80 dark:border-zinc-700/80 shadow-2xs"
            >
              {cat.name}
            </LocalizedClientLink>
          ))}
        </div>
      </div>
    </div>
  )
}
