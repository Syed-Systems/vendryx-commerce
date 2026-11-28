import { z } from "zod"

export const THEME_SCHEMA_VERSION = 1 as const

export const HEX_COLOR_REGEX = /^#[0-9A-Fa-f]{6}$/
const SAFE_RELATIVE_OR_HTTPS_URL = /^(\/[a-zA-Z0-9\-_/#?&=.%]*|#[a-zA-Z0-9\-_]+|https:\/\/[^\s<>"]+)$/
const FORBIDDEN_MARKUP_REGEX = /<\s*script|javascript:|data:text\/html|on\w+\s*=/i

const safeTextSchema = (maxLen: number) =>
  z
    .string()
    .trim()
    .min(1)
    .max(maxLen)
    .refine((val) => !FORBIDDEN_MARKUP_REGEX.test(val), {
      message: "Executable markup or script URI is forbidden in theme config.",
    })

const safeHrefSchema = z
  .string()
  .trim()
  .refine(
    (val) =>
      SAFE_RELATIVE_OR_HTTPS_URL.test(val) && !FORBIDDEN_MARKUP_REGEX.test(val),
    { message: "Href must be a safe relative path, anchor, or https:// URL." }
  )

const hexColorSchema = z.string().regex(HEX_COLOR_REGEX, "Must be a 6-digit hex color (#RRGGBB)")

export const ThemeIdSchema = z.enum(["maison-cuir", "hydro-vessel", "homedine-eco"])
export type ThemeId = z.infer<typeof ThemeIdSchema>

export const FontPairSchema = z.enum(["cormorant-jakarta", "outfit-dmsans", "playfair-jakarta"])
export type FontPairId = z.infer<typeof FontPairSchema>

export const DesignTokensSchema = z.object({
  canvas: hexColorSchema,
  surface: hexColorSchema,
  surfaceAlt: hexColorSchema,
  ink: hexColorSchema,
  inkMuted: hexColorSchema,
  border: hexColorSchema,
  accent: hexColorSchema,
  accentInk: hexColorSchema,
  heroBg: hexColorSchema,
  heroInk: hexColorSchema,
  fontPair: FontPairSchema,
  radiusCardPx: z.number().int().min(0).max(28),
  radiusButtonPx: z.number().int().min(0).max(999),
  headerStyle: z.enum(["editorial-centered", "kinetic-split", "organic-floating"]),
  cardAspect: z.enum(["portrait-3-4", "studio-4-3"]),
  iconStrokeWeight: z.enum(["hairline-1.1", "industrial-2.2", "botanical-1.75"]),
})

export type DesignTokens = z.infer<typeof DesignTokensSchema>

export const SectionConfigSchema = z.discriminatedUnion("type", [
  z.object({
    id: safeTextSchema(40),
    type: z.literal("hero"),
    headline: safeTextSchema(110),
    subcopy: safeTextSchema(240),
    ctaLabel: safeTextSchema(40),
    ctaHref: safeHrefSchema,
    mediaPath: safeHrefSchema,
    mediaAlt: safeTextSchema(140),
  }),
  z.object({
    id: safeTextSchema(40),
    type: z.literal("featured_collection"),
    heading: safeTextSchema(90),
    subcopy: safeTextSchema(180),
  }),
  z.object({
    id: safeTextSchema(40),
    type: z.literal("craft_story"),
    heading: safeTextSchema(100),
    body: safeTextSchema(420),
    originNote: safeTextSchema(120),
    mediaPath: safeHrefSchema,
    mediaAlt: safeTextSchema(140),
  }),
  z.object({
    id: safeTextSchema(40),
    type: z.literal("performance_specs"),
    heading: safeTextSchema(100),
    subcopy: safeTextSchema(200),
    specs: z
      .array(
        z.object({
          metric: safeTextSchema(24),
          label: safeTextSchema(48),
          detail: safeTextSchema(140),
        })
      )
      .min(2)
      .max(4),
  }),
])

export type ThemeSectionConfig = z.infer<typeof SectionConfigSchema>

export const ThemeConfigSchema = z.object({
  schemaVersion: z.literal(THEME_SCHEMA_VERSION),
  themeId: ThemeIdSchema,
  versionTag: safeTextSchema(32),
  tokens: DesignTokensSchema,
  navigation: z
    .array(
      z.object({
        label: safeTextSchema(24),
        href: safeHrefSchema,
      })
    )
    .min(3)
    .max(5),
  sections: z.array(SectionConfigSchema).min(2).max(5),
  footerNote: safeTextSchema(160),
})

export type ValidatedThemeConfig = z.infer<typeof ThemeConfigSchema>

export interface ThemeValidationOutcome {
  valid: boolean
  config: ValidatedThemeConfig
  errors: string[]
  fellBackToDefault: boolean
}

export function validateThemeConfig(
  rawInput: unknown,
  safeFallback: ValidatedThemeConfig
): ThemeValidationOutcome {
  const parsed = ThemeConfigSchema.safeParse(rawInput)
  if (parsed.success) {
    return {
      valid: true,
      config: parsed.data,
      errors: [],
      fellBackToDefault: false,
    }
  }

  const errors = parsed.error.issues.map(
    (issue) => `${issue.path.join(".") || "root"}: ${issue.message}`
  )
  return {
    valid: false,
    config: safeFallback,
    errors,
    fellBackToDefault: true,
  }
}

export function buildScopedCssVariables(tokens: DesignTokens): Record<string, string> {
  return {
    "--theme-canvas": tokens.canvas,
    "--theme-surface": tokens.surface,
    "--theme-surface-alt": tokens.surfaceAlt,
    "--theme-ink": tokens.ink,
    "--theme-ink-muted": tokens.inkMuted,
    "--theme-border": tokens.border,
    "--theme-accent": tokens.accent,
    "--theme-accent-ink": tokens.accentInk,
    "--theme-hero-bg": tokens.heroBg,
    "--theme-hero-ink": tokens.heroInk,
    "--theme-radius-card": `${tokens.radiusCardPx}px`,
    "--theme-radius-button": `${tokens.radiusButtonPx}px`,
  }
}
