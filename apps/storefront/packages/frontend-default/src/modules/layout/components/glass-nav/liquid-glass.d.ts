// Types for the vendored liquid-glass.js, which installs itself on `window`.
//
// The module is plain JS and attaches `liquidGlass` as a global rather than
// exporting it, so the only way to reach it is through this declaration.
// Upstream: https://github.com/deepika-builds/liquid-glass

export interface LiquidGlassOptions {
  /** Displacement strength in px. Negative magnifies (pulls the rim inward). */
  scale?: number
  /** Per-channel scale stagger — the prism fringe. 0 disables it. */
  chroma?: number
  /** Neutral inset as a fraction of the smaller side. */
  border?: number
  /** Edge-curvature softness of the displacement map, in px. */
  mapBlur?: number
  /** Backdrop blur behind the glass interior, in px. */
  blur?: number
  /** Backdrop saturation boost. */
  saturate?: number
  /** Corner radius override in px. Defaults to reading computed border-radius. */
  radius?: number | null
  /** Frosted blur used where refraction is unsupported (Safari, Firefox). */
  fallbackBlur?: number
}

export interface LiquidGlassInstance {
  /** False on Safari/Firefox, where the frosted-blur fallback is used instead. */
  supported: boolean
  /** Rebuild the displacement map. Call after the element's size or radius changes. */
  refresh: () => void
  /** Remove the filter, the observer, and the inline backdrop-filter. */
  destroy: () => void
}

declare global {
  interface Window {
    liquidGlass?: (
      el: Element,
      opts?: LiquidGlassOptions
    ) => LiquidGlassInstance
  }
}
