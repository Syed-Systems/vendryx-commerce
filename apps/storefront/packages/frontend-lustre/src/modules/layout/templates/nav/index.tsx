import { Suspense } from "react"
import type { ReactNode } from "react"
import type { Locale } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import SideMenu from "@modules/layout/components/side-menu"
import HeaderRegionSelect from "@modules/layout/components/header-region-select"
import { User } from "@phosphor-icons/react/dist/ssr"
import NavShell from "./nav-shell"

export type NavProps = {
  regions: HttpTypes.StoreRegion[]
  locales: Locale[] | null
  currentLocale: string | null
  cartButton: ReactNode
}

/**
 * Lustre Nav (Contract Slot 42).
 * 72px floating header with frosted material, high contrast, clean typography,
 * and Apple-informed hierarchy matching FRONTEND_LUSTRE_PLAN §4.1:
 * - Left: approved merchant wordmark or store name
 * - Center: store, collections, and category entry points received from core
 * - Right: locale, region, account, and bag
 * - Mobile: wordmark, search if supplied by core, and bag remain visible. Navigation opens as a full-height Headless UI dialog.
 */
export default async function Nav({
  regions,
  locales,
  currentLocale,
  cartButton,
}: NavProps) {
  return (
    <header className="sticky top-0 inset-x-0 z-40 w-full p-4 pointer-events-none">
      <NavShell>
        {/* Left: Approved merchant wordmark or store name */}
        <div className="flex items-center flex-1 basis-0">
          <LocalizedClientLink
            href="/"
            className="flex flex-col group py-1"
            data-testid="nav-home-link"
          >
            <span className="text-base sm:text-lg font-semibold tracking-[0.2em] uppercase text-[var(--text-primary)] group-hover:opacity-80 transition-opacity">
              Store
            </span>
            <span className="text-[9px] tracking-[0.25em] text-[var(--text-muted)] uppercase -mt-0.5">
              Watches &amp; Objects
            </span>
          </LocalizedClientLink>
        </div>

        {/* Center: store, collections, and category entry points */}
        <div className="hidden md:flex items-center justify-center gap-8 text-xs font-medium tracking-[0.15em] uppercase text-[var(--text-secondary)]">
          <LocalizedClientLink
            href="/store"
            className="hover:text-[var(--text-primary)] transition-colors py-1"
            data-testid="nav-store-link"
          >
            Catalog
          </LocalizedClientLink>
          <LocalizedClientLink
            href="/store?sortBy=created_at"
            className="hover:text-[var(--text-primary)] transition-colors py-1"
          >
            Latest Drops
          </LocalizedClientLink>
        </div>

        {/* Right: locale, region, account, and bag */}
        <div className="flex items-center justify-end gap-3 sm:gap-4 flex-1 basis-0">
          {/* Desktop Region Selector */}
          <div className="hidden md:flex items-center">
            <HeaderRegionSelect regions={regions} />
          </div>

          {/* Account */}
          <LocalizedClientLink
            href="/account"
            aria-label="Account Overview"
            data-testid="nav-account-link"
            className="p-2 rounded-full text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-raised)] transition-colors lustre-btn-press"
          >
            <User size={18} />
          </LocalizedClientLink>

          {/* Bag */}
          <Suspense
            fallback={
              <LocalizedClientLink
                href="/cart"
                data-testid="nav-cart-link"
                className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium bg-[var(--bg-raised)] text-[var(--text-primary)]"
              >
                <span>Bag</span>
              </LocalizedClientLink>
            }
          >
            {cartButton}
          </Suspense>

          {/* Mobile Navigation Trigger (md:hidden) */}
          <div className="md:hidden flex items-center">
            <SideMenu
              regions={regions}
              locales={locales}
              currentLocale={currentLocale}
            />
          </div>
        </div>
      </NavShell>
    </header>
  )
}
