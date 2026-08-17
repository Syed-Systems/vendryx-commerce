import { Suspense } from "react"
import { listRegions } from "@lib/data/regions"
import { listLocales } from "@lib/data/locales"
import { getLocale } from "@lib/data/locale-actions"
import { StoreRegion } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import CartButton from "@modules/layout/components/cart-button"
import SideMenu from "@modules/layout/components/side-menu"

export default async function Nav() {
  const [regions, locales, currentLocale] = await Promise.all([
    listRegions().then((regions: StoreRegion[]) => regions),
    listLocales(),
    getLocale(),
  ])

  return (
    <div className="sticky top-0 inset-x-0 z-50 group">
      {/* Promotional / Announcement Bar */}
      <div className="bg-zinc-950 text-zinc-100 text-[11px] font-medium tracking-widest uppercase py-2 px-4 text-center flex items-center justify-center gap-2 border-b border-zinc-800">
        <span className="inline-block w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
        <span>Complimentary Express Shipping On Orders Over $150</span>
        <span className="hidden sm:inline text-zinc-500">•</span>
        <span className="hidden sm:inline text-zinc-300">Code: LUXE2026</span>
      </div>

      {/* Main Navigation Header */}
      <header className="relative h-16 sm:h-20 mx-auto border-b duration-200 bg-white/95 backdrop-blur-md border-zinc-200/80 transition-all">
        <nav className="content-container flex items-center justify-between w-full h-full text-small-regular">
          {/* Mobile menu trigger & Desktop Category links */}
          <div className="flex-1 basis-0 h-full flex items-center gap-x-6">
            <div className="h-full flex items-center">
              <SideMenu regions={regions} locales={locales} currentLocale={currentLocale} />
            </div>

            {/* Desktop Navigation Links */}
            <div className="hidden lg:flex items-center gap-x-8 text-sm font-medium tracking-wide text-zinc-700">
              <LocalizedClientLink
                href="/store"
                className="hover:text-zinc-950 transition-colors py-1 relative group-nav"
                data-testid="nav-store-link"
              >
                Shop All
              </LocalizedClientLink>
              <LocalizedClientLink
                href="/store?sortBy=created_at"
                className="hover:text-zinc-950 transition-colors py-1"
              >
                New Arrivals
              </LocalizedClientLink>
              <LocalizedClientLink
                href="/collections/summer-collection"
                className="hover:text-zinc-950 transition-colors py-1"
              >
                Collections
              </LocalizedClientLink>
            </div>
          </div>

          {/* Brand Logo / Logotype */}
          <div className="flex items-center justify-center h-full">
            <LocalizedClientLink
              href="/"
              className="flex flex-col items-center group py-2"
              data-testid="nav-store-link"
            >
              <span className="text-xl sm:text-2xl font-serif tracking-[0.25em] uppercase font-bold text-zinc-950 group-hover:opacity-80 transition-opacity">
                STHENO
              </span>
              <span className="text-[9px] tracking-[0.35em] text-zinc-500 uppercase -mt-1 font-sans">
                Atelier • Edition
              </span>
            </LocalizedClientLink>
          </div>

          {/* Right actions: Search, Account, Cart */}
          <div className="flex items-center gap-x-5 sm:gap-x-7 h-full flex-1 basis-0 justify-end">
            <LocalizedClientLink
              className="hidden sm:flex items-center gap-1.5 text-xs font-medium uppercase tracking-wider text-zinc-700 hover:text-zinc-950 transition-colors"
              href="/store"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={1.75}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
              <span>Search</span>
            </LocalizedClientLink>

            <LocalizedClientLink
              className="hidden small:flex items-center gap-1.5 text-xs font-medium uppercase tracking-wider text-zinc-700 hover:text-zinc-950 transition-colors"
              href="/account"
              data-testid="nav-account-link"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={1.75}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                />
              </svg>
              <span>Account</span>
            </LocalizedClientLink>

            <Suspense
              fallback={
                <LocalizedClientLink
                  className="hover:text-zinc-950 flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-zinc-800"
                  href="/cart"
                  data-testid="nav-cart-link"
                >
                  <span className="p-2 rounded-full bg-zinc-100 hover:bg-zinc-200 transition-colors">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      className="h-4 w-4"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                      strokeWidth={1.75}
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
              <CartButton />
            </Suspense>
          </div>
        </nav>
      </header>
    </div>
  )
}
