import { Cinzel, Inter, Noto_Sans_Bengali, Playfair_Display } from "next/font/google"

/**
 * The faces the bundled themes actually name (see `FONT_STACKS` in branding.ts).
 *
 * These were never loaded. `FONT_STACKS` resolved to bare family names —
 * `'Playfair Display', Georgia, serif` — so every "branded" heading silently
 * fell through to Georgia and all four themes rendered in the same two faces.
 * Self-hosting via `next/font` keeps them out of the CSS payload, adds no render
 * blocking, and exposes each family as a CSS variable that `FONT_STACKS` can
 * point at.
 *
 * Only the four families the shipped presets reference are fetched. A tenant who
 * picks `geist`, `plus_jakarta_sans` or `roboto` in the admin still falls back to
 * the system stack until those are added here — adding all seven would cost every
 * shopper the weight of faces most stores never render.
 */
export const fontInter = Inter({
  subsets: ["latin"],
  display: "swap",
  variable: "--font-inter",
})

export const fontPlayfair = Playfair_Display({
  subsets: ["latin"],
  display: "swap",
  variable: "--font-playfair",
})

export const fontCinzel = Cinzel({
  subsets: ["latin"],
  display: "swap",
  variable: "--font-cinzel",
})

// Mahal's body face. Bengali matters here: the store sells saris, and a Latin-only
// subset would drop the script out of the body copy entirely.
export const fontNotoBengali = Noto_Sans_Bengali({
  subsets: ["latin", "bengali"],
  display: "swap",
  variable: "--font-noto-bengali",
})

/**
 * Class list for `<html>`. next/font defines each `--font-*` on the element that
 * carries these classes, and `<html>` is `:root`, so the variables resolve for
 * every rule below it — including the SSR-injected `--brand-font-*` tokens.
 */
export const fontVariables = [
  fontInter.variable,
  fontPlayfair.variable,
  fontCinzel.variable,
  fontNotoBengali.variable,
].join(" ")