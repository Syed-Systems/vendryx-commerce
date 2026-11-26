import { CheckCircle2, X } from "lucide-react"
import { useEffect, useState, type FormEvent } from "react"
import {
  calculatePlanTermQuote,
  validateOnboardingForm,
  type CurrencyCode,
  type PlanSelection,
  type PrepaidMonths,
  type SocialChannel,
} from "../lib/marketing-logic"

interface OnboardingModalProps {
  open: boolean
  plan: PlanSelection
  currency: CurrencyCode
  months: PrepaidMonths
  onClose: () => void
}

export function OnboardingModal({
  open,
  plan,
  currency,
  months,
  onClose,
}: OnboardingModalProps) {
  const [storeName, setStoreName] = useState("")
  const [email, setEmail] = useState("")
  const [channel, setChannel] = useState<SocialChannel | "storefront">("whatsapp")
  const [errors, setErrors] = useState<{ storeName?: string; email?: string }>({})
  const [submitted, setSubmitted] = useState(false)

  useEffect(() => {
    if (!open) return
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose()
    }
    window.addEventListener("keydown", handleKey)
    return () => window.removeEventListener("keydown", handleKey)
  }, [open, onClose])

  if (!open) return null

  const paidTier = plan === "scale" ? "scale" : "pro"
  const quote = calculatePlanTermQuote(paidTier, months, currency)

  function handleSubmit(e: FormEvent) {
    e.preventDefault()
    const result = validateOnboardingForm({ storeName, email, channel })
    if (!result.valid) {
      setErrors(result.errors)
      return
    }
    setErrors({})
    setSubmitted(true)
  }

  function handleResetAndClose() {
    setSubmitted(false)
    setErrors({})
    setStoreName("")
    setEmail("")
    onClose()
  }

  const planHeading =
    plan === "scale"
      ? `Scale · ${quote.formattedMonthly}/mo (${months}m · ${quote.formattedTotal})`
      : plan === "pro"
        ? `Pro · ${quote.formattedMonthly}/mo (${months}m · ${quote.formattedTotal})`
        : plan === "free"
          ? "Free Plan + 14-Day Pro Trial"
          : "14-Day Full Pro Trial (No Card Required)"

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="onboarding-modal-title"
    >
      <div className="relative w-full max-w-[460px] rounded-xl border border-[#FFFCF8]/[0.12] bg-[#0F1115] p-6 shadow-2xl">
        <button
          type="button"
          onClick={handleResetAndClose}
          className="absolute right-4 top-4 inline-flex h-8 w-8 items-center justify-center rounded-md text-[#94A3B8] hover:bg-[#1B1E24] hover:text-[#FFFCF8]"
          aria-label="Close modal"
        >
          <X className="h-4 w-4" />
        </button>

        {submitted ? (
          <div className="py-3">
            <CheckCircle2 className="h-9 w-9 text-[#4ADE80]" />
            <h3
              id="onboarding-modal-title"
              className="mt-3 text-[20px] font-semibold text-[#FFFCF8]"
            >
              Store request queued for {storeName.trim()}
            </h3>
            <p className="mt-2 text-[14px] leading-[1.55] text-[#94A3B8]">
              We sent your verification link to{" "}
              <span className="font-mono text-[#FFFCF8]">{email.trim()}</span>. Your 14-day Pro
              trial starts only after you submit your store request details, and you can switch to
              Free or Pro anytime with zero data loss.
            </p>
            <button
              type="button"
              onClick={handleResetAndClose}
              className="mt-6 w-full rounded-md bg-[#FF7A00] px-4 py-2.5 text-[14px] font-semibold text-[#080A0C] hover:bg-[#D95700]"
            >
              Back to Vendryx
            </button>
          </div>
        ) : (
          <>
            <div className="flex items-center gap-2.5">
              <img
                src="/brand/vendryx/vendryx-mark.svg"
                alt=""
                width={22}
                height={22}
                className="h-[22px] w-[22px] rounded-[5px]"
              />
              <span className="font-mono text-[11px] uppercase tracking-[0.12em] text-[#FFAF24]">
                {planHeading}
              </span>
            </div>

            <h3
              id="onboarding-modal-title"
              className="mt-2.5 text-[22px] font-semibold tracking-[-0.02em] text-[#FFFCF8]"
            >
              Launch your Vendryx store
            </h3>
            <p className="mt-1.5 text-[13.5px] leading-[1.5] text-[#94A3B8]">
              Get your Vendryx dashboard, branded online store, and WhatsApp order inbox ready in
              minutes. Zero commission on every sale.
            </p>

            <form onSubmit={handleSubmit} className="mt-5 space-y-4" noValidate>
              <div>
                <label
                  htmlFor="vendryx-store-name"
                  className="block text-[12px] font-medium text-[#CBD5E1]"
                >
                  Store or brand name
                </label>
                <input
                  id="vendryx-store-name"
                  type="text"
                  value={storeName}
                  onChange={(e) => setStoreName(e.target.value)}
                  placeholder="e.g. Dhaka Loom Co."
                  className="mt-1.5 w-full rounded-md border border-[#FFFCF8]/12 bg-[#080A0C] px-3 py-2 text-[14px] text-[#FFFCF8] placeholder:text-[#64748B] focus:border-[#FF7A00] focus:outline-none"
                />
                {errors.storeName && (
                  <p className="mt-1 text-[12px] text-[#F87171]" role="alert">
                    {errors.storeName}
                  </p>
                )}
              </div>

              <div>
                <label
                  htmlFor="vendryx-work-email"
                  className="block text-[12px] font-medium text-[#CBD5E1]"
                >
                  Work email
                </label>
                <input
                  id="vendryx-work-email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="owner@yourbrand.com"
                  className="mt-1.5 w-full rounded-md border border-[#FFFCF8]/12 bg-[#080A0C] px-3 py-2 text-[14px] text-[#FFFCF8] placeholder:text-[#64748B] focus:border-[#FF7A00] focus:outline-none"
                />
                {errors.email && (
                  <p className="mt-1 text-[12px] text-[#F87171]" role="alert">
                    {errors.email}
                  </p>
                )}
              </div>

              <div>
                <label
                  htmlFor="vendryx-primary-channel"
                  className="block text-[12px] font-medium text-[#CBD5E1]"
                >
                  Primary order channel today
                </label>
                <select
                  id="vendryx-primary-channel"
                  value={channel}
                  onChange={(e) =>
                    setChannel(e.target.value as SocialChannel | "storefront")
                  }
                  className="mt-1.5 w-full rounded-md border border-[#FFFCF8]/12 bg-[#080A0C] px-3 py-2 text-[14px] text-[#FFFCF8] focus:border-[#FF7A00] focus:outline-none"
                >
                  <option value="whatsapp">WhatsApp</option>
                  <option value="facebook">Facebook &amp; Messenger</option>
                  <option value="instagram">Instagram DMs &amp; Stories</option>
                  <option value="tiktok">TikTok Live &amp; DMs</option>
                  <option value="storefront">Web Storefront (Order Now)</option>
                </select>
              </div>

              <button
                type="submit"
                className="w-full rounded-md bg-[#FF7A00] px-4 py-2.5 text-[14px] font-semibold text-[#080A0C] transition-colors hover:bg-[#D95700]"
              >
                Create store &amp; start 14-day trial
              </button>
            </form>
          </>
        )}
      </div>
    </div>
  )
}
