import LocalizedClientLink from "@modules/common/components/localized-client-link"

export default function NewsletterSection() {
  return (
    <section className="content-container py-12 sm:py-16">
      <div className="relative rounded-[var(--brand-radius-card)] bg-[color:var(--brand-surface)]/80 overflow-hidden border border-[color:var(--brand-hairline)] p-8 sm:p-16 text-center shadow-sm">
        {/* Ambient glow, tinted from the accent token so it follows the theme. */}
        <div
          aria-hidden="true"
          className="absolute -top-24 -left-24 size-72 rounded-full blur-3xl pointer-events-none"
          style={{ background: "color-mix(in oklab, var(--brand-accent) 12%, transparent)" }}
        />
        <div
          aria-hidden="true"
          className="absolute -bottom-24 -right-24 size-72 rounded-full blur-3xl pointer-events-none"
          style={{ background: "color-mix(in oklab, var(--brand-accent) 8%, transparent)" }}
        />

        <div className="relative z-10 max-w-2xl mx-auto space-y-5">
          <span className="label-eyebrow text-[color:var(--ink-faint)] font-mono">Private Client Register</span>

          <h2 className="display-section font-heading font-bold text-[color:var(--ink)]">
            Join the Atelier Salon
          </h2>

          <p className="body-copy-lg measure mx-auto text-[color:var(--ink-muted)]">
            Create an account to follow your orders, save pieces you are considering,
            and check out in fewer steps.
          </p>

          {/* The audit found this form could not submit: `action="#"`, and the
              button was `type="button"`. There is no newsletter endpoint in the
              engine and AGENTS.md §12.3 forbids the storefront from routing, so
              there is nowhere honest for it to post. It routes to the one action
              that actually exists. */}
          <div className="flex flex-col sm:flex-row gap-3 max-w-md mx-auto pt-4">
            <LocalizedClientLink
              href="/account"
              className="min-h-[44px] inline-flex flex-1 items-center justify-center px-8 py-3.5 bg-[color:var(--band-dark)] text-[color:var(--on-dark)] text-xs font-semibold uppercase tracking-[0.18em] hover:bg-[color:var(--band-dark)] active:scale-[0.98] transition-all shadow-md"
            >
              Create an account
            </LocalizedClientLink>
            <LocalizedClientLink
              href="/store"
              className="glass min-h-[44px] glass-chip inline-flex flex-1 items-center justify-center px-8 py-3.5 text-[color:var(--ink)] text-xs font-semibold uppercase tracking-[0.18em] hover:bg-[color:var(--brand-surface)]/70 transition-colors"
            >
              Browse the collection
            </LocalizedClientLink>
          </div>

          <p className="text-xs text-[color:var(--ink-faint)] font-light pt-2">
            Your details are used to manage your orders. See our{" "}
            <LocalizedClientLink
              href="/policies/privacy"
              className="underline underline-offset-4 hover:text-[color:var(--ink)]"
            >
              privacy policy
            </LocalizedClientLink>
            .
          </p>
        </div>
      </div>
    </section>
  )
}
