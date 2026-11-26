"use client"

import React, { useEffect, useRef, useState } from "react"
import gsap from "gsap"
import { ScrollTrigger } from "gsap/ScrollTrigger"
import Lenis from "lenis"
import { AnimatePresence, motion, useReducedMotion, type PanInfo } from "motion/react"
import { formatMinorPrice } from "../../core/checkout-engine"
import type { CatalogProduct } from "../../core/tenant-resolver"
import type { StorefrontThemeProps } from "../../core/theme-contract"
import {
  AtelierBagIcon,
  HairlineArrowIcon,
  HairlineCloseIcon,
  HairlineCompareIcon,
  HairlineMinusIcon,
  HairlinePlusIcon,
  StitchMarkIcon,
} from "./icons"
import styles from "./maison-cuir.module.css"

gsap.registerPlugin(ScrollTrigger)

const EASE_OUT: [number, number, number, number] = [0.23, 1, 0.32, 1]
const EASE_DRAWER: [number, number, number, number] = [0.32, 0.72, 0, 1]

/** Apple's exact exponential-decay momentum projection (Designing Fluid Interfaces). */
function projectMomentum(velocityPxPerSec: number, decelerationRate = 0.99): number {
  return ((velocityPxPerSec / 1000) * decelerationRate) / (1 - decelerationRate)
}

const ATELIER_CHAPTERS = [
  {
    numeral: "I",
    src: "/media/maison-stage-tannery.jpg",
    alt: "Sunlit Tuscan vegetable-tanning workshop in Santa Croce sull'Arno with cognac calfskin hides draped over oak timber horses",
    title: "Forty Days in Chestnut Bark",
    sub: "Single-bend Tuscan calfskin slowly vegetable-tanned in Santa Croce sull'Arno with zero synthetic pigment.",
    targetProgress: 0.04,
  },
  {
    numeral: "II",
    src: "/media/maison-stage-stitch.jpg",
    alt: "Artisan hands saddle-stitching espresso full-grain leather with twin brass needles and beeswaxed linen thread",
    title: "Twin-Needle Saddle Stitch",
    sub: "Hand-sewn with beeswaxed French linen thread so every stitch locks independently for life.",
    targetProgress: 0.34,
  },
  {
    numeral: "III",
    src: "/media/leather-craft.jpg",
    alt: "Close-up of espresso full-grain leather edge burnishing and solid brass saddlery tools on limestone",
    title: "Seven-Coat Edge Burnishing",
    sub: "Raw hide edges skived, hand-dyed, and hot-creased with solid brass irons until glass-smooth.",
    targetProgress: 0.64,
  },
  {
    numeral: "IV",
    src: "/media/leather-hero.jpg",
    alt: "Maison Véla saddle-cognac weekender resting on a sunlit travertine plinth",
    title: "Architecture in Motion",
    sub: "Numbered silhouettes sculpted to soften, darken, and carry the quiet record of every journey.",
    targetProgress: 0.94,
  },
] as const

function MaisonProductCard({
  product,
  currency,
  onSelect,
  onQuickAdd,
}: {
  product: CatalogProduct
  currency: "USD" | "BDT"
  onSelect: () => void
  onQuickAdd: (variantIdx: number) => void
}) {
  const [variantIdx, setVariantIdx] = useState(0)
  const activeVariant = product.variants[variantIdx] ?? product.variants[0]

  return (
    <article data-maison-card className="flex flex-col">
      <button
        type="button"
        onClick={onSelect}
        className={`${styles.cardButton} ${styles.cardFrame} group relative aspect-[3/4] w-full text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--theme-ink)] focus-visible:ring-offset-2`}
      >
        <img
          src={activeVariant.imagePath}
          alt={`${product.title} in ${activeVariant.colorName}`}
          referrerPolicy="no-referrer"
          className={`${styles.cardImage} h-full w-full object-cover`}
        />
      </button>

      <div className="mt-4 flex flex-col gap-1.5 px-0.5">
        <div className="flex items-baseline justify-between gap-4">
          <button
            type="button"
            onClick={onSelect}
            className={`${styles.displayFont} ${styles.pressable} text-left text-[24px] font-medium leading-[1.12] text-[var(--theme-ink)] hover:opacity-75`}
          >
            {product.title}
          </button>
          <span className="shrink-0 text-[14px] font-medium tabular-nums text-[var(--theme-ink)]">
            {formatMinorPrice(activeVariant.priceMinor, currency)}
          </span>
        </div>

        <p className="text-[12.5px] text-[var(--theme-ink-muted)]">
          {product.category} · {activeVariant.colorName}
        </p>

        <div className="mt-1.5 flex items-center justify-between gap-3 pt-1">
          <div className="flex items-center gap-2" role="group" aria-label={`Finishes for ${product.title}`}>
            {product.variants.map((variant, idx) => {
              const selected = idx === variantIdx
              return (
                <button
                  key={variant.id}
                  type="button"
                  onClick={() => setVariantIdx(idx)}
                  aria-label={`${variant.colorName} leather finish`}
                  aria-pressed={selected}
                  className={`${styles.swatchRing} flex h-5 w-5 items-center justify-center rounded-full border ${
                    selected ? "border-[var(--theme-ink)] p-[2px]" : "border-transparent p-[2px]"
                  }`}
                >
                  <span
                    className="block h-full w-full rounded-full border border-black/15"
                    style={{ backgroundColor: variant.swatchHex }}
                  />
                </button>
              )
            })}
          </div>

          <button
            type="button"
            onClick={() => onQuickAdd(variantIdx)}
            className={`${styles.pressable} rounded-full border border-[var(--theme-border)] px-3.5 py-1.5 text-[12px] font-medium text-[var(--theme-ink)] hover:border-[var(--theme-ink)] whitespace-nowrap shrink-0`}
          >
            Add to Bag
          </button>
        </div>
      </div>
    </article>
  )
}

function AtelierProductModal({
  product,
  currency,
  onClose,
  onAddToCart,
}: {
  product: CatalogProduct
  currency: "USD" | "BDT"
  onClose: () => void
  onAddToCart: StorefrontThemeProps["onAddToCart"]
}) {
  const [selectedIdx, setSelectedIdx] = useState(0)
  const [openAccordion, setOpenAccordion] = useState<"anatomy" | "provenance">("anatomy")
  const reduceMotion = useReducedMotion()
  const activeVariant = product.variants[selectedIdx] ?? product.variants[0]

  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4 backdrop-blur-[6px] sm:p-8"
      role="dialog"
      aria-modal="true"
      aria-labelledby="atelier-pdp-title"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.22, ease: EASE_OUT }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose()
      }}
    >
      {/* Concentric Apple Dialog Shell: 28px outer radius, 12px gallery inset -> 16px inner photo radius */}
      <motion.div
        className={`${styles.modalShell} relative grid max-h-[90vh] w-full max-w-[1020px] grid-cols-1 overflow-y-auto md:grid-cols-[1.08fr_1fr]`}
        initial={reduceMotion ? { opacity: 0 } : { opacity: 0, scale: 0.96, y: 10 }}
        animate={reduceMotion ? { opacity: 1 } : { opacity: 1, scale: 1, y: 0 }}
        exit={reduceMotion ? { opacity: 0 } : { opacity: 0, scale: 0.97, y: 6 }}
        transition={
          reduceMotion
            ? { duration: 0.18 }
            : { type: "spring", duration: 0.4, bounce: 0 }
        }
      >
        <button
          type="button"
          onClick={onClose}
          aria-label="Close product view"
          className={`${styles.iconCircleBtn} absolute right-4 top-4 z-10`}
        >
          <HairlineCloseIcon className="h-4 w-4" />
        </button>

        {/* Left: Concentric 12px-inset Portrait Studio Gallery (28 - 12 = 16px inner radius) */}
        <div className="flex p-3">
          <div className={`${styles.modalGalleryInner} aspect-[3/4] w-full md:aspect-auto md:h-full`}>
            <img
              src={activeVariant.imagePath}
              alt={`${product.title} — ${activeVariant.colorName}`}
              referrerPolicy="no-referrer"
              className="h-full w-full object-cover"
            />
          </div>
        </div>

        {/* Right: Contiguous Atelier Purchase Module */}
        <div className="flex flex-col justify-between p-7 sm:p-9">
          <div>
            <p className="text-[12px] tracking-[0.06em] text-[var(--theme-ink-muted)]">
              {product.category} · {activeVariant.inStock ? "Available now" : "Waitlist"}
            </p>
            <h2
              id="atelier-pdp-title"
              className={`${styles.displayFont} mt-2 text-[36px] font-medium leading-[1.06]`}
            >
              {product.title}
            </h2>
            <p className="mt-2 text-[18px] font-medium tabular-nums">
              {formatMinorPrice(activeVariant.priceMinor, currency)}
            </p>
            <p className="mt-4 text-[14.5px] leading-[1.68] text-[var(--theme-ink-muted)]">
              {product.description}
            </p>

            {/* Hide Finish Selector */}
            <div className="mt-6 border-t pt-5" style={{ borderColor: "var(--theme-border)" }}>
              <div className="flex items-center justify-between text-[13px]">
                <span className="text-[var(--theme-ink-muted)]">Leather Finish</span>
                <span className="font-medium text-[var(--theme-ink)]">
                  {activeVariant.colorName}
                </span>
              </div>
              <div className="mt-3 flex flex-wrap items-center gap-2.5">
                {product.variants.map((variant, idx) => {
                  const active = idx === selectedIdx
                  return (
                    <button
                      key={variant.id}
                      type="button"
                      onClick={() => setSelectedIdx(idx)}
                      aria-label={variant.colorName}
                      className={`${styles.pressable} flex items-center gap-2 rounded-[12px] border px-3.5 py-2 text-[12.5px] ${
                        active
                          ? "border-[var(--theme-ink)] bg-[var(--theme-surface)] text-[var(--theme-ink)]"
                          : "border-[var(--theme-border)] text-[var(--theme-ink-muted)] hover:border-[var(--theme-ink)]"
                      }`}
                    >
                      <span
                        className="h-3.5 w-3.5 rounded-full border border-black/15"
                        style={{ backgroundColor: variant.swatchHex }}
                      />
                      <span>{variant.colorName}</span>
                    </button>
                  )
                })}
              </div>
            </div>

            {/* Interior Anatomy & Provenance Accordions (200ms strong ease-out) */}
            <div className="mt-6 divide-y border-y" style={{ borderColor: "var(--theme-border)" }}>
              <div>
                <button
                  type="button"
                  onClick={() =>
                    setOpenAccordion(openAccordion === "anatomy" ? "provenance" : "anatomy")
                  }
                  aria-expanded={openAccordion === "anatomy"}
                  className="flex w-full items-center justify-between py-3.5 text-left text-[13px] font-medium"
                >
                  <span>Dimensions &amp; Interior Anatomy</span>
                  {openAccordion === "anatomy" ? (
                    <HairlineMinusIcon className="h-4 w-4" />
                  ) : (
                    <HairlinePlusIcon className="h-4 w-4" />
                  )}
                </button>
                <AnimatePresence initial={false}>
                  {openAccordion === "anatomy" && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.2, ease: EASE_OUT }}
                      className="overflow-hidden"
                    >
                      <dl className="space-y-2 pb-4 text-[13px]">
                        {product.details.map((row) => (
                          <div key={row.label} className="flex justify-between gap-4">
                            <dt className="text-[var(--theme-ink-muted)]">{row.label}</dt>
                            <dd className="text-right text-[var(--theme-ink)]">{row.value}</dd>
                          </div>
                        ))}
                      </dl>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>

              <div>
                <button
                  type="button"
                  onClick={() =>
                    setOpenAccordion(openAccordion === "provenance" ? "anatomy" : "provenance")
                  }
                  aria-expanded={openAccordion === "provenance"}
                  className="flex w-full items-center justify-between py-3.5 text-left text-[13px] font-medium"
                >
                  <span>Tannery Provenance &amp; Care</span>
                  {openAccordion === "provenance" ? (
                    <HairlineMinusIcon className="h-4 w-4" />
                  ) : (
                    <HairlinePlusIcon className="h-4 w-4" />
                  )}
                </button>
                <AnimatePresence initial={false}>
                  {openAccordion === "provenance" && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.2, ease: EASE_OUT }}
                      className="overflow-hidden"
                    >
                      <p className="pb-4 text-[13px] leading-[1.65] text-[var(--theme-ink-muted)]">
                        {product.provenanceOrSpec}. Store inside the included unbleached linen dust
                        bag away from direct heat. Condition lightly once a season with neutral
                        beeswax balm.
                      </p>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            </div>
          </div>

          <div className="mt-7 space-y-3">
            <button
              type="button"
              onClick={() => {
                onAddToCart({
                  variantId: activeVariant.id,
                  productId: product.id,
                  title: product.title,
                  variantLabel: activeVariant.colorName,
                  imagePath: activeVariant.imagePath,
                  unitPriceMinor: activeVariant.priceMinor,
                })
                onClose()
              }}
              className={`${styles.primaryAction} flex w-full items-center justify-center gap-3 px-6 py-4 text-[13.5px] font-medium`}
            >
              <span>Add to Bag — {formatMinorPrice(activeVariant.priceMinor, currency)}</span>
              <HairlineArrowIcon className="h-4 w-4" />
            </button>
            <p className="text-center text-[12px] text-[var(--theme-ink-muted)]">
              Complimentary insured courier delivery on orders over $150 · Guest checkout available
            </p>
          </div>
        </div>
      </motion.div>
    </motion.div>
  )
}

export function MaisonCuirTheme({
  tenant,
  cartItems,
  cartTotals,
  cartOpen,
  activeProduct,
  onSelectProduct,
  onAddToCart,
  onUpdateQuantity,
  onRemoveItem,
  onToggleCart,
  onProceedToCheckout,
}: StorefrontThemeProps) {
  const reduceMotion = useReducedMotion()
  const hero = tenant.themeConfig.sections.find((s) => s.type === "hero")
  const featured = tenant.themeConfig.sections.find((s) => s.type === "featured_collection")
  const craft = tenant.themeConfig.sections.find((s) => s.type === "craft_story")

  // Pinned 4-stage atelier showcase state (GSAP ScrollTrigger scrub)
  const [activeStage, setActiveStage] = useState(0)
  const [captionBlur, setCaptionBlur] = useState(false)
  const stageIdxRef = useRef(0)
  const stageTimerRef = useRef<number | null>(null)

  // Interactive patina comparison drag state (0 = full Year 5 Patina, 100 = full Day 1 Raw)
  const [splitPct, setSplitPct] = useState(50)
  const [splitAnimate, setSplitAnimate] = useState(false)
  const [isDraggingPatina, setIsDraggingPatina] = useState(false)
  const activePointerIdRef = useRef<number | null>(null)

  // Refs for Lenis & GSAP ScrollTrigger choreography
  const rootRef = useRef<HTMLDivElement | null>(null)
  const heroSectionRef = useRef<HTMLElement | null>(null)
  const heroImgRef = useRef<HTMLImageElement | null>(null)
  const showcaseRef = useRef<HTMLElement | null>(null)
  const frameRefs = useRef<(HTMLElement | null)[]>([])
  const collectionGridRef = useRef<HTMLDivElement | null>(null)
  const craftSectionRef = useRef<HTMLElement | null>(null)
  const lenisRef = useRef<Lenis | null>(null)
  const showcaseTriggerRef = useRef<ScrollTrigger | null>(null)

  // Initialize Lenis smooth inertial scrolling synced to GSAP ticker + ScrollTrigger animations
  useEffect(() => {
    const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches

    let lenis: Lenis | null = null
    let lenisTicker: ((time: number) => void) | null = null

    if (!prefersReduced) {
      lenis = new Lenis({
        duration: 1.15,
        easing: (t: number) => Math.min(1, 1.001 - Math.pow(2, -10 * t)),
        smoothWheel: true,
      })
      lenisRef.current = lenis
      lenis.on("scroll", ScrollTrigger.update)
      lenisTicker = (time: number) => {
        lenis?.raf(time * 1000)
      }
      gsap.ticker.add(lenisTicker)
      gsap.ticker.lagSmoothing(0)
    }

    const gsapCtx = gsap.context(() => {
      if (prefersReduced) return

      // 1. Subtle weighted parallax drift on the full-bleed campaign hero
      if (heroSectionRef.current && heroImgRef.current) {
        gsap.fromTo(
          heroImgRef.current,
          { yPercent: 0, scale: 1 },
          {
            yPercent: 9,
            scale: 1.05,
            ease: "none",
            scrollTrigger: {
              trigger: heroSectionRef.current,
              start: "top top",
              end: "bottom top",
              scrub: true,
            },
          }
        )
      }

      // 2. Pinned 4-stage Atelier Showcase (pin: true, scrub: 0.65, +=280% horizontal clip-path wipes)
      const frames = frameRefs.current.filter((el): el is HTMLElement => el !== null)
      if (showcaseRef.current && frames.length === ATELIER_CHAPTERS.length) {
        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: showcaseRef.current,
            start: "top top",
            end: "+=280%",
            pin: true,
            scrub: 0.65,
            anticipatePin: 1,
            onUpdate: (self) => {
              const t = self.progress * 3.5
              let nextIdx = 0
              if (t >= 3.07) nextIdx = 3
              else if (t >= 2.02) nextIdx = 2
              else if (t >= 0.97) nextIdx = 1

              if (nextIdx !== stageIdxRef.current) {
                stageIdxRef.current = nextIdx
                setCaptionBlur(true)
                if (stageTimerRef.current !== null) {
                  window.clearTimeout(stageTimerRef.current)
                }
                stageTimerRef.current = window.setTimeout(() => {
                  stageTimerRef.current = null
                  setActiveStage(nextIdx)
                  setCaptionBlur(false)
                }, 100)
              }
            },
          },
        })

        showcaseTriggerRef.current = tl.scrollTrigger ?? null

        const wipeStarts = [0.25, 1.3, 2.35]
        frames.forEach((frame, i) => {
          if (i === 0) return
          tl.fromTo(
            frame,
            { clipPath: "inset(0 0 0 100%)" },
            { clipPath: "inset(0 0 0 0%)", ease: "none", duration: 0.8 },
            wipeStarts[i - 1]
          )
        })
        tl.set({}, {}, 3.5)
      }

      // 3. Staggered editorial entrance (Emil's 30-80ms non-uniform cascade: 0ms, 50ms, 90ms)
      if (collectionGridRef.current) {
        const cards = collectionGridRef.current.querySelectorAll("[data-maison-card]")
        if (cards.length > 0) {
          gsap.from(cards, {
            y: 16,
            opacity: 0,
            duration: 0.45,
            stagger: (idx: number) => (idx === 0 ? 0 : idx === 1 ? 0.05 : 0.09),
            ease: "power3.out",
            clearProps: "transform,opacity",
            scrollTrigger: {
              trigger: collectionGridRef.current,
              start: "top 84%",
              once: true,
            },
          })
        }
      }

      // 4. Calm editorial reveal on Tannery Provenance split
      if (craftSectionRef.current) {
        const craftCols = craftSectionRef.current.querySelectorAll("[data-craft-reveal]")
        if (craftCols.length > 0) {
          gsap.from(craftCols, {
            y: 18,
            opacity: 0,
            duration: 0.5,
            stagger: 0.07,
            ease: "power3.out",
            clearProps: "transform,opacity",
            scrollTrigger: {
              trigger: craftSectionRef.current,
              start: "top 84%",
              once: true,
            },
          })
        }
      }
    }, rootRef)

    const refreshId = window.setTimeout(() => {
      ScrollTrigger.refresh()
    }, 150)

    return () => {
      window.clearTimeout(refreshId)
      if (stageTimerRef.current !== null) {
        window.clearTimeout(stageTimerRef.current)
        stageTimerRef.current = null
      }
      showcaseTriggerRef.current = null
      gsapCtx.revert()
      if (lenisTicker) gsap.ticker.remove(lenisTicker)
      lenis?.destroy()
      lenisRef.current = null
    }
  }, [])

  // Pause background Lenis scroll while product modal or bag drawer is open
  useEffect(() => {
    if (cartOpen || activeProduct) {
      lenisRef.current?.stop()
    } else {
      lenisRef.current?.start()
    }
  }, [cartOpen, activeProduct])

  // Close overlays on Escape (instant keyboard action)
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return
      if (activeProduct) onSelectProduct(null)
      else if (cartOpen) onToggleCart(false)
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [cartOpen, activeProduct, onSelectProduct, onToggleCart])

  const handleAnchorScroll = (e: React.MouseEvent<HTMLAnchorElement>, selector: string) => {
    if (!selector.startsWith("#")) return
    if (selector === "#") {
      e.preventDefault()
      if (lenisRef.current) lenisRef.current.scrollTo(0)
      else window.scrollTo({ top: 0, behavior: "smooth" })
      return
    }
    const target = document.querySelector(selector)
    if (!target) return
    e.preventDefault()
    if (lenisRef.current) {
      lenisRef.current.scrollTo(target as HTMLElement, { offset: 0 })
    } else {
      target.scrollIntoView({ behavior: "smooth" })
    }
  }

  const handleStageSelect = (idx: number) => {
    const trigger = showcaseTriggerRef.current
    const targetChapter = ATELIER_CHAPTERS[idx]
    if (trigger && targetChapter) {
      const targetY = trigger.start + (trigger.end - trigger.start) * targetChapter.targetProgress
      if (lenisRef.current) {
        lenisRef.current.scrollTo(targetY)
      } else {
        window.scrollTo({ top: targetY, behavior: "smooth" })
      }
      return
    }

    // Reduced-motion fallback
    stageIdxRef.current = idx
    setCaptionBlur(true)
    if (stageTimerRef.current !== null) window.clearTimeout(stageTimerRef.current)
    stageTimerRef.current = window.setTimeout(() => {
      setActiveStage(idx)
      setCaptionBlur(false)
      stageTimerRef.current = null
    }, 100)
    frameRefs.current.forEach((frame, i) => {
      if (!frame) return
      frame.style.clipPath = i <= idx ? "inset(0 0 0 0%)" : "inset(0 0 0 100%)"
    })
  }

  const updateSplit = (pct: number, animate: boolean) => {
    setSplitAnimate(animate)
    setSplitPct(Math.max(0, Math.min(100, pct)))
  }

  // Multi-touch guarded 1:1 pointer capture drag (Emil Kowalski RECIPES.md)
  const handlePatinaPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (activePointerIdRef.current !== null) return
    activePointerIdRef.current = e.pointerId
    setIsDraggingPatina(true)
    e.currentTarget.setPointerCapture(e.pointerId)
    const rect = e.currentTarget.getBoundingClientRect()
    updateSplit(((e.clientX - rect.left) / rect.width) * 100, false)
  }

  const handlePatinaPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (activePointerIdRef.current !== e.pointerId) return
    const rect = e.currentTarget.getBoundingClientRect()
    updateSplit(((e.clientX - rect.left) / rect.width) * 100, false)
  }

  const handlePatinaPointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    if (activePointerIdRef.current !== e.pointerId) return
    activePointerIdRef.current = null
    setIsDraggingPatina(false)
  }

  // Apple momentum projection drag-to-dismiss on the Bag Sheet
  const handleBagDragEnd = (_event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => {
    const projectedX = info.offset.x + projectMomentum(info.velocity.x, 0.99)
    if (info.offset.x > 120 || info.velocity.x > 450 || projectedX > 180) {
      onToggleCart(false)
    }
  }

  return (
    <div ref={rootRef} className={styles.root}>
      {/* Strict 3-Zone Apple Translucent Top Bar */}
      <header className={styles.header}>
        <div className="mx-auto flex h-full max-w-[1280px] items-center justify-between gap-8 px-6 lg:px-10">
          <a
            href="#"
            onClick={(e) => handleAnchorScroll(e, "#")}
            className={`${styles.displayFont} ${styles.pressable} shrink-0 whitespace-nowrap text-[24px] font-semibold tracking-[0.12em] text-[var(--theme-ink)]`}
          >
            {tenant.brandName}
          </a>

          <nav aria-label="Primary" className="hidden items-center gap-2 md:flex">
            {tenant.themeConfig.navigation.map((item) => (
              <a
                key={item.label}
                href={item.href}
                onClick={(e) => handleAnchorScroll(e, item.href)}
                className={`${styles.navLink} shrink-0 whitespace-nowrap`}
              >
                {item.label}
              </a>
            ))}
          </nav>

          <button
            type="button"
            onClick={() => onToggleCart(true)}
            className={`${styles.bagTrigger} shrink-0 whitespace-nowrap`}
            aria-label={`Open shopping bag with ${cartTotals.itemCount} items`}
            data-testid="maison-bag-trigger"
          >
            <AtelierBagIcon className="h-[17px] w-[17px]" />
            <span>Bag ({cartTotals.itemCount})</span>
          </button>
        </div>
      </header>

      <main>
        {/* Section 1: Full-Bleed Sunlit Atelier Campaign Hero with Scrubbed Parallax */}
        {hero && (
          <section id="hero" ref={heroSectionRef} className={styles.heroFrame}>
            <div className={styles.heroMediaWrap}>
              <img
                ref={heroImgRef}
                src={hero.mediaPath}
                alt={hero.mediaAlt}
                referrerPolicy="no-referrer"
                className={styles.heroImg}
              />
              <div className={`${styles.heroScrim} absolute inset-0`} />
            </div>

            <div className="relative z-10 mx-auto flex h-full max-w-[1280px] flex-col justify-end px-6 pb-14 sm:pb-20 lg:px-10">
              <div className="max-w-[600px]">
                <h1
                  className={`${styles.displayFont} text-[40px] font-medium leading-[1.03] text-[#FAF7F2] sm:text-[58px]`}
                  style={{ textWrap: "balance" }}
                >
                  {hero.headline}
                </h1>
                <p className="mt-4 max-w-[490px] text-[15px] leading-[1.68] text-[#EDE7DC]">
                  {hero.subcopy}
                </p>
                <div className="mt-8 flex flex-wrap items-center gap-3.5">
                  <a
                    href={hero.ctaHref}
                    onClick={(e) => handleAnchorScroll(e, hero.ctaHref)}
                    className={`${styles.heroPrimaryCta} whitespace-nowrap`}
                  >
                    <span>{hero.ctaLabel}</span>
                    <HairlineArrowIcon className="h-4 w-4" />
                  </a>
                  <a
                    href="#showcase"
                    onClick={(e) => handleAnchorScroll(e, "#showcase")}
                    className={`${styles.heroGlassCta} whitespace-nowrap`}
                  >
                    <span>Inside the Atelier</span>
                  </a>
                </div>
              </div>
            </div>
          </section>
        )}

        {/* Section 2: Pinned 4-Stage Atelier Craft Showcase (GSAP ScrollTrigger pin: true + scrub: 0.65) */}
        <section id="showcase" ref={showcaseRef} className={styles.showcaseSection}>
          <div className={styles.showcaseViewport} data-testid="maison-showcase-viewport">
            {ATELIER_CHAPTERS.map((chapter, idx) => (
              <figure
                key={chapter.src}
                ref={(el) => {
                  frameRefs.current[idx] = el
                }}
                className={styles.showcaseFrame}
                data-stage-index={idx}
                style={{
                  clipPath: idx === 0 ? "inset(0 0 0 0%)" : "inset(0 0 0 100%)",
                }}
              >
                <img
                  src={chapter.src}
                  alt={chapter.alt}
                  referrerPolicy="no-referrer"
                  className={styles.showcaseImg}
                />
              </figure>
            ))}

            <div className={styles.showcaseScrim} aria-hidden="true" />

            <div className={styles.showcaseHud}>
              <div
                className={`${styles.showcaseCaption} ${
                  captionBlur ? styles.showcaseCaptionBlur : ""
                }`}
              >
                <p className={styles.chapterIndex}>
                  Chapter {ATELIER_CHAPTERS[activeStage].numeral} / IV
                </p>
                <h2 className={styles.showcaseTitle} data-testid="maison-showcase-title">
                  {ATELIER_CHAPTERS[activeStage].title}
                </h2>
                <p className={styles.showcaseSub}>{ATELIER_CHAPTERS[activeStage].sub}</p>
              </div>

              <div className={styles.stageBars} role="tablist" aria-label="Atelier craft chapters">
                {ATELIER_CHAPTERS.map((chapter, idx) => (
                  <button
                    key={chapter.numeral}
                    type="button"
                    role="tab"
                    aria-selected={activeStage === idx}
                    aria-label={`View Chapter ${chapter.numeral}: ${chapter.title}`}
                    onClick={() => handleStageSelect(idx)}
                    className={`${styles.stageBar} ${
                      activeStage === idx ? styles.stageBarActive : ""
                    }`}
                  />
                ))}
              </div>
            </div>
          </div>
        </section>

        {/* Section 3: Permanent Leather Objects (20px Apple Rounded Portrait Grid) */}
        <section id="collection" className="mx-auto max-w-[1280px] px-6 py-20 sm:py-24 lg:px-10">
          {featured && (
            <div
              className="flex flex-col justify-between gap-4 border-b pb-8 md:flex-row md:items-end"
              style={{ borderColor: "var(--theme-border)" }}
            >
              <div>
                <h2
                  className={`${styles.displayFont} text-[34px] font-medium leading-[1.08] text-[var(--theme-ink)] sm:text-[42px]`}
                >
                  {featured.heading}
                </h2>
              </div>
              <p className="max-w-[420px] text-[14.5px] leading-[1.65] text-[var(--theme-ink-muted)]">
                {featured.subcopy}
              </p>
            </div>
          )}

          <div
            ref={collectionGridRef}
            className="mt-12 grid grid-cols-1 gap-x-8 gap-y-14 sm:grid-cols-2 lg:grid-cols-3"
          >
            {tenant.catalog.map((product) => (
              <MaisonProductCard
                key={product.id}
                product={product}
                currency={tenant.currency}
                onSelect={() => onSelectProduct(product)}
                onQuickAdd={(variantIdx) => {
                  const chosen = product.variants[variantIdx] ?? product.variants[0]
                  onAddToCart({
                    variantId: chosen.id,
                    productId: product.id,
                    title: product.title,
                    variantLabel: chosen.colorName,
                    imagePath: chosen.imagePath,
                    unitPriceMinor: chosen.priceMinor,
                  })
                }}
              />
            ))}
          </div>
        </section>

        {/* Section 4: Interactive Patina Evolution Comparison (Day 1 Raw Vachetta vs Year 5 Patina) */}
        <section
          id="patina"
          className="border-t"
          style={{
            backgroundColor: "var(--theme-surface)",
            borderColor: "var(--theme-border)",
          }}
        >
          <div className="mx-auto grid max-w-[1280px] grid-cols-1 items-center gap-12 px-6 py-20 lg:grid-cols-[1.15fr_1fr] lg:gap-16 lg:px-10 lg:py-24">
            <div
              className={styles.patinaWrap}
              onPointerDown={handlePatinaPointerDown}
              onPointerMove={handlePatinaPointerMove}
              onPointerUp={handlePatinaPointerUp}
              onPointerCancel={handlePatinaPointerUp}
              role="slider"
              aria-label="Drag to compare Day 1 raw Vachetta leather against Year 5 sunlit amber patina"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={Math.round(splitPct)}
              data-testid="maison-patina-slider"
              tabIndex={0}
              onKeyDown={(e) => {
                // Emil Kowalski rule: never animate keyboard-initiated steps
                if (e.key === "ArrowLeft") updateSplit(splitPct - 10, false)
                if (e.key === "ArrowRight") updateSplit(splitPct + 10, false)
              }}
            >
              {/* Bottom layer: Year 5 Sunlit Amber Patina */}
              <div className={styles.patinaLayer}>
                <img
                  src="/media/maison-patina-year5.jpg"
                  alt="Maison Véla calfskin folio aged five years into a deep warm amber-cognac patina"
                  referrerPolicy="no-referrer"
                  className={styles.patinaImg}
                />
              </div>

              {/* Top clipped layer: Day 1 Raw Vachetta */}
              <div
                className={styles.patinaLayer}
                style={{
                  clipPath: `inset(0 ${100 - splitPct}% 0 0)`,
                  transition: splitAnimate
                    ? "clip-path 240ms cubic-bezier(0.23, 1, 0.32, 1)"
                    : "none",
                }}
              >
                <img
                  src="/media/maison-patina-day1.jpg"
                  alt="Freshly cut pale honey-sand raw vegetable-tanned Vachetta calfskin folio on travertine"
                  referrerPolicy="no-referrer"
                  className={styles.patinaImg}
                />
              </div>

              <span className={styles.patinaTag} style={{ left: 14 }}>
                Day 1 · Raw Vachetta
              </span>
              <span className={styles.patinaTag} style={{ right: 14 }}>
                Year 5 · Amber Patina
              </span>

              <div
                className={styles.patinaDivider}
                style={{
                  left: `${splitPct}%`,
                  transition: splitAnimate ? "left 240ms cubic-bezier(0.23, 1, 0.32, 1)" : "none",
                }}
              >
                <div
                  className={`${styles.patinaHandle} ${
                    isDraggingPatina ? styles.patinaHandleDragging : ""
                  }`}
                >
                  <HairlineCompareIcon className="h-4 w-4" />
                </div>
              </div>
            </div>

            <div>
              <h2
                className={`${styles.displayFont} text-[34px] font-medium leading-[1.08] text-[var(--theme-ink)] sm:text-[44px]`}
                style={{ textWrap: "balance" }}
              >
                Living leather, never sealed in plastic.
              </h2>
              <p className="mt-5 text-[15px] leading-[1.72] text-[var(--theme-ink-muted)]">
                Drag across the folio to trace five years of daylight, natural hand oils, and
                travel. Full-grain vegetable-tanned Vachetta deepens from pale honey-sand into
                burnished Tuscan amber without ever peeling or cracking.
              </p>

              <div className="mt-8 grid grid-cols-1 gap-3 sm:grid-cols-2">
                <button
                  type="button"
                  onClick={() => updateSplit(84, true)}
                  className={`${styles.patinaBtn} ${splitPct >= 55 ? styles.patinaBtnActive : ""}`}
                >
                  <div className={`${styles.displayFont} text-[24px] font-medium leading-none`}>
                    Day 1
                  </div>
                  <div className="mt-1.5 text-[12.5px] text-[var(--theme-ink-muted)]">
                    Unfinished raw Vachetta grain
                  </div>
                </button>
                <button
                  type="button"
                  onClick={() => updateSplit(16, true)}
                  className={`${styles.patinaBtn} ${splitPct <= 45 ? styles.patinaBtnActive : ""}`}
                >
                  <div className={`${styles.displayFont} text-[24px] font-medium leading-none`}>
                    Year 5
                  </div>
                  <div className="mt-1.5 text-[12.5px] text-[var(--theme-ink-muted)]">
                    Deep sunlit amber patina
                  </div>
                </button>
              </div>
            </div>
          </div>
        </section>

        {/* Section 5: Tannery Provenance & Macro Leathercraft Editorial Split */}
        {craft && (
          <section
            id="craft"
            ref={craftSectionRef}
            className="border-t"
            style={{
              backgroundColor: "var(--theme-canvas)",
              borderColor: "var(--theme-border)",
            }}
          >
            <div className="mx-auto grid max-w-[1280px] grid-cols-1 items-stretch gap-12 px-6 py-20 lg:grid-cols-[1.05fr_1fr] lg:gap-16 lg:px-10 lg:py-24">
              <div data-craft-reveal className={styles.craftMediaFrame}>
                <img
                  src={craft.mediaPath}
                  alt={craft.mediaAlt}
                  referrerPolicy="no-referrer"
                  className="h-full min-h-[320px] w-full object-cover"
                />
              </div>

              <div data-craft-reveal className="flex flex-col justify-center">
                <h2
                  className={`${styles.displayFont} text-[34px] font-medium leading-[1.08] text-[var(--theme-ink)] sm:text-[44px]`}
                  style={{ textWrap: "balance" }}
                >
                  {craft.heading}
                </h2>
                <p className="mt-5 text-[15px] leading-[1.72] text-[var(--theme-ink-muted)]">
                  {craft.body}
                </p>
                <div className="mt-6 flex items-center gap-2.5 text-[12.5px] tracking-[0.03em] text-[var(--theme-ink-muted)]">
                  <StitchMarkIcon className="h-4 w-4 shrink-0" />
                  <span>{craft.originNote}</span>
                </div>
                <div className="mt-7 border-t pt-6" style={{ borderColor: "var(--theme-border)" }}>
                  <button
                    type="button"
                    onClick={() => onSelectProduct(tenant.catalog[0] ?? null)}
                    className={`${styles.primaryAction} inline-flex items-center gap-3 px-6 py-3.5 text-[13.5px] font-medium`}
                  >
                    <span>Inspect Le Cabas Structuré</span>
                    <HairlineArrowIcon className="h-4 w-4" />
                  </button>
                </div>
              </div>
            </div>
          </section>
        )}
      </main>

      {/* Quiet Editorial Store Footer */}
      <footer className="border-t" style={{ borderColor: "var(--theme-border)" }}>
        <div className="mx-auto flex max-w-[1280px] flex-col justify-between gap-6 px-6 py-12 sm:flex-row sm:items-center lg:px-10">
          <div>
            <p
              className={`${styles.displayFont} text-[20px] font-semibold tracking-[0.12em] text-[var(--theme-ink)]`}
            >
              {tenant.brandName}
            </p>
            <p className="mt-1.5 text-[13px] text-[var(--theme-ink-muted)]">
              {tenant.themeConfig.footerNote}
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-6 text-[13px] text-[var(--theme-ink-muted)]">
            <a
              href="#showcase"
              onClick={(e) => handleAnchorScroll(e, "#showcase")}
              className="hover:text-[var(--theme-ink)]"
            >
              Showcase
            </a>
            <a
              href="#collection"
              onClick={(e) => handleAnchorScroll(e, "#collection")}
              className="hover:text-[var(--theme-ink)]"
            >
              Collection
            </a>
            <a
              href="#patina"
              onClick={(e) => handleAnchorScroll(e, "#patina")}
              className="hover:text-[var(--theme-ink)]"
            >
              Patina
            </a>
            <a href={`mailto:${tenant.contactEmail}`} className="hover:text-[var(--theme-ink)]">
              {tenant.contactEmail}
            </a>
          </div>
        </div>
      </footer>

      {/* Atelier PDP Modal (Apple critically damped spring + symmetric faster exit) */}
      <AnimatePresence>
        {activeProduct && (
          <AtelierProductModal
            key={activeProduct.id}
            product={activeProduct}
            currency={tenant.currency}
            onClose={() => onSelectProduct(null)}
            onAddToCart={onAddToCart}
          />
        )}
      </AnimatePresence>

      {/* Floating Apple Rounded Bag Sheet (Symmetric slide + Momentum Drag-to-Dismiss) */}
      <AnimatePresence>
        {cartOpen && (
          <motion.div
            key="maison-bag-backdrop"
            className="fixed inset-0 z-50 flex justify-end bg-black/40 p-2.5 sm:p-3.5"
            role="dialog"
            aria-modal="true"
            aria-label="Shopping bag"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.24, ease: EASE_OUT }}
            onClick={(e) => {
              if (e.target === e.currentTarget) onToggleCart(false)
            }}
          >
            <motion.aside
              data-testid="maison-bag-sheet"
              className={`${styles.sheetShell} flex h-full w-full max-w-[430px] flex-col overflow-hidden`}
              initial={reduceMotion ? { opacity: 0 } : { x: "106%" }}
              animate={reduceMotion ? { opacity: 1 } : { x: 0 }}
              exit={
                reduceMotion
                  ? { opacity: 0 }
                  : { x: "106%", transition: { duration: 0.28, ease: EASE_DRAWER } }
              }
              transition={
                reduceMotion
                  ? { duration: 0.18 }
                  : { type: "spring", duration: 0.42, bounce: 0 }
              }
              drag={reduceMotion ? false : "x"}
              dragConstraints={{ left: 0 }}
              dragElastic={{ left: 0.06, right: 0.35 }}
              dragSnapToOrigin
              onDragEnd={handleBagDragEnd}
            >
              <div
                className="flex items-center justify-between border-b px-6 py-5"
                style={{ borderColor: "var(--theme-border)" }}
              >
                <h2 className={`${styles.displayFont} text-[26px] font-medium`}>
                  Your Bag ({cartTotals.itemCount})
                </h2>
                <button
                  type="button"
                  onClick={() => onToggleCart(false)}
                  aria-label="Close bag"
                  className={styles.iconCircleBtn}
                >
                  <HairlineCloseIcon className="h-4 w-4" />
                </button>
              </div>

              <div className="flex-1 overflow-y-auto px-6 py-6">
                {cartItems.length === 0 ? (
                  <div className="py-16 text-center">
                    <p className={`${styles.displayFont} text-[24px] text-[var(--theme-ink)]`}>
                      Your bag is currently empty.
                    </p>
                    <p className="mt-2 text-[13.5px] text-[var(--theme-ink-muted)]">
                      Select a piece from our permanent leather collection to begin.
                    </p>
                  </div>
                ) : (
                  <ul className="divide-y" style={{ borderColor: "var(--theme-border)" }}>
                    {cartItems.map((item) => (
                      <li key={item.variantId} className="flex gap-4 py-5 first:pt-0">
                        <img
                          src={item.imagePath}
                          alt={item.title}
                          referrerPolicy="no-referrer"
                          className="h-24 w-20 shrink-0 rounded-[12px] object-cover"
                          style={{ backgroundColor: "var(--theme-surface)" }}
                        />
                        <div className="flex flex-1 flex-col justify-between">
                          <div className="flex items-start justify-between gap-2">
                            <div>
                              <p className={`${styles.displayFont} text-[21px] leading-tight`}>
                                {item.title}
                              </p>
                              <p className="mt-0.5 text-[12.5px] text-[var(--theme-ink-muted)]">
                                {item.variantLabel}
                              </p>
                            </div>
                            <span className="text-[13.5px] font-medium tabular-nums">
                              {formatMinorPrice(
                                item.unitPriceMinor * item.quantity,
                                tenant.currency
                              )}
                            </span>
                          </div>

                          <div className="mt-3 flex items-center justify-between">
                            <div
                              className="inline-flex items-center overflow-hidden rounded-full border"
                              style={{ borderColor: "var(--theme-border)" }}
                            >
                              <button
                                type="button"
                                onClick={() => onUpdateQuantity(item.variantId, -1)}
                                aria-label={`Decrease quantity of ${item.title}`}
                                className={`${styles.pressable} flex h-8 w-8 items-center justify-center hover:bg-[var(--theme-surface)]`}
                              >
                                <HairlineMinusIcon className="h-3.5 w-3.5" />
                              </button>
                              <span className="w-7 text-center text-[12.5px] tabular-nums">
                                {item.quantity}
                              </span>
                              <button
                                type="button"
                                onClick={() => onUpdateQuantity(item.variantId, 1)}
                                aria-label={`Increase quantity of ${item.title}`}
                                className={`${styles.pressable} flex h-8 w-8 items-center justify-center hover:bg-[var(--theme-surface)]`}
                              >
                                <HairlinePlusIcon className="h-3.5 w-3.5" />
                              </button>
                            </div>
                            <button
                              type="button"
                              onClick={() => onRemoveItem(item.variantId)}
                              className={`${styles.pressable} text-[12px] text-[var(--theme-ink-muted)] underline-offset-4 hover:text-[var(--theme-ink)] hover:underline`}
                            >
                              Remove
                            </button>
                          </div>
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
              </div>

              {cartItems.length > 0 && (
                <div
                  className="border-t px-6 py-5"
                  style={{ borderColor: "var(--theme-border)" }}
                >
                  <div className="flex items-center justify-between text-[14px]">
                    <span className="text-[var(--theme-ink-muted)]">Subtotal</span>
                    <span className="font-medium tabular-nums">
                      {formatMinorPrice(cartTotals.subtotalMinor, tenant.currency)}
                    </span>
                  </div>
                  <p className="mt-1 text-[12px] text-[var(--theme-ink-muted)]">
                    {cartTotals.qualifiesForFreeShipping
                      ? "Complimentary insured courier shipping unlocked."
                      : "Courier fee calculated at locked guest checkout."}
                  </p>
                  <button
                    type="button"
                    onClick={onProceedToCheckout}
                    className={`${styles.primaryAction} mt-4 flex w-full items-center justify-center gap-3 px-6 py-3.5 text-[13.5px] font-medium`}
                  >
                    <span>Proceed to Checkout</span>
                    <HairlineArrowIcon className="h-4 w-4" />
                  </button>
                </div>
              )}
            </motion.aside>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}
