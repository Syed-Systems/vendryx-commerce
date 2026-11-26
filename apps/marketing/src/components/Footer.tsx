import { ArrowRight } from "lucide-react"

interface FooterProps {
  onOpenOnboarding: (plan: "trial" | "free" | "pro") => void
}

export function Footer({ onOpenOnboarding }: FooterProps) {
  return (
    <footer className="bg-[#080A0C]">
      {/* Closing CTA Band */}
      <div className="border-b border-[#FFFCF8]/[0.08] py-16">
        <div className="mx-auto flex max-w-[1200px] flex-col items-start justify-between gap-6 px-4 sm:px-6 lg:flex-row lg:items-center">
          <div className="max-w-[640px]">
            <h2 className="text-[28px] font-semibold leading-[1.12] tracking-[-0.03em] text-[#FFFCF8] sm:text-[34px]">
              Start free today. Keep 100% of every order you ship.
            </h2>
            <p className="mt-2 text-[14.5px] text-[#94A3B8]">
              Try Pro free for 14 days—then stay on Free forever, or grow with Pro ($20/mo) and
              Scale ($50/mo) with bKash, SSLCommerz, Pathao, Steadfast, RedX, and your AI Store
              Assistant.
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <button
              type="button"
              onClick={() => onOpenOnboarding("trial")}
              className="inline-flex shrink-0 items-center gap-2 rounded-md bg-[#FF7A00] px-5 py-2.5 text-[14px] font-semibold text-[#080A0C] transition-colors hover:bg-[#D95700]"
            >
              Start free trial
              <ArrowRight className="h-4 w-4" />
            </button>
            <a
              href="mailto:sales@syedsystems.com"
              className="inline-flex shrink-0 items-center rounded-md border border-[#FFFCF8]/12 bg-[#1B1E24]/70 px-4 py-2.5 text-[14px] font-medium text-[#FFFCF8] hover:border-[#FFFCF8]/25"
            >
              Talk to our team
            </a>
          </div>
        </div>
      </div>

      {/* Official Vendryx Primary Dark Lockup Footer */}
      <div className="mx-auto flex max-w-[1200px] flex-col justify-between gap-8 px-4 py-10 sm:px-6 md:flex-row md:items-center">
        <div>
          <img
            src="/brand/vendryx/vendryx-primary-dark.svg"
            alt="Vendryx by Syed Systems"
            width={240}
            height={52}
            className="h-auto w-[220px] sm:w-[240px]"
          />
          <p className="mt-2.5 text-[12.5px] text-[#64748B]">
            One cart. Every channel. One commerce OS.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-x-6 gap-y-2 text-[13px] text-[#94A3B8]">
          <a href="#cockpit" className="hover:text-[#FFFCF8]">
            Dashboard
          </a>
          <a href="#social-okf" className="hover:text-[#FFFCF8]">
            AI &amp; Social
          </a>
          <a href="#operations" className="hover:text-[#FFFCF8]">
            Features
          </a>
          <a href="#engine" className="hover:text-[#FFFCF8]">
            Reliability
          </a>
          <a href="#pricing" className="hover:text-[#FFFCF8]">
            Pricing
          </a>
          <a href="mailto:support@syedsystems.com" className="hover:text-[#FFFCF8]">
            support@syedsystems.com
          </a>
        </div>
      </div>
    </footer>
  )
}
