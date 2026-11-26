"use client"

import React from "react"
import type { ResolvedTenantContext } from "./tenant-resolver"

interface PlatformAttributionFooterProps {
  tenant: ResolvedTenantContext
}

/**
 * Core-enforced platform attribution bar.
 *
 * Security & Plan Policy:
 * - Rendered by StorefrontShell (core), NEVER inside theme code, so Free stores cannot hide or remove it.
 * - Automatically returns null on Pro and Scale white-label stores — zero Vendryx footprint in DOM.
 */
export function PlatformAttributionFooter({ tenant }: PlatformAttributionFooterProps) {
  if (!tenant.showPlatformBadge) {
    return null
  }

  return (
    <aside
      aria-label="Platform attribution"
      data-testid="powered-by-vendryx-footer"
      className="border-t border-[var(--theme-border)] bg-[var(--theme-surface)] px-6 py-4 text-[var(--theme-ink-muted)]"
    >
      <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-2 text-xs sm:flex-row">
        <div className="flex items-center gap-2.5">
          <img
            src="/brand/vendryx-mark.svg"
            alt="Vendryx"
            width={16}
            height={16}
            className="h-4 w-4 opacity-85"
          />
          <span>
            Powered by <strong className="font-semibold text-[var(--theme-ink)]">Vendryx</strong>
          </span>
        </div>
        <span className="text-[11px] text-[var(--theme-ink-muted)]">
          Free storefront hosted on <span className="font-mono">{tenant.resolvedHost}</span> · Upgrade to Pro for full white-label
        </span>
      </div>
    </aside>
  )
}
