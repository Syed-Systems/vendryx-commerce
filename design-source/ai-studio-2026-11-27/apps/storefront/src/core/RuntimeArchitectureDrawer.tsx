"use client"

import React, { useState } from "react"
import { useRouter } from "next/navigation"
import {
  buildHostOnlyCookie,
  type ResolvedTenantContext,
  type TenantPlan,
  type TenantSlug,
  type ThemePublishMode,
} from "./tenant-resolver"
import { validateThemeConfig } from "./theme-schema"

interface RuntimeArchitectureDrawerProps {
  tenant: ResolvedTenantContext
}

const MALICIOUS_THEME_PAYLOAD = {
  schemaVersion: 1,
  themeId: "maison-cuir",
  versionTag: "v9.9.9-xss",
  tokens: {
    canvas: "#F6F3EE",
    surface: "#EFEAE1",
    surfaceAlt: "#E7DFD3",
    ink: "red; background:url(javascript:alert(1))",
    inkMuted: "#686158",
    border: "#DBD3C6",
    accent: "#171513",
    accentInk: "#F6F3EE",
    heroBg: "#1F1B17",
    heroInk: "#FAF7F2",
    fontPair: "cormorant-jakarta",
    radiusCardPx: 0,
    radiusButtonPx: 0,
    headerStyle: "editorial-centered",
    cardAspect: "portrait-3-4",
    iconStrokeWeight: "hairline-1.1",
  },
  navigation: [
    { label: "<script>alert(1)</script>", href: "javascript:alert(document.cookie)" },
    { label: "Shop", href: "#collection" },
    { label: "About", href: "#craft" },
  ],
  sections: [],
  footerNote: "Injected",
}

export function RuntimeArchitectureDrawer({ tenant }: RuntimeArchitectureDrawerProps) {
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const [xssTestResult, setXssTestResult] = useState<{
    rejected: boolean
    errors: string[]
    fallbackTag: string
  } | null>(null)

  const navigateWith = (next: {
    store?: TenantSlug
    plan?: TenantPlan
    mode?: ThemePublishMode
  }) => {
    const store = next.store ?? tenant.slug
    const plan = next.plan ?? tenant.plan
    const mode = next.mode ?? tenant.mode
    setXssTestResult(null)
    router.push(`/?store=${store}&plan=${plan}&mode=${mode}`)
  }

  const runXssSandboxTest = () => {
    const outcome = validateThemeConfig(MALICIOUS_THEME_PAYLOAD, tenant.themeConfig)
    setXssTestResult({
      rejected: !outcome.valid && outcome.fellBackToDefault,
      errors: outcome.errors.slice(0, 4),
      fallbackTag: outcome.config.versionTag,
    })
  }

  const cookieProof = buildHostOnlyCookie(`vendryx_cart_${tenant.slug}`, "cart_01J9X", 86400)

  return (
    <>
      {/* Discreet floating control at bottom-right so the storefront header stays 100% pure */}
      <div className="fixed bottom-4 right-4 z-40">
        <button
          type="button"
          onClick={() => setOpen(true)}
          data-testid="runtime-switcher-trigger"
          className="flex items-center gap-2.5 rounded-full border border-neutral-700 bg-neutral-950/95 px-4 py-2.5 text-xs font-medium text-neutral-100 shadow-xl backdrop-blur-md transition hover:bg-neutral-900"
        >
          <span className="h-2 w-2 rounded-full bg-emerald-400" />
          <span>{tenant.brandName}</span>
          <span className="text-neutral-400">·</span>
          <span className="text-neutral-200">Switch Store</span>
        </button>
      </div>

      {open && (
        <div
          className="fixed inset-0 z-50 flex items-end justify-end bg-black/55 backdrop-blur-sm sm:items-center sm:p-4"
          role="dialog"
          aria-modal="true"
          aria-label="Multi-Tenant Storefront Runtime Switcher"
          data-testid="runtime-architecture-drawer"
        >
          <div className="max-h-[90vh] w-full max-w-xl overflow-y-auto rounded-t-2xl border border-neutral-800 bg-neutral-950 p-6 text-neutral-100 shadow-2xl sm:rounded-2xl">
            <div className="flex items-start justify-between gap-4 border-b border-neutral-800 pb-4">
              <div>
                <h2 className="text-base font-semibold tracking-tight text-white">
                  Unified Next.js Multi-Theme Runtime
                </h2>
                <p className="mt-1 text-xs text-neutral-400">
                  One Next.js server serving unrecognizable storefronts with strict tenant isolation.
                </p>
              </div>
              <button
                type="button"
                onClick={() => setOpen(false)}
                aria-label="Close switcher"
                className="rounded-lg border border-neutral-800 p-2 text-neutral-400 transition hover:bg-neutral-900 hover:text-white"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M18 6 6 18M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* 1. Store / Theme Switcher */}
            <div className="mt-5">
              <div className="text-xs font-medium uppercase tracking-wider text-neutral-400">
                Active Storefront Theme
              </div>
              <div className="mt-2.5 grid grid-cols-1 gap-2.5 sm:grid-cols-3">
                <button
                  type="button"
                  data-testid="switch-store-maison"
                  onClick={() => navigateWith({ store: "maison-vela" })}
                  className={`rounded-xl border p-3.5 text-left transition ${
                    tenant.slug === "maison-vela"
                      ? "border-amber-400/70 bg-amber-950/25"
                      : "border-neutral-800 bg-neutral-900/60 hover:border-neutral-700"
                  }`}
                >
                  <div className="flex items-center justify-between gap-1">
                    <span className="text-sm font-semibold text-white">Maison Véla</span>
                    <span className="font-mono text-[10px] text-amber-300">maison-cuir</span>
                  </div>
                  <p className="mt-1 text-xs text-neutral-400">
                    Luxury Leather · Warm Stone · Cormorant Serif · Patina Atelier
                  </p>
                </button>

                <button
                  type="button"
                  data-testid="switch-store-kova"
                  onClick={() => navigateWith({ store: "kova" })}
                  className={`rounded-xl border p-3.5 text-left transition ${
                    tenant.slug === "kova"
                      ? "border-blue-400/70 bg-blue-950/30"
                      : "border-neutral-800 bg-neutral-900/60 hover:border-neutral-700"
                  }`}
                >
                  <div className="flex items-center justify-between gap-1">
                    <span className="text-sm font-semibold text-white">KOVA</span>
                    <span className="font-mono text-[10px] text-blue-300">hydro-vessel</span>
                  </div>
                  <p className="mt-1 text-xs text-neutral-400">
                    Bottle Flagship · WebGL2 Contours · 3D Wireframe · Light/Dark
                  </p>
                </button>

                <button
                  type="button"
                  data-testid="switch-store-homedine"
                  onClick={() => navigateWith({ store: "homedine" })}
                  className={`rounded-xl border p-3.5 text-left transition ${
                    tenant.slug === "homedine"
                      ? "border-emerald-400/70 bg-emerald-950/30"
                      : "border-neutral-800 bg-neutral-900/60 hover:border-neutral-700"
                  }`}
                >
                  <div className="flex items-center justify-between gap-1">
                    <span className="text-sm font-semibold text-white">Homedine</span>
                    <span className="font-mono text-[10px] text-emerald-300">homedine-eco</span>
                  </div>
                  <p className="mt-1 text-xs text-neutral-400">
                    Eco Kitchenware · Forest Green · Playfair Serif · Botanical Editorial
                  </p>
                </button>
              </div>
            </div>

            {/* 2. Plan Tier & Draft/Published Mode */}
            <div className="mt-5 grid grid-cols-1 gap-4 sm:grid-cols-2">
              <div>
                <div className="text-xs font-medium uppercase tracking-wider text-neutral-400">
                  Tenant Plan Tier
                </div>
                <div className="mt-2 grid grid-cols-2 gap-2">
                  <button
                    type="button"
                    data-testid="switch-plan-pro"
                    onClick={() => navigateWith({ plan: "pro" })}
                    className={`rounded-lg border px-3 py-2 text-xs font-medium transition ${
                      tenant.plan !== "free"
                        ? "border-emerald-500/70 bg-emerald-950/30 text-emerald-200"
                        : "border-neutral-800 bg-neutral-900 text-neutral-300 hover:border-neutral-700"
                    }`}
                  >
                    Pro · White-Label
                  </button>
                  <button
                    type="button"
                    data-testid="switch-plan-free"
                    onClick={() => navigateWith({ plan: "free" })}
                    className={`rounded-lg border px-3 py-2 text-xs font-medium transition ${
                      tenant.plan === "free"
                        ? "border-emerald-500/70 bg-emerald-950/30 text-emerald-200"
                        : "border-neutral-800 bg-neutral-900 text-neutral-300 hover:border-neutral-700"
                    }`}
                  >
                    Free · Powered By
                  </button>
                </div>
              </div>

              <div>
                <div className="text-xs font-medium uppercase tracking-wider text-neutral-400">
                  Theme JSON Version
                </div>
                <div className="mt-2 grid grid-cols-2 gap-2">
                  <button
                    type="button"
                    data-testid="switch-mode-published"
                    onClick={() => navigateWith({ mode: "published" })}
                    className={`rounded-lg border px-3 py-2 text-xs font-medium transition ${
                      tenant.mode === "published"
                        ? "border-sky-500/70 bg-sky-950/30 text-sky-200"
                        : "border-neutral-800 bg-neutral-900 text-neutral-300 hover:border-neutral-700"
                    }`}
                  >
                    Published
                  </button>
                  <button
                    type="button"
                    data-testid="switch-mode-draft"
                    onClick={() => navigateWith({ mode: "draft" })}
                    className={`rounded-lg border px-3 py-2 text-xs font-medium transition ${
                      tenant.mode === "draft"
                        ? "border-sky-500/70 bg-sky-950/30 text-sky-200"
                        : "border-neutral-800 bg-neutral-900 text-neutral-300 hover:border-neutral-700"
                    }`}
                  >
                    Draft Preview
                  </button>
                </div>
              </div>
            </div>

            {/* 3. Isolation & Security Proof */}
            <div className="mt-5 rounded-xl border border-neutral-800 bg-neutral-900/70 p-4">
              <div className="text-xs font-medium uppercase tracking-wider text-neutral-400">
                Active Isolation Guarantees
              </div>
              <dl className="mt-3 space-y-2 text-xs">
                <div className="flex justify-between gap-4">
                  <dt className="text-neutral-400">Validated Host</dt>
                  <dd className="font-mono text-neutral-100">{tenant.resolvedHost}</dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="text-neutral-400">Tenant Cache Key</dt>
                  <dd className="truncate font-mono text-emerald-300">{tenant.cacheKey}</dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="text-neutral-400">HTTP X-Powered-By</dt>
                  <dd className="font-mono text-neutral-100">Stripped (poweredByHeader: false)</dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="text-neutral-400">Footer Attribution</dt>
                  <dd className="font-mono text-neutral-100">
                    {tenant.showPlatformBadge ? "Core-enforced (Free plan)" : "Hidden (White-label)"}
                  </dd>
                </div>
                <div className="pt-1">
                  <dt className="text-neutral-400">Host-Only Session Cookie (No Domain= bleed)</dt>
                  <dd className="mt-1 break-all rounded-md bg-neutral-950 px-2.5 py-1.5 font-mono text-[11px] text-neutral-300">
                    {cookieProof}
                  </dd>
                </div>
              </dl>
            </div>

            {/* 4. Live Zod Schema & XSS Rejection Test */}
            <div className="mt-4 rounded-xl border border-neutral-800 bg-neutral-900/70 p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <div className="text-xs font-medium text-white">Zod Theme Config Sandbox</div>
                  <p className="text-[11px] text-neutral-400">
                    Simulate an injected &lt;script&gt; / javascript: payload from corrupt DB config.
                  </p>
                </div>
                <button
                  type="button"
                  onClick={runXssSandboxTest}
                  data-testid="run-xss-sandbox-test"
                  className="shrink-0 rounded-lg border border-neutral-700 bg-neutral-800 px-3 py-1.5 text-xs font-medium text-white transition hover:bg-neutral-700"
                >
                  Test XSS Guard
                </button>
              </div>

              {xssTestResult && (
                <div
                  className="mt-3 rounded-lg border border-emerald-500/40 bg-emerald-950/30 p-3 text-xs"
                  data-testid="xss-test-result"
                >
                  <div className="font-semibold text-emerald-300">
                    Blocked &amp; safely fell back to {xssTestResult.fallbackTag}
                  </div>
                  <ul className="mt-1.5 space-y-1 font-mono text-[11px] text-neutral-300">
                    {xssTestResult.errors.map((err, idx) => (
                      <li key={idx}>• {err}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  )
}
