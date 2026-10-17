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
import React, { useState } from "react"
import { Link } from "react-router-dom"
import { useBranding } from "../../../hooks/api/branding"
import { STHENO_STOREFRONT_URL } from "../../../lib/storefront"
import { CURATED_FONTS, DEFAULT_BRANDING_VALUES } from "../../branding/branding-view"
import { BrandingQuickDrawer } from "./branding-quick-drawer"

export const StorefrontPreviewCard: React.FC = () => {
  const { data: branding } = useBranding()
  const [device, setDevice] = useState<"desktop" | "mobile">("desktop")
  const [isDrawerOpen, setIsDrawerOpen] = useState(false)
  const [iframeKey, setIframeKey] = useState(0)

  const storeName = branding?.store_name || DEFAULT_BRANDING_VALUES.store_name
  const fontKey = branding?.font_key || DEFAULT_BRANDING_VALUES.font_key
  const fontLabel =
    CURATED_FONTS.find((f) => f.key === fontKey)?.label.split(" ")[0] || "Inter"

  const primaryColor = branding?.color_primary || DEFAULT_BRANDING_VALUES.color_primary
  const accentColor = branding?.color_accent || DEFAULT_BRANDING_VALUES.color_accent
  const backgroundColor =
    branding?.color_background || DEFAULT_BRANDING_VALUES.color_background

  const handleRefresh = () => {
    setIframeKey((prev) => prev + 1)
  }

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
                  Live · Port 8000
                </span>
              </div>
              <Text size="xsmall" className="text-ui-fg-subtle">
                Real-time preview of customer shopping experience & theme styling
              </Text>
            </div>
          </div>

          <div className="flex items-center gap-2">
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
                href={STHENO_STOREFRONT_URL}
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

        {/* Browser Mockup Chrome */}
        <div className="flex items-center justify-between px-4 py-2 bg-ui-bg-subtle/80 border-b border-ui-border-base text-xs text-ui-fg-subtle">
          {/* The three macOS traffic-light dots that used to sit here are gone.
              They were decoration on a live <iframe>, not a screenshot, and
              re-drawing window chrome is a tell that reads as a mockup. The
              address bar below stays: it carries the real URL, the refresh
              control and the device toggle. */}

          {/* Browser Address Bar */}
          <div className="flex items-center gap-2 px-3 py-1 rounded-md bg-ui-bg-base border border-ui-border-base text-ui-fg-muted font-mono text-[11px] max-w-[420px] w-full mr-4 shadow-sm">
            <LockClosedSolid className="w-3 h-3 text-emerald-600 shrink-0" />
            <span className="truncate flex-1 text-ui-fg-base">{STHENO_STOREFRONT_URL}</span>
            <button
              type="button"
              onClick={handleRefresh}
              className="text-ui-fg-muted hover:text-ui-fg-base p-0.5 rounded transition-colors"
              title="Refresh storefront preview"
            >
              <ArrowPath className="w-3 h-3" />
            </button>
          </div>

          {/* Device Toggle */}
          <div className="flex items-center bg-ui-bg-base border border-ui-border-base rounded-md p-0.5 shadow-sm">
            <button
              type="button"
              onClick={() => setDevice("desktop")}
              className={`p-1 rounded flex items-center gap-1 text-xs transition-colors ${
                device === "desktop"
                  ? "bg-ui-bg-component text-ui-fg-base shadow-sm font-medium"
                  : "text-ui-fg-muted hover:text-ui-fg-base"
              }`}
              title="Desktop View"
            >
              <ComputerDesktop className="w-3.5 h-3.5" />
            </button>
            <button
              type="button"
              onClick={() => setDevice("mobile")}
              className={`p-1 rounded flex items-center gap-1 text-xs transition-colors ${
                device === "mobile"
                  ? "bg-ui-bg-component text-ui-fg-base shadow-sm font-medium"
                  : "text-ui-fg-muted hover:text-ui-fg-base"
              }`}
              title="Mobile View"
            >
              <Phone className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* Viewport Canvas */}
        <div className="relative w-full bg-ui-bg-subtle/40 p-4 flex items-center justify-center overflow-hidden min-h-[380px]">
          {device === "desktop" ? (
            <div className="w-full h-[380px] bg-ui-bg-base rounded-lg border border-ui-border-base overflow-hidden shadow-sm">
              <iframe
                key={iframeKey}
                src={STHENO_STOREFRONT_URL}
                title="Storefront Desktop Preview"
                className="w-full h-full border-none"
                loading="lazy"
              />
            </div>
          ) : (
            <div className="w-[375px] h-[460px] bg-ui-bg-base rounded-[24px] border-[6px] border-ui-border-strong overflow-hidden shadow-elevation-modal relative">
              {/* Simulated Mobile Notch / Speaker */}
              <div className="absolute top-1 left-1/2 -translate-x-1/2 w-20 h-3 bg-ui-border-strong rounded-full z-10 pointer-events-none" />
              <iframe
                key={iframeKey}
                src={STHENO_STOREFRONT_URL}
                title="Storefront Mobile Preview"
                className="w-full h-full border-none pt-4"
                loading="lazy"
              />
            </div>
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
                  className="w-2.5 h-2.5 rounded-full border border-black/10"
                  style={{ backgroundColor: primaryColor }}
                />
                <span className="font-mono text-[10px] text-ui-fg-subtle uppercase">
                  {primaryColor}
                </span>
              </div>
              <div
                className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-ui-border-base bg-ui-bg-base"
                title={`Accent: ${accentColor}`}
              >
                <span
                  className="w-2.5 h-2.5 rounded-full border border-black/10"
                  style={{ backgroundColor: accentColor }}
                />
                <span className="font-mono text-[10px] text-ui-fg-subtle uppercase">
                  {accentColor}
                </span>
              </div>
              <div
                className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-ui-border-base bg-ui-bg-base"
                title={`Background: ${backgroundColor}`}
              >
                <span
                  className="w-2.5 h-2.5 rounded-full border border-black/10"
                  style={{ backgroundColor: backgroundColor }}
                />
                <span className="font-mono text-[10px] text-ui-fg-subtle uppercase">
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
