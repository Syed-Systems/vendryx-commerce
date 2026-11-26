export type CurrencyCode = "USD" | "BDT"
export type PrepaidMonths = 1 | 3 | 6 | 12 | 24
export type PaidPlanTier = "pro" | "scale"
export type PlanSelection = "trial" | "free" | "pro" | "scale"
export type SocialChannel = "whatsapp" | "facebook" | "instagram" | "tiktok"
export type DeliveryZone = "inside_dhaka" | "sub_dhaka" | "outside_dhaka"
export type StorefrontAccentId = "vendryx" | "emerald" | "ivory"

export interface ProTermQuote {
  tier: PaidPlanTier
  months: PrepaidMonths
  currency: CurrencyCode
  discountPercent: number
  subtotalUsdCents: number
  discountUsdCents: number
  totalUsdCents: number
  effectiveMonthlyUsdCents: number
  payableBdt: number
  effectiveMonthlyBdt: number
  formattedMonthly: string
  formattedTotal: string
  formattedSavings: string | null
  includesFreeCustomStorefront: boolean
}

// Margin-protected 3-tier pricing rules (exact integer math, zero float drift):
// Pro:   USD 20/mo (2000 cents) | BDT 2,200/mo — includes 1,500 AI Store Assistant chats/mo
// Scale: USD 50/mo (5000 cents) | BDT 5,400/mo — includes 6,000 AI Store Assistant chats/mo
// Long-term prepaid discount = 10% flat when months >= 12 (includes custom storefront setup)
export const PRO_BASE_MONTHLY_USD_CENTS = 2000
export const PRO_BASE_MONTHLY_BDT = 2200
export const SCALE_BASE_MONTHLY_USD_CENTS = 5000
export const SCALE_BASE_MONTHLY_BDT = 5400
export const ANNUAL_DISCOUNT_THRESHOLD_MONTHS = 12
export const LONG_TERM_DISCOUNT_PERCENT = 10

const TIER_BASE_PRICING: Record<
  PaidPlanTier,
  { monthlyUsdCents: number; monthlyBdt: number }
> = {
  pro: {
    monthlyUsdCents: PRO_BASE_MONTHLY_USD_CENTS,
    monthlyBdt: PRO_BASE_MONTHLY_BDT,
  },
  scale: {
    monthlyUsdCents: SCALE_BASE_MONTHLY_USD_CENTS,
    monthlyBdt: SCALE_BASE_MONTHLY_BDT,
  },
}

export function formatUsdCents(cents: number): string {
  const major = Math.trunc(cents / 100)
  const remainder = Math.abs(cents % 100)
  const formattedMajor = major.toLocaleString("en-US")
  if (remainder === 0) return `$${formattedMajor}`
  return `$${formattedMajor}.${String(remainder).padStart(2, "0")}`
}

export function formatWholeBdt(taka: number): string {
  return `৳${Math.round(taka).toLocaleString("en-US")}`
}

export function calculatePlanTermQuote(
  tier: PaidPlanTier,
  months: PrepaidMonths,
  currency: CurrencyCode
): ProTermQuote {
  const base = TIER_BASE_PRICING[tier]
  const hasDiscount = months >= ANNUAL_DISCOUNT_THRESHOLD_MONTHS
  const discountPercent = hasDiscount ? LONG_TERM_DISCOUNT_PERCENT : 0

  const subtotalUsdCents = base.monthlyUsdCents * months
  const discountUsdCents = hasDiscount ? Math.trunc((subtotalUsdCents * discountPercent) / 100) : 0
  const totalUsdCents = subtotalUsdCents - discountUsdCents
  const effectiveMonthlyUsdCents = Math.trunc(totalUsdCents / months)

  const subtotalBdt = base.monthlyBdt * months
  const discountBdt = hasDiscount ? Math.trunc((subtotalBdt * discountPercent) / 100) : 0
  const payableBdt = subtotalBdt - discountBdt
  const effectiveMonthlyBdt = Math.trunc(payableBdt / months)

  const formattedMonthly =
    currency === "USD" ? formatUsdCents(effectiveMonthlyUsdCents) : formatWholeBdt(effectiveMonthlyBdt)
  const formattedTotal =
    currency === "USD" ? formatUsdCents(totalUsdCents) : formatWholeBdt(payableBdt)
  const formattedSavings =
    hasDiscount
      ? currency === "USD"
        ? formatUsdCents(discountUsdCents)
        : formatWholeBdt(discountBdt)
      : null

  return {
    tier,
    months,
    currency,
    discountPercent,
    subtotalUsdCents,
    discountUsdCents,
    totalUsdCents,
    effectiveMonthlyUsdCents,
    payableBdt,
    effectiveMonthlyBdt,
    formattedMonthly,
    formattedTotal,
    formattedSavings,
    includesFreeCustomStorefront: hasDiscount,
  }
}

export function calculateProTermQuote(
  months: PrepaidMonths,
  currency: CurrencyCode
): ProTermQuote {
  return calculatePlanTermQuote("pro", months, currency)
}

export interface CourierQuoteInput {
  zone: DeliveryZone
  weightKg: number
  codAmountBdt: number
}

export interface CourierQuote {
  zoneLabel: string
  deliveryFeeBdt: number
  codFeeBdt: number
  netMerchantPayoutBdt: number
  estimatedWindow: string
  recommendedCarrier: "Pathao" | "Steadfast" | "RedX"
  hierarchyExample: string
}

const ZONE_BASE_RATES_BDT: Record<
  DeliveryZone,
  {
    label: string
    baseFee: number
    extraKgFee: number
    eta: string
    carrier: "Pathao" | "Steadfast" | "RedX"
    hierarchyExample: string
  }
> = {
  inside_dhaka: {
    label: "Inside Dhaka",
    baseFee: 60,
    extraKgFee: 20,
    eta: "Same-day / 24h",
    carrier: "Pathao",
    hierarchyExample: "Dhaka → Dhanmondi → Road 27",
  },
  sub_dhaka: {
    label: "Sub-Dhaka",
    baseFee: 100,
    extraKgFee: 25,
    eta: "24–48 hours",
    carrier: "Steadfast",
    hierarchyExample: "Dhaka → Gazipur → Tongi",
  },
  outside_dhaka: {
    label: "Outside Dhaka",
    baseFee: 130,
    extraKgFee: 30,
    eta: "48–72 hours",
    carrier: "RedX",
    hierarchyExample: "Chattogram → Agrabad → Zone 04",
  },
}

export function calculateCourierQuote(input: CourierQuoteInput): CourierQuote {
  const safeWeight = Math.max(1, Math.ceil(input.weightKg))
  const safeCod = Math.max(0, Math.trunc(input.codAmountBdt))
  const config = ZONE_BASE_RATES_BDT[input.zone]
  const extraKg = Math.max(0, safeWeight - 1)
  const deliveryFeeBdt = config.baseFee + extraKg * config.extraKgFee
  const codFeeBdt = input.zone === "inside_dhaka" ? 0 : Math.trunc((safeCod + 50) / 100)
  const netMerchantPayoutBdt = Math.max(0, safeCod - deliveryFeeBdt - codFeeBdt)

  return {
    zoneLabel: config.label,
    deliveryFeeBdt,
    codFeeBdt,
    netMerchantPayoutBdt,
    estimatedWindow: config.eta,
    recommendedCarrier: config.carrier,
    hierarchyExample: config.hierarchyExample,
  }
}

export interface OnboardingFormInput {
  storeName: string
  email: string
  channel: SocialChannel | "storefront"
}

export interface OnboardingValidationResult {
  valid: boolean
  errors: {
    storeName?: string
    email?: string
  }
}

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/

export function validateOnboardingForm(input: OnboardingFormInput): OnboardingValidationResult {
  const errors: OnboardingValidationResult["errors"] = {}
  const trimmedName = input.storeName.trim()
  const trimmedEmail = input.email.trim()

  if (trimmedName.length < 2) {
    errors.storeName = "Enter your store or brand name (at least 2 characters)."
  }

  if (!EMAIL_PATTERN.test(trimmedEmail)) {
    errors.email = "Enter a valid work email address."
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors,
  }
}
