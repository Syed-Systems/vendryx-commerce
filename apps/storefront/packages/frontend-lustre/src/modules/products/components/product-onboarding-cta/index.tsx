import React from "react"

export default function ProductOnboardingCta({
  isOnboarding,
}: {
  isOnboarding: boolean
}) {
  if (!isOnboarding) {
    return null
  }

  return (
    <div className="mx-auto max-w-3xl rounded-[var(--radius-lg)] border border-[var(--accent-sapphire)]/30 bg-[var(--bg-surface)] p-8 text-center shadow-sm">
      <div className="flex flex-col items-center gap-y-3">
        <span className="text-xs font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
          Demo Onboarding
        </span>
        <h3 className="text-xl font-semibold text-[var(--text-primary)]">
          Your demonstration product is active
        </h3>
        <p className="max-w-md text-xs text-[var(--text-secondary)]">
          Continue managing inventory, currencies, and sales channels in the administrative workspace.
        </p>
        <a
          href="http://localhost:7001/a/orders?onboarding_step=create_order_nextjs"
          className="mt-2 inline-flex h-10 items-center justify-center rounded-full bg-[var(--accent-sapphire)] px-6 text-xs font-semibold text-white lustre-btn-press transition hover:bg-[var(--accent-sapphire-hover)]"
        >
          Return to Admin Workspace
        </a>
      </div>
    </div>
  )
}
