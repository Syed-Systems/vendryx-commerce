/**
 * Outbound links shown to merchants.
 *
 * The fork hardcoded these to docs.medusajs.com and medusajs.com/changelog, so a merchant
 * clicking "Documentation" inside our dashboard landed on another company's website.
 *
 * Configured rather than hardcoded, and every call site must handle the empty case by
 * hiding the affordance: a missing link is better than a broken one, and both are better
 * than sending our customers somewhere else. Set these once the docs site is published.
 */
export const DOCS_URL = import.meta.env.VITE_DOCS_URL || ""
export const CHANGELOG_URL = import.meta.env.VITE_CHANGELOG_URL || ""

/** Deep link into the docs, or "" when no docs site is configured. */
export const docsLink = (path: string): string =>
  DOCS_URL ? `${DOCS_URL.replace(/\/$/, "")}/${path.replace(/^\//, "")}` : ""
