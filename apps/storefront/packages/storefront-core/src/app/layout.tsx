import { getBaseURL } from "@lib/util/env"
import { Metadata } from "next"
import { headers } from "next/headers"
import "../styles/globals.css"
// P27-03: the bound frontend's token layer, loaded after globals so a frontend can
// re-point the UI preset's CSS variables. Every frontend supplies this file.
import "@frontend/styles/tokens.css"
import { emitBrandingCss, resolveThemeByHost } from "../lib/branding"
import { fontVariables } from "../lib/fonts"
import DevThemeSwitcher from "@frontend/layout/components/dev-theme-switcher"

export const metadata: Metadata = {
  metadataBase: new URL(getBaseURL()),
}

export default async function RootLayout(props: { children: React.ReactNode }) {
  const headersList = await headers()
  const host = headersList.get("host")
  const mockBranding = resolveThemeByHost(host)
  const themeName = (mockBranding?.store_name || "default").toLowerCase().replace(/\s+/g, '-')
  const brandCss = emitBrandingCss(mockBranding)

  return (
    // `fontVariables` defines the `--font-*` custom properties on <html>, which is
    // also `:root` — so the `--brand-font-*` tokens injected below can resolve them
    // for every element on the page.
    <html lang="en" data-mode="light" className={fontVariables}>
      <head>
        <style
          id="stheno-brand-tokens"
          dangerouslySetInnerHTML={{ __html: brandCss }}
        />
        {process.env.NODE_ENV === "development" && (
          <script
            dangerouslySetInnerHTML={{
              __html: `
              if (typeof window !== "undefined") {
                window.addEventListener("message", function(e) {
                  // Same-origin only. Without this, any page open in another tab can
                  // postMessage arbitrary CSS into this storefront's <style> tag and
                  // restyle it under the shopper - hide prices, restyle the checkout
                  // button, or exfiltrate the visitor through a remote CSS background.
                  if (e.origin !== window.location.origin) { return; }
                  if (e.data && e.data.type === "STHENO_PREVIEW_TOKENS") {
                    if (e.data.css) {
                      var el = document.getElementById("stheno-brand-tokens");
                      if (el) { el.textContent = e.data.css; }
                    }
                    if (e.data.themeName) {
                      document.body.setAttribute('data-theme', e.data.themeName);
                    }
                  }
                });
              }
            `,
            }}
          />
        )}
      </head>
      {/* `--brand-font-body`, not `--brand-font-family`: the latter resolves to the
          tenant's single `font_key`, which gave every theme one face for body copy.
          `font_key` still seeds heading/body/mono when a tenant sets none, so a
          store that only picks a key behaves exactly as before. */}
      {/* No `group` class here on purpose. Tailwind compiles `group-hover:x` to
          `.group:hover .group-hover\:x`, and `<body>` is `:hover` whenever the
          cursor is anywhere in the viewport — so a `group` on body made that
          selector match EVERY `group-hover:` utility in the app, permanently.
          Every product and category title was stuck on its hover accent
          (Mahal's gold) instead of `--ink`. Theme isolation does not need it:
          `mahal:`, `lustre:` and `matrix:` resolve from `data-theme` alone. */}
      <body data-theme={themeName} className="bg-brand-background text-brand-text" style={{ fontFamily: "var(--brand-font-body)" }}>
        <main className="relative">{props.children}</main>
        {process.env.NODE_ENV === "development" && <DevThemeSwitcher />}
      </body>
    </html>
  )
}
