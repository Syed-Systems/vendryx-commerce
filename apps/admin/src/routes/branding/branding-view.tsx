import {
  ArrowPath,
  ArrowUpRightOnBox,
  BuildingStorefront,
  CloudArrowUp,
  ComputerDesktop,
  GlobeEurope,
  LockClosedSolid,
  Phone,
  Photo,
  Trash,
} from "@medusajs/icons"
import { Button, Container, Heading, Input, Select, Text, toast } from "@medusajs/ui"
import React, { useEffect, useRef, useState } from "react"
import { SingleColumnPage } from "../../components/layout/pages"
import { useBranding, useStorefrontAppearance, useUpsertBranding, useUpsertStorefrontAppearance, type StorefrontAppearance } from "../../hooks/api/branding"
import { STHENO_STOREFRONT_URL } from "../../lib/storefront"
import { extractLogoPalette, type LogoPaletteSuggestion } from "./logo-palette"

export const CURATED_FONTS = [
  { key: "inter", label: "Inter (Clean Modern UI)", family: "'Inter', sans-serif" },
  { key: "geist", label: "Geist Sans (Tech / Minimal)", family: "'Geist Sans', sans-serif" },
  { key: "noto_sans_bengali", label: "Noto Sans Bengali (Bengali UI)", family: "'Noto Sans Bengali', sans-serif" },
  { key: "playfair", label: "Playfair Display (Luxury / Editorial)", family: "'Playfair Display', Georgia, serif" },
  { key: "plus_jakarta_sans", label: "Plus Jakarta Sans (Consumer)", family: "'Plus Jakarta Sans', sans-serif" },
  { key: "roboto", label: "Roboto (Universal Sans)", family: "Roboto, sans-serif" },
  { key: "cinzel", label: "Cinzel (Classical Serif)", family: "Cinzel, serif" },
]

export interface BrandingFormState {
  store_name: string
  tagline: string
  logo_url: string
  favicon_url: string
  font_key: string
  color_primary: string
  color_secondary: string
  color_accent: string
  color_background: string
  color_surface: string
  color_text: string
}

export const DEFAULT_BRANDING_VALUES: BrandingFormState = {
  store_name: "Stheno Store",
  tagline: "Excellence in handcrafted commerce, with real-time token rendering and sub-millisecond styling response.",
  logo_url: "",
  favicon_url: "",
  font_key: "inter",
  color_primary: "#111827",
  color_secondary: "#4B5563",
  color_accent: "#2563EB",
  color_background: "#FFFFFF",
  color_surface: "#F9FAFB",
  color_text: "#111827",
}

export const BrandingView = () => {
  const { data: brandingData } = useBranding()
  const { data: appearanceData } = useStorefrontAppearance()
  const { mutateAsync: upsertBranding, isPending: isSaving } = useUpsertBranding()
  const { mutateAsync: upsertAppearance, isPending: isSavingAppearance } = useUpsertStorefrontAppearance()

  const [formState, setFormState] = useState<BrandingFormState>(DEFAULT_BRANDING_VALUES)
  const [previewMode, setPreviewMode] = useState<"storefront" | "component">("storefront")
  const [device, setDevice] = useState<"desktop" | "mobile">("desktop")
  const [iframeKey, setIframeKey] = useState(0)
  const [logoPalette, setLogoPalette] = useState<LogoPaletteSuggestion | null>(null)
  const [appearance, setAppearance] = useState<StorefrontAppearance>({
    storefront_archetype: "baseline" as const,
    product_card_style: "standard" as const,
    product_image_aspect: "portrait" as const,
  })
  const [isAnalyzingLogo, setIsAnalyzingLogo] = useState(false)
  const logoAnalysisId = useRef(0)
  const iframeRef = useRef<HTMLIFrameElement>(null)
  const logoInputRef = useRef<HTMLInputElement>(null)
  const faviconInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (brandingData) {
      setFormState({
        store_name: brandingData.store_name || DEFAULT_BRANDING_VALUES.store_name,
        tagline: DEFAULT_BRANDING_VALUES.tagline,
        logo_url: brandingData.logo_url || brandingData.logo_file_id || "",
        favicon_url: brandingData.favicon_url || brandingData.favicon_file_id || "",
        font_key: brandingData.font_key || DEFAULT_BRANDING_VALUES.font_key,
        color_primary: brandingData.color_primary || DEFAULT_BRANDING_VALUES.color_primary,
        color_secondary: brandingData.color_secondary || DEFAULT_BRANDING_VALUES.color_secondary,
        color_accent: brandingData.color_accent || DEFAULT_BRANDING_VALUES.color_accent,
        color_background: brandingData.color_background || DEFAULT_BRANDING_VALUES.color_background,
        color_surface: brandingData.color_surface || DEFAULT_BRANDING_VALUES.color_surface,
        color_text: brandingData.color_text || DEFAULT_BRANDING_VALUES.color_text,
      })
    }
  }, [brandingData])

  useEffect(() => {
    if (appearanceData) setAppearance(appearanceData)
  }, [appearanceData])

  const selectedFont = CURATED_FONTS.find((f) => f.key === formState.font_key) || CURATED_FONTS[0]

  // Broadcast live branding tokens to embedded storefront iframe in real-time
  useEffect(() => {
    if (!iframeRef.current?.contentWindow) return
    const css = `:root {
      --brand-primary: ${formState.color_primary};
      --brand-secondary: ${formState.color_secondary};
      --brand-accent: ${formState.color_accent};
      --brand-background: ${formState.color_background};
      --brand-surface: ${formState.color_surface};
      --brand-text: ${formState.color_text};
      --brand-font-family: ${selectedFont.family};
    }`
    iframeRef.current.contentWindow.postMessage(
      {
        type: "STHENO_PREVIEW_TOKENS",
        css,
        store_name: formState.store_name,
        logo_url: formState.logo_url,
        favicon_url: formState.favicon_url,
        appearance,
      },
      "*"
    )
  }, [formState, selectedFont, iframeKey, appearance])

  const handleFieldChange = (field: keyof BrandingFormState, value: string) => {
    if (field === "logo_url") {
      logoAnalysisId.current += 1
      setLogoPalette(null)
      setIsAnalyzingLogo(false)
    }
    setFormState((prev) => ({ ...prev, [field]: value }))
  }

  const handleAnalyzeLogo = async () => {
    const analysisId = ++logoAnalysisId.current
    setIsAnalyzingLogo(true)
    setLogoPalette(null)
    const palette = await extractLogoPalette(formState.logo_url)
    if (analysisId !== logoAnalysisId.current) return
    setIsAnalyzingLogo(false)
    if (!palette) {
      toast.error("Could not extract colors", { description: "Use a same-origin PNG, JPEG, or WebP logo under 5MB. SVG and remote logos are not analyzed." })
      return
    }
    setLogoPalette(palette)
  }

  const handleApplyLogoPalette = () => {
    if (!logoPalette) return
    setFormState((previous) => ({
      ...previous,
      color_primary: logoPalette.primary,
      color_secondary: logoPalette.secondary,
      color_accent: logoPalette.accent,
    }))
    toast.success("Palette applied to the live preview", { description: "Review the preview and save your branding changes to publish them." })
  }

  const handleLogoFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    if (!["image/png", "image/jpeg", "image/webp", "image/svg+xml"].includes(file.type)) {
      toast.error("Unsupported logo file", { description: "Choose a PNG, JPEG, WebP, or SVG image." })
      e.target.value = ""
      return
    }
    if (file.size > 5 * 1024 * 1024) {
      toast.error("Logo file exceeds 5MB limit")
      e.target.value = ""
      return
    }
    const reader = new FileReader()
    reader.onload = () => {
      const dataUrl = reader.result as string
      logoAnalysisId.current += 1
      setLogoPalette(null)
      setIsAnalyzingLogo(false)
      setFormState((prev) => ({ ...prev, logo_url: dataUrl }))
      toast.success("Logo uploaded to preview")
    }
    reader.onerror = () => toast.error("Logo file could not be read")
    reader.readAsDataURL(file)
  }

  const handleFaviconFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    if (file.size > 1024 * 1024) {
      toast.error("Favicon file exceeds 1MB limit")
      return
    }
    const reader = new FileReader()
    reader.onload = () => {
      const dataUrl = reader.result as string
      setFormState((prev) => ({ ...prev, favicon_url: dataUrl }))
      toast.success("Favicon uploaded to preview")
    }
    reader.readAsDataURL(file)
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await upsertBranding({
        store_name: formState.store_name,
        logo_file_id: formState.logo_url || null,
        favicon_file_id: formState.favicon_url || null,
        font_key: formState.font_key,
        color_primary: formState.color_primary,
        color_secondary: formState.color_secondary,
        color_accent: formState.color_accent,
        color_background: formState.color_background,
        color_surface: formState.color_surface,
        color_text: formState.color_text,
      })
      await upsertAppearance(appearance)
      toast.success("Branding updated successfully")
      setIframeKey((k) => k + 1)
    } catch (err: any) {
      toast.error(err.message || "Failed to update branding")
    }
  }

  const handleReset = () => {
    logoAnalysisId.current += 1
    setLogoPalette(null)
    setIsAnalyzingLogo(false)
    setFormState({ ...DEFAULT_BRANDING_VALUES })
    setAppearance({ storefront_archetype: "baseline", product_card_style: "standard", product_image_aspect: "portrait" })
  }

  return (
    <SingleColumnPage widgets={{ before: [], after: [] }}>
      <div className="flex flex-col gap-y-6 pb-12">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <Heading level="h1">Storefront Branding</Heading>
            <Text className="text-ui-fg-subtle text-sm">
              Customise brand identity, curated typography, and colour tokens. Resolves per request with zero rebuilds.
            </Text>
          </div>
          <div className="flex items-center gap-x-2">
            <Button variant="secondary" size="small" onClick={handleReset} disabled={isSaving || isSavingAppearance}>
              Reset Defaults
            </Button>
            <Button variant="primary" size="small" onClick={handleSave} isLoading={isSaving || isSavingAppearance}>
              Save Changes
            </Button>
          </div>
        </div>

        {/* Two-column layout: Form (left) and Live Preview (right) */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Settings Form Column */}
          <div className="lg:col-span-6 flex flex-col gap-y-6">
            {/* 1. Core Brand Identity */}
            <Container>
              <div className="flex flex-col gap-y-4">
                <div>
                  <Heading level="h2" className="text-base font-semibold">Storefront Composition</Heading>
                  <Text className="text-xs text-ui-fg-muted mt-1">Choosing an archetype sets matching card and image defaults. You can adjust either below; changes apply only after you save.</Text>
                </div>
                <label className="text-sm font-medium">Category archetype
                  <Select value={appearance.storefront_archetype} onValueChange={(value) => setAppearance(value === "editorial"
                    ? { storefront_archetype: "editorial", product_card_style: "editorial", product_image_aspect: "portrait" }
                    : value === "dense_catalog"
                      ? { storefront_archetype: "dense_catalog", product_card_style: "dense", product_image_aspect: "square" }
                      : { storefront_archetype: "baseline", product_card_style: "standard", product_image_aspect: "portrait" })}>
                    <Select.Trigger aria-label="Category archetype"><Select.Value /></Select.Trigger><Select.Content>
                      <Select.Item value="baseline">Baseline</Select.Item><Select.Item value="editorial">Editorial</Select.Item><Select.Item value="dense_catalog">Dense catalog</Select.Item>
                    </Select.Content>
                  </Select>
                </label>
                <label className="text-sm font-medium">Product card style
                  <Select value={appearance.product_card_style} onValueChange={(value) => setAppearance((p) => ({ ...p, product_card_style: value as typeof p.product_card_style }))}>
                    <Select.Trigger aria-label="Product card style"><Select.Value /></Select.Trigger><Select.Content>
                      <Select.Item value="standard">Standard</Select.Item><Select.Item value="editorial">Editorial</Select.Item><Select.Item value="dense">Dense</Select.Item>
                    </Select.Content>
                  </Select>
                </label>
                <label className="text-sm font-medium">Product image aspect
                  <Select value={appearance.product_image_aspect} onValueChange={(value) => setAppearance((p) => ({ ...p, product_image_aspect: value as typeof p.product_image_aspect }))}>
                    <Select.Trigger aria-label="Product image aspect"><Select.Value /></Select.Trigger><Select.Content>
                      <Select.Item value="square">Square</Select.Item><Select.Item value="portrait">Portrait</Select.Item><Select.Item value="landscape">Landscape</Select.Item>
                    </Select.Content>
                  </Select>
                </label>
              </div>
            </Container>

            <Container>
              <div className="flex flex-col gap-y-4">
                <Heading level="h2" className="text-base font-semibold">
                  Brand Identity
                </Heading>

                <div>
                  <Text className="text-sm font-medium mb-1">Store Name</Text>
                  <Input
                    name="store_name"
                    aria-label="Store Name"
                    value={formState.store_name}
                    onChange={(e) => handleFieldChange("store_name", e.target.value)}
                    placeholder="e.g. Stheno Flagship"
                  />
                  <Text className="text-xs text-ui-fg-muted mt-1">
                    Overrides default store title across header, title tags, and checkout.
                  </Text>
                </div>

                <div>
                  <Text className="text-sm font-medium mb-1">Brand Tagline</Text>
                  <Input
                    name="tagline"
                    aria-label="Brand Tagline"
                    value={formState.tagline}
                    onChange={(e) => handleFieldChange("tagline", e.target.value)}
                    placeholder="e.g. Excellence in handcrafted commerce"
                  />
                  <Text className="text-xs text-ui-fg-muted mt-1">
                    Displayed in hero banners, footer metadata, and social previews.
                  </Text>
                </div>

                <div>
                  <Text className="text-sm font-medium mb-1">Curated Typography</Text>
                  <Select
                    value={formState.font_key}
                    onValueChange={(val) => handleFieldChange("font_key", val)}
                  >
                    <Select.Trigger aria-label="Curated Typography">
                      <Select.Value />
                    </Select.Trigger>
                    <Select.Content>
                      {CURATED_FONTS.map((font) => (
                        <Select.Item key={font.key} value={font.key}>
                          {font.label}
                        </Select.Item>
                      ))}
                    </Select.Content>
                  </Select>
                  <div
                    className="mt-2 p-2.5 rounded-md border border-ui-border-base bg-ui-bg-subtle text-xs"
                    style={{ fontFamily: selectedFont.family }}
                  >
                    <span className="text-ui-fg-muted mr-2">Specimen:</span>
                    <span className="text-ui-fg-base font-medium">
                      The quick brown fox jumps over the lazy dog
                    </span>
                  </div>
                  <Text className="text-xs text-ui-fg-muted mt-1">
                    Self-hosted, licence-cleared WOFF2 fonts complying with checkout script isolation (ADR-013).
                  </Text>
                </div>
              </div>
            </Container>

            {/* 2. Visual Brand Assets (Logo & Favicon) */}
            <Container>
              <div className="flex flex-col gap-y-5">
                <div>
                  <Heading level="h2" className="text-base font-semibold">
                    Visual Brand Assets
                  </Heading>
                  <Text className="text-xs text-ui-fg-muted mt-0.5">
                    Official merchant logo and browser favicon emitted to all storefront templates.
                  </Text>
                </div>

                {/* Store Logo Section */}
                <div className="flex flex-col gap-y-2">
                  <div className="flex items-center justify-between">
                    <Text className="text-xs font-semibold text-ui-fg-base flex items-center gap-1.5">
                      <Photo className="w-3.5 h-3.5 text-ui-fg-subtle" />
                      Storefront Logo
                    </Text>
                    {formState.logo_url && (
                      <button
                        type="button"
                        onClick={() => handleFieldChange("logo_url", "")}
                        className="text-[11px] text-rose-600 hover:underline flex items-center gap-1"
                      >
                        <Trash className="w-3 h-3" />
                        Remove Logo
                      </button>
                    )}
                  </div>

                  {formState.logo_url ? (
                    <div className="flex items-center gap-3 p-3 rounded-xl border border-ui-border-base bg-ui-bg-subtle/40">
                      <div className="w-24 h-12 rounded-lg border border-ui-border-base bg-ui-bg-base flex items-center justify-center p-1 overflow-hidden shadow-sm">
                        <img
                          src={formState.logo_url}
                          alt="Store Logo Preview"
                          className="max-h-full max-w-full object-contain"
                        />
                      </div>
                      <div className="flex-1 min-w-0">
                        <Text size="small" weight="plus" className="text-ui-fg-base truncate">
                          Active Store Logo
                        </Text>
                        <Text size="xsmall" className="text-ui-fg-muted">
                          Displayed in header and customer invoices
                        </Text>
                      </div>
                      <Button
                        type="button"
                        variant="secondary"
                        size="small"
                        onClick={() => logoInputRef.current?.click()}
                      >
                        Replace
                      </Button>
                    </div>
                  ) : (
                    <div
                      onClick={() => logoInputRef.current?.click()}
                      className="flex flex-col items-center justify-center p-5 rounded-xl border border-dashed border-ui-border-base bg-ui-bg-subtle/30 hover:bg-ui-bg-subtle/60 cursor-pointer transition-colors text-center group"
                    >
                      <div className="w-8 h-8 rounded-full bg-ui-bg-component border border-ui-border-base flex items-center justify-center mb-2 group-hover:scale-105 transition-transform">
                        <CloudArrowUp className="w-4 h-4 text-ui-fg-subtle" />
                      </div>
                      <Text size="small" weight="plus" className="text-ui-fg-base">
                        Click or drag to upload Storefront Logo
                      </Text>
                      <Text size="xsmall" className="text-ui-fg-muted mt-0.5">
                        SVG, PNG, WebP or JPEG (max 5MB, transparent background recommended)
                      </Text>
                    </div>
                  )}

                  <input
                    ref={logoInputRef}
                    type="file"
                    accept="image/png,image/svg+xml,image/jpeg,image/webp"
                    className="hidden"
                    onChange={handleLogoFileChange}
                  />

                  {/* Optional Direct URL field */}
                  <div className="flex items-center gap-2 pt-1">
                    <span className="text-[11px] text-ui-fg-muted shrink-0">Or Image URL:</span>
                    <Input
                      name="logo_url"
                      aria-label="Logo URL"
                      value={formState.logo_url.startsWith("data:") ? "(Uploaded image data)" : formState.logo_url}
                      onChange={(e) => handleFieldChange("logo_url", e.target.value)}
                      placeholder="https://cdn.example.com/logo.svg"
                      className="text-xs font-mono h-7"
                    />
                  </div>
                  <div className="flex flex-wrap items-center gap-2">
                    <Button type="button" variant="secondary" size="small" onClick={handleAnalyzeLogo} isLoading={isAnalyzingLogo} disabled={!formState.logo_url}>
                      Suggest logo colors
                    </Button>
                    {logoPalette && <>
                      {[logoPalette.primary, logoPalette.secondary, logoPalette.accent].map((color, index) => (
                        <span key={`${color}-${index}`} title={color} aria-label={`Suggested color ${index + 1}: ${color}`} className="h-6 w-6 rounded-full border border-ui-border-base" style={{ backgroundColor: color }} />
                      ))}
                      <Button type="button" variant="secondary" size="small" onClick={handleApplyLogoPalette}>Use suggested colors</Button>
                    </>}
                  </div>
                </div>

                {/* Favicon Section */}
                <div className="flex flex-col gap-y-2 pt-2 border-t border-ui-border-base">
                  <div className="flex items-center justify-between">
                    <Text className="text-xs font-semibold text-ui-fg-base flex items-center gap-1.5">
                      <GlobeEurope className="w-3.5 h-3.5 text-ui-fg-subtle" />
                      Browser Favicon
                    </Text>
                    {formState.favicon_url && (
                      <button
                        type="button"
                        onClick={() => setFormState((p) => ({ ...p, favicon_url: "" }))}
                        className="text-[11px] text-rose-600 hover:underline flex items-center gap-1"
                      >
                        <Trash className="w-3 h-3" />
                        Remove Favicon
                      </button>
                    )}
                  </div>

                  {formState.favicon_url ? (
                    <div className="flex items-center gap-3 p-3 rounded-xl border border-ui-border-base bg-ui-bg-subtle/40">
                      <div className="w-10 h-10 rounded-lg border border-ui-border-base bg-ui-bg-base flex items-center justify-center p-1 shadow-sm">
                        <img
                          src={formState.favicon_url}
                          alt="Favicon Preview"
                          className="w-6 h-6 object-contain"
                        />
                      </div>
                      <div className="flex-1 min-w-0">
                        <Text size="small" weight="plus" className="text-ui-fg-base truncate">
                          Active Browser Favicon
                        </Text>
                        <Text size="xsmall" className="text-ui-fg-muted">
                          Displayed in customer browser tabs & bookmarks
                        </Text>
                      </div>
                      <Button
                        type="button"
                        variant="secondary"
                        size="small"
                        onClick={() => faviconInputRef.current?.click()}
                      >
                        Replace
                      </Button>
                    </div>
                  ) : (
                    <div
                      onClick={() => faviconInputRef.current?.click()}
                      className="flex flex-col items-center justify-center p-4 rounded-xl border border-dashed border-ui-border-base bg-ui-bg-subtle/30 hover:bg-ui-bg-subtle/60 cursor-pointer transition-colors text-center group"
                    >
                      <div className="w-7 h-7 rounded-full bg-ui-bg-component border border-ui-border-base flex items-center justify-center mb-1.5 group-hover:scale-105 transition-transform">
                        <CloudArrowUp className="w-3.5 h-3.5 text-ui-fg-subtle" />
                      </div>
                      <Text size="small" weight="plus" className="text-ui-fg-base">
                        Click to upload Browser Favicon
                      </Text>
                      <Text size="xsmall" className="text-ui-fg-muted mt-0.5">
                        ICO, PNG, SVG (32×32 or 48×48 px, max 1MB)
                      </Text>
                    </div>
                  )}

                  <input
                    ref={faviconInputRef}
                    type="file"
                    accept="image/x-icon,image/png,image/svg+xml"
                    className="hidden"
                    onChange={handleFaviconFileChange}
                  />

                  <div className="flex items-center gap-2 pt-1">
                    <span className="text-[11px] text-ui-fg-muted shrink-0">Or Favicon URL:</span>
                    <Input
                      name="favicon_url"
                      aria-label="Favicon URL"
                      value={formState.favicon_url.startsWith("data:") ? "(Uploaded icon data)" : formState.favicon_url}
                      onChange={(e) => handleFieldChange("favicon_url", e.target.value)}
                      placeholder="https://cdn.example.com/favicon.ico"
                      className="text-xs font-mono h-7"
                    />
                  </div>
                </div>
              </div>
            </Container>

            {/* 3. Colour Tokens */}
            <Container>
              <div className="flex flex-col gap-y-4">
                <Heading level="h2" className="text-base font-semibold">
                  Colour Tokens
                </Heading>
                <Text className="text-xs text-ui-fg-muted">
                  Tokens emit into CSS variables (<code className="text-xs">--brand-*</code>) consumed by storefront layouts.
                </Text>

                <div className="grid grid-cols-2 gap-4">
                  {/* Primary Color */}
                  <div>
                    <Text className="text-xs font-medium mb-1">Primary Color</Text>
                    <div className="flex items-center gap-x-2">
                      <input
                        type="color"
                        aria-label="Primary Color Picker"
                        value={formState.color_primary}
                        onChange={(e) => handleFieldChange("color_primary", e.target.value)}
                        className="w-8 h-8 rounded border border-ui-border-base cursor-pointer p-0"
                      />
                      <Input
                        name="color_primary"
                        aria-label="Primary Color"
                        value={formState.color_primary}
                        onChange={(e) => handleFieldChange("color_primary", e.target.value)}
                        className="font-mono text-xs"
                      />
                    </div>
                  </div>

                  {/* Secondary Color */}
                  <div>
                    <Text className="text-xs font-medium mb-1">Secondary Color</Text>
                    <div className="flex items-center gap-x-2">
                      <input
                        type="color"
                        aria-label="Secondary Color Picker"
                        value={formState.color_secondary}
                        onChange={(e) => handleFieldChange("color_secondary", e.target.value)}
                        className="w-8 h-8 rounded border border-ui-border-base cursor-pointer p-0"
                      />
                      <Input
                        name="color_secondary"
                        aria-label="Secondary Color"
                        value={formState.color_secondary}
                        onChange={(e) => handleFieldChange("color_secondary", e.target.value)}
                        className="font-mono text-xs"
                      />
                    </div>
                  </div>

                  {/* Accent Color */}
                  <div>
                    <Text className="text-xs font-medium mb-1">Accent Color</Text>
                    <div className="flex items-center gap-x-2">
                      <input
                        type="color"
                        aria-label="Accent Color Picker"
                        value={formState.color_accent}
                        onChange={(e) => handleFieldChange("color_accent", e.target.value)}
                        className="w-8 h-8 rounded border border-ui-border-base cursor-pointer p-0"
                      />
                      <Input
                        name="color_accent"
                        aria-label="Accent Color"
                        value={formState.color_accent}
                        onChange={(e) => handleFieldChange("color_accent", e.target.value)}
                        className="font-mono text-xs"
                      />
                    </div>
                  </div>

                  {/* Text Color */}
                  <div>
                    <Text className="text-xs font-medium mb-1">Text Color</Text>
                    <div className="flex items-center gap-x-2">
                      <input
                        type="color"
                        aria-label="Text Color Picker"
                        value={formState.color_text}
                        onChange={(e) => handleFieldChange("color_text", e.target.value)}
                        className="w-8 h-8 rounded border border-ui-border-base cursor-pointer p-0"
                      />
                      <Input
                        name="color_text"
                        aria-label="Text Color"
                        value={formState.color_text}
                        onChange={(e) => handleFieldChange("color_text", e.target.value)}
                        className="font-mono text-xs"
                      />
                    </div>
                  </div>

                  {/* Surface Color */}
                  <div>
                    <Text className="text-xs font-medium mb-1">Surface Color</Text>
                    <div className="flex items-center gap-x-2">
                      <input
                        type="color"
                        aria-label="Surface Color Picker"
                        value={formState.color_surface}
                        onChange={(e) => handleFieldChange("color_surface", e.target.value)}
                        className="w-8 h-8 rounded border border-ui-border-base cursor-pointer p-0"
                      />
                      <Input
                        name="color_surface"
                        aria-label="Surface Color"
                        value={formState.color_surface}
                        onChange={(e) => handleFieldChange("color_surface", e.target.value)}
                        className="font-mono text-xs"
                      />
                    </div>
                  </div>

                  {/* Background Color */}
                  <div>
                    <Text className="text-xs font-medium mb-1">Background Color</Text>
                    <div className="flex items-center gap-x-2">
                      <input
                        type="color"
                        aria-label="Background Color Picker"
                        value={formState.color_background}
                        onChange={(e) => handleFieldChange("color_background", e.target.value)}
                        className="w-8 h-8 rounded border border-ui-border-base cursor-pointer p-0"
                      />
                      <Input
                        name="color_background"
                        aria-label="Background Color"
                        value={formState.color_background}
                        onChange={(e) => handleFieldChange("color_background", e.target.value)}
                        className="font-mono text-xs"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </Container>
          </div>

          {/* Live Preview Column */}
          <div className="lg:col-span-6 flex flex-col">
            <Container className="h-full flex flex-col p-0 overflow-hidden border border-ui-border-base rounded-xl shadow-elevation-card-rest bg-ui-bg-base">
              {/* Preview Header & Controls */}
              <div className="flex flex-wrap items-center justify-between p-4 border-b border-ui-border-base bg-ui-bg-subtle/40 gap-2">
                <div className="flex items-center gap-x-2">
                  <div className="flex items-center justify-center w-7 h-7 rounded bg-ui-bg-component border border-ui-border-base text-ui-fg-base shadow-sm">
                    <BuildingStorefront className="w-3.5 h-3.5 text-ui-fg-subtle" />
                  </div>
                  <div>
                    <div className="flex items-center gap-1.5">
                      <Heading level="h2" className="text-sm font-semibold text-ui-fg-base">
                        Live Storefront Preview
                      </Heading>
                      <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/10 text-emerald-600 border border-emerald-500/20">
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                        Live · Port 8000
                      </span>
                    </div>
                    <Text className="text-[11px] text-ui-fg-muted font-mono">
                      Font: {selectedFont.label.split(" ")[0]}
                    </Text>
                  </div>
                </div>

                {/* View Mode Switcher */}
                <div className="flex items-center bg-ui-bg-base border border-ui-border-base rounded-lg p-0.5 shadow-sm text-xs">
                  <button
                    type="button"
                    onClick={() => setPreviewMode("storefront")}
                    className={`px-2.5 py-1 rounded-md transition-colors ${
                      previewMode === "storefront"
                        ? "bg-ui-bg-component text-ui-fg-base font-semibold shadow-sm"
                        : "text-ui-fg-muted hover:text-ui-fg-base"
                    }`}
                  >
                    Live Website
                  </button>
                  <button
                    type="button"
                    onClick={() => setPreviewMode("component")}
                    className={`px-2.5 py-1 rounded-md transition-colors ${
                      previewMode === "component"
                        ? "bg-ui-bg-component text-ui-fg-base font-semibold shadow-sm"
                        : "text-ui-fg-muted hover:text-ui-fg-base"
                    }`}
                  >
                    Component Canvas
                  </button>
                </div>
              </div>

              {/* Simulated Browser Chrome */}
              <div className="flex items-center justify-between px-4 py-2 bg-ui-bg-subtle/80 border-b border-ui-border-base text-xs text-ui-fg-subtle">
                {/* macOS window dots */}
                <div className="flex items-center gap-1.5">
                  <span className="w-2.5 h-2.5 rounded-full bg-rose-400/80 border border-rose-500/30" />
                  <span className="w-2.5 h-2.5 rounded-full bg-amber-400/80 border border-amber-500/30" />
                  <span className="w-2.5 h-2.5 rounded-full bg-emerald-400/80 border border-emerald-500/30" />
                </div>

                {/* Address Bar with Favicon */}
                <div className="flex items-center gap-2 px-3 py-1 rounded-md bg-ui-bg-base border border-ui-border-base text-ui-fg-muted font-mono text-[11px] max-w-[320px] w-full mx-2 shadow-sm">
                  {formState.favicon_url ? (
                    <img
                      src={formState.favicon_url}
                      alt="Favicon"
                      className="w-3.5 h-3.5 object-contain shrink-0 rounded-sm"
                    />
                  ) : (
                    <LockClosedSolid className="w-3 h-3 text-emerald-600 shrink-0" />
                  )}
                  <span className="truncate flex-1 text-ui-fg-base">{STHENO_STOREFRONT_URL}</span>
                  <button
                    type="button"
                    onClick={() => setIframeKey((k) => k + 1)}
                    className="text-ui-fg-muted hover:text-ui-fg-base p-0.5 rounded transition-colors"
                    title="Refresh storefront preview"
                  >
                    <ArrowPath className="w-3 h-3" />
                  </button>
                </div>

                {/* Device toggle & external link */}
                <div className="flex items-center gap-1.5">
                  <div className="flex items-center bg-ui-bg-base border border-ui-border-base rounded-md p-0.5 shadow-sm">
                    <button
                      type="button"
                      onClick={() => setDevice("desktop")}
                      className={`p-1 rounded transition-colors ${
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
                      className={`p-1 rounded transition-colors ${
                        device === "mobile"
                          ? "bg-ui-bg-component text-ui-fg-base shadow-sm font-medium"
                          : "text-ui-fg-muted hover:text-ui-fg-base"
                      }`}
                      title="Mobile View"
                    >
                      <Phone className="w-3.5 h-3.5" />
                    </button>
                  </div>
                  <a
                    href={STHENO_STOREFRONT_URL}
                    target="_blank"
                    rel="noreferrer"
                    className="p-1 rounded text-ui-fg-muted hover:text-ui-fg-base inline-flex items-center"
                    title="Open live storefront in new tab"
                  >
                    <ArrowUpRightOnBox className="w-3.5 h-3.5" />
                  </a>
                </div>
              </div>

              {/* Viewport Area */}
              <div className="flex-1 p-4 bg-ui-bg-subtle/30 flex items-center justify-center overflow-hidden min-h-[540px]">
                {/* 1. Real Storefront Iframe Container */}
                <div
                  className={`w-full h-full flex items-center justify-center ${
                    previewMode === "storefront" ? "flex" : "hidden"
                  }`}
                >
                  {device === "desktop" ? (
                    <div className="w-full h-[540px] bg-ui-bg-base rounded-lg border border-ui-border-base overflow-hidden shadow-sm">
                      <iframe
                        ref={iframeRef}
                        key={iframeKey}
                        src={STHENO_STOREFRONT_URL}
                        title="Storefront Desktop Preview"
                        className="w-full h-full border-none"
                        loading="lazy"
                        onLoad={() => {
                          const css = `:root {
                            --brand-primary: ${formState.color_primary};
                            --brand-secondary: ${formState.color_secondary};
                            --brand-accent: ${formState.color_accent};
                            --brand-background: ${formState.color_background};
                            --brand-surface: ${formState.color_surface};
                            --brand-text: ${formState.color_text};
                            --brand-font-family: ${selectedFont.family};
                          }`
                          iframeRef.current?.contentWindow?.postMessage(
                            {
                              type: "STHENO_PREVIEW_TOKENS",
                              css,
                              store_name: formState.store_name,
                              logo_url: formState.logo_url,
                              favicon_url: formState.favicon_url,
                            },
                            "*"
                          )
                        }}
                      />
                    </div>
                  ) : (
                    <div className="w-[375px] h-[540px] bg-ui-bg-base rounded-[24px] border-[6px] border-ui-border-strong overflow-hidden shadow-elevation-modal relative">
                      <div className="absolute top-1 left-1/2 -translate-x-1/2 w-20 h-3 bg-ui-border-strong rounded-full z-10 pointer-events-none" />
                      <iframe
                        ref={iframeRef}
                        key={iframeKey}
                        src={STHENO_STOREFRONT_URL}
                        title="Storefront Mobile Preview"
                        className="w-full h-full border-none pt-4"
                        loading="lazy"
                        onLoad={() => {
                          const css = `:root {
                            --brand-primary: ${formState.color_primary};
                            --brand-secondary: ${formState.color_secondary};
                            --brand-accent: ${formState.color_accent};
                            --brand-background: ${formState.color_background};
                            --brand-surface: ${formState.color_surface};
                            --brand-text: ${formState.color_text};
                            --brand-font-family: ${selectedFont.family};
                          }`
                          iframeRef.current?.contentWindow?.postMessage(
                            {
                              type: "STHENO_PREVIEW_TOKENS",
                              css,
                              store_name: formState.store_name,
                              logo_url: formState.logo_url,
                              favicon_url: formState.favicon_url,
                            },
                            "*"
                          )
                        }}
                      />
                    </div>
                  )}
                </div>

                {/* 2. Simulated Component Preview Canvas */}
                <div
                  data-testid="branding-preview-frame"
                  className={`flex-1 rounded-lg border border-ui-border-base overflow-hidden shadow-sm transition-all duration-150 flex flex-col h-full ${
                    previewMode === "component" ? "flex" : "hidden"
                  }`}
                  style={{
                    backgroundColor: formState.color_background,
                    color: formState.color_text,
                    fontFamily: selectedFont.family,
                  }}
                >
                  {/* Store Header Preview with Logo Support */}
                  <div
                    className="px-4 py-3 border-b flex items-center justify-between transition-colors"
                    style={{
                      backgroundColor: formState.color_surface,
                      borderColor: `${formState.color_secondary}33`,
                    }}
                  >
                    <div className="flex items-center gap-2">
                      {formState.logo_url ? (
                        <img
                          src={formState.logo_url}
                          alt={formState.store_name}
                          className="h-7 max-w-[140px] object-contain"
                        />
                      ) : null}
                      <div
                        className={`font-bold text-base tracking-tight ${formState.logo_url ? "hidden sm:block text-xs font-semibold opacity-80" : ""}`}
                        data-testid="preview-store-name"
                      >
                        {formState.store_name || "Store Name"}
                      </div>
                    </div>
                    <div className="flex items-center gap-x-4 text-xs font-medium opacity-80">
                      <span>Products</span>
                      <span>Collections</span>
                      <span>About</span>
                      <span
                        className="px-2 py-1 rounded text-xs text-white"
                        style={{ backgroundColor: formState.color_primary }}
                      >
                        Cart (0)
                      </span>
                    </div>
                  </div>

                  {/* Hero Banner Preview */}
                  <div className="p-6 flex-1 flex flex-col justify-center items-center text-center">
                    <span
                      className="text-xs font-semibold px-2 py-0.5 rounded-full mb-3 tracking-wider uppercase"
                      style={{
                        backgroundColor: `${formState.color_accent}22`,
                        color: formState.color_accent,
                      }}
                    >
                      Curated Collection
                    </span>
                    <h3 className="text-xl font-bold mb-2">
                      Welcome to {formState.store_name}
                    </h3>
                    <p
                      className="text-xs max-w-sm mb-5 leading-relaxed"
                      style={{ color: formState.color_secondary }}
                    >
                      {formState.tagline || DEFAULT_BRANDING_VALUES.tagline}
                    </p>
                    <div className="flex items-center gap-x-3">
                      <button
                        type="button"
                        className="px-4 py-2 rounded text-xs font-semibold text-white shadow-sm transition-transform active:scale-95"
                        style={{ backgroundColor: formState.color_primary }}
                      >
                        Explore Catalog
                      </button>
                      <button
                        type="button"
                        className="px-4 py-2 rounded text-xs font-semibold border transition-colors"
                        style={{
                          borderColor: formState.color_secondary,
                          color: formState.color_text,
                        }}
                      >
                        View Lookbook
                      </button>
                    </div>
                  </div>

                  {/* Palette Swatches Bar */}
                  <div
                    className="p-3 border-t grid grid-cols-6 gap-2 text-center text-[10px]"
                    style={{
                      backgroundColor: formState.color_surface,
                      borderColor: `${formState.color_secondary}33`,
                    }}
                  >
                    <div>
                      <div className="h-4 rounded mb-1" style={{ backgroundColor: formState.color_primary }} />
                      <span className="opacity-70">Primary</span>
                    </div>
                    <div>
                      <div className="h-4 rounded mb-1" style={{ backgroundColor: formState.color_secondary }} />
                      <span className="opacity-70">Secondary</span>
                    </div>
                    <div>
                      <div className="h-4 rounded mb-1" style={{ backgroundColor: formState.color_accent }} />
                      <span className="opacity-70">Accent</span>
                    </div>
                    <div>
                      <div className="h-4 rounded mb-1" style={{ backgroundColor: formState.color_text }} />
                      <span className="opacity-70">Text</span>
                    </div>
                    <div>
                      <div className="h-4 rounded border mb-1" style={{ backgroundColor: formState.color_surface }} />
                      <span className="opacity-70">Surface</span>
                    </div>
                    <div>
                      <div className="h-4 rounded border mb-1" style={{ backgroundColor: formState.color_background }} />
                      <span className="opacity-70">Bg</span>
                    </div>
                  </div>
                </div>
              </div>
            </Container>
          </div>
        </div>
      </div>
    </SingleColumnPage>
  )
}
