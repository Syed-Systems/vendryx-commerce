import React from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

const SignInPrompt: React.FC = () => {
  return (
    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-5">
      <div>
        <h3 className="text-base font-semibold text-[var(--text-primary)]">
          Already have an account?
        </h3>
        <p className="text-xs text-[var(--text-secondary)] mt-1">
          Sign in to access your saved addresses and past orders.
        </p>
      </div>

      <LocalizedClientLink
        href="/account"
        className="inline-flex h-10 shrink-0 items-center justify-center rounded-full border border-[var(--accent-sapphire)] px-5 text-xs font-semibold text-[var(--accent-sapphire)] transition hover:bg-[var(--accent-sapphire)] hover:text-white lustre-btn-press"
        data-testid="sign-in-button"
      >
        Sign In
      </LocalizedClientLink>
    </div>
  )
}

export default SignInPrompt
