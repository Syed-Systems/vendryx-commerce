// Types for the vendored liquid-glass.js, which installs itself on `window`.
export interface LiquidGlassOptions {
  scale?: number
  chroma?: number
  border?: number
  mapBlur?: number
  blur?: number
  saturate?: number
  radius?: number | null
  fallbackBlur?: number
}

export interface LiquidGlassInstance {
  supported: boolean
  refresh: () => void
  destroy: () => void
}

declare global {
  interface Window {
    liquidGlass?: (el: Element, opts?: LiquidGlassOptions) => LiquidGlassInstance
  }
}

export {}