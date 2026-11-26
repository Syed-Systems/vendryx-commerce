"use client"

import { useEffect, useRef, useState } from "react"
import type { LiquidGlassInstance } from "@modules/layout/components/glass-nav/liquid-glass"
// Imported from core's public entry point, not `@lib/branding`. A frontend package may
// only reach core through `@stheno/storefront-core`; the private alias is what
// scripts/check-storefront-boundaries.sh fails on. Everything used here is re-exported
// there. Introduced by 95afd25, which left the boundary gate red.
import {
  DEFAULT_BRANDING,
  MAHAL_BRANDING,
  LUSTRE_BRANDING,
  MATRIX_BRANDING,
  emitBrandingCss,
} from "@stheno/storefront-core"

const THEMES = [
  { name: "Default", data: DEFAULT_BRANDING },
  { name: "Mahal", data: MAHAL_BRANDING },
  { name: "Lustre", data: LUSTRE_BRANDING },
  { name: "Matrix", data: MATRIX_BRANDING },
]

const slug = (name: string) => name.toLowerCase().replace(/\s+/g, "-")

/**
 * The switcher's own tokens, fixed on purpose.
 *
 * Every value here deliberately shadows the tenant's, so the control renders
 * identically in all four themes. Set on the root `<div>` rather than on the
 * panel, so the whole subtree — trigger, listbox, options, swatch borders —
 * inherits one instrument palette instead of four.
 *
 * A dark neutral glass, because it has to stay legible floating over all four
 * stores: Default and Mahal are light pages, Lustre and Matrix are black ones.
 * A light glass would vanish on two of them; this reads on all four.
 *
 * Note `radius` is a literal `14px` rather than `--brand-radius-card`. Reading
 * the tenant's radius is what previously switched refraction off entirely on
 * Lustre (0px) and Matrix (4px), since a displacement map needs a rim to bend.
 * A constant rim keeps the effect present everywhere. The module reads this
 * computed value to build the map, so it does not need to be 14px specifically —
 * it just must not be 0.
 */
const SWITCHER_TOKENS = {
  "--brand-radius-card": "14px",
  "--brand-radius-button": "8px",
  "--chrome-bg": "rgb(18 18 20 / 0.72)",
  "--chrome-fg": "rgb(250 250 250)",
  "--chrome-fg-muted": "rgb(250 250 250 / 0.72)",
  "--chrome-edge": "rgb(250 250 250 / 0.18)",
  "--glass-tint-top": "rgb(255 255 255 / 0.1)",
  "--glass-tint-bottom": "rgb(255 255 255 / 0.05)",
  "--glass-edge": "rgb(255 255 255 / 0.16)",
  "--glass-rim": "inset 0 1px 0 0 rgb(255 255 255 / 0.14)",
  "--glass-lift": "0 10px 30px -12px rgb(0 0 0 / 0.6)",
  "--glass-blur": "16px",
  "--glass-saturate": "1.5",
} as React.CSSProperties

/** Background + accent dot, painted inline from the preset's own hex values so
 *  the swatch shows the target palette rather than whatever is active.
 *
 *  The ring is white at low alpha, not black: the panel these sit on is now a
 *  fixed dark neutral in every theme, and a black ring disappeared against it. */
function Swatch({ theme }: { theme: (typeof THEMES)[number] }) {
  return (
    <span aria-hidden="true" className="relative flex size-3.5 shrink-0 items-center">
      <span
        className="size-3.5 rounded-full border border-white/25"
        style={{ background: theme.data.color_background }}
      />
      <span
        className="absolute left-0 top-0 size-3.5 rounded-full border border-white/25"
        style={{ background: theme.data.color_accent }}
      />
    </span>
  )
}

export default function DevThemeSwitcher() {
  const [activeTheme, setActiveTheme] = useState("Default")
  const [open, setOpen] = useState(false)
  const rootRef = useRef<HTMLDivElement>(null)
  const triggerRef = useRef<HTMLButtonElement>(null)
  const optionRefs = useRef<(HTMLButtonElement | null)[]>([])
  const panelRef = useRef<HTMLDivElement>(null)
  const glass = useRef<LiquidGlassInstance | null>(null)

  /**
   * Real refraction on the switcher, unconditionally.
   *
   * Reuses the vendored module rather than re-deriving it (it already sets
   * `color-interpolation-filters="sRGB"`, which without it ghosts the backdrop
   * up-and-left).
   *
   * Deliberately NOT gated on `--brand-material-style`, unlike the nav. The nav
   * is part of the store's identity, so it should obey the tenant's declared
   * material. The switcher is a *control*, and a control that restyles itself
   * on every click makes it impossible to judge what actually changed about a
   * theme. It is a fixed neutral glass in all four, so the variable is the
   * store beneath it and the constant is the instrument.
   *
   * That decoupling is also what lets this run in every theme: `SWITCHER_TOKENS`
   * below forces a radius, so Matrix's 4px and Lustre's 0px cannot switch it off.
   *
   * Rebuilt when the panel opens, because the box changes from `w-auto` to
   * `w-56` and the displacement map is baked to the element's dimensions. The
   * nav does this on `transitionend`; here there is no transition.
   *
   * ~224x46, well under the ~800px/side where GPU cost starts to matter, and
   * it is a dev-only affordance.
   */
  useEffect(() => {
    const el = panelRef.current
    if (!open || !el) return

    let disposed = false
    void import("@modules/layout/components/glass-nav/liquid-glass").then(() => {
      if (disposed || !window.liquidGlass) return
      glass.current = window.liquidGlass(el, {
        scale: -72,
        chroma: 5,
        border: 0.06,
        mapBlur: 10,
        blur: 2,
        saturate: 1.5,
        fallbackBlur: 14,
      })
    })

    return () => {
      disposed = true
      glass.current?.destroy()
      glass.current = null
    }
  }, [open])

  useEffect(() => {
    const match = THEMES.find((theme) => slug(theme.name) === document.body.dataset.theme)
    if (match) {
      setActiveTheme(match.name)
    }
  }, [])

  // A popover has to survive a click that lands outside it, but not the click
  // that opened it. `pointerdown` rather than `click` so the panel closes before
  // the press can activate whatever was underneath it.
  useEffect(() => {
    if (!open) return
    const onPointerDown = (e: PointerEvent) => {
      if (!rootRef.current?.contains(e.target as Node)) setOpen(false)
    }
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setOpen(false)
        triggerRef.current?.focus()
      }
    }
    document.addEventListener("pointerdown", onPointerDown)
    document.addEventListener("keydown", onKeyDown)
    return () => {
      document.removeEventListener("pointerdown", onPointerDown)
      document.removeEventListener("keydown", onKeyDown)
    }
  }, [open])

  const active = THEMES.find((t) => t.name === activeTheme) ?? THEMES[0]
  const activeIndex = THEMES.findIndex((t) => t.name === activeTheme)

  const selectTheme = (themeName: string) => {
    setActiveTheme(themeName)
    const theme = THEMES.find((t) => t.name === themeName)
    if (theme) {
      const style = document.getElementById("stheno-brand-tokens")
      if (style) {
        style.textContent = emitBrandingCss(theme.data)
      }
      document.body.dataset.theme = slug(themeName)
    }
    setOpen(false)
    triggerRef.current?.focus()
  }

  const move = (from: number, delta: number) => {
    const next = (from + delta + THEMES.length) % THEMES.length
    optionRefs.current[next]?.focus()
  }

  const onOptionKeyDown = (e: React.KeyboardEvent, index: number) => {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault()
        move(index, 1)
        break
      case "ArrowUp":
        e.preventDefault()
        move(index, -1)
        break
      case "Home":
        e.preventDefault()
        optionRefs.current[0]?.focus()
        break
      case "End":
        e.preventDefault()
        optionRefs.current[THEMES.length - 1]?.focus()
        break
    }
  }

  return (
    <div
      ref={rootRef}
      style={SWITCHER_TOKENS}
      className="fixed bottom-4 right-4 z-[9999] font-heading"
    >
      {/* Collapsed by default. A floating dev panel that permanently covers a
          corner of the page competes with the thing being debugged; opening it
          is one click and it closes again the moment a theme is picked. */}
      <div
        ref={panelRef}
        className={[
          "glass overflow-hidden rounded-[var(--brand-radius-card)] shadow-2xl transition-[width,height]",
          open ? "w-56" : "w-auto",
        ].join(" ")}
      >
        <button
          ref={triggerRef}
          type="button"
          aria-haspopup="listbox"
          aria-expanded={open}
          onClick={() => setOpen((v) => !v)}
          className="tap flex w-full items-center gap-2 px-2.5 py-2 text-[color:var(--chrome-fg)] transition-colors hover:bg-[color:var(--chrome-fg)]/10"
        >
          <Swatch theme={active} />
          <span className="text-[13px] font-sans font-medium">{active.name}</span>
          <span className="ml-auto pl-1 font-mono text-[10px] uppercase opacity-70">
            {active.data.material_style}
          </span>
          <svg
            aria-hidden="true"
            viewBox="0 0 12 12"
            className={["size-3 shrink-0 transition-transform", open ? "rotate-180" : ""].join(" ")}
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
          >
            <path d="M2.5 4.5 6 8l3.5-3.5" />
          </svg>
        </button>

        {open && (
          <div
            role="listbox"
            aria-label="Store theme"
            className="flex flex-col gap-0.5 border-t border-[color:var(--chrome-edge)] p-1.5"
          >
            {THEMES.map((theme, i) => {
              const selected = theme.name === activeTheme
              return (
                <button
                  key={theme.name}
                  ref={(el) => {
                    optionRefs.current[i] = el
                  }}
                  type="button"
                  role="option"
                  aria-selected={selected}
                  tabIndex={i === activeIndex ? 0 : -1}
                  onKeyDown={(e) => onOptionKeyDown(e, i)}
                  onClick={() => selectTheme(theme.name)}
                  className={[
                    "tap flex items-center gap-2.5 rounded-[var(--brand-radius-button)] px-2.5 py-2 text-left transition-colors",
                    selected
                      ? "bg-[color:var(--chrome-fg)] text-[color:var(--chrome-bg)]"
                      : "text-[color:var(--chrome-fg-muted)] hover:bg-[color:var(--chrome-fg)]/10 hover:text-[color:var(--chrome-fg)]",
                  ].join(" ")}
                >
                  <Swatch theme={theme} />
                  <span className="text-[13px] font-sans font-medium">{theme.name}</span>
                  <span className="ml-auto pl-2 font-mono text-[10px] uppercase opacity-70">
                    {theme.data.material_style}
                  </span>
                </button>
              )
            })}
          </div>
        )}
      </div>
    </div>
  )
}