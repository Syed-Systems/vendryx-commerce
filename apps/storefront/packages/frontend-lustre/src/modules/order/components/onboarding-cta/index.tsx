"use client"

import React from "react"
import { resetOnboardingState } from "@stheno/storefront-core"

export type OnboardingCtaProps = {
  orderId: string
}

const OnboardingCta: React.FC<OnboardingCtaProps> = ({ orderId }) => {
  return (
    <div className="w-full rounded-[var(--radius-lg)] border border-[var(--accent-sapphire)]/30 bg-[var(--bg-surface)] p-6 text-center shadow-sm">
      <span className="text-xs font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
        Demonstration Mode
      </span>
      <h3 className="text-lg font-semibold text-[var(--text-primary)] mt-1">
        Test Order Placed Successfully
      </h3>
      <p className="mt-1 text-xs text-[var(--text-secondary)]">
        You may now conclude checkout onboarding and transition to store administration.
      </p>
      <button
        type="button"
        onClick={() => resetOnboardingState(orderId)}
        className="mt-4 inline-flex h-10 items-center justify-center rounded-full bg-[var(--accent-sapphire)] px-6 text-xs font-semibold text-white transition hover:bg-[var(--accent-sapphire-hover)] lustre-btn-press"
      >
        Complete Setup in Admin
      </button>
    </div>
  )
}

export default OnboardingCta
