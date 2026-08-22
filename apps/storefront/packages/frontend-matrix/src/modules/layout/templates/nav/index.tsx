import { Suspense } from "react"
import type { ReactNode } from "react"
import type { Locale } from "@stheno/storefront-core"
import { StoreRegion } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import SideMenu from "@modules/layout/components/side-menu"
import Navbar from "./navbar"
import styles from "./nav.module.css"

/**
 * Server half of the ported Navbar. Core resolves regions, locales and the cart;
 * this hands them to the client half as slots so the cart's Suspense boundary
 * stays exactly where core put it.
 */
export default async function Nav({
  regions,
  locales,
  currentLocale,
  cartButton,
}: {
  regions: StoreRegion[]
  locales: Locale[] | null
  currentLocale: string | null
  cartButton: ReactNode
}) {
  return (
    <Navbar
      logoSrc="/frontends/matrix/logo.png"
      storeName="The Matrix Hours"
      links={[
        { label: "Home", href: "/" },
        { label: "Collection", href: "/store" },
        { label: "About", href: "#story" },
      ]}
      sideMenu={
        <SideMenu
          regions={regions}
          locales={locales}
          currentLocale={currentLocale}
        />
      }
      cartButton={
        <Suspense
          fallback={
            <LocalizedClientLink
              className={styles.iconButton}
              href="/cart"
              data-testid="nav-cart-link"
              aria-label="Cart"
            >
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden
              >
                <path d="M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4Z" />
                <path d="M3 6h18M16 10a4 4 0 0 1-8 0" />
              </svg>
            </LocalizedClientLink>
          }
        >
          {cartButton}
        </Suspense>
      }
    />
  )
}
