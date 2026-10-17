import {
  ArrowPath,
  ArrowUpRightOnBox,
  BuildingStorefront,
  ComputerDesktop,
  LockClosedSolid,
  Palette,
  Phone,
} from "@medusajs/icons"
import { Badge, Button, Container, Heading, Text } from "@medusajs/ui"
import React, { useEffect, useState } from "react"
import { Link } from "react-router-dom"
import { useBranding } from "../../../hooks/api/branding"
import {
  STHENO_STOREFRONT_URL,
  STOREFRONT_THEMES,
  getStorefrontUrl,
  type StorefrontTheme,
} from "../../../lib/storefront"
import { CURATED_FONTS, DEFAULT_BRANDING_VALUES } from "../../branding/branding-view"
import { BrandingQuickDrawer } from "./branding-quick-drawer"

export const StorefrontPreviewCard: React.FC = () => {
  const { data: branding } = useBranding()
  const [device, setDevice] = useState<"desktop" | "mobile">("desktop")
  const [theme, setTheme] = useState<StorefrontTheme>("default")
  const [iframeKey, setIframeKey] = useState(0)
  const [loaded, setLoaded] = useState(false)
  const [failed, setFailed] = useState(false)
  const [isDrawerOpen, setIsDrawerOpen] = useState(false)

  const url = getStorefrontUrl(theme)

  // The iframe is keyed on url + iframeKey, so a theme or device change
  // remounts it. Reset the loading flags in lockstep so the skeleton and the
  // error state track the new request rather than the previous one.
  useEffect(() => {
    setLoaded(false)
    setFailed(false)
  }, [url, iframeKey])

  // If nothing has painted after 15s, the storefront is unreachable. The iframe
  // `error` event is unreliable for HTTP failures, so a timeout is the honest
  // signal. Cleared on unmount and on every url/key change.
  useEffect(() => {
    const t = setTimeout(() => {
      setFailed((prev) => (loaded ? prev : true))
    }, 15000)
    return () => clearTimeout(t)
  }, [url, iframeKey, loaded])

  const storeName = branding?.store_name || DEFAULT_BRANDING_VALUES.store_name
  const fontKey = branding?.font_key || DEFAULT_BRANDING_VALUES.font_key
  const fontLabel =
    CURATED_FONTS.find((f) => f.key === fontKey)?.label.split(" ")[0] || "Inter"

  const primaryColor = branding?.color_primary || DEFAULT_BRANDING_VALUES.color_primary
  const accentColor = branding?.color_accent || DEFAULT_BRANDING_VALUES.color_accent
  const backgroundColor =
    branding?.color_background || DEFAULT_BRANDING_VALUES.color_background

  const handleRefresh = () => setIframeKey((k) => k + 1)

  return (
    <>
      <Container className="flex flex-col p-0 overflow-hidden border border-ui-border-base shadow-elevation-card-rest rounded-xl bg-ui-bg-base">
        {/* Top Header */}
        <div className="flex flex-wrap items-center justify-between px-6 py-4 border-b border-ui-border-base gap-3 bg-ui-bg-subtle/30">
          <div className="flex items-center gap-2.5">
            <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-ui-bg-component border border-ui-border-base text-ui-fg-base shadow-sm">
              <BuildingStorefront className="w-4 h-4 text-ui-fg-subtle" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <Heading level="h3" className="text-sm font-semibold text-ui-fg-base">
                  Storefront Live Studio
                </Heading>
                <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[11px] font-medium bg-emerald-500/10 text-emerald-600 border border-emerald-500/20">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                  Live · Port {new URL(STHENO_STOREFRONT_URL).port || 80}
                </span>
              </div>
              <Text size="xsmall" className="text-ui-fg-subtle">
                Real-time preview of customer shopping experience &amp; theme styling
              </Text>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {/* Theme selector — one viewport at a time, so the merchant can
                compare how the same page reads under each theme without four
                iframes competing for the GPU. */}
            <div className="flex items-center bg-ui-bg-base border border-ui-border-base rounded-md p-0.5 shadow-sm">
              {STOREFRONT_THEMES.map((t) => (
                <button
                  key={t.key}
                  type="button"
                  onClick={() => setTheme(t.key)}
                  title={`${t.label} theme`}
                  className={`px-2 py-1 rounded text-xs font-medium transition-colors ${
                    theme === t.key
                      ? "bg-ui-bg-component text-ui-fg-base shadow-sm"
                      : "text-ui-fg-muted hover:text-ui-fg-base"
                  }`}
                >
                  {t.label}
                </button>
              ))}
            </div>

            {/* Device toggle */}
            <div className="flex items-center bg-ui-bg-base border border-ui-border-base rounded-md p-0.5 shadow-sm">
              <button
                type="button"
                onClick={() => setDevice("desktop")}
                title="Desktop view"
                className={`p-1 rounded flex items-center gap-1 text-xs transition-colors ${
                  device === "desktop"
                    ? "bg-ui-bg-component text-ui-fg-base shadow-sm font-medium"
                    : "text-ui-fg-muted hover:text-ui-fg-base"
                }`}
              >
                <ComputerDesktop className="w-3.5 h-3.5" />
              </button>
              <button
                type="button"
                onClick={() => setDevice("mobile")}
                title="Mobile view"
                className={`p-1 rounded flex items-center gap-1 text-xs transition-colors ${
                  device === "mobile"
                    ? "bg-ui-bg-component text-ui-fg-base shadow-sm font-medium"
                    : "text-ui-fg-muted hover:text-ui-fg-base"
                }`}
              >
                <Phone className="w-3.5 h-3.5" />
              </button>
            </div>

            <Button
              variant="secondary"
              size="small"
              onClick={() => setIsDrawerOpen(true)}
              className="gap-x-1.5 text-xs font-medium"
            >
              <Palette className="w-3.5 h-3.5" />
              Quick Theme Edit
            </Button>
            <Button variant="transparent" size="small" asChild className="text-xs">
              <a
                href={url}
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-1 text-ui-fg-subtle hover:text-ui-fg-base"
              >
                Open Store
                <ArrowUpRightOnBox className="w-3.5 h-3.5 ml-0.5" />
              </a>
            </Button>
          </div>
        </div>

        {/* Browser Address Bar */}
        <div className="flex items-center gap-2 px-4 py-2 bg-ui-bg-subtle/80 border-b border-ui-border-base text-xs text-ui-fg-subtle">
          <div className="flex items-center gap-2 px-3 py-1 rounded-md bg-ui-bg-base border border-ui-border-base text-ui-fg-muted font-mono text-[11px] max-w-[420px] w-full shadow-sm">
            <LockClosedSolid className="w-3 h-3 text-emerald-600 shrink-0" />
            <span className="truncate flex-1 text-ui-fg-base">{url}</span>
            <button
              type="button"
              onClick={handleRefresh}
              className="text-ui-fg-muted hover:text-ui-fg-base p-0.5 rounded transition-colors"
              title="Refresh storefront preview"
            >
              <ArrowPath className="w-3 h-3" />
            </button>
          </div>
        </div>

        {/* Viewport Canvas */}
        <div className="relative w-full bg-ui-bg-subtle/40 p-4 flex items-center justify-center overflow-hidden min-h-[360px]">
          {failed ? (
            <div className="flex flex-col items-center justify-center gap-3 text-center p-6">
              <BuildingStorefront className="w-10 h-10 text-ui-fg-muted" />
              <Heading level="h3" className="text-sm font-semibold text-ui-fg-base">
                Storefront not reachable
              </Heading>
              <Text size="xsmall" className="text-ui-fg-subtle max-w-xs">
                Nothing answered at {url}. Check that the storefront dev server is
                running on port 8000, then refresh.
              </Text>
              <Button variant="secondary" size="small" onClick={handleRefresh}>
                Retry
              </Button>
            </div>
          ) : (
            <>
              {!loaded && (
                <div className="absolute inset-0 flex items-center justify-center">
                  <div className="w-3/4 h-3/4 bg-ui-bg-component rounded-lg animate-pulse" />
                </div>
              )}
              {device === "desktop" ? (
                <div className="w-full aspect-video bg-ui-bg-base rounded-lg border border-ui-border-base overflow-hidden shadow-sm">
                  <iframe
                    key={`${url}-${iframeKey}`}
                    src={url}
                    title="Storefront Desktop Preview"
                    className="w-full h-full border-none"
                    loading="lazy"
                    onLoad={() => setLoaded(true)}
                    onError={() => setFailed(true)}
                  />
                </div>
              ) : (
                <div
                  className="relative overflow-hidden shadow-2xl"
                  style={{ width: 375, height: 780, transform: "scale(0.5)", transformOrigin: "top center" }}
                >
                  <div className="absolute inset-0 rounded-[2.5rem] border-[10px] border-zinc-800 bg-zinc-800 overflow-hidden">
                    {/* Notch — part of the frame, not an overlay on the iframe */}
                    <div className="absolute top-0 left-1/2 -translate-x-1/2 w-28 h-6 bg-zinc-800 rounded-b-2xl z-10" />
                    <iframe
                      key={`${url}-${iframeKey}`}
                      src={url}
                      title="Storefront Mobile Preview"
                      className="w-full h-full border-none pt-5"
                      loading="lazy"
                      onLoad={() => setLoaded(true)}
                      onError={() => setFailed(true)}
                    />
                  </div>
                </div>
              )}
            </>
          )}
        </div>

        {/* Live Branding Identity Bar */}
        <div className="flex flex-wrap items-center justify-between px-6 py-3 border-t border-ui-border-base bg-ui-bg-subtle/20 gap-3 text-xs">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-1.5">
              <span className="text-ui-fg-muted">Store Name:</span>
              <span className="font-semibold text-ui-fg-base">{storeName}</span>
            </div>

            <span className="text-ui-border-strong">|</span>

            <div className="flex items-center gap-1.5">
              <span className="text-ui-fg-muted">Typography:</span>
              <Badge size="2xsmall" color="grey" className="font-medium">
                {fontLabel}
              </Badge>
            </div>

            <span className="text-ui-border-strong">|</span>

            {/* Colors */}
            <div className="flex items-center gap-2">
              <span className="text-ui-fg-muted">Palette:</span>
              <div
                className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-ui-border-base bg-ui-bg-base"
                title={`Primary: ${primaryColor}`}
              >
                <span
                  className="w-2.5 h-2.5 rounded-full border border-ui-border-base"
                  style={{ backgroundColor: primaryColor }}
                />
                <span className="font-mono text-xs text-ui-fg-subtle uppercase">
                  {primaryColor}
                </span>
              </div>
              <div
                className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-ui-border-base bg-ui-bg-base"
                title={`Accent: ${accentColor}`}
              >
                <span
                  className="w-2.5 h-2.5 rounded-full border border-ui-border-base"
                  style={{ backgroundColor: accentColor }}
                />
                <span className="font-mono text-xs text-ui-fg-subtle uppercase">
                  {accentColor}
                </span>
              </div>
              <div
                className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-ui-border-base bg-ui-bg-base"
                title={`Background: ${backgroundColor}`}
              >
                <span
                  className="w-2.5 h-2.5 rounded-full border border-ui-border-base"
                  style={{ backgroundColor: backgroundColor }}
                />
                <span className="font-mono text-xs text-ui-fg-subtle uppercase">
                  {backgroundColor}
                </span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Link
              to="/settings/branding"
              className="text-xs font-medium text-ui-fg-interactive hover:underline inline-flex items-center gap-1"
            >
              Full Theme Studio →
            </Link>
          </div>
        </div>
      </Container>

      {/* Quick Brand Edit Drawer */}
      <BrandingQuickDrawer open={isDrawerOpen} onClose={() => setIsDrawerOpen(false)} />
    </>
  )
}