"use client"

import { useState, useEffect, useMemo, useCallback, useRef } from "react"
import Lenis from "lenis"
import gsap from "gsap"
import { ScrollTrigger } from "gsap/ScrollTrigger"
import type { StorefrontThemeProps } from "../../core/theme-contract"
import {
  CATEGORIES,
  GALLERY_ITEMS,
  HOMEDINE_STATS,
  MOTION_TOKENS,
  PRODUCTS,
  REVIEWS,
  SCROLL_TOKENS,
  SPOTLIGHTS,
  filterHomedineProducts,
  getActiveSwatch,
  type HomedineGalleryItem,
  type HomedineProduct,
} from "./catalog"
import { HeroSection } from "./components/HeroSection"
import { BestsellingProducts } from "./components/BestsellingProducts"
import { LifestyleBanner } from "./components/LifestyleBanner"
import { CategoriesCarousel } from "./components/CategoriesCarousel"
import { EditorialSpotlights } from "./components/EditorialSpotlights"
import { InspirationGallery } from "./components/InspirationGallery"
import { ReviewsSection } from "./components/ReviewsSection"
import { SustainabilityClosing } from "./components/SustainabilityClosing"
import { Footer } from "./components/Footer"
import { ProductModal } from "./components/ProductModal"
import { CartDrawer } from "./components/CartDrawer"
import { AccountDrawer } from "./components/AccountDrawer"
import styles from "./homedine-eco.module.css"

gsap.registerPlugin(ScrollTrigger)

interface ActiveModalState {
  product: HomedineProduct
  swatchId?: string
  contextNote?: string
}

export function HomedineEcoTheme({
  tenant,
  cartItems,
  cartTotals,
  cartOpen,
  onAddToCart,
  onUpdateQuantity,
  onRemoveItem,
  onToggleCart,
  onProceedToCheckout,
}: StorefrontThemeProps) {
  const rootRef = useRef<HTMLDivElement | null>(null)
  const lenisRef = useRef<Lenis | null>(null)

  const [savedIds, setSavedIds] = useState<string[]>([])
  const [searchQuery, setSearchQuery] = useState("")
  const [activeCategoryId, setActiveCategoryId] = useState<string | null>(null)
  const [activeCategoryLabel, setActiveCategoryLabel] = useState<string | null>(null)
  const [activeModal, setActiveModal] = useState<ActiveModalState | null>(null)
  const [accountOpen, setAccountOpen] = useState(false)

  const isOverlayOpen = cartOpen || accountOpen || activeModal !== null

  const heroSectionConfig = useMemo(() => {
    return tenant.themeConfig.sections.find((s) => s.type === "hero")
  }, [tenant.themeConfig.sections])

  // 1. Lenis smooth scroll synced cleanly with GSAP ticker + complete teardown on unmount
  useEffect(() => {
    const prefersReduced =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches
    if (prefersReduced) return

    const lenis = new Lenis({
      duration: SCROLL_TOKENS.duration,
      easing: SCROLL_TOKENS.easing,
      smoothWheel: true,
      wheelMultiplier: SCROLL_TOKENS.wheelMultiplier,
      touchMultiplier: SCROLL_TOKENS.touchMultiplier,
      infinite: false,
    })
    lenisRef.current = lenis

    const handleScroll = () => ScrollTrigger.update()
    lenis.on("scroll", handleScroll)

    const rafCallback = (time: number) => {
      lenis.raf(time * 1000)
    }
    gsap.ticker.add(rafCallback)
    gsap.ticker.lagSmoothing(0)

    const handleAnchorClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null
      const anchor = target?.closest('a[href^="#"]') as HTMLAnchorElement | null
      if (!anchor) return
      const href = anchor.getAttribute("href")
      if (!href || href === "#" || href === "#top") {
        e.preventDefault()
        lenis.scrollTo(0, { duration: 1.1 })
        return
      }
      const el = document.querySelector<HTMLElement>(href)
      if (el) {
        e.preventDefault()
        lenis.scrollTo(el, { offset: SCROLL_TOKENS.anchorOffset, duration: 1.15 })
      }
    }

    document.addEventListener("click", handleAnchorClick)

    return () => {
      document.removeEventListener("click", handleAnchorClick)
      gsap.ticker.remove(rafCallback)
      lenis.destroy()
      lenisRef.current = null
    }
  }, [])

  // Pause Lenis while modal or drawer is open
  useEffect(() => {
    const lenis = lenisRef.current
    if (!lenis) return
    if (isOverlayOpen) {
      lenis.stop()
    } else {
      lenis.start()
    }
  }, [isOverlayOpen])

  // 2. GSAP hero entrance, counter, ScrollTrigger reveals, parallax & magnetic buttons
  useEffect(() => {
    const root = rootRef.current
    if (!root) return

    const prefersReduced =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches
    if (prefersReduced) return

    const ctx = gsap.context(() => {
      // Hero entrance timeline
      const heroTl = gsap.timeline({
        defaults: { ease: MOTION_TOKENS.ease.expoOut },
      })

      const heroBg = root.querySelector('[data-gsap="hero-bg"]')
      if (heroBg) {
        heroTl.fromTo(
          heroBg,
          { scale: 1.07 },
          { scale: 1, duration: 1.4, ease: MOTION_TOKENS.ease.power3Out },
          0
        )
      }

      const heroNav = root.querySelectorAll('[data-gsap="hero-nav"]')
      if (heroNav.length > 0) {
        heroTl.from(
          heroNav,
          {
            y: -14,
            opacity: 0,
            duration: 0.7,
            stagger: MOTION_TOKENS.stagger.tight,
            clearProps: "transform,opacity",
          },
          0.1
        )
      }

      const heroLines = root.querySelectorAll('[data-gsap="hero-line"]')
      if (heroLines.length > 0) {
        heroTl.from(
          heroLines,
          {
            y: 36,
            opacity: 0,
            duration: MOTION_TOKENS.duration.heroTitle,
            stagger: 0.1,
            clearProps: "transform,opacity",
          },
          0.2
        )
      }

      const heroFade = root.querySelectorAll('[data-gsap="hero-fade"]')
      if (heroFade.length > 0) {
        heroTl.from(
          heroFade,
          {
            y: 20,
            opacity: 0,
            duration: 0.85,
            stagger: 0.12,
            clearProps: "transform,opacity",
          },
          0.48
        )
      }

      const heroStat = root.querySelector('[data-gsap="hero-stat"]')
      if (heroStat) {
        heroTl.from(
          heroStat,
          {
            y: 24,
            opacity: 0,
            scale: 0.96,
            duration: 0.9,
            clearProps: "transform,opacity",
          },
          0.55
        )
      }

      // Animated numeric counter 0% -> 96%
      const counterEl = root.querySelector<HTMLElement>('[data-gsap="eco-counter"]')
      if (counterEl) {
        const counterObj = { value: 0 }
        heroTl.to(
          counterObj,
          {
            value: HOMEDINE_STATS.ecoMaterialStat,
            duration: MOTION_TOKENS.duration.counter,
            ease: MOTION_TOKENS.ease.power2Out,
            onUpdate: () => {
              counterEl.textContent = `${Math.round(counterObj.value)}%`
            },
          },
          0.6
        )
      }

      // Scrubbed image parallax
      root.querySelectorAll<HTMLElement>('[data-gsap="parallax-media"]').forEach((el) => {
        const triggerEl = el.closest("section") ?? el.parentElement ?? el
        gsap.fromTo(
          el,
          { yPercent: -5 },
          {
            yPercent: 5,
            ease: MOTION_TOKENS.ease.none,
            scrollTrigger: {
              trigger: triggerEl,
              start: "top bottom",
              end: "bottom top",
              scrub: MOTION_TOKENS.scrollTrigger.parallaxScrub,
            },
          }
        )
      })

      // Single-element editorial reveals
      root.querySelectorAll<HTMLElement>('[data-gsap="reveal"]').forEach((el) => {
        gsap.from(el, {
          y: 28,
          opacity: 0,
          duration: MOTION_TOKENS.duration.reveal,
          ease: MOTION_TOKENS.ease.power3Out,
          clearProps: "transform,opacity",
          scrollTrigger: {
            trigger: el,
            start: MOTION_TOKENS.scrollTrigger.earlyStart,
            once: true,
          },
        })
      })

      // Staggered grids
      root.querySelectorAll<HTMLElement>('[data-gsap="stagger-group"]').forEach((group) => {
        const items = group.querySelectorAll<HTMLElement>("[data-gsap-item]")
        if (items.length === 0) return
        gsap.from(items, {
          y: 22,
          opacity: 0,
          duration: 0.56,
          stagger: MOTION_TOKENS.stagger.cards,
          ease: MOTION_TOKENS.ease.power3Out,
          clearProps: "transform,opacity",
          scrollTrigger: {
            trigger: group,
            start: MOTION_TOKENS.scrollTrigger.earlyStart,
            once: true,
          },
        })
      })

      // Editorial spotlight alternating reveal
      root.querySelectorAll<HTMLElement>('[data-gsap="spotlight-row"]').forEach((row) => {
        const media = row.querySelector<HTMLElement>("[data-gsap-media]")
        const copy = row.querySelector<HTMLElement>("[data-gsap-copy]")
        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: row,
            start: MOTION_TOKENS.scrollTrigger.defaultStart,
            once: true,
          },
        })

        if (media) {
          tl.from(
            media,
            {
              scale: 0.95,
              y: 26,
              opacity: 0,
              duration: 0.9,
              ease: MOTION_TOKENS.ease.expoOut,
              clearProps: "transform,opacity",
            },
            0
          )
        }
        if (copy) {
          tl.from(
            copy,
            {
              y: 24,
              opacity: 0,
              duration: 0.85,
              ease: MOTION_TOKENS.ease.power3Out,
              clearProps: "transform,opacity",
            },
            0.14
          )
        }
      })

      // Layered 3-card collage fan-out reveal
      const collageContainer = root.querySelector<HTMLElement>('[data-gsap="collage-fan"]')
      if (collageContainer) {
        const leftCard = collageContainer.querySelector('[data-gsap-collage="left"]')
        const centerCard = collageContainer.querySelector('[data-gsap-collage="center"]')
        const rightCard = collageContainer.querySelector('[data-gsap-collage="right"]')

        const collageTl = gsap.timeline({
          scrollTrigger: {
            trigger: collageContainer,
            start: MOTION_TOKENS.scrollTrigger.defaultStart,
            once: true,
          },
        })

        if (centerCard) {
          collageTl.from(
            centerCard,
            {
              y: 30,
              scale: 0.92,
              opacity: 0,
              duration: 0.85,
              ease: MOTION_TOKENS.ease.expoOut,
              clearProps: "transform,opacity",
            },
            0
          )
        }
        if (leftCard) {
          collageTl.from(
            leftCard,
            {
              x: 42,
              rotate: 2,
              opacity: 0,
              duration: 0.9,
              ease: MOTION_TOKENS.ease.expoOut,
              clearProps: "transform,opacity",
            },
            0.14
          )
        }
        if (rightCard) {
          collageTl.from(
            rightCard,
            {
              x: -42,
              rotate: -2,
              opacity: 0,
              duration: 0.9,
              ease: MOTION_TOKENS.ease.expoOut,
              clearProps: "transform,opacity",
            },
            0.14
          )
        }
      }
    }, root)

    const refreshTimer = window.setTimeout(() => {
      ScrollTrigger.refresh()
    }, 350)

    // Magnetic subtle cursor pull for desktop fine pointers
    const magneticCleanups: Array<() => void> = []
    const isFinePointer = window.matchMedia("(hover: hover) and (pointer: fine)").matches
    if (isFinePointer) {
      const elements = Array.from(root.querySelectorAll<HTMLElement>("[data-magnetic]"))
      elements.forEach((el) => {
        const strength = Number(el.dataset.magneticStrength ?? "0.22")
        const xTo = gsap.quickTo(el, "x", { duration: 0.35, ease: "power3.out" })
        const yTo = gsap.quickTo(el, "y", { duration: 0.35, ease: "power3.out" })

        const handleMove = (e: MouseEvent) => {
          const rect = el.getBoundingClientRect()
          const relX = e.clientX - (rect.left + rect.width / 2)
          const relY = e.clientY - (rect.top + rect.height / 2)
          xTo(relX * strength)
          yTo(relY * strength)
        }
        const handleLeave = () => {
          xTo(0)
          yTo(0)
        }

        el.addEventListener("mousemove", handleMove)
        el.addEventListener("mouseleave", handleLeave)
        magneticCleanups.push(() => {
          el.removeEventListener("mousemove", handleMove)
          el.removeEventListener("mouseleave", handleLeave)
          gsap.set(el, { clearProps: "x,y" })
        })
      })
    }

    return () => {
      window.clearTimeout(refreshTimer)
      magneticCleanups.forEach((fn) => fn())
      ctx.revert()
    }
  }, [])

  const filteredProducts = useMemo(
    () => filterHomedineProducts(PRODUCTS, searchQuery, activeCategoryId),
    [searchQuery, activeCategoryId]
  )

  const savedProducts = useMemo(
    () => PRODUCTS.filter((p) => savedIds.includes(p.id)),
    [savedIds]
  )

  const handleAddToCart = useCallback(
    (product: HomedineProduct, swatchId?: string) => {
      const swatch = getActiveSwatch(product, swatchId)
      onAddToCart(
        {
          variantId: `${product.id}::${swatch.id}`,
          productId: product.id,
          title: product.shortName,
          variantLabel: swatch.name,
          imagePath: product.image,
          unitPriceMinor: product.priceMinor,
        },
        1
      )
    },
    [onAddToCart]
  )

  const handleToggleSave = useCallback((productId: string) => {
    setSavedIds((prev) =>
      prev.includes(productId) ? prev.filter((id) => id !== productId) : [...prev, productId]
    )
  }, [])

  const scrollToElement = useCallback((id: string) => {
    const el = document.getElementById(id)
    if (!el) return
    if (lenisRef.current) {
      lenisRef.current.scrollTo(el, { offset: SCROLL_TOKENS.anchorOffset, duration: 1.15 })
    } else {
      el.scrollIntoView({ behavior: "smooth", block: "start" })
    }
  }, [])

  const handleSelectCategory = useCallback(
    (categoryId: string, label: string) => {
      setActiveCategoryId(categoryId)
      setActiveCategoryLabel(label)
      setSearchQuery("")
      scrollToElement("bestsellers")
    },
    [scrollToElement]
  )

  const handleClearFilters = useCallback(() => {
    setActiveCategoryId(null)
    setActiveCategoryLabel(null)
    setSearchQuery("")
  }, [])

  const handleShopSpotlight = useCallback((productId: string) => {
    const found = PRODUCTS.find((p) => p.id === productId)
    if (found) setActiveModal({ product: found })
  }, [])

  const handleInspectGalleryItem = useCallback((item: HomedineGalleryItem) => {
    const found = PRODUCTS.find((p) => p.id === item.linkedProductId) ?? PRODUCTS[0]
    setActiveModal({ product: found, contextNote: item.title })
  }, [])

  return (
    <div id="top" ref={rootRef} className={styles.root} data-theme="homedine-eco">
      <main>
        <HeroSection
          products={PRODUCTS}
          cartItemCount={cartTotals.itemCount}
          savedCount={savedIds.length}
          searchQuery={searchQuery}
          isDraftMode={tenant.mode === "draft"}
          draftSubcopy={heroSectionConfig?.type === "hero" ? heroSectionConfig.subcopy : undefined}
          draftCtaLabel={heroSectionConfig?.type === "hero" ? heroSectionConfig.ctaLabel : undefined}
          onSearchChange={setSearchQuery}
          onOpenCart={() => onToggleCart(true)}
          onOpenAccount={() => setAccountOpen(true)}
          onSelectProduct={(product) => setActiveModal({ product })}
          onQuickAdd={(product) => handleAddToCart(product)}
          onShopNow={() => scrollToElement("bestsellers")}
        />

        <BestsellingProducts
          products={filteredProducts}
          activeCategoryLabel={activeCategoryLabel}
          searchQuery={searchQuery}
          savedIds={savedIds}
          onClearFilters={handleClearFilters}
          onSelectProduct={(product, swatchId) => setActiveModal({ product, swatchId })}
          onAddToCart={handleAddToCart}
          onToggleSave={handleToggleSave}
        />

        <LifestyleBanner />

        <CategoriesCarousel
          categories={CATEGORIES}
          activeCategoryId={activeCategoryId}
          onSelectCategory={handleSelectCategory}
        />

        <EditorialSpotlights
          spotlights={SPOTLIGHTS}
          onShopSpotlight={handleShopSpotlight}
        />

        <InspirationGallery
          items={GALLERY_ITEMS}
          onInspectGalleryItem={handleInspectGalleryItem}
        />

        <ReviewsSection reviews={REVIEWS} />

        <SustainabilityClosing />
      </main>

      <Footer footerNote={tenant.themeConfig.footerNote} />

      <ProductModal
        product={activeModal?.product ?? null}
        initialSwatchId={activeModal?.swatchId}
        contextNote={activeModal?.contextNote}
        isSaved={activeModal ? savedIds.includes(activeModal.product.id) : false}
        onClose={() => setActiveModal(null)}
        onAddToCart={handleAddToCart}
        onToggleSave={handleToggleSave}
      />

      <CartDrawer
        isOpen={cartOpen}
        items={cartItems}
        totals={cartTotals}
        currency={tenant.currency}
        onClose={() => onToggleCart(false)}
        onUpdateQuantity={onUpdateQuantity}
        onRemoveItem={onRemoveItem}
        onProceedToCheckout={onProceedToCheckout}
      />

      <AccountDrawer
        isOpen={accountOpen}
        savedProducts={savedProducts}
        onClose={() => setAccountOpen(false)}
        onToggleSave={handleToggleSave}
        onAddToCart={(product) => handleAddToCart(product)}
      />
    </div>
  )
}
