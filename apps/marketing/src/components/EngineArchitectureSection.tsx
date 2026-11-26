import { CheckCircle2, ShieldCheck } from "lucide-react"
import { useState } from "react"
import { RELIABILITY_PILLARS } from "../data/marketing-content"

interface OrderSafetyStep {
  id: string
  title: string
  whatHappens: string
  safetyNet: string
}

const SAFETY_STEPS: OrderSafetyStep[] = [
  {
    id: "hold",
    title: "Hold Stock",
    whatHappens:
      "As soon as a shopper starts checking out, their item is reserved for 15 minutes.",
    safetyNet:
      "If they close the page or cancel, the item goes right back on sale automatically.",
  },
  {
    id: "payment",
    title: "Check Payment or COD",
    whatHappens:
      "bKash, card payments, or Cash on Delivery phone history are verified safely.",
    safetyNet:
      "Even if mobile internet drops mid-payment, the customer is never charged twice.",
  },
  {
    id: "confirm",
    title: "Confirm Order",
    whatHappens:
      "The order appears instantly in your dashboard and alerts your team right away.",
    safetyNet:
      "Your numbers, stock counts, and revenue totals always match down to the last Taka.",
  },
  {
    id: "ship",
    title: "Book Courier",
    whatHappens:
      "Send the parcel to Pathao, Steadfast, or RedX in one click with live tracking.",
    safetyNet:
      "If a courier service is busy, your order stays saved safely until you retry.",
  },
]

export function EngineArchitectureSection() {
  const [activeStepIndex, setActiveStepIndex] = useState<number>(0)
  const activeStep = SAFETY_STEPS[activeStepIndex]

  return (
    <section
      id="engine"
      className="scroll-mt-16 border-b border-[#FFFCF8]/[0.08] py-16 sm:py-20"
    >
      <div className="mx-auto max-w-[1200px] px-4 sm:px-6">
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-[1fr_1.1fr] lg:items-end">
          <div>
            <h2 className="text-[28px] font-semibold leading-[1.12] tracking-[-0.03em] text-[#FFFCF8] sm:text-[36px]">
              Fast, private, and safe—even during your biggest Eid or flash sale.
            </h2>
          </div>
          <p className="text-[15px] leading-[1.6] text-[#94A3B8]">
            You shouldn’t have to worry about broken plugins, oversold stock, or double charges.
            Vendryx handles the hard work quietly behind the scenes so your store just works.
          </p>
        </div>

        {/* 4 Simple Reliability Promises */}
        <div className="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {RELIABILITY_PILLARS.map((pillar) => (
            <article
              key={pillar.number}
              className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.08] bg-[#0F1115] p-5"
            >
              <div>
                <h3 className="text-[16px] font-semibold leading-[1.3] text-[#FFFCF8]">
                  {pillar.title}
                </h3>
                <p className="mt-2 text-[13.5px] leading-[1.55] text-[#94A3B8]">
                  {pillar.description}
                </p>
              </div>
              <p className="mt-4 border-t border-[#FFFCF8]/[0.07] pt-3 text-[12px] font-medium text-[#CBD5E1]">
                {pillar.detail}
              </p>
            </article>
          ))}
        </div>

        {/* Interactive 4-Step Order Journey */}
        <div className="mt-5 rounded-xl border border-[#FFFCF8]/[0.08] vendryx-card-surface p-5 sm:p-6">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-[#FFFCF8]/[0.08] pb-3.5">
            <p className="text-[15px] font-semibold text-[#FFFCF8]">
              How every order stays protected from checkout to delivery
            </p>
            <span className="text-[12px] text-[#94A3B8]">Click any step to explore</span>
          </div>

          <div className="mt-4 grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-4">
            {SAFETY_STEPS.map((step, index) => {
              const active = index === activeStepIndex
              return (
                <button
                  key={step.id}
                  type="button"
                  onClick={() => setActiveStepIndex(index)}
                  className={`flex items-center gap-2.5 rounded-lg border px-3.5 py-3 text-left transition-colors ${
                    active
                      ? "border-[#FF7A00] bg-[#1B1E24]"
                      : "border-[#FFFCF8]/[0.08] bg-[#0F1115] hover:border-[#FFFCF8]/20"
                  }`}
                >
                  <span
                    className={`flex h-6 w-6 shrink-0 items-center justify-center rounded-md font-mono text-[11px] font-semibold ${
                      active
                        ? "bg-[#FF7A00] text-[#080A0C]"
                        : "bg-[#1B1E24] text-[#94A3B8]"
                    }`}
                  >
                    {index + 1}
                  </span>
                  <span className="text-[13.5px] font-semibold text-[#FFFCF8]">{step.title}</span>
                </button>
              )
            })}
          </div>

          <div className="mt-4 grid grid-cols-1 gap-3 rounded-lg border border-[#FFFCF8]/[0.07] bg-[#0B0D11] p-4 md:grid-cols-2">
            <div className="flex items-start gap-2.5">
              <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-[#4ADE80]" />
              <div>
                <p className="text-[12.5px] font-semibold text-[#4ADE80]">
                  What happens ({activeStep.title})
                </p>
                <p className="mt-1 text-[13.5px] leading-[1.55] text-[#FFFCF8]">
                  {activeStep.whatHappens}
                </p>
              </div>
            </div>

            <div className="flex items-start gap-2.5 border-t border-[#FFFCF8]/[0.07] pt-3 md:border-l md:border-t-0 md:pl-4 md:pt-0">
              <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-[#FFAF24]" />
              <div>
                <p className="text-[12.5px] font-semibold text-[#FFAF24]">
                  If anything goes wrong
                </p>
                <p className="mt-1 text-[13.5px] leading-[1.55] text-[#CBD5E1]">
                  {activeStep.safetyNet}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
