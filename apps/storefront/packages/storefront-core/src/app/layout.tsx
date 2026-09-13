import { getBaseURL } from "@lib/util/env"
import { Metadata } from "next"
import { headers } from "next/headers"
import "../styles/globals.css"
// P27-03: the bound frontend's token layer, loaded after globals so a frontend can
// re-point the UI preset's CSS variables. Every frontend supplies this file.
import "@frontend/styles/tokens.css"
import { emitBrandingCss, resolveThemeByHost } from "../lib/branding"
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
    <html lang="en" data-mode="light">
      <head>
        <style
          id="stheno-brand-tokens"
          dangerouslySetInnerHTML={{ __html: brandCss }}
        />
      </head>
      <body data-theme={themeName} className="group bg-brand-background text-brand-text" style={{ fontFamily: "var(--brand-font-family)" }}>
        <main className="relative">{props.children}</main>
        {process.env.NODE_ENV === "development" && <DevThemeSwitcher />}
      </body>
    </html>
  )
}
