/**
 * @stheno/storefront-core - Branding & Token Resolution (P26-05)
 *
 * Core resolves branding per request and emits CSS custom properties.
 * No per-tenant build, no dynamic component loading (ADR-013 decisions 1, 2, 5).
 */

export interface TenantBranding {
  id?: string | null
  store_name?: string | null
  logo_url?: string | null
  favicon_url?: string | null
  font_key?: string | null
  color_primary?: string | null
  color_secondary?: string | null
  color_accent?: string | null
  color_background?: string | null
  color_surface?: string | null
  color_text?: string | null
  updated_at?: string | null
}

export interface ResolvedBranding {
  store_name: string
  logo_url: string | null
  favicon_url: string | null
  font_key: string
  color_primary: string
  color_secondary: string
  color_accent: string
  color_background: string
  color_surface: string
  color_text: string
}

export const DEFAULT_BRANDING = {
  store_name: "Stheno Store",
  logo_url: null,
  favicon_url: null,
  font_key: "inter",
  color_primary: "#111827",
  color_secondary: "#4B5563",
  color_accent: "#2563EB",
  color_background: "#FFFFFF",
  color_surface: "#F9FAFB",
  color_text: "#111827",
} as const

export const FONT_STACKS: Record<string, string> = {
  inter: "var(--font-inter, 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif)",
  geist: "var(--font-geist, 'Geist Sans', -apple-system, BlinkMacSystemFont, sans-serif)",
  noto_sans_bengali: "'Noto Sans Bengali', 'SolaimanLipi', -apple-system, sans-serif",
  playfair: "'Playfair Display', Georgia, serif",
  plus_jakarta_sans: "'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif",
  roboto: "Roboto, -apple-system, BlinkMacSystemFont, sans-serif",
  cinzel: "Cinzel, 'Times New Roman', serif",
}

/**
 * Resolves raw tenant branding data into concrete token values, applying documented defaults.
 */
export function resolveBrandingTokens(
  branding?: Partial<TenantBranding> | null
): ResolvedBranding {
  const fontKey = branding?.font_key?.trim().toLowerCase() || DEFAULT_BRANDING.font_key

  return {
    store_name: branding?.store_name?.trim() || DEFAULT_BRANDING.store_name,
    logo_url: branding?.logo_url || null,
    favicon_url: branding?.favicon_url || null,
    font_key: fontKey,
    color_primary: branding?.color_primary?.trim() || DEFAULT_BRANDING.color_primary,
    color_secondary: branding?.color_secondary?.trim() || DEFAULT_BRANDING.color_secondary,
    color_accent: branding?.color_accent?.trim() || DEFAULT_BRANDING.color_accent,
    color_background: branding?.color_background?.trim() || DEFAULT_BRANDING.color_background,
    color_surface: branding?.color_surface?.trim() || DEFAULT_BRANDING.color_surface,
    color_text: branding?.color_text?.trim() || DEFAULT_BRANDING.color_text,
  }
}

/**
 * Returns CSS custom properties mapping to the resolved brand tokens.
 */
export function resolveBrandingCssVars(
  branding?: Partial<TenantBranding> | null
): Record<string, string> {
  const resolved = resolveBrandingTokens(branding)
  const fontStack = FONT_STACKS[resolved.font_key] || FONT_STACKS[DEFAULT_BRANDING.font_key]

  return {
    "--brand-primary": resolved.color_primary,
    "--brand-secondary": resolved.color_secondary,
    "--brand-accent": resolved.color_accent,
    "--brand-background": resolved.color_background,
    "--brand-surface": resolved.color_surface,
    "--brand-text": resolved.color_text,
    "--brand-font-family": fontStack,
  }
}

/**
 * Emits a CSS string suitable for injection into a `<style>` block in document `<head>`.
 */
export function emitBrandingCss(branding?: Partial<TenantBranding> | null): string {
  const vars = resolveBrandingCssVars(branding)
  const declarations = Object.entries(vars)
    .map(([key, value]) => `  ${key}: ${value};`)
    .join("\n")

  return `:root {\n${declarations}\n}`
}
