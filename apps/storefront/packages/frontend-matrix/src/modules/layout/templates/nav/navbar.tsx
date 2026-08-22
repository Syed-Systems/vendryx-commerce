"use client"

import { useEffect, useState } from "react"
import type { ReactNode } from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import useScrollDirection from "./use-scroll-direction"
import styles from "./nav.module.css"

/**
 * PORT of The Matrix Hours `Navbar` — its structure (left / center / right),
 * its scroll-away behaviour, its shrink-on-scroll logo, and its stylesheet.
 *
 * Two things differ, both forced by the engine rather than chosen:
 *
 *  - The cart affordance is a slot core renders, because a frontend never reads
 *    the cart (ADR-013 decision 4).
 *  - The source's own mobile overlay is replaced by core's `SideMenu`, which
 *    carries region and locale selection the source shop had no need for. It
 *    sits in the source's `.mobileMenuBtn` slot, so the stylesheet still hides
 *    it above 768px exactly as before.
 *
 * The source's `onSearchOpen` opened a `SearchBar` over `ShopContext`'s
 * client-side query. Search has no contract slot — PHASE3_STATUS lists it as a
 * missing feature, not a theming gap — so it links to the catalog for now.
 */
export default function Navbar({
  logoSrc,
  storeName,
  links,
  cartButton,
  sideMenu,
}: {
  logoSrc: string
  storeName: string
  links: { label: string; href: string }[]
  cartButton: ReactNode
  sideMenu: ReactNode
}) {
  const [isScrolled, setIsScrolled] = useState(false)
  const scrollDirection = useScrollDirection()

  useEffect(() => {
    const handleScroll = () => setIsScrolled(window.scrollY > 50)
    window.addEventListener("scroll", handleScroll, { passive: true })
    handleScroll()
    return () => window.removeEventListener("scroll", handleScroll)
  }, [])

  return (
    <nav
      className={`${styles.navbar} ${isScrolled ? styles.scrolled : ""} ${
        scrollDirection === "down" ? styles.hidden : ""
      }`}
      aria-label="Main Navigation"
    >
      <div className={styles.container}>
        <div className={styles.left}>
          <div className={styles.mobileMenuBtn}>{sideMenu}</div>
          <LocalizedClientLink
            href="/"
            className={styles.logoLink}
            data-testid="nav-store-link"
          >
            {/* Not next/image: the source shipped a plain <img> here, and the
                logo is a fixed-height mark the stylesheet already sizes. */}
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              src={logoSrc}
              alt={`${storeName} logo`}
              className={styles.logoImage}
            />
          </LocalizedClientLink>
        </div>

        <div className={styles.center}>
          {links.map((link) =>
            link.href.startsWith("#") ? (
              <a key={link.label} href={link.href} className={styles.navLink}>
                {link.label}
              </a>
            ) : (
              <LocalizedClientLink
                key={link.label}
                href={link.href}
                className={styles.navLink}
              >
                {link.label}
              </LocalizedClientLink>
            )
          )}
        </div>

        <div className={styles.right}>
          <LocalizedClientLink
            href="/store"
            className={`${styles.iconButton} ${styles.searchBtn}`}
            aria-label="Search"
          >
            <SearchIcon />
          </LocalizedClientLink>

          <LocalizedClientLink
            href="/account"
            className={styles.iconButton}
            aria-label="Account"
            data-testid="nav-account-link"
          >
            <UserIcon />
          </LocalizedClientLink>

          {cartButton}
        </div>
      </div>
    </nav>
  )
}

/*
 * The source used `lucide-react`. These are the same glyphs inlined: adding a
 * 1.4MB icon dependency for two paths is the ladder's rung 6.
 */
const icon = {
  width: 20,
  height: 20,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.75,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
}

const SearchIcon = () => (
  <svg {...icon} aria-hidden>
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.3-4.3" />
  </svg>
)

const UserIcon = () => (
  <svg {...icon} aria-hidden>
    <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
    <circle cx="12" cy="7" r="4" />
  </svg>
)
