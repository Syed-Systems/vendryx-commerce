import { Check, CreditCard, FileSpreadsheet, Globe, Truck } from "lucide-react"
import { useMemo, useState } from "react"
import {
  COURIER_PROVIDERS,
  PAYMENT_PROVIDERS,
  SPREADSHEET_MAPPINGS,
  STOREFRONT_BRAND_PRESETS,
} from "../data/marketing-content"
import {
  calculateCourierQuote,
  formatWholeBdt,
  type DeliveryZone,
  type StorefrontAccentId,
} from "../lib/marketing-logic"

const ZONES: { id: DeliveryZone; label: string }[] = [
  { id: "inside_dhaka", label: "Inside Dhaka" },
  { id: "sub_dhaka", label: "Sub-Dhaka" },
  { id: "outside_dhaka", label: "Outside Dhaka" },
]

const WEIGHTS = [1, 2, 3] as const
const PRESET_IDS: StorefrontAccentId[] = ["vendryx", "emerald", "ivory"]

export function BentoOperationsSection() {
  const [zone, setZone] = useState<DeliveryZone>("inside_dhaka")
  const [weightKg, setWeightKg] = useState<number>(1)
  const [brandPreset, setBrandPreset] = useState<StorefrontAccentId>("vendryx")
  const [sheetMode, setSheetMode] = useState<"import" | "export">("import")

  const quote = useMemo(
    () =>
      calculateCourierQuote({
        zone,
        weightKg,
        codAmountBdt: 3450,
      }),
    [zone, weightKg]
  )

  const activeTheme = STOREFRONT_BRAND_PRESETS[brandPreset]

  return (
    <section
      id="operations"
      className="scroll-mt-16 border-b border-[#FFFCF8]/[0.08] py-16 sm:py-20"
    >
      <div className="mx-auto max-w-[1200px] px-4 sm:px-6">
        <div className="max-w-[720px]">
          <h2 className="text-[28px] font-semibold leading-[1.12] tracking-[-0.03em] text-[#FFFCF8] sm:text-[36px]">
            Built for how you actually pack, ship, and get paid.
          </h2>
        </div>

        <div className="mt-8 grid grid-cols-1 gap-4 lg:grid-cols-12">
          {/* Tile 1 (7 cols): Local Courier Booking & Cash on Delivery */}
          <div className="rounded-xl border border-[#FFFCF8]/[0.08] vendryx-card-surface p-5 sm:p-6 lg:col-span-7">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="flex items-start gap-2.5">
                <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-[#FFFCF8]/10 bg-[#1B1E24] text-[#FF7A00]">
                  <Truck className="h-4 w-4" />
                </div>
                <h3 className="text-[20px] font-semibold leading-[1.25] tracking-[-0.02em] text-[#FFFCF8]">
                  Pathao, Steadfast, and RedX built right into your orders
                </h3>
              </div>

              <div className="flex items-center gap-2">
                {COURIER_PROVIDERS.map((carrier) => (
                  <div
                    key={carrier.name}
                    className="flex h-8 items-center rounded-md border border-[#FFFCF8]/[0.08] bg-[#FFFCF8] px-2.5"
                    title={carrier.name}
                  >
                    <img
                      src={carrier.logo}
                      alt={carrier.name}
                      className="h-4 w-auto max-w-[64px] object-contain"
                    />
                  </div>
                ))}
              </div>
            </div>

            <p className="mt-3 text-[14px] leading-[1.55] text-[#94A3B8]">
              Pick the buyer’s city and area, spot repeat fake-order numbers before shipping, and
              book pickup without copying addresses into separate courier portals.
            </p>

            {/* Interactive Zone & Weight Controls */}
            <div className="mt-5 flex flex-wrap items-center justify-between gap-3">
              <div className="flex flex-wrap gap-1.5">
                {ZONES.map((z) => (
                  <button
                    key={z.id}
                    type="button"
                    onClick={() => setZone(z.id)}
                    className={`shrink-0 whitespace-nowrap rounded-md px-3 py-1.5 text-[12px] font-medium transition-colors ${
                      zone === z.id
                        ? "bg-[#FF7A00] text-[#080A0C] font-semibold"
                        : "border border-[#FFFCF8]/[0.08] bg-[#0F1115] text-[#94A3B8] hover:text-[#FFFCF8]"
                    }`}
                  >
                    {z.label}
                  </button>
                ))}
              </div>

              <div className="flex items-center gap-1.5">
                <span className="text-[12px] text-[#94A3B8]">Weight:</span>
                {WEIGHTS.map((kg) => (
                  <button
                    key={kg}
                    type="button"
                    onClick={() => setWeightKg(kg)}
                    className={`shrink-0 rounded-md px-2.5 py-1 font-mono text-[12px] transition-colors ${
                      weightKg === kg
                        ? "border border-[#FF7A00] bg-[#1B1E24] text-[#FFFCF8]"
                        : "border border-[#FFFCF8]/[0.08] bg-[#0F1115] text-[#94A3B8] hover:text-[#FFFCF8]"
                    }`}
                  >
                    {kg}kg
                  </button>
                ))}
              </div>
            </div>

            {/* Live Payout Preview */}
            <div className="mt-4 rounded-lg border border-[#FFFCF8]/[0.08] bg-[#0B0D11] p-4">
              <div className="flex flex-wrap items-center justify-between gap-2 border-b border-[#FFFCF8]/[0.07] pb-2.5">
                <span className="text-[12px] text-[#94A3B8]">
                  Delivery area:{" "}
                  <span className="font-medium text-[#FFFCF8]">{quote.hierarchyExample}</span>
                </span>
                <span className="font-mono text-[11.5px] text-[#FFAF24]">
                  {quote.recommendedCarrier} · {quote.estimatedWindow}
                </span>
              </div>

              <div className="mt-3 grid grid-cols-2 gap-3 sm:grid-cols-4">
                <div>
                  <p className="text-[11.5px] text-[#94A3B8]">Cash collected</p>
                  <p className="mt-0.5 font-mono text-[15px] font-semibold tabular-nums text-[#FFFCF8]">
                    ৳3,450
                  </p>
                </div>
                <div>
                  <p className="text-[11.5px] text-[#94A3B8]">Delivery charge</p>
                  <p className="mt-0.5 font-mono text-[15px] font-semibold tabular-nums text-[#CBD5E1]">
                    {formatWholeBdt(quote.deliveryFeeBdt)}
                  </p>
                </div>
                <div>
                  <p className="text-[11.5px] text-[#94A3B8]">COD charge</p>
                  <p className="mt-0.5 font-mono text-[15px] font-semibold tabular-nums text-[#CBD5E1]">
                    {formatWholeBdt(quote.codFeeBdt)}
                  </p>
                </div>
                <div>
                  <p className="text-[11.5px] text-[#94A3B8]">You receive</p>
                  <p className="mt-0.5 font-mono text-[15px] font-semibold tabular-nums text-[#4ADE80]">
                    {formatWholeBdt(quote.netMerchantPayoutBdt)}
                  </p>
                </div>
              </div>
            </div>
          </div>

          {/* Tile 2 (5 cols): Direct Payments with 0% Commission */}
          <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.08] vendryx-card-surface p-5 sm:p-6 lg:col-span-5">
            <div>
              <div className="flex items-start gap-2.5">
                <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-[#FFFCF8]/10 bg-[#1B1E24] text-[#FF7A00]">
                  <CreditCard className="h-4 w-4" />
                </div>
                <h3 className="text-[20px] font-semibold leading-[1.25] tracking-[-0.02em] text-[#FFFCF8]">
                  Connect your own accounts. Keep 100% of every sale.
                </h3>
              </div>
              <p className="mt-2.5 text-[14px] leading-[1.55] text-[#94A3B8]">
                Customer payments go directly into your own bKash, bank, or card account. Vendryx
                never holds your money and charges{" "}
                <span className="font-medium text-[#FFFCF8]">0% per order</span>.
              </p>

              <div className="mt-4 space-y-2">
                {PAYMENT_PROVIDERS.map((provider) => (
                  <div
                    key={provider.name}
                    className="flex items-center justify-between gap-3 rounded-lg border border-[#FFFCF8]/[0.07] bg-[#0F1115] px-3 py-2.5"
                  >
                    <div className="flex items-center gap-3 min-w-0">
                      <div className="flex h-7 w-16 shrink-0 items-center justify-center rounded bg-[#FFFCF8] px-1.5">
                        <img
                          src={provider.logo}
                          alt={provider.name}
                          className="h-4 w-auto max-w-[52px] object-contain"
                        />
                      </div>
                      <div className="min-w-0">
                        <p className="text-[13px] font-semibold text-[#FFFCF8]">{provider.name}</p>
                        <p className="truncate text-[11.5px] text-[#94A3B8]">{provider.note}</p>
                      </div>
                    </div>
                    <span className="shrink-0 font-mono text-[10.5px] text-[#4ADE80]">Direct</span>
                  </div>
                ))}
              </div>
            </div>

            <p className="mt-4 border-t border-[#FFFCF8]/[0.07] pt-3 text-[12px] text-[#94A3B8]">
              Cash on Delivery (COD) is included free on every store.
            </p>
          </div>

          {/* Tile 3 (5 cols): Branded Storefront & No-Login Checkout */}
          <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.08] vendryx-card-surface p-5 sm:p-6 lg:col-span-5">
            <div>
              <div className="flex items-start gap-2.5">
                <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-[#FFFCF8]/10 bg-[#1B1E24] text-[#FF7A00]">
                  <Globe className="h-4 w-4" />
                </div>
                <h3 className="text-[20px] font-semibold leading-[1.25] tracking-[-0.02em] text-[#FFFCF8]">
                  Buyers tap “Order Now” — no password or sign-up needed
                </h3>
              </div>
              <p className="mt-2.5 text-[14px] leading-[1.55] text-[#94A3B8]">
                Start free on{" "}
                <span className="font-mono text-[#FFFCF8]">yourstore.vendryx.shop</span> with your
                brand colors, or connect your own{" "}
                <span className="font-mono text-[#FFFCF8]">.com</span> domain on Pro with a custom
                storefront built by our team.
              </p>

              <div className="mt-4 flex flex-wrap gap-1.5">
                {PRESET_IDS.map((id) => {
                  const preset = STOREFRONT_BRAND_PRESETS[id]
                  const active = brandPreset === id
                  return (
                    <button
                      key={id}
                      type="button"
                      onClick={() => setBrandPreset(id)}
                      className={`shrink-0 rounded-md px-2.5 py-1.5 text-[12px] font-medium transition-colors ${
                        active
                          ? "bg-[#FF7A00] text-[#080A0C] font-semibold"
                          : "border border-[#FFFCF8]/[0.08] bg-[#0F1115] text-[#94A3B8] hover:text-[#FFFCF8]"
                      }`}
                    >
                      {preset.name}
                    </button>
                  )
                })}
              </div>

              <ul className="mt-4 space-y-2 border-t border-[#FFFCF8]/[0.07] pt-3.5 text-[12.5px] text-[#CBD5E1]">
                <li className="flex items-center gap-2">
                  <Check className="h-3.5 w-3.5 shrink-0 text-[#FF7A00]" />
                  <span>Name, phone, and address only — buyers check out in seconds</span>
                </li>
                <li className="flex items-center gap-2">
                  <Check className="h-3.5 w-3.5 shrink-0 text-[#FF7A00]" />
                  <span>Repeat customers recognized automatically by phone number</span>
                </li>
                <li className="flex items-center gap-2">
                  <Check className="h-3.5 w-3.5 shrink-0 text-[#FF7A00]" />
                  <span>Custom domain &amp; bespoke storefront included on Pro</span>
                </li>
              </ul>
            </div>

            {/* Live Storefront Mini Preview */}
            <div
              className="mt-4 rounded-lg p-4 transition-colors"
              style={{
                backgroundColor: activeTheme.bg,
                border: `1px solid ${activeTheme.border}`,
                color: activeTheme.text,
              }}
            >
              <div className="flex items-center justify-between gap-2 text-[11px] font-mono">
                <span className="truncate opacity-85">https://{activeTheme.domain}</span>
                <span style={{ color: activeTheme.accent }} className="shrink-0 font-semibold">
                  {activeTheme.tierLabel}
                </span>
              </div>

              <div className="mt-3.5 flex items-center justify-between gap-3">
                <div className="min-w-0">
                  <p className="truncate text-[14px] font-semibold">Jamdani Linen Overshirt</p>
                  <p className="mt-0.5 text-[12px]" style={{ color: activeTheme.muted }}>
                    Olive · Size M · ৳2,450 · Fast guest checkout
                  </p>
                </div>
                <span
                  className="shrink-0 whitespace-nowrap rounded-md px-3 py-1.5 text-[12px] font-semibold"
                  style={{
                    backgroundColor: activeTheme.accent,
                    color: activeTheme.accentText,
                  }}
                >
                  Order Now
                </span>
              </div>
            </div>
          </div>

          {/* Tile 4 (7 cols): Simple Excel Upload & Bengali-Friendly Export */}
          <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.08] vendryx-card-surface p-5 sm:p-6 lg:col-span-7">
            <div>
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="flex items-start gap-2.5">
                  <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-[#FFFCF8]/10 bg-[#1B1E24] text-[#FF7A00]">
                    <FileSpreadsheet className="h-4 w-4" />
                  </div>
                  <h3 className="text-[20px] font-semibold leading-[1.25] tracking-[-0.02em] text-[#FFFCF8]">
                    Upload your existing Excel sheet without starting over
                  </h3>
                </div>

                <div className="flex items-center gap-1.5 rounded-md border border-[#FFFCF8]/[0.08] bg-[#0B0D11] p-1">
                  <button
                    type="button"
                    onClick={() => setSheetMode("import")}
                    className={`rounded px-2.5 py-1 text-[12px] font-medium transition-colors ${
                      sheetMode === "import"
                        ? "bg-[#FF7A00] text-[#080A0C] font-semibold"
                        : "text-[#94A3B8] hover:text-[#FFFCF8]"
                    }`}
                  >
                    Upload Excel
                  </button>
                  <button
                    type="button"
                    onClick={() => setSheetMode("export")}
                    className={`rounded px-2.5 py-1 text-[12px] font-medium transition-colors ${
                      sheetMode === "export"
                        ? "bg-[#FF7A00] text-[#080A0C] font-semibold"
                        : "text-[#94A3B8] hover:text-[#FFFCF8]"
                    }`}
                  >
                    Bengali Export
                  </button>
                </div>
              </div>

              <p className="mt-3 text-[14px] leading-[1.55] text-[#94A3B8]">
                {sheetMode === "import"
                  ? "Match your Excel or CSV columns once and save the layout. Extra columns are skipped safely and your products appear live on your storefront right away."
                  : "Download your orders anytime and open them straight in Excel—Bengali customer names, Dhaka street addresses, and Taka totals display cleanly without broken characters."}
              </p>

              {sheetMode === "import" ? (
                <div className="mt-4 overflow-hidden rounded-lg border border-[#FFFCF8]/[0.08] bg-[#0B0D11]">
                  <div className="grid grid-cols-[1fr_1fr_auto] gap-3 border-b border-[#FFFCF8]/[0.07] px-3.5 py-2 text-[11px] font-semibold uppercase tracking-[0.06em] text-[#64748B]">
                    <span>Your Excel Column</span>
                    <span>In Your Store</span>
                    <span>Status</span>
                  </div>
                  <div className="divide-y divide-[#FFFCF8]/[0.06]">
                    {SPREADSHEET_MAPPINGS.map((row) => (
                      <div
                        key={row.sellerCol}
                        className="grid grid-cols-[1fr_1fr_auto] items-center gap-3 px-3.5 py-2 text-[13px]"
                      >
                        <span className="truncate text-[#CBD5E1]">{row.sellerCol}</span>
                        <span className="truncate font-medium text-[#FFFCF8]">
                          {row.vendryxField}
                        </span>
                        <span className="shrink-0 whitespace-nowrap font-mono text-[11.5px] text-[#FFAF24]">
                          {row.status}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="mt-4 rounded-lg border border-[#FFFCF8]/[0.08] bg-[#0B0D11] p-4 font-mono text-[12px] leading-[1.7] text-[#CBD5E1]">
                  <p className="text-[#94A3B8]">
                    # Ready for Microsoft Excel — Bengali text opens cleanly
                  </p>
                  <p className="mt-2 text-[#FFAF24]">
                    Order,Customer,Address,Product,Total,Courier
                  </p>
                  <p className="mt-1 text-[#FFFCF8]">
                    #1048,নাদিয়া রহমান,ধানমন্ডি ২৭,&quot;জামদানি লিনেন শার্ট (M)&quot;,৳3450,Pathao
                  </p>
                  <p className="text-[#FFFCF8]">
                    #1047,ফারহান চৌধুরী,উত্তরা সেক্টর ৭,&quot;লেদার উইকেন্ডার ব্যাগ&quot;,৳6800,Steadfast
                  </p>
                </div>
              )}
            </div>

            <div className="mt-4 flex flex-wrap items-center gap-x-5 gap-y-1.5 border-t border-[#FFFCF8]/[0.07] pt-3 text-[12.5px] text-[#94A3B8]">
              <span className="inline-flex items-center gap-1.5">
                <Check className="h-3.5 w-3.5 text-[#FF7A00]" />
                Save column templates
              </span>
              <span className="inline-flex items-center gap-1.5">
                <Check className="h-3.5 w-3.5 text-[#FF7A00]" />
                Up to 100 products free
              </span>
              <span className="inline-flex items-center gap-1.5">
                <Check className="h-3.5 w-3.5 text-[#FF7A00]" />
                Instant new-order alerts
              </span>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
