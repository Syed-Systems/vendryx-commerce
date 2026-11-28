"use client"

import { useState, useRef, useEffect } from "react"
import { ArrowRight, Leaf, Menu, Search, ShoppingBag, User, X } from "lucide-react"
import {
  ASSETS,
  filterHomedineProducts,
  formatUsd,
  type HomedineProduct,
} from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

interface HeroSectionProps {
  products: HomedineProduct[]
  cartItemCount: number
  savedCount: number
  searchQuery: string
  isDraftMode?: boolean
  draftSubcopy?: string
  draftCtaLabel?: string
  onSearchChange: (value: string) => void
  onOpenCart: () => void
  onOpenAccount: () => void
  onSelectProduct: (product: HomedineProduct) => void
  onQuickAdd: (product: HomedineProduct) => void
  onShopNow: () => void
}

const NAV_LINKS = [
  { label: "Shop", href: "#bestsellers" },
  { label: "Bestsellers", href: "#spotlights" },
  { label: "Gallery", href: "#gallery" },
  { label: "About", href: "#about" },
]

export function HeroSection({
  products,
  cartItemCount,
  savedCount,
  searchQuery,
  isDraftMode = false,
  draftSubcopy,
  draftCtaLabel,
  onSearchChange,
  onOpenCart,
  onOpenAccount,
  onSelectProduct,
  onQuickAdd,
  onShopNow,
}: HeroSectionProps) {
  const [searchFocused, setSearchFocused] = useState(false)
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)
  const searchWrapperRef = useRef<HTMLDivElement | null>(null)

  const searchResults =
    searchQuery.trim().length > 0 ? filterHomedineProducts(products, searchQuery) : []

  useEffect(() => {
    const handleOutsideClick = (event: MouseEvent) => {
      if (
        searchWrapperRef.current &&
        !searchWrapperRef.current.contains(event.target as Node)
      ) {
        setSearchFocused(false)
      }
    }
    document.addEventListener("mousedown", handleOutsideClick)
    return () => document.removeEventListener("mousedown", handleOutsideClick)
  }, [])

  return (
    <section
      aria-label="Homedine eco-friendly kitchenware showcase"
      className="mx-auto max-w-[1360px] px-4 pt-4 sm:px-6 sm:pt-5 lg:px-8"
    >
      <div className="relative overflow-hidden rounded-[var(--radius-hero)] bg-[#10251c] min-h-[540px] sm:min-h-[600px] lg:min-h-[650px] flex flex-col justify-between">
        {/* Background Hero Kitchen Photo with subtle parallax */}
        <div
          data-gsap="hero-bg"
          className="absolute inset-0 pointer-events-none overflow-hidden"
        >
          <div data-gsap="parallax-media" className="w-full h-[112%] -mt-[6%]">
            <ResilientImage
              src={ASSETS.heroKitchen}
              alt="Sunlit emerald-green eco-friendly modern kitchen with herbs and ceramic bowls"
              loading="eager"
              className="w-full h-full object-cover object-center"
            />
          </div>
          {/* Measured contrast scrims for WCAG AA legibility over sunlit kitchen */}
          <div className="absolute inset-0 bg-gradient-to-b from-black/[0.52] via-black/[0.18] to-black/[0.62]" />
          <div className="absolute inset-0 bg-gradient-to-r from-[#071c13]/[0.82] via-[#071c13]/[0.36] to-transparent" />
        </div>

        {/* Integrated Top Navigation Bar */}
        <header className="relative z-30 flex items-center justify-between gap-4 px-5 pt-5 pb-3 sm:px-8 sm:pt-7 lg:px-10">
          {/* Left Navigation Links (Desktop) & Menu Button (Mobile) */}
          <div className="flex items-center gap-6 flex-1 min-w-0">
            <button
              type="button"
              onClick={() => setMobileMenuOpen((prev) => !prev)}
              aria-label={mobileMenuOpen ? "Close navigation menu" : "Open navigation menu"}
              aria-expanded={mobileMenuOpen}
              className="md:hidden inline-flex items-center justify-center w-10 h-10 rounded-full bg-white/[0.15] backdrop-blur-md border border-white/25 text-white hover:bg-white/25 transition-colors shrink-0"
            >
              {mobileMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
            </button>

            <nav
              aria-label="Primary navigation"
              className="hidden md:flex items-center gap-7 text-[14px] font-medium text-white/95"
            >
              {NAV_LINKS.map((link) => (
                <a
                  key={link.label}
                  href={link.href}
                  data-gsap="hero-nav"
                  className={`${styles.navLinkUnderline} whitespace-nowrap shrink-0 hover:text-white transition-colors`}
                >
                  {link.label}
                </a>
              ))}
            </nav>
          </div>

          {/* Center Brand Wordmark */}
          <a
            href="#top"
            data-gsap="hero-nav"
            aria-label="Homedine Home"
            className="text-[24px] sm:text-[26px] leading-none tracking-[-0.02em] text-white shrink-0 select-none"
          >
            <span className="font-semibold">Home</span>
            <span className={`${styles.fontEditorial} font-normal ml-[1px]`}>dine</span>
          </a>

          {/* Right Search Pill + Cart & Account Controls */}
          <div className="flex items-center justify-end gap-2.5 flex-1 min-w-0">
            <div ref={searchWrapperRef} className="relative hidden sm:block w-[180px] lg:w-[224px]">
              <label htmlFor="hero-product-search" className="sr-only">
                Search eco-friendly products
              </label>
              <Search
                className="w-4 h-4 text-[#4e5651] absolute left-3.5 top-1/2 -translate-y-1/2 pointer-events-none"
                strokeWidth={1.9}
                aria-hidden="true"
              />
              <input
                id="hero-product-search"
                type="search"
                value={searchQuery}
                onFocus={() => setSearchFocused(true)}
                onChange={(e) => {
                  onSearchChange(e.target.value)
                  setSearchFocused(true)
                }}
                placeholder="Search Product..."
                className="w-full h-10 pl-9 pr-4 rounded-full bg-white/95 text-[13px] text-[#1a1f1c] placeholder:text-[#656f69] shadow-sm focus:bg-white focus:outline-none"
              />

              {/* Live Search Results Popover */}
              {searchFocused && searchQuery.trim().length > 0 && (
                <div
                  data-lenis-prevent
                  className="absolute right-0 mt-2 w-[310px] rounded-2xl bg-white shadow-2xl border border-black/[0.08] p-2.5 z-50 max-h-[340px] overflow-y-auto"
                >
                  {searchResults.length === 0 ? (
                    <p className="py-5 px-3 text-center text-[13px] text-[#4e5651]">
                      No eco kitchenware matched &ldquo;{searchQuery}&rdquo;.
                    </p>
                  ) : (
                    <ul className="divide-y divide-black/[0.06]">
                      {searchResults.map((item) => (
                        <li key={item.id} className="py-2 first:pt-1 last:pb-1 flex items-center gap-3">
                          <button
                            type="button"
                            onClick={() => {
                              onSelectProduct(item)
                              setSearchFocused(false)
                            }}
                            className="flex items-center gap-3 flex-1 min-w-0 text-left cursor-pointer"
                          >
                            <ResilientImage
                              src={item.image}
                              alt={item.alt}
                              className="w-11 h-11 rounded-lg object-cover bg-[#cac8be] shrink-0"
                            />
                            <div className="min-w-0 flex-1">
                              <p className="text-[13px] font-semibold text-[#1a1f1c] truncate">
                                {item.shortName}
                              </p>
                              <p className="text-[12px] text-[#4e5651] tabular-nums">
                                {formatUsd(item.price)} · {item.category}
                              </p>
                            </div>
                          </button>
                          <button
                            type="button"
                            onClick={() => onQuickAdd(item)}
                            aria-label={`Add ${item.shortName} to cart`}
                            className="px-2.5 py-1.5 rounded-full bg-[#0e5437] text-white text-[11px] font-medium hover:bg-[#0a3f29] whitespace-nowrap shrink-0 transition-colors cursor-pointer"
                          >
                            + Add
                          </button>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              )}
            </div>

            <button
              type="button"
              onClick={onOpenCart}
              data-magnetic
              data-magnetic-strength="0.18"
              data-testid="homedine-cart-trigger"
              aria-label={`Open shopping cart with ${cartItemCount} items`}
              className="relative inline-flex items-center justify-center w-10 h-10 rounded-full bg-white/[0.18] backdrop-blur-md border border-white/30 text-white hover:bg-white hover:text-[#1a1f1c] transition-colors shrink-0 cursor-pointer"
            >
              <ShoppingBag className="w-[18px] h-[18px]" strokeWidth={1.8} />
              {cartItemCount > 0 && (
                <span
                  data-testid="cart-badge-count"
                  className="absolute -top-1 -right-1 min-w-[20px] h-5 px-1 rounded-full bg-[#f4f2db] text-[#0e5437] text-[11px] font-bold flex items-center justify-center tabular-nums shadow-sm"
                >
                  {cartItemCount}
                </span>
              )}
            </button>

            <button
              type="button"
              onClick={onOpenAccount}
              data-magnetic
              data-magnetic-strength="0.18"
              aria-label={`Open saved items (${savedCount} saved)`}
              className="relative inline-flex items-center justify-center w-10 h-10 rounded-full bg-white/[0.18] backdrop-blur-md border border-white/30 text-white hover:bg-white hover:text-[#1a1f1c] transition-colors shrink-0 cursor-pointer"
            >
              <User className="w-[18px] h-[18px]" strokeWidth={1.8} />
              {savedCount > 0 && (
                <span className="absolute -top-1 -right-1 min-w-[18px] h-[18px] px-1 rounded-full bg-[#0e5437] text-white text-[10px] font-semibold flex items-center justify-center tabular-nums border border-white/50">
                  {savedCount}
                </span>
              )}
            </button>
          </div>
        </header>

        {/* Mobile Drawer Nav */}
        {mobileMenuOpen && (
          <div className="relative z-30 md:hidden mx-5 mb-2 rounded-2xl bg-[#10251c]/95 backdrop-blur-md border border-white/20 p-4 text-white">
            <div className="mb-3">
              <label htmlFor="mobile-product-search" className="sr-only">
                Search eco-friendly products
              </label>
              <div className="relative">
                <Search className="w-4 h-4 text-[#4e5651] absolute left-3.5 top-1/2 -translate-y-1/2" />
                <input
                  id="mobile-product-search"
                  type="search"
                  value={searchQuery}
                  onChange={(e) => onSearchChange(e.target.value)}
                  placeholder="Search Product..."
                  className="w-full h-10 pl-9 pr-4 rounded-full bg-white text-[13px] text-[#1a1f1c] placeholder:text-[#656f69]"
                />
              </div>
            </div>
            <nav aria-label="Mobile navigation" className="flex flex-col gap-2">
              {NAV_LINKS.map((link) => (
                <a
                  key={link.label}
                  href={link.href}
                  onClick={() => setMobileMenuOpen(false)}
                  className="py-2 px-3 rounded-xl text-sm font-medium text-white/90 hover:bg-white/10 transition-colors"
                >
                  {link.label}
                </a>
              ))}
            </nav>
          </div>
        )}

        {/* Hero Main Content Area */}
        <div className="relative z-20 flex flex-col md:flex-row md:items-end justify-between gap-8 px-6 pb-8 pt-16 sm:px-10 sm:pb-11 lg:px-12 lg:pb-12">
          {/* Left Hero Editorial Copy & Primary CTA */}
          <div className="max-w-[540px]">
            {isDraftMode && (
              <span className="inline-flex items-center gap-1.5 mb-4 px-3 py-1 rounded-full bg-[#f4f2db]/90 text-[#0e5437] text-[11px] font-semibold uppercase tracking-wider">
                Draft v1.1.0 Preview
              </span>
            )}
            <h1 className="text-[clamp(2.35rem,4.6vw,3.9rem)] leading-[1.07] tracking-[-0.025em] text-white font-medium">
              <span data-gsap="hero-line" className="block">
                Eco-Friendly
              </span>
              <span data-gsap="hero-line" className="block">
                <span className={`${styles.fontEditorial} font-normal pr-2`}>Kitchenware</span>{" "}
                <span>for</span>
              </span>
              <span data-gsap="hero-line" className="block">
                a greener home
              </span>
            </h1>

            <p
              data-gsap="hero-fade"
              className="mt-5 max-w-[400px] text-[14px] sm:text-[15px] leading-[1.6] text-white/85 font-normal"
            >
              {isDraftMode && draftSubcopy
                ? draftSubcopy
                : "Thoughtfully crafted, non-toxic kitchen essentials made from sustainable materials for calmer, planet-first everyday cooking."}
            </p>

            <div data-gsap="hero-fade" className="mt-7">
              <button
                type="button"
                onClick={onShopNow}
                data-magnetic
                data-magnetic-strength="0.2"
                className={`${styles.btnArrowGlide} inline-flex items-center gap-2.5 h-11 px-6 rounded-full bg-[var(--color-brand-cream)] hover:bg-[var(--color-brand-cream-hover)] text-[#18211b] text-[14px] font-semibold shadow-lg transition-colors whitespace-nowrap shrink-0 cursor-pointer`}
              >
                <span>{isDraftMode && draftCtaLabel ? draftCtaLabel : "Shop now"}</span>
                <ArrowRight className={`${styles.arrowIcon} w-4 h-4`} strokeWidth={2} />
              </button>
            </div>
          </div>

          {/* Bottom-Right Frosted Eco Material Stat Card */}
          <aside
            data-gsap="hero-stat"
            aria-label="Sustainability material highlight"
            className={`${styles.glassSurface} w-full md:w-[214px] rounded-[18px] p-4 sm:p-5 text-white flex flex-row md:flex-col items-center md:items-stretch justify-between gap-4 md:min-h-[188px]`}
          >
            <div className="flex items-start justify-between gap-3">
              <p className="text-[12px] sm:text-[13px] leading-[1.35] text-white/[0.92] font-medium">
                Natural.
                <br />
                Sustainable.
                <br />
                Eco-conscious.
              </p>
              <span className="hidden md:flex w-8 h-8 rounded-full bg-white/[0.12] items-center justify-center shrink-0">
                <Leaf className="w-4 h-4 text-[#f4f2db]" strokeWidth={1.75} />
              </span>
            </div>

            <div className="md:pt-6 text-right md:text-left">
              <p
                data-gsap="eco-counter"
                className="text-[34px] sm:text-[42px] leading-none font-medium tracking-[-0.03em] tabular-nums text-white"
              >
                96%
              </p>
              <p className="mt-1 text-[11px] text-white/75">Recycled &amp; bio-based</p>
            </div>
          </aside>
        </div>
      </div>
    </section>
  )
}
