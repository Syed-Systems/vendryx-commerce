import { getBaseURL } from "@lib/util/env"
import { Metadata } from "next"
import { headers } from "next/headers"
import "../styles/globals.css"
// P27-03: the bound frontend's token layer, loaded after globals so a frontend can
// re-point the UI preset's CSS variables. Every frontend supplies this file.
import "@frontend/styles/tokens.css"
import {
  DEFAULT_BRANDING,
  LUSTRE_BRANDING,
  MAHAL_BRANDING,
  MATRIX_BRANDING,
  emitBrandingCss,
  resolveThemeByHost,
} from "../lib/branding"
import { getStoreBranding } from "../lib/data/branding"
import { getStorefrontAppearance } from "../lib/data/appearance"
import DevThemeSwitcher from "@frontend/layout/components/dev-theme-switcher"

const DEV_THEME_OPTIONS = [
  { name: "Default", css: emitBrandingCss(DEFAULT_BRANDING) },
  { name: "Mahal", css: emitBrandingCss(MAHAL_BRANDING) },
  { name: "Lustre", css: emitBrandingCss(LUSTRE_BRANDING) },
  { name: "Matrix", css: emitBrandingCss(MATRIX_BRANDING) },
]

export const metadata: Metadata = {
  metadataBase: new URL(getBaseURL()),
}

export default async function RootLayout(props: { children: React.ReactNode }) {
  const headersList = await headers()
  const host = headersList.get("host")
  const mockBranding = resolveThemeByHost(host)
  const tenantBranding = await getStoreBranding(host)
  const appearance = await getStorefrontAppearance(host)
  const branding = tenantBranding ?? (process.env.NODE_ENV === "development" ? mockBranding : null)
  const themeName = branding === mockBranding
    ? (mockBranding?.store_name || "default").toLowerCase().replace(/\s+/g, '-')
    : "default"
  const brandCss = emitBrandingCss(branding)

  return (
    <html lang="en" data-mode="light">
      <head>
        <style
          id="stheno-brand-tokens"
          dangerouslySetInnerHTML={{ __html: brandCss }}
        />
        <script
          dangerouslySetInnerHTML={{
            __html: `
              if (typeof window !== "undefined") {
                window.addEventListener("message", function(e) {
                  if (e.data && e.data.type === "STHENO_PREVIEW_TOKENS") {
                    if (e.data.css) {
                      var el = document.getElementById("stheno-brand-tokens");
                      if (el) { el.textContent = e.data.css; }
                    }
                    if (e.data.themeName) {
                      document.body.setAttribute('data-theme', e.data.themeName);
                    }
                    if (e.data.appearance) {
                      document.body.setAttribute('data-storefront-archetype', e.data.appearance.storefront_archetype);
                      document.body.setAttribute('data-product-card-style', e.data.appearance.product_card_style);
                      document.body.setAttribute('data-product-image-aspect', e.data.appearance.product_image_aspect);
                    }
                  }
                });
              }
            `,
          }}
        />
      </head>
      <body data-theme={themeName} data-storefront-archetype={appearance.storefront_archetype} data-product-card-style={appearance.product_card_style} data-product-image-aspect={appearance.product_image_aspect} className="group bg-brand-background text-brand-text" style={{ fontFamily: "var(--brand-font-family)" }}>
        <main className="relative">{props.children}</main>
        {process.env.NODE_ENV === "development" && <DevThemeSwitcher themes={DEV_THEME_OPTIONS} />}
      </body>
    </html>
  )
}
