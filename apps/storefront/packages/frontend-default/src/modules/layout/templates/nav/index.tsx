import { Suspense } from "react"
import type { ReactNode } from "react"
import type { Locale } from "@stheno/storefront-core"
import { HttpTypes, StoreRegion } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import GlassNavShell from "@modules/layout/components/glass-nav"
import SideMenu from "@modules/layout/components/side-menu"

/**
 * Presentation only. Core resolves regions, locales and the cart, per ADR-013
 * decision 4 — the cart arrives as a rendered slot so its Suspense boundary
 * stays exactly where it was.
 */
export default async function Nav({
  regions,
  locales,
  currentLocale,
  cartButton,
  storeName,
}: {
  regions: StoreRegion[]
  locales: Locale[] | null
  currentLocale: string | null
  cartButton: ReactNode
  categories?: HttpTypes.StoreProductCategory[] | null
  enableCustomerAccounts?: boolean
  customer?: HttpTypes.StoreCustomer | null
  /** Tenant's resolved display name. Core supplies it (ADR-013 decision 4). */
  storeName: string
}) {

  return (
    <GlassNavShell
      announcement={
        /* The audit flagged this bar for rendering unsourced figures — a
           "$150" threshold and a "LUXE2026" code, both invented, in the exact
           stat position AGENTS.md's standing rule bans. It now carries brand
           copy with no number in it, and the whole bar is the link, so the
           click target is the full width rather than an inert sentence. */
        <LocalizedClientLink
          href="/store"
          className="group flex min-h-[44px] items-center justify-center gap-2 bg-[color:var(--chrome-bg)] px-4 py-2.5 text-center text-[0.6875rem] font-semibold uppercase tracking-[0.22em] indent-[0.22em] text-[color:var(--chrome-fg)]"
        >
          <span
            aria-hidden="true"
            className="inline-block size-1.5 shrink-0 rounded-full bg-emerald-400 motion-safe:animate-pulse"
          />
          <span>New season, made to measure</span>
          <span aria-hidden="true" className="text-[color:var(--chrome-fg-faint)]">—</span>
          <span className="underline decoration-[color:var(--chrome-edge)] underline-offset-4 group-hover:decoration-[color:var(--chrome-fg)]">
            Explore the atelier
          </span>
        </LocalizedClientLink>
      }
    >
      <nav className="content-container flex items-center justify-between w-full h-full">
          {/* Mobile menu trigger & Desktop Category links */}
          <div className="flex-1 basis-0 min-w-0 h-full flex items-center gap-x-6">
            <div className="h-full flex items-center shrink-0">
              <SideMenu regions={regions} locales={locales} currentLocale={currentLocale} />
            </div>

            {/* Desktop Navigation Links.

                `whitespace-nowrap` is load-bearing. These are `label-nav`
                uppercase at `0.14em` tracking, and the two sides are
                `flex-1 basis-0` — so a wide wordmark between them squeezes
                them until the link text breaks across two lines and collides
                with the wordmark. They must not wrap; the layout gives up
                space elsewhere first (see the side containers' `min-w-0`). */}
            <div className="hidden lg:flex items-center gap-x-8 label-nav text-[color:var(--chrome-fg-muted)] whitespace-nowrap">
              <LocalizedClientLink
                href="/store"
                className="tap hover:text-[color:var(--chrome-fg)] transition-colors relative group-nav whitespace-nowrap"
                data-testid="nav-store-link"
              >
                Shop All
              </LocalizedClientLink>
              <LocalizedClientLink
                href="/store?sortBy=created_at"
                className="tap hover:text-[color:var(--chrome-fg)] transition-colors whitespace-nowrap"
              >
                New Arrivals
              </LocalizedClientLink>
              <LocalizedClientLink
                href="/collections/summer-collection"
                className="tap hover:text-[color:var(--chrome-fg)] transition-colors whitespace-nowrap"
              >
                Collections
              </LocalizedClientLink>
            </div>
          </div>

          {/* Brand Logo / Logotype.

                `min-w-0` + `truncate`: a tenant name is free text and can be any
                length. The two flanking sides are `flex-1 basis-0`, so a long
                name would otherwise squeeze them until their links wrapped. The
                wordmark yields first — `truncate` with `title` for the full
                name on hover.

                Tracking is `0.08em`, not the `0.25em` this used to carry. That
                value was tuned against the fixed six-character "STHENO"; on a
                13-character "Stheno Store" it added ~50px of pure letter-space
                and pushed the nav links into wrapping. Short names still read as
                letterspaced caps, and long ones now fit. */}
          <div className="flex items-center justify-center h-full min-w-0 px-2">
            <LocalizedClientLink
              href="/"
              className="group py-2 min-w-0 max-w-full"
              title={storeName}
              data-testid="nav-store-link"
            >
              <span className="block truncate text-lg sm:text-xl font-heading tracking-[0.08em] indent-[0.08em] uppercase font-bold leading-none text-[color:var(--chrome-fg)] group-hover:opacity-70 transition-opacity">
                {storeName}
              </span>
              <span className="block text-[9px] leading-tight tracking-[0.28em] indent-[0.28em] text-[color:var(--chrome-fg-faint)] uppercase font-sans">
                Atelier • Edition
              </span>
            </LocalizedClientLink>
          </div>

          {/* Right actions: Search, Account, Cart */}
          <div className="flex items-center gap-x-4 sm:gap-x-6 h-full flex-1 basis-0 min-w-0 justify-end">
            {/* Action labels are hidden below `small` via `sr-only`, and from
                `small` up they must stay on one line — a wrapped "SEARCH" beside
                an icon reads as two stacked items. */}
            <div className="flex min-w-0 items-center gap-x-4 sm:gap-x-6 justify-end">
            {/* Search and Account were `text-xs` with no vertical padding — an
                18px tap target. `.tap` restores the 44px minimum. */}
            <LocalizedClientLink
              className="tap sm:flex items-center gap-1.5 label-nav whitespace-nowrap text-[color:var(--chrome-fg-muted)] hover:text-[color:var(--chrome-fg)] transition-colors max-sm:sr-only"
              href="/store"
            >
              <svg
                aria-hidden="true"
                xmlns="http://www.w3.org/2000/svg"
                className="size-4"
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
              className="tap small:flex items-center gap-1.5 label-nav whitespace-nowrap text-[color:var(--chrome-fg-muted)] hover:text-[color:var(--chrome-fg)] transition-colors max-small:sr-only"
              href="/account"
              data-testid="nav-account-link"
            >
              <svg
                aria-hidden="true"
                xmlns="http://www.w3.org/2000/svg"
                className="size-4"
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
                  className="tap hover:text-[color:var(--chrome-fg)] items-center gap-2 label-nav whitespace-nowrap text-[color:var(--chrome-fg-muted)]"
                  href="/cart"
                  data-testid="nav-cart-link"
                >
                  <span className="p-2 rounded-full bg-[color:var(--chrome-bg)] hover:bg-[color:var(--chrome-fg)] hover:text-[color:var(--chrome-bg)] transition-colors">
                    <svg
                      aria-hidden="true"
                      xmlns="http://www.w3.org/2000/svg"
                      className="size-4"
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
              {cartButton}
            </Suspense>
            </div>
          </div>
        </nav>
    </GlassNavShell>
  )
}
