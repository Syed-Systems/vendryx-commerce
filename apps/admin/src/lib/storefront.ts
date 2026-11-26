/**
 * Storefront URLs for the admin's Live Studio preview.
 *
 * The storefront resolves its tenant from the Host header (ADR-022), so a
 * themed preview is just a different hostname on the same port. The engine
 * does the same thing with pro/free/trial.localhost.
 *
 * `localhost` is left alone on purpose: browsers resolve *.localhost to the
 * loopback address themselves, so mahal.localhost:8000 reaches the same dev
 * server without a DNS entry or a hosts-file edit. In production the storefront
 * is one deployment and the tenant comes from the request's Host, so the helper
 * falls back to the configured URL rather than inventing a hostname.
 */
export const STHENO_STOREFRONT_URL =
  typeof __STOREFRONT_URL__ !== "undefined"
    ? __STOREFRONT_URL__
    : "http://localhost:8000"

const THEMES = ["default", "mahal", "lustre", "matrix"] as const
export type StorefrontTheme = (typeof THEMES)[number]

export function getStorefrontUrl(theme: StorefrontTheme): string {
  if (theme === "default") return STHENO_STOREFRONT_URL
  try {
    const url = new URL(STHENO_STOREFRONT_URL)
    // Only rewrite the host when it is a localhost name. A production URL
    // (https://shop.example.com) has no tenant subdomain to swap, and guessing
    // one would point the preview at a host that does not exist.
    if (!url.hostname.endsWith("localhost")) return STHENO_STOREFRONT_URL
    url.hostname = `${theme}.${url.hostname}`
    return url.toString().replace(/\/$/, "")
  } catch {
    return STHENO_STOREFRONT_URL
  }
}

export const STOREFRONT_THEMES: { key: StorefrontTheme; label: string }[] = [
  { key: "default", label: "Default" },
  { key: "mahal", label: "Mahal" },
  { key: "lustre", label: "Lustre" },
  { key: "matrix", label: "Matrix" },
]