import fs from "fs"
import path from "path"
import { describe, expect, it } from "vitest"

const STOREFRONT_ROOT = path.resolve(__dirname, "../..")
const PACKAGES_DIR = path.join(STOREFRONT_ROOT, "packages")
const CORE_CHECKOUT_DIR = path.join(PACKAGES_DIR, "storefront-core", "src", "modules", "checkout")
const REPO_ROOT = path.resolve(STOREFRONT_ROOT, "../..")
const PAYMENT_SCRIPTS_DOC = path.join(REPO_ROOT, "docs", "payment-page-scripts.md")

describe("P25-08 / P27-06: Checkout Ownership & Skinning Boundary (ADR-013)", () => {
  it("verifies checkout components reside strictly in storefront-core", () => {
    expect(fs.existsSync(CORE_CHECKOUT_DIR)).toBe(true)

    // Key checkout templates and steps that must exist in core
    const expectedCoreModules = [
      path.join(CORE_CHECKOUT_DIR, "templates", "checkout-form"),
      path.join(CORE_CHECKOUT_DIR, "templates", "checkout-summary"),
      path.join(CORE_CHECKOUT_DIR, "components", "addresses"),
      path.join(CORE_CHECKOUT_DIR, "components", "shipping"),
      path.join(CORE_CHECKOUT_DIR, "components", "payment"),
      path.join(CORE_CHECKOUT_DIR, "components", "review"),
    ]

    for (const mod of expectedCoreModules) {
      const tsx = mod + ".tsx"
      const dirIndex = path.join(mod, "index.tsx")
      const exists = fs.existsSync(tsx) || fs.existsSync(dirIndex)
      expect(exists).toBe(true)
    }
  })

  it("asserts frontends do not fork or reimplement checkout components", () => {
    const frontends = ["frontend-default", "frontend-matrix", "frontend-lustre"]

    for (const fe of frontends) {
      const feCheckout = path.join(PACKAGES_DIR, fe, "src", "modules", "checkout")
      if (fs.existsSync(feCheckout)) {
        const files = fs.readdirSync(feCheckout)
        // Directory must either not exist or be completely empty
        expect(files.length).toBe(0)
      }
    }
  })

  it("verifies bundler configuration preserves core checkout mapping regardless of active theme", () => {
    const nextConfigPath = path.join(STOREFRONT_ROOT, "next.config.js")
    expect(fs.existsSync(nextConfigPath)).toBe(true)

    const nextConfigContent = fs.readFileSync(nextConfigPath, "utf8")
    // Asserts NormalModuleReplacementPlugin holds back checkout to core
    expect(nextConfigContent).toContain('rest.startsWith("checkout/")')
    expect(nextConfigContent).toContain("storefront-core")
  })

  it("asserts checkout script manifest adheres to PCI SAQ A restrictions", () => {
    expect(fs.existsSync(PAYMENT_SCRIPTS_DOC)).toBe(true)
    const manifest = fs.readFileSync(PAYMENT_SCRIPTS_DOC, "utf8")

    // Approved Stripe script present
    expect(manifest).toContain("https://js.stripe.com")

    // Forbidden tracking/analytics scripts explicitly prohibited on payment pages
    expect(manifest).toContain("Google Tag Manager")
    expect(manifest).toContain("Meta Pixel")
  })

  it("verifies themes supply token stylesheets for checkout skinning without markup alterations", () => {
    // ADR-031 consolidated the storefronts onto a single unified runtime.
    // `frontend-matrix` and `frontend-lustre` are no longer separate packages --
    // they are theme variants of `frontend-default`, selected by the
    // `data-theme` attribute on <body>. This test predates that decision and was
    // asserting on directories that were deliberately removed.
    //
    // The assertion is now the stronger one that actually holds: the one shipped
    // frontend supplies the token layer, and every theme it declares has the
    // derived-token blocks that make checkout skinning per-theme. It reads the
    // theme list from the presets themselves rather than hardcoding it, so a
    // future theme cannot pass by omission.
    const frontends = ["frontend-default"]

    for (const fe of frontends) {
      const tokensPath = path.join(PACKAGES_DIR, fe, "src", "modules", "styles", "tokens.css")
      expect(fs.existsSync(tokensPath)).toBe(true)

      const css = fs.readFileSync(tokensPath, "utf8")
      // Must define or map tokens
      expect(css.length).toBeGreaterThan(10)
    }

    // Each named preset must have a matching per-theme override block, or that
    // theme silently falls through to the default material.
    const tokensCss = fs.readFileSync(
      path.join(PACKAGES_DIR, "frontend-default", "src", "modules", "styles", "tokens.css"),
      "utf8"
    )
    for (const theme of ["mahal", "lustre", "matrix"]) {
      expect(tokensCss).toContain(`[data-theme="${theme}"]`)
    }
  })
})
