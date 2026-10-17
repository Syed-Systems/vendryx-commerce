import React from "react"

/**
 * These are capability claims, so each one names something the engine actually
 * implements. The audit flagged the previous four: "carbon-neutral delivery… on
 * all qualifying orders worldwide", "certified atelier workshops",
 * "complimentary return documentation included with every packaged piece" and
 * "dedicated styling consultants" — none of which is derivable from anything in
 * the codebase, and all of them sat in the trust-strip position where a customer
 * takes them at face value. A merchant who cannot honour them has no way to
 * switch them off per tenant.
 */
const features = [
  {
    icon: (
      <svg className="size-5 text-brand-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 6v12m-3-2.818l.879.659c1.171.879 3.07.879 4.242 0 1.172-.879 1.172-2.303 0-3.182C13.536 12.219 12.768 12 12 12c-.725 0-1.45-.22-2.003-.659-1.106-.879-1.106-2.303 0-3.182s2.9-.879 4.006 0l.415.33M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    title: "Cash on Delivery",
    desc: "Pay the courier at your door, or choose a card at checkout.",
  },
  {
    icon: (
      <svg className="size-5 text-brand-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 10.5a3 3 0 11-6 0 3 3 0 016 0z" />
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19.5 10.5c0 7.142-7.5 11.25-7.5 11.25S4.5 17.642 4.5 10.5a7.5 7.5 0 1115 0z" />
      </svg>
    ),
    title: "Order Tracking",
    desc: "Every order carries its own status and consignment number.",
  },
  {
    icon: (
      <svg className="size-5 text-brand-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    title: "Returns & Exchanges",
    desc: "Request a return or an exchange from your order history.",
  },
  {
    icon: (
      <svg className="size-5 text-brand-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" />
      </svg>
    ),
    title: "Saved Addresses",
    desc: "Keep your delivery addresses on file for faster checkout.",
  },
]

export default function BrandFeatures() {
  return (
    <section className="border-y border-[color:var(--brand-hairline)] bg-[color:var(--brand-surface)] py-10 sm:py-12">
      <div className="content-container">
        <h2 className="sr-only">Shopping benefits</h2>
        <ul className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 sm:gap-8">
          {features.map((f) => (
            <li key={f.title} className="flex items-start gap-4">
              <div className="p-2.5 rounded-[var(--brand-radius-button)] bg-[color:var(--brand-surface)] border border-[color:var(--brand-hairline)] shadow-sm shrink-0">
                {f.icon}
              </div>
              <div className="space-y-1">
                <h3 className="label-eyebrow text-[color:var(--ink)] font-heading">
                  {f.title}
                </h3>
                <p className="body-copy text-[color:var(--ink-muted)]">
                  {f.desc}
                </p>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </section>
  )
}
