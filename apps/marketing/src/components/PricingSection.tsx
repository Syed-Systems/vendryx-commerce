import { ArrowRight, Check, Info, Sparkles, Store } from "lucide-react"
import {
  calculatePlanTermQuote,
  type CurrencyCode,
  type PlanSelection,
  type PrepaidMonths,
} from "../lib/marketing-logic"

interface PricingSectionProps {
  currency: CurrencyCode
  months: PrepaidMonths
  onCurrencyChange: (currency: CurrencyCode) => void
  onMonthsChange: (months: PrepaidMonths) => void
  onSelectPlan: (plan: PlanSelection) => void
}

const PREPAID_TERMS: { months: PrepaidMonths; label: string; tag?: string }[] = [
  { months: 1, label: "1 mo" },
  { months: 3, label: "3 mo" },
  { months: 6, label: "6 mo" },
  { months: 12, label: "12 mo", tag: "-10%" },
  { months: 24, label: "24 mo", tag: "-10%" },
]

const FREE_FEATURES = [
  "Full Vendryx Merchant Dashboard & live order desk",
  "Up to 100 active products (never deleted)",
  "Up to 3 staff accounts (you + 2 team members)",
  "yourstore.vendryx.shop storefront & fast guest checkout",
  "Cash on Delivery + WhatsApp, Messenger & IG order inbox",
  "Excel/CSV catalog upload & Bengali-ready order sheets",
  "Instant new-order alerts & 30-day sales reports",
  "0% commission on every order forever",
] as const

const PRO_FEATURES = [
  "Unlimited products & variants + up to 10 staff accounts",
  "Full sales history & analytics with no time limit",
  "Connect your own bKash, SSLCommerz, Stripe & Nagad (0% fee)",
  "1-click Pathao, Steadfast & RedX booking + COD tracking",
  "Fake-order & COD return risk check before shipping",
  "AI Store Assistant (OKF): 1,500 AI chats/mo across all 4 channels",
  "Connect your own .com domain with free SSL + storefront themes",
  "Custom team-built storefront included on 12-mo / 24-mo plans",
] as const

const SCALE_FEATURES = [
  "Everything in Pro, built for high-volume & flash-sale brands",
  "AI Store Assistant (OKF): 6,000 AI chats/mo included",
  "Up to 25 staff accounts with role-based permissions",
  "Multi-warehouse stock locations & live inventory transfers",
  "Priority WhatsApp & engineering onboarding support",
  "Dedicated custom storefront included on 12-mo / 24-mo plans",
  "High-traffic Eid & TikTok Live drop protection",
  "0% commission on every order — keep 100% of your revenue",
] as const

export function PricingSection({
  currency,
  months,
  onCurrencyChange,
  onMonthsChange,
  onSelectPlan,
}: PricingSectionProps) {
  const proQuote = calculatePlanTermQuote("pro", months, currency)
  const scaleQuote = calculatePlanTermQuote("scale", months, currency)

  return (
    <section
      id="pricing"
      className="scroll-mt-16 border-b border-[#FFFCF8]/[0.08] py-16 sm:py-20"
    >
      <div className="mx-auto max-w-[1200px] px-4 sm:px-6">
        <div className="flex flex-col justify-between gap-6 lg:flex-row lg:items-end">
          <div className="max-w-[640px]">
            <h2 className="text-[28px] font-semibold leading-[1.12] tracking-[-0.03em] text-[#FFFCF8] sm:text-[36px]">
              Simple pricing that scales with your store. Always 0% commission.
            </h2>
            <p className="mt-2.5 text-[15px] leading-[1.6] text-[#94A3B8]">
              Every new store starts with a{" "}
              <span className="font-medium text-[#FFFCF8]">14-day free Pro trial</span>. Stay on
              Free forever, or move to Pro or Scale as your order volume grows—your products and
              orders are never deleted.
            </p>
          </div>

          {/* Currency & Prepaid Duration Controls */}
          <div className="flex flex-col gap-2.5 sm:items-end">
            <div className="inline-flex self-start rounded-md border border-[#FFFCF8]/[0.10] bg-[#0F1115] p-1 sm:self-auto">
              {(["USD", "BDT"] as const).map((code) => (
                <button
                  key={code}
                  type="button"
                  onClick={() => onCurrencyChange(code)}
                  className={`rounded px-3 py-1 font-mono text-[12px] font-medium transition-colors ${
                    currency === code
                      ? "bg-[#FF7A00] text-[#080A0C] font-semibold"
                      : "text-[#94A3B8] hover:text-[#FFFCF8]"
                  }`}
                >
                  {code === "USD" ? "USD ($)" : "BDT (৳)"}
                </button>
              ))}
            </div>

            <div
              className="flex flex-wrap items-center gap-1 rounded-md border border-[#FFFCF8]/[0.10] bg-[#0F1115] p-1"
              role="group"
              aria-label="Prepaid subscription duration"
            >
              {PREPAID_TERMS.map((term) => {
                const active = months === term.months
                return (
                  <button
                    key={term.months}
                    type="button"
                    onClick={() => onMonthsChange(term.months)}
                    className={`inline-flex shrink-0 items-center gap-1 rounded px-2.5 py-1 font-mono text-[12px] transition-colors ${
                      active
                        ? "bg-[#1B1E24] text-[#FFFCF8] ring-1 ring-[#FF7A00]"
                        : "text-[#94A3B8] hover:text-[#FFFCF8]"
                    }`}
                  >
                    <span>{term.label}</span>
                    {term.tag && (
                      <span className="text-[10px] font-semibold text-[#FFAF24]">{term.tag}</span>
                    )}
                  </button>
                )
              })}
            </div>
          </div>
        </div>

        {/* Three Plan Cards: Free, Pro, Scale */}
        <div className="mt-9 grid grid-cols-1 gap-5 lg:grid-cols-3">
          {/* Free Tier */}
          <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.09] bg-[#0F1115] p-6">
            <div>
              <div className="flex items-center justify-between gap-3">
                <h3 className="text-[20px] font-semibold text-[#FFFCF8]">Free</h3>
                <span className="font-mono text-[11px] text-[#94A3B8]">No card needed</span>
              </div>

              <div className="mt-4 flex items-baseline gap-2">
                <span className="font-mono text-[38px] font-semibold tracking-[-0.03em] tabular-nums text-[#FFFCF8]">
                  {currency === "USD" ? "$0" : "৳0"}
                </span>
                <span className="text-[13px] text-[#94A3B8]">/ mo forever</span>
              </div>

              <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[12px] font-medium">
                <span className="text-[#4ADE80]">0% commission forever</span>
              </div>

              <p className="mt-2 text-[13.5px] leading-[1.55] text-[#94A3B8]">
                For new social sellers moving off spreadsheets and manual DM tracking.
              </p>

              <ul className="mt-5 space-y-2.5 border-t border-[#FFFCF8]/[0.08] pt-5">
                {FREE_FEATURES.map((feature) => (
                  <li
                    key={feature}
                    className="flex items-start gap-2.5 text-[13px] leading-[1.45] text-[#CBD5E1]"
                  >
                    <Check className="mt-0.5 h-4 w-4 shrink-0 text-[#94A3B8]" />
                    <span>{feature}</span>
                  </li>
                ))}
              </ul>
            </div>

            <div className="mt-7">
              <button
                type="button"
                onClick={() => onSelectPlan("free")}
                className="inline-flex w-full items-center justify-center gap-2 rounded-md border border-[#FFFCF8]/15 bg-[#1B1E24] px-4 py-2.5 text-[13.5px] font-semibold text-[#FFFCF8] transition-colors hover:border-[#FFFCF8]/30 hover:bg-[#232730]"
              >
                Start free forever
                <ArrowRight className="h-4 w-4" />
              </button>
            </div>
          </div>

          {/* Pro Tier (Primary Recommended) */}
          <div className="flex flex-col justify-between rounded-xl border border-[#FF7A00]/55 vendryx-card-surface p-6 shadow-[0_0_0_1px_rgba(255,122,0,0.14)]">
            <div>
              <div className="flex items-center justify-between gap-3">
                <h3 className="text-[20px] font-semibold text-[#FFFCF8]">Pro</h3>
                <span className="font-mono text-[11px] font-semibold text-[#FFAF24]">
                  {proQuote.discountPercent > 0 ? "Most popular · 10% off" : "Most popular"}
                </span>
              </div>

              <div className="mt-4 flex flex-wrap items-baseline gap-x-2 gap-y-1">
                <span className="font-mono text-[38px] font-semibold tracking-[-0.03em] tabular-nums text-[#FFFCF8]">
                  {proQuote.formattedMonthly}
                </span>
                <span className="text-[13px] text-[#94A3B8]">
                  / mo{" "}
                  {months > 1 ? `· ${proQuote.formattedTotal} (${months}m)` : "billed monthly"}
                </span>
              </div>

              <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[12px] font-medium">
                <span className="text-[#4ADE80]">0% commission</span>
                {proQuote.formattedSavings && (
                  <span className="text-[#FFAF24]">Save {proQuote.formattedSavings}</span>
                )}
              </div>

              <p className="mt-2 text-[13.5px] leading-[1.55] text-[#CBD5E1]">
                For growing D2C &amp; F-commerce brands automating payments, couriers, and AI chat.
              </p>

              <ul className="mt-5 space-y-2.5 border-t border-[#FFFCF8]/[0.08] pt-5">
                {PRO_FEATURES.map((feature) => (
                  <li
                    key={feature}
                    className="flex items-start gap-2.5 text-[13px] leading-[1.45] text-[#FFFCF8]"
                  >
                    <Check className="mt-0.5 h-4 w-4 shrink-0 text-[#FF7A00]" />
                    <span>{feature}</span>
                  </li>
                ))}
              </ul>
            </div>

            <div className="mt-7">
              <button
                type="button"
                onClick={() => onSelectPlan("pro")}
                className="inline-flex w-full items-center justify-center gap-2 rounded-md bg-[#FF7A00] px-4 py-2.5 text-[13.5px] font-semibold text-[#080A0C] transition-colors hover:bg-[#D95700]"
              >
                Start 14-day trial · Pro ({proQuote.formattedTotal})
                <ArrowRight className="h-4 w-4" />
              </button>
            </div>
          </div>

          {/* Scale Tier */}
          <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.12] bg-[#0F1115] p-6">
            <div>
              <div className="flex items-center justify-between gap-3">
                <h3 className="text-[20px] font-semibold text-[#FFFCF8]">Scale</h3>
                <span className="font-mono text-[11px] text-[#94A3B8]">
                  {scaleQuote.discountPercent > 0 ? "High volume · 10% off" : "High volume"}
                </span>
              </div>

              <div className="mt-4 flex flex-wrap items-baseline gap-x-2 gap-y-1">
                <span className="font-mono text-[38px] font-semibold tracking-[-0.03em] tabular-nums text-[#FFFCF8]">
                  {scaleQuote.formattedMonthly}
                </span>
                <span className="text-[13px] text-[#94A3B8]">
                  / mo{" "}
                  {months > 1 ? `· ${scaleQuote.formattedTotal} (${months}m)` : "billed monthly"}
                </span>
              </div>

              <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[12px] font-medium">
                <span className="text-[#4ADE80]">0% commission</span>
                {scaleQuote.formattedSavings && (
                  <span className="text-[#FFAF24]">Save {scaleQuote.formattedSavings}</span>
                )}
              </div>

              <p className="mt-2 text-[13.5px] leading-[1.55] text-[#94A3B8]">
                For busy multi-location brands, Eid drops, and TikTok Live sellers.
              </p>

              <ul className="mt-5 space-y-2.5 border-t border-[#FFFCF8]/[0.08] pt-5">
                {SCALE_FEATURES.map((feature) => (
                  <li
                    key={feature}
                    className="flex items-start gap-2.5 text-[13px] leading-[1.45] text-[#CBD5E1]"
                  >
                    <Check className="mt-0.5 h-4 w-4 shrink-0 text-[#FFAF24]" />
                    <span>{feature}</span>
                  </li>
                ))}
              </ul>
            </div>

            <div className="mt-7">
              <button
                type="button"
                onClick={() => onSelectPlan("scale")}
                className="inline-flex w-full items-center justify-center gap-2 rounded-md border border-[#FFFCF8]/20 bg-[#1B1E24] px-4 py-2.5 text-[13.5px] font-semibold text-[#FFFCF8] transition-colors hover:border-[#FF7A00]/60 hover:bg-[#232730]"
              >
                Start 14-day trial · Scale ({scaleQuote.formattedTotal})
                <ArrowRight className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>

        {/* Clear Add-on & Custom Storefront Terms Strip */}
        <div className="mt-5 grid grid-cols-1 gap-3 md:grid-cols-3">
          <div className="flex items-start gap-3 rounded-lg border border-[#FFFCF8]/[0.08] bg-[#0F1115] p-4">
            <Sparkles className="mt-0.5 h-4 w-4 shrink-0 text-[#FF7A00]" />
            <div className="text-[12.5px] leading-[1.5] text-[#94A3B8]">
              <span className="font-semibold text-[#FFFCF8]">Extra AI chats anytime: </span>
              Need more during a busy Eid or Live drop? Add 1,500 extra AI Store Assistant chats
              for{" "}
              <span className="font-mono font-medium text-[#FFFCF8]">
                {currency === "USD" ? "$10" : "৳1,100"}
              </span>{" "}
              — no surprise overage bills.
            </div>
          </div>

          <div className="flex items-start gap-3 rounded-lg border border-[#FFFCF8]/[0.08] bg-[#0F1115] p-4">
            <Store className="mt-0.5 h-4 w-4 shrink-0 text-[#FFAF24]" />
            <div className="text-[12.5px] leading-[1.5] text-[#94A3B8]">
              <span className="font-semibold text-[#FFFCF8]">Custom storefront setup: </span>
              Included free when you prepay 12 or 24 months on Pro or Scale, or add a one-time
              team setup for{" "}
              <span className="font-mono font-medium text-[#FFFCF8]">
                {currency === "USD" ? "$99" : "৳10,000"}
              </span>{" "}
              on monthly billing.
            </div>
          </div>

          <div className="flex items-start gap-3 rounded-lg border border-[#FFFCF8]/[0.08] bg-[#0F1115] p-4">
            <Info className="mt-0.5 h-4 w-4 shrink-0 text-[#4ADE80]" />
            <div className="text-[12.5px] leading-[1.5] text-[#94A3B8]">
              <span className="font-semibold text-[#FFFCF8]">Local &amp; global billing: </span>
              {currency === "BDT"
                ? "Pay easily in BDT via bKash, Nagad, Rocket, or Bank Transfer. Switch to USD anytime for international cards."
                : "Pay in USD via Stripe or switch to BDT for local Bangladesh pricing via bKash, Nagad, or Bank Transfer."}
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
