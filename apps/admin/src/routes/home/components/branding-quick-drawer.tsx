import { Photo, Trash } from "@medusajs/icons"
import { Button, Drawer, Heading, Input, Label, Select, Text, toast } from "@medusajs/ui"
import React, { useEffect, useRef, useState } from "react"
import { useBranding, useUpsertBranding } from "../../../hooks/api/branding"
import {
  BrandingFormState,
  CURATED_FONTS,
  DEFAULT_BRANDING_VALUES,
} from "../../branding/branding-view"

interface BrandingQuickDrawerProps {
  open: boolean
  onClose: () => void
}

export const BrandingQuickDrawer: React.FC<BrandingQuickDrawerProps> = ({ open, onClose }) => {
  const { data: brandingData } = useBranding()
  const { mutateAsync: upsertBranding, isPending: isSaving } = useUpsertBranding()

  const [formState, setFormState] = useState<BrandingFormState>(DEFAULT_BRANDING_VALUES)
  const logoInputRef = useRef<HTMLInputElement>(null)

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
  }, [brandingData, open])

  const handleFieldChange = (field: keyof BrandingFormState, value: string) => {
    setFormState((prev) => ({ ...prev, [field]: value }))
  }

  const handleLogoFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    if (file.size > 5 * 1024 * 1024) {
      toast.error("Logo file exceeds 5MB limit")
      return
    }
    const reader = new FileReader()
    reader.onload = () => {
      const dataUrl = reader.result as string
      setFormState((prev) => ({ ...prev, logo_url: dataUrl }))
      toast.success("Logo uploaded to quick studio")
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
      toast.success("Storefront branding updated in live theme")
      onClose()
    } catch (err: any) {
      toast.error(err.message || "Failed to update branding")
    }
  }

  const handleReset = () => {
    setFormState({ ...DEFAULT_BRANDING_VALUES })
  }

  return (
    <Drawer open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <Drawer.Content className="flex flex-col max-w-[480px]">
        <Drawer.Header className="border-b border-ui-border-base p-6 bg-ui-bg-subtle/40">
          <div className="flex flex-col gap-y-1">
            <Drawer.Title asChild>
              <Heading level="h2" className="text-lg font-bold text-ui-fg-base">
                Quick Brand Studio
              </Heading>
            </Drawer.Title>
            <Drawer.Description className="text-xs text-ui-fg-subtle">
              Tweak logo, typography and live CSS tokens with zero rebuilds.
            </Drawer.Description>
          </div>
        </Drawer.Header>

        <form onSubmit={handleSave} className="flex flex-1 flex-col overflow-hidden">
          <Drawer.Body className="flex flex-col gap-y-5 overflow-y-auto p-6 flex-1">
            {/* Store Title */}
            <div className="flex flex-col gap-y-1.5">
              <Label size="small" weight="plus" className="text-ui-fg-base">
                Storefront Name
              </Label>
              <Input
                value={formState.store_name}
                onChange={(e) => handleFieldChange("store_name", e.target.value)}
                placeholder="e.g. Stheno Store"
                className="w-full text-sm"
              />
              <Text size="xsmall" className="text-ui-fg-muted">
                Emitted to customer browser tab, header title, and metadata.
              </Text>
            </div>

            {/* Store Logo Section */}
            <div className="flex flex-col gap-y-2">
              <div className="flex items-center justify-between">
                <Label size="small" weight="plus" className="text-ui-fg-base">
                  Store Logo
                </Label>
                {formState.logo_url && (
                  <button
                    type="button"
                    onClick={() => setFormState((p) => ({ ...p, logo_url: "" }))}
                    className="text-[11px] text-rose-600 hover:underline flex items-center gap-1"
                  >
                    <Trash className="w-3 h-3" />
                    Remove
                  </button>
                )}
              </div>

              {formState.logo_url ? (
                <div className="flex items-center gap-3 p-2.5 rounded-xl border border-ui-border-base bg-ui-bg-subtle/40">
                  <div className="w-16 h-10 rounded border border-ui-border-base bg-ui-bg-base flex items-center justify-center p-1 overflow-hidden shadow-sm">
                    <img
                      src={formState.logo_url}
                      alt="Logo"
                      className="max-h-full max-w-full object-contain"
                    />
                  </div>
                  <div className="flex-1 min-w-0">
                    <Text size="small" weight="plus" className="text-ui-fg-base truncate">
                      Active Logo
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
                <Button
                  type="button"
                  variant="secondary"
                  size="small"
                  className="w-full justify-center gap-2"
                  onClick={() => logoInputRef.current?.click()}
                >
                  <Photo className="w-3.5 h-3.5" />
                  Upload Store Logo
                </Button>
              )}

              <input
                ref={logoInputRef}
                type="file"
                accept="image/png,image/svg+xml,image/jpeg,image/webp"
                className="hidden"
                onChange={handleLogoFileChange}
              />
            </div>

            {/* Typography Selector */}
            <div className="flex flex-col gap-y-1.5">
              <Label size="small" weight="plus" className="text-ui-fg-base">
                Curated Typography
              </Label>
              <Select
                value={formState.font_key}
                onValueChange={(val) => handleFieldChange("font_key", val)}
              >
                <Select.Trigger className="w-full text-sm">
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
              <Text size="xsmall" className="text-ui-fg-muted">
                Self-hosted WOFF2 font with zero third-party CDN tracking.
              </Text>
            </div>

            {/* Color Palette Grid */}
            <div className="flex flex-col gap-y-3 pt-2">
              <Label size="small" weight="plus" className="text-ui-fg-base">
                Core Colour Tokens
              </Label>

              {/* Primary Color */}
              <div className="flex items-center justify-between p-3 rounded-xl border border-ui-border-base bg-ui-bg-subtle/30">
                <div className="flex flex-col">
                  <span className="text-xs font-semibold text-ui-fg-base">Primary Brand</span>
                  <span className="text-[11px] text-ui-fg-muted">Headers, main CTA buttons</span>
                </div>
                <div className="flex items-center gap-x-2">
                  <input
                    type="color"
                    value={formState.color_primary}
                    onChange={(e) => handleFieldChange("color_primary", e.target.value)}
                    className="w-7 h-7 rounded border border-ui-border-base cursor-pointer p-0 shrink-0 bg-transparent"
                  />
                  <Input
                    value={formState.color_primary}
                    onChange={(e) => handleFieldChange("color_primary", e.target.value)}
                    className="w-24 font-mono text-xs uppercase"
                  />
                </div>
              </div>

              {/* Accent Color */}
              <div className="flex items-center justify-between p-3 rounded-xl border border-ui-border-base bg-ui-bg-subtle/30">
                <div className="flex flex-col">
                  <span className="text-xs font-semibold text-ui-fg-base">Accent / Highlight</span>
                  <span className="text-[11px] text-ui-fg-muted">Badges, links, interactive focus</span>
                </div>
                <div className="flex items-center gap-x-2">
                  <input
                    type="color"
                    value={formState.color_accent}
                    onChange={(e) => handleFieldChange("color_accent", e.target.value)}
                    className="w-7 h-7 rounded border border-ui-border-base cursor-pointer p-0 shrink-0 bg-transparent"
                  />
                  <Input
                    value={formState.color_accent}
                    onChange={(e) => handleFieldChange("color_accent", e.target.value)}
                    className="w-24 font-mono text-xs uppercase"
                  />
                </div>
              </div>

              {/* Background Color */}
              <div className="flex items-center justify-between p-3 rounded-xl border border-ui-border-base bg-ui-bg-subtle/30">
                <div className="flex flex-col">
                  <span className="text-xs font-semibold text-ui-fg-base">Canvas Background</span>
                  <span className="text-[11px] text-ui-fg-muted">Storefront body background</span>
                </div>
                <div className="flex items-center gap-x-2">
                  <input
                    type="color"
                    value={formState.color_background}
                    onChange={(e) => handleFieldChange("color_background", e.target.value)}
                    className="w-7 h-7 rounded border border-ui-border-base cursor-pointer p-0 shrink-0 bg-transparent"
                  />
                  <Input
                    value={formState.color_background}
                    onChange={(e) => handleFieldChange("color_background", e.target.value)}
                    className="w-24 font-mono text-xs uppercase"
                  />
                </div>
              </div>
            </div>
          </Drawer.Body>

          <Drawer.Footer className="border-t border-ui-border-base p-4 flex items-center justify-between bg-ui-bg-subtle/20">
            <Button
              type="button"
              variant="secondary"
              size="small"
              onClick={handleReset}
              disabled={isSaving}
            >
              Reset Defaults
            </Button>
            <div className="flex items-center gap-x-2">
              <Button type="button" variant="secondary" size="small" onClick={onClose}>
                Cancel
              </Button>
              <Button type="submit" variant="primary" size="small" isLoading={isSaving}>
                Apply & Save
              </Button>
            </div>
          </Drawer.Footer>
        </form>
      </Drawer.Content>
    </Drawer>
  )
}
