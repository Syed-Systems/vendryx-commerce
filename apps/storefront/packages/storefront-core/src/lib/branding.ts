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
  font_heading?: string | null
  font_body?: string | null
  font_mono?: string | null
  color_primary?: string | null
  color_secondary?: string | null
  color_accent?: string | null
  color_background?: string | null
  color_surface?: string | null
  color_text?: string | null
  radius_card?: string | null
  radius_input?: string | null
  radius_button?: string | null
  material_style?: string | null
  updated_at?: string | null
}

export interface ResolvedBranding {
  store_name: string
  logo_url: string | null
  favicon_url: string | null
  font_key: string
  font_heading: string
  font_body: string
  font_mono: string
  color_primary: string
  color_secondary: string
  color_accent: string
  color_background: string
  color_surface: string
  color_text: string
  radius_card: string
  radius_input: string
  radius_button: string
  material_style: string
}

export const DEFAULT_BRANDING = {
  store_name: "Stheno Store",
  logo_url: null,
  favicon_url: null,
  font_key: "inter",
  font_heading: "inter",
  font_body: "inter",
  font_mono: "inter",
  color_primary: "#111827",
  color_secondary: "#4B5563",
  color_accent: "#2563EB",
  color_background: "#FFFFFF",
  color_surface: "#F9FAFB",
  color_text: "#111827",
  radius_card: "8px",
  radius_input: "4px",
  radius_button: "4px",
  material_style: "shadow",
} as const

export const MAHAL_BRANDING = {
  store_name: "Mahal",
  logo_url: null,
  favicon_url: null,
  font_key: "noto_sans_bengali",
  font_heading: "playfair",
  font_body: "noto_sans_bengali",
  font_mono: "inter",
  color_primary: "#1A0F14", // Deep rich black-red
  color_secondary: "#8C6A5D", // Warm clay
  color_accent: "#D4AF37", // Gold
  color_background: "#FAFAF8", // Warm white
  color_surface: "#FFFFFF",
  color_text: "#2C1E16",
  radius_card: "16px",
  radius_input: "8px",
  radius_button: "9999px",
  material_style: "glass",
} as const

export const LUSTRE_BRANDING = {
  store_name: "Lustre",
  logo_url: null,
  favicon_url: null,
  font_key: "cinzel",
  font_heading: "cinzel",
  font_body: "inter",
  font_mono: "inter",
  color_primary: "#050505",
  color_secondary: "#A3A3A3",
  color_accent: "#E5E5E5",
  color_background: "#000000",
  color_surface: "#0A0A0A",
  color_text: "#FAFAFA",
  radius_card: "0px",
  radius_input: "0px",
  radius_button: "0px",
  material_style: "flat",
} as const

export const MATRIX_BRANDING = {
  store_name: "Matrix",
  logo_url: null,
  favicon_url: null,
  font_key: "inter",
  font_heading: "inter",
  font_body: "inter",
  font_mono: "inter",
  color_primary: "#00FF41", // Matrix green
  color_secondary: "#008F11",
  color_accent: "#00FF41",
  color_background: "#0D0208", // Very dark green/black
  color_surface: "#000000",
  color_text: "#00FF41",
  radius_card: "4px",
  radius_input: "2px",
  radius_button: "2px",
  material_style: "border",
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
    font_heading: branding?.font_heading?.trim().toLowerCase() || fontKey,
    font_body: branding?.font_body?.trim().toLowerCase() || fontKey,
    font_mono: branding?.font_mono?.trim().toLowerCase() || fontKey,
    color_primary: branding?.color_primary?.trim() || DEFAULT_BRANDING.color_primary,
    color_secondary: branding?.color_secondary?.trim() || DEFAULT_BRANDING.color_secondary,
    color_accent: branding?.color_accent?.trim() || DEFAULT_BRANDING.color_accent,
    color_background: branding?.color_background?.trim() || DEFAULT_BRANDING.color_background,
    color_surface: branding?.color_surface?.trim() || DEFAULT_BRANDING.color_surface,
    color_text: branding?.color_text?.trim() || DEFAULT_BRANDING.color_text,
    radius_card: branding?.radius_card?.trim() || DEFAULT_BRANDING.radius_card,
    radius_input: branding?.radius_input?.trim() || DEFAULT_BRANDING.radius_input,
    radius_button: branding?.radius_button?.trim() || DEFAULT_BRANDING.radius_button,
    material_style: branding?.material_style?.trim() || DEFAULT_BRANDING.material_style,
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
  const fontStackHeading = FONT_STACKS[resolved.font_heading] || fontStack
  const fontStackBody = FONT_STACKS[resolved.font_body] || fontStack
  const fontStackMono = FONT_STACKS[resolved.font_mono] || fontStack

  return {
    "--brand-primary": resolved.color_primary,
    "--brand-secondary": resolved.color_secondary,
    "--brand-accent": resolved.color_accent,
    "--brand-background": resolved.color_background,
    "--brand-surface": resolved.color_surface,
    "--brand-text": resolved.color_text,
    "--brand-font-family": fontStack,
    "--brand-font-heading": fontStackHeading,
    "--brand-font-body": fontStackBody,
    "--brand-font-mono": fontStackMono,
    "--brand-radius-card": resolved.radius_card,
    "--brand-radius-input": resolved.radius_input,
    "--brand-radius-button": resolved.radius_button,
    "--brand-material-style": resolved.material_style,
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

/**
 * Resolves the correct mock theme branding based on the local hostname for development.
 */
export function resolveThemeByHost(host: string | null): Partial<TenantBranding> | null {
  if (!host) return null
  const h = host.toLowerCase()
  if (h.includes("mahal")) return MAHAL_BRANDING
  if (h.includes("lustre")) return LUSTRE_BRANDING
  if (h.includes("matrix")) return MATRIX_BRANDING
  return null
}
