import { describe, expect, it } from "vitest"
import {
  DEFAULT_BRANDING,
  FONT_STACKS,
  emitBrandingCss,
  resolveBrandingCssVars,
  resolveBrandingTokens,
} from "../../packages/storefront-core/src/lib/branding"

describe("P26-05: Branding tokens & runtime resolution", () => {
  it("resolves documented defaults when branding row is absent or null", () => {
    const tokens = resolveBrandingTokens(null)

    expect(tokens.store_name).toBe(DEFAULT_BRANDING.store_name)
    expect(tokens.font_key).toBe("inter")
    expect(tokens.color_primary).toBe(DEFAULT_BRANDING.color_primary)
    expect(tokens.color_secondary).toBe(DEFAULT_BRANDING.color_secondary)
    expect(tokens.color_accent).toBe(DEFAULT_BRANDING.color_accent)
    expect(tokens.color_background).toBe(DEFAULT_BRANDING.color_background)
    expect(tokens.color_surface).toBe(DEFAULT_BRANDING.color_surface)
    expect(tokens.color_text).toBe(DEFAULT_BRANDING.color_text)
    expect(tokens.logo_url).toBeNull()
    expect(tokens.favicon_url).toBeNull()
  })

  it("emits CSS custom properties for defaults", () => {
    const vars = resolveBrandingCssVars(null)

    expect(vars["--brand-primary"]).toBe(DEFAULT_BRANDING.color_primary)
    expect(vars["--brand-secondary"]).toBe(DEFAULT_BRANDING.color_secondary)
    expect(vars["--brand-accent"]).toBe(DEFAULT_BRANDING.color_accent)
    expect(vars["--brand-background"]).toBe(DEFAULT_BRANDING.color_background)
    expect(vars["--brand-surface"]).toBe(DEFAULT_BRANDING.color_surface)
    expect(vars["--brand-text"]).toBe(DEFAULT_BRANDING.color_text)
    expect(vars["--brand-font-family"]).toBe(FONT_STACKS.inter)
  })

  it("resolves tenant-specific overrides while preserving unspecified defaults", () => {
    const tenantA = {
      store_name: "Artisan Leather",
      color_primary: "#8B4513",
      color_accent: "#D2691E",
      font_key: "playfair",
    }

    const tokens = resolveBrandingTokens(tenantA)
    expect(tokens.store_name).toBe("Artisan Leather")
    expect(tokens.color_primary).toBe("#8B4513")
    expect(tokens.color_accent).toBe("#D2691E")
    expect(tokens.font_key).toBe("playfair")
    // Preserves defaults
    expect(tokens.color_secondary).toBe(DEFAULT_BRANDING.color_secondary)
    expect(tokens.color_background).toBe(DEFAULT_BRANDING.color_background)

    const vars = resolveBrandingCssVars(tenantA)
    expect(vars["--brand-primary"]).toBe("#8B4513")
    expect(vars["--brand-accent"]).toBe("#D2691E")
    expect(vars["--brand-font-family"]).toBe(FONT_STACKS.playfair)
  })

  it("falls back to default font stack on unrecognised font key", () => {
    const vars = resolveBrandingCssVars({ font_key: "unknown_mystery_font" })
    expect(vars["--brand-font-family"]).toBe(FONT_STACKS.inter)
  })

  it("emits a valid :root CSS block", () => {
    const css = emitBrandingCss({
      color_primary: "#1A56DB",
      color_accent: "#E02424",
    })

    expect(css).toContain(":root {")
    expect(css).toContain("--brand-primary: #1A56DB;")
    expect(css).toContain("--brand-accent: #E02424;")
    expect(css).toContain(`--brand-secondary: ${DEFAULT_BRANDING.color_secondary};`)
    expect(css).toContain(`--brand-font-family: ${FONT_STACKS.inter};`)
    expect(css).toContain("}")
  })
})
