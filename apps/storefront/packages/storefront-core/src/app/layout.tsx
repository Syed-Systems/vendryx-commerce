import { getBaseURL } from "@lib/util/env"
import { Metadata } from "next"
import "../styles/globals.css"
// P27-03: the bound frontend's token layer, loaded after globals so a frontend can
// re-point the UI preset's CSS variables. Every frontend supplies this file.
import "@frontend/styles/tokens.css"
import { emitBrandingCss } from "../lib/branding"

export const metadata: Metadata = {
  metadataBase: new URL(getBaseURL()),
}

export default function RootLayout(props: { children: React.ReactNode }) {
  const brandCss = emitBrandingCss(null)

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
                  if (e.data && e.data.type === "STHENO_PREVIEW_TOKENS" && e.data.css) {
                    var el = document.getElementById("stheno-brand-tokens");
                    if (el) { el.textContent = e.data.css; }
                  }
                });
              }
            `,
          }}
        />
      </head>
      <body style={{ fontFamily: "var(--brand-font-family)" }}>
        <main className="relative">{props.children}</main>
      </body>
    </html>
  )
}
