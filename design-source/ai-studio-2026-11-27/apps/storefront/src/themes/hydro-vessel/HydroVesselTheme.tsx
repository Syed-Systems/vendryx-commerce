"use client"

import React, { useEffect, useMemo, useRef, useState } from "react"
import gsap from "gsap"
import { ScrollTrigger } from "gsap/ScrollTrigger"
import Lenis from "lenis"
import { formatMinorPrice } from "../../core/checkout-engine"
import type { StorefrontThemeProps } from "../../core/theme-contract"
import {
  AlpinePeakMark,
  CompareDragIcon,
  DiagonalExploreIcon,
  MoonModeIcon,
  SunModeIcon,
  VesselBagIcon,
  VesselCloseIcon,
} from "./icons"
import styles from "./hydro-vessel.module.css"
import { initHeroCanvas } from "./webgl/heroCanvas"
import { initSpecsCanvas } from "./webgl/specsCanvas"

gsap.registerPlugin(ScrollTrigger)

type FinishId = "sage" | "basalt" | "glacier"
type CapacityLabel = "500 ml" | "750 ml" | "950 ml"

interface FinishMeta {
  id: FinishId
  name: string
  dot: string
  className: string
}

const FINISH_LIST: FinishMeta[] = [
  { id: "sage", name: "Alpine Sage", dot: "#48634B", className: styles.finishSage },
  { id: "basalt", name: "Basalt Black", dot: "#262B28", className: styles.finishBasalt },
  { id: "glacier", name: "Glacier White", dot: "#DCE3DF", className: styles.finishGlacier },
]

const CAPACITY_SCALE: Record<CapacityLabel, number> = {
  "500 ml": 0.9,
  "750 ml": 0.96,
  "950 ml": 1.03,
}

const SHOWCASE_STAGES = [
  {
    src: "/media/kova-showcase-alpine.jpg",
    alt: "KOVA matte pine-sage stainless flask resting on a wet basalt boulder beside an alpine stream",
    title: "Made for the Outside",
    sub: "Zero condensation from mist-veiled trails to everyday city commutes.",
    targetProgress: 0.04,
  },
  {
    src: "/media/kova-showcase-trail.jpg",
    alt: "KOVA flask resting on a sunlit granite boulder overlooking golden alpine meadows and mountain lake",
    title: "Machined Steel Collar",
    sub: "Leakproof chug cap with silent carry loop engineered to take a knock.",
    targetProgress: 0.34,
  },
  {
    src: "/media/kova-thermal-cold.jpg",
    alt: "KOVA insulated vessel surrounded by glacial ice blocks on dark slate",
    title: "24 Hours Glacial Cold",
    sub: "Copper-lined double-wall vacuum insulation locks in ice through peak summer heat.",
    targetProgress: 0.64,
  },
  {
    src: "/media/kova-thermal-hot.jpg",
    alt: "KOVA insulated vessel with gentle rising steam on volcanic rock at sunrise",
    title: "12 Hours Piping Hot",
    sub: "Electropolished 18/8 steel interior preserves pure coffee and tea flavor.",
    targetProgress: 0.94,
  },
] as const

export function HydroVesselTheme({
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
  const [vesselMode, setVesselMode] = useState<"light" | "dark">("light")
  const vesselModeRef = useRef<"light" | "dark">("light")
  const [finish, setFinish] = useState<FinishId>("sage")
  const [capacity, setCapacity] = useState<CapacityLabel>("950 ml")
  const [includeAddon, setIncludeAddon] = useState(false)

  // Environmental showcase stage state (driven by GSAP ScrollTrigger scrub & clickable dots)
  const [activeStage, setActiveStage] = useState(0)
  const [captionBlur, setCaptionBlur] = useState(false)
  const stageIdxRef = useRef(0)
  const stageTimerRef = useRef<number | null>(null)

  // Thermal comparison drag slider state (0 = full Hot, 100 = full Cold)
  const [splitPct, setSplitPct] = useState(50)
  const [splitAnimate, setSplitAnimate] = useState(false)
  const draggingRef = useRef(false)

  // Refs for WebGL canvases, 3D tilt, Lenis, and GSAP ScrollTrigger
  const rootRef = useRef<HTMLDivElement | null>(null)
  const heroCanvasRef = useRef<HTMLCanvasElement | null>(null)
  const specsCanvasRef = useRef<HTMLCanvasElement | null>(null)
  const heroStageRef = useRef<HTMLDivElement | null>(null)
  const tiltWrapRef = useRef<HTMLDivElement | null>(null)
  const shadowRef = useRef<HTMLDivElement | null>(null)
  const showcaseRef = useRef<HTMLElement | null>(null)
  const frameRefs = useRef<(HTMLElement | null)[]>([])
  const bentoGridRef = useRef<HTMLDivElement | null>(null)
  const lenisRef = useRef<Lenis | null>(null)
  const showcaseTriggerRef = useRef<ScrollTrigger | null>(null)

  useEffect(() => {
    vesselModeRef.current = vesselMode
    window.dispatchEvent(new Event("resize"))
  }, [vesselMode])

  // Initialize WebGL2 contour shader, 3D wireframe canvas, Lenis smooth scroll, 3D spring tilt & GSAP ScrollTrigger
  useEffect(() => {
    const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches

    const cleanupHero = initHeroCanvas(heroCanvasRef.current, () => vesselModeRef.current)
    const cleanupSpecs = initSpecsCanvas(specsCanvasRef.current, () => vesselModeRef.current)

    // 1. Lenis inertial smooth scrolling synced directly into GSAP ticker
    let lenis: Lenis | null = null
    let lenisTicker: ((time: number) => void) | null = null

    if (!prefersReduced) {
      lenis = new Lenis({
        duration: 1.05,
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

    // 2. Silky spring-physics 3D pointer tilt driven by GSAP ticker
    let targetRotX = 0
    let targetRotY = 0
    let curRotX = 0
    let curRotY = 0
    let velX = 0
    let velY = 0
    let tiltTicker: (() => void) | null = null
    const heroStage = heroStageRef.current

    const onHeroMove = (e: PointerEvent) => {
      if (!heroStage) return
      const rect = heroStage.getBoundingClientRect()
      if (!rect.width || !rect.height) return
      const nx = ((e.clientX - rect.left) / rect.width - 0.5) * 2
      const ny = ((e.clientY - rect.top) / rect.height - 0.5) * 2
      targetRotY = nx * 13
      targetRotX = -ny * 9
    }

    const onHeroLeave = () => {
      targetRotX = 0
      targetRotY = 0
    }

    if (heroStage && tiltWrapRef.current && !prefersReduced) {
      heroStage.addEventListener("pointermove", onHeroMove)
      heroStage.addEventListener("pointerleave", onHeroLeave)
      tiltTicker = () => {
        const ax = (targetRotX - curRotX) * 0.08
        const ay = (targetRotY - curRotY) * 0.08
        velX = (velX + ax) * 0.82
        velY = (velY + ay) * 0.82
        curRotX += velX
        curRotY += velY
        if (tiltWrapRef.current) {
          tiltWrapRef.current.style.transform = `rotateX(${curRotX.toFixed(2)}deg) rotateY(${curRotY.toFixed(2)}deg)`
        }
        if (shadowRef.current) {
          shadowRef.current.style.transform = `translateX(${(-curRotY * 1.4).toFixed(1)}px) scale(${(
            1 -
            Math.abs(curRotX) * 0.01
          ).toFixed(3)})`
        }
      }
      gsap.ticker.add(tiltTicker)
    }

    // 3. GSAP ScrollTrigger Pinned 4-Stage Horizontal Clip-Path Wipe + Bento Reveal
    const gsapCtx = gsap.context(() => {
      if (prefersReduced) return

      const frames = frameRefs.current.filter((el): el is HTMLElement => el !== null)

      if (showcaseRef.current && frames.length === SHOWCASE_STAGES.length) {
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

        // Dwell + horizontal clip-path wipe cadence matching flagship pacing
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

      if (bentoGridRef.current) {
        const tiles = bentoGridRef.current.querySelectorAll("[data-reveal-tile]")
        if (tiles.length > 0) {
          gsap.from(tiles, {
            y: 28,
            scale: 0.96,
            opacity: 0,
            duration: 0.65,
            stagger: 0.08,
            ease: "power3.out",
            clearProps: "transform,opacity",
            scrollTrigger: {
              trigger: bentoGridRef.current,
              start: "top 82%",
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
      cleanupHero()
      cleanupSpecs()
      showcaseTriggerRef.current = null
      gsapCtx.revert()
      if (tiltTicker) gsap.ticker.remove(tiltTicker)
      if (lenisTicker) gsap.ticker.remove(lenisTicker)
      lenis?.destroy()
      lenisRef.current = null
      heroStage?.removeEventListener("pointermove", onHeroMove)
      heroStage?.removeEventListener("pointerleave", onHeroLeave)
    }
  }, [])

  // Close bag drawer on Escape
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && cartOpen) onToggleCart(false)
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [cartOpen, onToggleCart])

  const flagshipProduct = tenant.catalog[0]
  const activeFinishMeta = FINISH_LIST.find((f) => f.id === finish) ?? FINISH_LIST[0]

  const capacityOptions = useMemo(() => {
    const variants = flagshipProduct?.variants ?? []
    const labels: CapacityLabel[] = ["500 ml", "750 ml", "950 ml"]
    return labels.map((label, idx) => {
      const matched =
        variants.find((v) => v.sizeLabel?.toLowerCase() === label.toLowerCase()) ??
        variants[idx] ??
        variants[0]
      return {
        label,
        variantId: matched?.id ?? `kova-${label}`,
        priceMinor: matched?.priceMinor ?? (idx === 0 ? 3800 : idx === 1 ? 4400 : 4800),
      }
    })
  }, [flagshipProduct])

  const selectedCapacity =
    capacityOptions.find((c) => c.label === capacity) ?? capacityOptions[capacityOptions.length - 1]
  const addon = flagshipProduct?.accessoryAddon
  const configuredTotalMinor =
    (selectedCapacity?.priceMinor ?? 4800) + (includeAddon && addon ? addon.priceMinor : 0)

  const heroSection = tenant.themeConfig.sections.find((s) => s.type === "hero")

  // Smoothly scroll via Lenis when clicking nav links or Hero CTAs
  const handleAnchorScroll = (e: React.MouseEvent<HTMLAnchorElement>, selector: string) => {
    if (!selector.startsWith("#")) return
    const target = document.querySelector(selector)
    if (!target) return
    e.preventDefault()
    if (lenisRef.current) {
      lenisRef.current.scrollTo(target as HTMLElement, { offset: 0 })
    } else {
      target.scrollIntoView({ behavior: "smooth" })
    }
  }

  // Clicking a showcase progress pill glides smoothly to that exact pinned scrub position
  const handleStageSelect = (idx: number) => {
    const trigger = showcaseTriggerRef.current
    const targetStage = SHOWCASE_STAGES[idx]
    if (trigger && targetStage) {
      const targetY = trigger.start + (trigger.end - trigger.start) * targetStage.targetProgress
      if (lenisRef.current) {
        lenisRef.current.scrollTo(targetY)
      } else {
        window.scrollTo({ top: targetY, behavior: "smooth" })
      }
      return
    }

    // Reduced-motion fallback: snap frames directly
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

  const handleCompPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    draggingRef.current = true
    e.currentTarget.setPointerCapture(e.pointerId)
    const rect = e.currentTarget.getBoundingClientRect()
    updateSplit(((e.clientX - rect.left) / rect.width) * 100, false)
  }

  const handleCompPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!draggingRef.current) return
    const rect = e.currentTarget.getBoundingClientRect()
    updateSplit(((e.clientX - rect.left) / rect.width) * 100, false)
  }

  const handleCompPointerUp = () => {
    draggingRef.current = false
  }

  const handleCommitToBag = () => {
    if (!flagshipProduct || !selectedCapacity) return
    onAddToCart({
      variantId: `${selectedCapacity.variantId}-${finish}`,
      productId: flagshipProduct.id,
      title: flagshipProduct.title,
      variantLabel: `${activeFinishMeta.name} · ${selectedCapacity.label}`,
      imagePath: "/media/kova-hero-studio.jpg",
      unitPriceMinor: selectedCapacity.priceMinor,
    })

    if (includeAddon && addon) {
      onAddToCart({
        variantId: addon.variantId,
        productId: flagshipProduct.id,
        title: addon.title,
        variantLabel: `${activeFinishMeta.name} Matched Fit`,
        imagePath: "/media/kova-hero-studio.jpg",
        unitPriceMinor: addon.priceMinor,
      })
    }
  }

  return (
    <div ref={rootRef} className={styles.root} data-vessel-mode={vesselMode}>
      <div className={styles.grainOverlay} aria-hidden="true" />

      {/* Strict 3-Zone Top Bar */}
      <header className={styles.siteHeader}>
        <div className={`${styles.container} ${styles.headerInner}`}>
          <a
            href="#hero"
            onClick={(e) => handleAnchorScroll(e, "#hero")}
            className={styles.brandLockup}
            aria-label={`${tenant.brandName} Home`}
          >
            <AlpinePeakMark className={`${styles.brandMark} h-[15px] w-[24px]`} />
            <span>{tenant.brandName}</span>
          </a>

          <nav className={styles.headerNav} aria-label="Primary">
            <a
              href="#showcase"
              onClick={(e) => handleAnchorScroll(e, "#showcase")}
              className={styles.navLink}
            >
              Showcase
            </a>
            <a
              href="#thermal"
              onClick={(e) => handleAnchorScroll(e, "#thermal")}
              className={styles.navLink}
            >
              Thermal
            </a>
            <a
              href="#craft"
              onClick={(e) => handleAnchorScroll(e, "#craft")}
              className={styles.navLink}
            >
              Specs
            </a>
            <a
              href="#order"
              onClick={(e) => handleAnchorScroll(e, "#order")}
              className={styles.navLink}
            >
              Configure
            </a>
          </nav>

          <div className={styles.headerActions}>
            <button
              type="button"
              onClick={() => setVesselMode((prev) => (prev === "light" ? "dark" : "light"))}
              className={styles.iconBtn}
              aria-label="Toggle mineral light or basalt dark theme"
              data-testid="kova-theme-toggle"
            >
              {vesselMode === "light" ? <MoonModeIcon /> : <SunModeIcon />}
            </button>

            <button
              type="button"
              onClick={() => onToggleCart(true)}
              className={styles.iconBtn}
              aria-label={`Open bag with ${cartTotals.itemCount} items`}
              data-testid="kova-bag-trigger"
            >
              <VesselBagIcon />
              <span className={styles.bagCount}>{cartTotals.itemCount}</span>
            </button>
          </div>
        </div>
      </header>

      <main>
        {/* SECTION 1: Asymmetric Split Hero with Live WebGL2 Topographic Field */}
        <section id="hero" className={styles.hero}>
          <canvas ref={heroCanvasRef} className={styles.heroCanvas} aria-hidden="true" />

          <div className={`${styles.container} ${styles.heroGrid}`}>
            <div className={styles.heroCopy}>
              <h1 className={styles.heroTitle}>
                {heroSection?.headline ?? "Built for altitude."}
              </h1>
              <p className={styles.heroSub}>
                {heroSection?.subcopy ??
                  "A copper-lined 18/8 stainless vessel built for long ridge days and everyday commutes—keeping drinks glacial cold or steaming without ever sweating in your bag."}
              </p>

              <div className={styles.heroActions}>
                <a
                  href="#order"
                  onClick={(e) => handleAnchorScroll(e, "#order")}
                  className={`${styles.btn} ${styles.btnPrimary}`}
                >
                  {heroSection?.ctaLabel ?? "Shop Flask"}
                </a>
                <a
                  href="#showcase"
                  onClick={(e) => handleAnchorScroll(e, "#showcase")}
                  className={`${styles.btn} ${styles.btnGhost}`}
                >
                  <span>Explore</span>
                  <DiagonalExploreIcon />
                </a>
              </div>
            </div>

            <div className={styles.heroStage} ref={heroStageRef}>
              <div className={styles.flaskTiltWrap} ref={tiltWrapRef}>
                <img
                  src="/media/kova-hero-studio.jpg"
                  alt={`${tenant.brandName} Alpine Flask in ${activeFinishMeta.name} finish`}
                  referrerPolicy="no-referrer"
                  className={`${styles.flaskHeroImg} ${activeFinishMeta.className}`}
                />
              </div>

              <div className={styles.flaskShadow} ref={shadowRef} aria-hidden="true" />

              <div className={styles.heroSwatches} role="radiogroup" aria-label="Select flask finish">
                {FINISH_LIST.map((item) => {
                  const active = finish === item.id
                  return (
                    <button
                      key={item.id}
                      type="button"
                      role="radio"
                      aria-checked={active}
                      onClick={() => setFinish(item.id)}
                      className={`${styles.swatchBtn} ${active ? styles.swatchBtnActive : ""}`}
                    >
                      <span className={styles.swatchDot} style={{ background: item.dot }} />
                      <span>{item.name}</span>
                    </button>
                  )
                })}
              </div>
            </div>
          </div>
        </section>

        {/* SECTION 2: Pinned 4-Angle Environmental Showcase (GSAP ScrollTrigger pin: true + scrub: 0.65) */}
        <section id="showcase" ref={showcaseRef} className={styles.showcaseSection}>
          <div className={styles.showcaseViewport} data-testid="kova-showcase-viewport">
            {SHOWCASE_STAGES.map((stage, idx) => (
              <figure
                key={stage.src}
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
                  src={stage.src}
                  alt={stage.alt}
                  referrerPolicy="no-referrer"
                  className={styles.showcaseImg}
                />
              </figure>
            ))}

            <div className={styles.showcaseScrim} aria-hidden="true" />

            <div className={`${styles.container} ${styles.showcaseHud}`}>
              <div
                className={`${styles.showcaseCaption} ${
                  captionBlur ? styles.showcaseCaptionBlur : ""
                }`}
              >
                <h2 className={styles.captionTitle} data-testid="showcase-caption-title">
                  {SHOWCASE_STAGES[activeStage].title}
                </h2>
                <p className={styles.captionSub}>{SHOWCASE_STAGES[activeStage].sub}</p>
              </div>

              <div className={styles.showcaseDots} role="tablist" aria-label="Showcase angles">
                {SHOWCASE_STAGES.map((stage, idx) => (
                  <button
                    key={stage.title}
                    type="button"
                    role="tab"
                    aria-selected={activeStage === idx}
                    aria-label={`View ${stage.title}`}
                    onClick={() => handleStageSelect(idx)}
                    className={`${styles.stageDot} ${
                      activeStage === idx ? styles.stageDotActive : ""
                    }`}
                  />
                ))}
              </div>
            </div>
          </div>
        </section>

        {/* SECTION 3: Interactive Thermal Before/After Drag Comparison */}
        <section id="thermal" className={styles.thermalSection}>
          <div className={`${styles.container} ${styles.thermalGrid}`}>
            <div
              className={styles.comparisonWrap}
              onPointerDown={handleCompPointerDown}
              onPointerMove={handleCompPointerMove}
              onPointerUp={handleCompPointerUp}
              onPointerCancel={handleCompPointerUp}
              role="slider"
              aria-label="Drag to compare 24-hour cold retention versus 12-hour hot retention"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={Math.round(splitPct)}
              tabIndex={0}
              onKeyDown={(e) => {
                if (e.key === "ArrowLeft") updateSplit(splitPct - 10, true)
                if (e.key === "ArrowRight") updateSplit(splitPct + 10, true)
              }}
            >
              <div className={styles.comparisonLayer}>
                <img
                  src="/media/kova-thermal-hot.jpg"
                  alt="KOVA Flask keeping drinks steaming hot for 12 hours on dark volcanic stone"
                  referrerPolicy="no-referrer"
                  className={styles.comparisonImg}
                />
              </div>

              <div
                className={styles.comparisonLayer}
                style={{
                  clipPath: `inset(0 ${100 - splitPct}% 0 0)`,
                  transition: splitAnimate ? "clip-path 360ms var(--hv-ease-out)" : "none",
                }}
              >
                <img
                  src="/media/kova-thermal-cold.jpg"
                  alt="KOVA Flask holding ice cold for 24 hours surrounded by glacial ice blocks"
                  referrerPolicy="no-referrer"
                  className={styles.comparisonImg}
                />
              </div>

              <span className={styles.comparisonTag} style={{ left: 16 }}>
                24H COLD
              </span>
              <span className={styles.comparisonTag} style={{ right: 16 }}>
                12H HOT
              </span>

              <div
                className={styles.comparisonDivider}
                style={{
                  left: `${splitPct}%`,
                  transition: splitAnimate ? "left 360ms var(--hv-ease-out)" : "none",
                }}
              >
                <div className={styles.comparisonHandle}>
                  <CompareDragIcon />
                </div>
              </div>
            </div>

            <div>
              <h2 className={styles.sectionTitle}>Double-wall vacuum shield.</h2>
              <p className={styles.sectionSub}>
                An ultra-thin copper barrier inside the vacuum chamber reflects radiant heat back
                into your drink while keeping the matte exterior completely dry to the touch.
              </p>

              <div className={styles.thermalMetrics}>
                <button
                  type="button"
                  onClick={() => updateSplit(84, true)}
                  className={`${styles.metricBtn} ${
                    splitPct >= 55 ? styles.metricBtnActive : ""
                  }`}
                >
                  <div className={styles.metricValue}>24 hrs</div>
                  <div className={styles.metricLabel}>Glacial cold retention</div>
                </button>
                <button
                  type="button"
                  onClick={() => updateSplit(16, true)}
                  className={`${styles.metricBtn} ${
                    splitPct <= 45 ? styles.metricBtnActive : ""
                  }`}
                >
                  <div className={styles.metricValue}>12 hrs</div>
                  <div className={styles.metricLabel}>Steaming hot retention</div>
                </button>
              </div>
            </div>
          </div>
        </section>

        {/* SECTION 4: Asymmetric 12-Column 3-Tile Editorial Bento + 3D Architectural Wireframe */}
        <section id="craft" className={styles.bentoSection}>
          <div className={styles.container}>
            <div className={styles.sectionHeaderStack}>
              <h2 className={styles.sectionTitle}>Engineered down to the collar.</h2>
            </div>

            <div className={styles.bentoGrid} ref={bentoGridRef}>
              <article data-reveal-tile className={`${styles.bentoTile} ${styles.bentoTall}`}>
                <img
                  src="/media/kova-showcase-alpine.jpg"
                  alt="KOVA Flask resting beside a rushing alpine stream"
                  referrerPolicy="no-referrer"
                  className={styles.bentoImg}
                />
                <div className={styles.bentoFooter}>
                  <h3 className={styles.bentoTitle}>Powder-Coat Armor</h3>
                  <p className={styles.bentoSub}>
                    Micro-textured mineral finish engineered for wet grip and rock-proof durability.
                  </p>
                </div>
              </article>

              <article data-reveal-tile className={`${styles.bentoTile} ${styles.bentoTop}`}>
                <img
                  src="/media/kova-showcase-trail.jpg"
                  alt="KOVA Flask with leakproof chug cap on sunlit granite"
                  referrerPolicy="no-referrer"
                  className={styles.bentoImg}
                />
                <div className={styles.bentoFooter}>
                  <h3 className={styles.bentoTitle}>Throw-In-Bag Seal</h3>
                  <p className={styles.bentoSub}>
                    Food-grade silicone gasket rated leakproof under cabin pressure.
                  </p>
                </div>
              </article>

              <article data-reveal-tile className={`${styles.bentoTile} ${styles.bentoBottom}`}>
                <canvas
                  ref={specsCanvasRef}
                  className={styles.specsCanvas}
                  aria-label="Rotating 3D architectural wireframe cross-section of KOVA flask"
                />
                <div className={styles.bentoFooterSpecs}>
                  <div>
                    <h3 className={styles.bentoTitle}>950 ml</h3>
                    <p className={styles.bentoSub}>520 g empty weight</p>
                  </div>
                  <span className={styles.specReadout}>18/8 Pro-Grade Steel</span>
                </div>
              </article>
            </div>
          </div>
        </section>

        {/* SECTION 5: Interactive Finish & Capacity Configurator + Order */}
        <section id="order" className={styles.orderSection}>
          <div className={`${styles.container} ${styles.orderGrid}`}>
            <div className={styles.orderPreview}>
              <img
                src="/media/kova-hero-studio.jpg"
                alt={`Configured ${flagshipProduct?.title ?? "KOVA Flask"} in ${activeFinishMeta.name} (${capacity})`}
                referrerPolicy="no-referrer"
                className={`${styles.orderFlaskImg} ${activeFinishMeta.className}`}
                style={{ transform: `scale(${CAPACITY_SCALE[capacity]})` }}
              />
            </div>

            <div>
              <h2 className={styles.sectionTitle}>
                {flagshipProduct?.title ?? "KOVA Alpine Flask"}
              </h2>
              <p className={styles.orderPrice} data-testid="kova-configured-price">
                {formatMinorPrice(configuredTotalMinor, tenant.currency)}
              </p>

              <div className={styles.configGroup}>
                <span className={styles.configLabel}>Finish</span>
                <div className={styles.optionRow}>
                  {FINISH_LIST.map((item) => {
                    const active = finish === item.id
                    return (
                      <button
                        key={item.id}
                        type="button"
                        onClick={() => setFinish(item.id)}
                        className={`${styles.pillOption} ${active ? styles.pillOptionActive : ""}`}
                      >
                        <span className={styles.swatchDot} style={{ background: item.dot }} />
                        <span>{item.name}</span>
                      </button>
                    )
                  })}
                </div>
              </div>

              <div className={styles.configGroup}>
                <span className={styles.configLabel}>Capacity</span>
                <div className={styles.optionRow}>
                  {capacityOptions.map((opt) => {
                    const active = capacity === opt.label
                    return (
                      <button
                        key={opt.label}
                        type="button"
                        onClick={() => setCapacity(opt.label)}
                        className={`${styles.pillOption} ${active ? styles.pillOptionActive : ""}`}
                      >
                        {opt.label}
                      </button>
                    )
                  })}
                </div>
              </div>

              {addon && (
                <label className={styles.addonBox}>
                  <input
                    type="checkbox"
                    checked={includeAddon}
                    onChange={(e) => setIncludeAddon(e.target.checked)}
                    data-testid="kova-addon-checkbox"
                    className="h-4 w-4 accent-[#3d5740]"
                  />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center justify-between gap-2 text-sm font-medium">
                      <span>Add {addon.title}</span>
                      <span className={styles.mono}>
                        +{formatMinorPrice(addon.priceMinor, tenant.currency)}
                      </span>
                    </div>
                    <p className="text-xs opacity-75">{addon.subtitle}</p>
                  </div>
                </label>
              )}

              <div className={styles.specList}>
                <div className={styles.specRow}>
                  <span className={styles.specKey}>Insulation</span>
                  <span className={styles.specVal}>24h cold, 12h hot</span>
                </div>
                <div className={styles.specRow}>
                  <span className={styles.specKey}>Steel</span>
                  <span className={styles.specVal}>18/8 electropolished double wall</span>
                </div>
                <div className={styles.specRow}>
                  <span className={styles.specKey}>Warranty</span>
                  <span className={styles.specVal}>Lifetime thermal seal guarantee</span>
                </div>
              </div>

              <button
                type="button"
                onClick={handleCommitToBag}
                data-testid="kova-add-to-bag"
                className={`${styles.btn} ${styles.btnPrimary} ${styles.btnFull}`}
              >
                Add to Bag — {formatMinorPrice(configuredTotalMinor, tenant.currency)}
              </button>
            </div>
          </div>
        </section>
      </main>

      {/* Minimal Single-Row Footer */}
      <footer className={styles.siteFooter}>
        <div className={`${styles.container} ${styles.footerInner}`}>
          <span className={styles.footerBrand}>{tenant.brandName}</span>
          <span>{tenant.themeConfig.footerNote}</span>
          <a href={`mailto:${tenant.contactEmail}`}>{tenant.contactEmail}</a>
        </div>
      </footer>

      {/* Slide-Over Order Bag Drawer wired to Locked Vendryx Core Checkout */}
      {cartOpen && (
        <>
          <div
            className={styles.drawerBackdrop}
            onClick={() => onToggleCart(false)}
            aria-hidden="true"
          />
          <aside
            className={styles.bagDrawer}
            role="dialog"
            aria-modal="true"
            aria-label="Shopping bag"
          >
            <div className={styles.drawerHeader}>
              <h2 className="text-xl font-bold tracking-tight">
                Bag ({cartTotals.itemCount})
              </h2>
              <button
                type="button"
                onClick={() => onToggleCart(false)}
                className={styles.iconBtn}
                aria-label="Close bag"
              >
                <VesselCloseIcon />
              </button>
            </div>

            <div className={styles.drawerBody}>
              {cartItems.length === 0 ? (
                <div className="flex h-full flex-col items-center justify-center gap-4 text-center">
                  <p className="text-base opacity-75">Your bag is empty.</p>
                  <a
                    href="#order"
                    onClick={(e) => {
                      onToggleCart(false)
                      handleAnchorScroll(e, "#order")
                    }}
                    className={`${styles.btn} ${styles.btnPrimary}`}
                  >
                    Configure Flask
                  </a>
                </div>
              ) : (
                <ul className="space-y-4">
                  {cartItems.map((item) => (
                    <li
                      key={item.variantId}
                      className="flex items-center gap-4 border-b pb-4"
                      style={{ borderColor: "var(--hv-border)" }}
                    >
                      <img
                        src={item.imagePath}
                        alt={item.title}
                        referrerPolicy="no-referrer"
                        className="h-20 w-16 shrink-0 rounded-lg object-cover"
                      />
                      <div className="min-w-0 flex-1">
                        <div className="flex items-start justify-between gap-2">
                          <div className="min-w-0">
                            <p className="truncate text-[15.5px] font-bold">{item.title}</p>
                            <p className="truncate text-xs opacity-75">{item.variantLabel}</p>
                          </div>
                          <span className={`${styles.mono} text-sm font-medium`}>
                            {formatMinorPrice(item.unitPriceMinor * item.quantity, tenant.currency)}
                          </span>
                        </div>

                        <div className="mt-3 flex items-center justify-between">
                          <div className="inline-flex items-center gap-2.5">
                            <button
                              type="button"
                              onClick={() => onUpdateQuantity(item.variantId, -1)}
                              aria-label={`Decrease quantity of ${item.title}`}
                              className="flex h-7 w-7 items-center justify-center rounded-full border text-sm"
                              style={{ borderColor: "var(--hv-border-strong)" }}
                            >
                              -
                            </button>
                            <span className={`${styles.mono} text-sm`}>{item.quantity}</span>
                            <button
                              type="button"
                              onClick={() => onUpdateQuantity(item.variantId, 1)}
                              aria-label={`Increase quantity of ${item.title}`}
                              className="flex h-7 w-7 items-center justify-center rounded-full border text-sm"
                              style={{ borderColor: "var(--hv-border-strong)" }}
                            >
                              +
                            </button>
                          </div>
                          <button
                            type="button"
                            onClick={() => onRemoveItem(item.variantId)}
                            className="text-xs opacity-70 hover:opacity-100"
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
              <div className={styles.drawerFooter}>
                <div className="mb-4 flex items-center justify-between text-lg font-bold">
                  <span>Total</span>
                  <span className={styles.mono}>
                    {formatMinorPrice(cartTotals.subtotalMinor, tenant.currency)}
                  </span>
                </div>
                <button
                  type="button"
                  onClick={onProceedToCheckout}
                  data-testid="kova-checkout-btn"
                  className={`${styles.btn} ${styles.btnPrimary} ${styles.btnFull}`}
                >
                  Checkout
                </button>
              </div>
            )}
          </aside>
        </>
      )}
    </div>
  )
}
