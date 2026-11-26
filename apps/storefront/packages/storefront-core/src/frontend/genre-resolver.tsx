import React from "react"
import LocalizedClientLink from "../modules/common/components/localized-client-link"

export type GenreType = "appliance" | "clothing" | "tech" | "default"

/**
 * Resolves the storefront genre from the Host header or tenant settings.
 *
 * Free/Trial merchants select from pre-built genre DOMs (Appliance, Clothing, Tech).
 * Pro merchants can select from any pre-built genre or deploy a bespoke DOM.
 */
export function getGenreForHost(host?: string | null): GenreType {
  if (!host) return "default"
  const h = host.toLowerCase()
  if (h.includes("appliance")) return "appliance"
  if (h.includes("clothing") || h.includes("fashion") || h.includes("mahal")) return "clothing"
  if (h.includes("tech") || h.includes("matrix")) return "tech"
  return "default"
}

export type StoreLifecycleState = "trial_active" | "trial_expired" | "pro_active"

export function getLifecycleState(tier?: string | null, daysRemaining?: number | null): StoreLifecycleState {
  if (tier === "trial") return "trial_active"
  if (tier === "free") return "trial_expired"
  if (tier === "pro") return "pro_active"
  return "pro_active"
}

/**
 * High-aesthetic banner for 7-Day Free Trial merchants.
 */
export function TrialBanner({ daysRemaining = 7 }: { daysRemaining?: number }) {
  return (
    <aside
      aria-label="Trial status"
      className="w-full bg-gradient-to-r from-amber-600 via-amber-500 to-amber-600 text-zinc-950 px-4 py-2 text-xs font-medium tracking-wide flex items-center justify-between border-b border-amber-400/40 shadow-sm transition-all"
    >
      <div className="flex items-center gap-2 mx-auto">
        <span className="inline-block w-2 h-2 rounded-full bg-zinc-950 animate-ping" />
        <span className="font-semibold uppercase tracking-wider text-[11px]">
          7-Day Full-Access Trial Active
        </span>
        <span className="hidden sm:inline text-zinc-800">•</span>
        <span className="hidden sm:inline text-zinc-900">
          {daysRemaining} {daysRemaining === 1 ? "day" : "days"} remaining
        </span>
        <span className="hidden md:inline text-zinc-800">•</span>
        <span className="hidden md:inline text-zinc-900">
          Explore all flagship features &amp; pre-built genre DOMs
        </span>
      </div>
      <LocalizedClientLink
        href="/account"
        className="ml-auto px-3 py-1 rounded-full bg-zinc-950 text-white hover:bg-zinc-800 text-[11px] font-semibold tracking-wider uppercase transition-colors shrink-0"
      >
        Upgrade to Pro
      </LocalizedClientLink>
    </aside>
  )
}

/**
 * High-visibility banner for Expired Trial merchants (Free Tier).
 */
export function TrialExpiredBanner() {
  return (
    <aside
      aria-label="Trial expired notice"
      className="w-full bg-zinc-900 text-zinc-100 px-4 py-2.5 text-xs border-b border-zinc-700 flex items-center justify-between shadow-md"
    >
      <div className="flex items-center gap-2.5 mx-auto">
        <span className="px-2 py-0.5 rounded bg-red-500/20 text-red-400 border border-red-500/30 text-[10px] font-mono uppercase font-semibold">
          Trial Expired
        </span>
        <span className="text-zinc-300">
          Store catalog is active in read-only mode. Upgrade to Pro to reactivate checkout and custom domains.
        </span>
      </div>
      <LocalizedClientLink
        href="/account"
        className="ml-auto px-3.5 py-1 rounded-full bg-amber-400 text-zinc-950 hover:bg-amber-300 font-semibold text-[11px] tracking-wider uppercase transition-colors shrink-0"
      >
        Unlock Pro
      </LocalizedClientLink>
    </aside>
  )
}
