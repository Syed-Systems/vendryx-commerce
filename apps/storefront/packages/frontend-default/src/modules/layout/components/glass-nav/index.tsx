"use client"

import { useCallback, useEffect, useRef, useState, type ReactNode } from "react"
import type { LiquidGlassInstance } from "./liquid-glass"

/** Scroll distance before the bar collapses into the pill. */
const PILL_AT = 32

/**
 * The sticky nav shell: liquid-glass on the header, and a shorter pill once the
 * page scrolls.
 *
 * This is a client component because both effects need the DOM. `Nav` stays a
 * server component and passes its already-rendered `<nav>` in as children, so
 * the region/cart Suspense boundary is untouched.
 *
 * Presentation only — core still resolves the data (ADR-013 decision 4).
 */
export default function GlassNavShell({
  children,
  announcement,
}: {
  children: ReactNode
  announcement?: ReactNode
}) {
  const headerRef = useRef<HTMLElement | null>(null)
  const sentinelRef = useRef<HTMLDivElement | null>(null)
  const glass = useRef<LiquidGlassInstance | null>(null)
  const [scrolled, setScrolled] = useState(false)

  useEffect(() => {
    /**
     * An observer on a sentinel rather than a `scroll` listener.
     *
     * A scroll listener runs on every scroll frame whether or not the answer
     * changed, which is the pattern the design guidance bans. The answer here is
     * a single boolean that flips once at `PILL_AT` and then holds, so the
     * browser can do the sampling: the sentinel is a real element in document
     * flow, parked `PILL_AT` pixels down, and it leaves the viewport exactly
     * once. The observer then fires twice for the life of the page.
     *
     * The sentinel MUST sit in normal flow, as a sibling *before* the sticky
     * shell. The earlier attempt appended it inside `.glass-nav-shell`, which is
     * `position: sticky` — a sticky box never scrolls, so an absolutely
     * positioned child pinned inside it stays intersecting forever,
     * `data-scrolled` never becomes `true`, and the pill silently never forms.
     */
    const node = sentinelRef.current
    if (!node) return

    const observer = new IntersectionObserver(
      ([entry]) => setScrolled(!entry.isIntersecting),
      { threshold: 0 }
    )
    observer.observe(node)

    // Covers a restored scroll position (refresh mid-page), where the observer's
    // first callback can land before layout has settled on the right answer.
    setScrolled(window.scrollY > PILL_AT)

    return () => observer.disconnect()
  }, [])

  /**
   * Attach the refraction once the header becomes the floating pill, and keep it.
   *
   * At the top of the page the header is a full-bleed 1440x80 bar with
   * `border-radius: 0`. There is no rim for the displacement map to bend, so the
   * refraction is invisible there — while being the *larger* of the two states
   * (115k px vs 74k). Until then the header carries the chrome tint from the base
   * CSS rule, which is all that reads at radius 0.
   *
   * Deferring to first scroll is the part that matters: module init measured
   * ~17ms, and it was running on the critical path, competing with first paint.
   *
   * Deliberately not torn down on scroll-back. Init is not cheap, so detaching
   * would re-pay it on every direction change, and steady-state compositing
   * measured 58-60fps either way — so there is nothing to reclaim by paying a
   * frame of jank each time the user scrolls back up.
   *
   * Gated on `material_style`. Lustre declares `flat` and Matrix `border`, and
   * both set a zero-radius chrome: a displacement map has no rim to bend at
   * `border-radius: 0`, so the refraction would cost a filter per frame to
   * produce nothing. Only `shadow` (Default) and `glass` (Mahal) get it.
   */
  useEffect(() => {
    if (!scrolled || glass.current) return
    const el = headerRef.current
    if (!el) return

    const material = getComputedStyle(document.documentElement)
      .getPropertyValue("--brand-material-style")
      .trim()
    if (material === "flat" || material === "border") return

    let disposed = false
    // Imported lazily on purpose: liquid-glass.js reads `navigator` and probes
    // canvas support the moment it evaluates, so it cannot be part of the server
    // render pass.
    void import("./liquid-glass").then(() => {
      if (disposed || !window.liquidGlass) return
      glass.current = window.liquidGlass(el, {
        scale: -96,
        chroma: 6,
        border: 0.055,
        mapBlur: 14,
        blur: 2,
        saturate: 1.6,
        fallbackBlur: 18,
      })
    })

    return () => {
      disposed = true
    }
  }, [scrolled])

  useEffect(
    () => () => {
      glass.current?.destroy()
      glass.current = null
    },
    []
  )

  // The map is sized in device pixels and the element animates height *and*
  // radius into the pill, so the ResizeObserver alone leaves the rim stale
  // mid-transition. This lands it exactly.
  const onTransitionEnd = useCallback(() => {
    glass.current?.refresh()
  }, [])

  return (
    <>
      {/* Scroll probe. `PILL_AT` tall and zero-width, in normal flow ahead of the
          sticky shell, so it scrolls off the top of the viewport exactly when the
          page passes the pill threshold. Decorative, and out of the a11y tree. */}
      <div
        ref={sentinelRef}
        aria-hidden="true"
        className="pointer-events-none h-8 w-px"
        style={{ marginTop: PILL_AT }}
      />
      <div className="glass-nav-shell group">
        {announcement ? (
          <div className="glass-nav-announce" data-collapsed={scrolled}>
            {announcement}
          </div>
        ) : null}
        <header
          ref={headerRef}
          className="glass-nav-header"
          data-scrolled={scrolled}
          onTransitionEnd={onTransitionEnd}
        >
          {children}
        </header>
      </div>
    </>
  )
}
