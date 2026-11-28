import type { StoreCurrency } from "./tenant-resolver"

export type DeliveryZone = "inside_city" | "suburb" | "nationwide"
export type PaymentMethod = "cod" | "bkash" | "sslcommerz" | "stripe"

export interface CartLineItem {
  variantId: string
  productId: string
  title: string
  variantLabel: string
  imagePath: string
  unitPriceMinor: number
  quantity: number
}

export interface CheckoutTotals {
  subtotalMinor: number
  shippingMinor: number
  codFeeMinor: number
  totalMinor: number
  itemCount: number
  qualifiesForFreeShipping: boolean
}

export interface CheckoutFormInput {
  fullName: string
  phone: string
  zone: DeliveryZone
  address: string
  paymentMethod: PaymentMethod
}

export interface CheckoutValidationResult {
  valid: boolean
  errors: Partial<Record<"fullName" | "phone" | "address", string>>
}

export interface OrderReceipt {
  orderNumber: string
  tenantId: string
  customerName: string
  phone: string
  zoneLabel: string
  paymentLabel: string
  totalFormatted: string
  itemCount: number
  createdAtIso: string
}

const FREE_SHIPPING_THRESHOLD_USD_CENTS = 15000 // $150
const ZONE_SHIPPING_USD_CENTS: Record<DeliveryZone, { fee: number; label: string }> = {
  inside_city: { fee: 600, label: "Standard Metro Express (24h)" },
  suburb: { fee: 900, label: "Suburban Courier (24–48h)" },
  nationwide: { fee: 1400, label: "Nationwide Insured Courier (2–3 days)" },
}

const PAYMENT_LABELS: Record<PaymentMethod, string> = {
  cod: "Cash on Delivery",
  bkash: "bKash Direct",
  sslcommerz: "SSLCommerz (Cards & Wallets)",
  stripe: "Stripe Card Checkout",
}

export function formatMinorPrice(minorUnits: number, currency: StoreCurrency = "USD"): string {
  const safeMinor = Math.max(0, Math.trunc(minorUnits))
  if (currency === "BDT") {
    return `৳${safeMinor.toLocaleString("en-US")}`
  }
  const dollars = Math.trunc(safeMinor / 100)
  const cents = safeMinor % 100
  const formattedDollars = dollars.toLocaleString("en-US")
  if (cents === 0) return `$${formattedDollars}`
  return `$${formattedDollars}.${String(cents).padStart(2, "0")}`
}

export function calculateLockedCheckout(
  items: CartLineItem[],
  zone: DeliveryZone,
  paymentMethod: PaymentMethod
): CheckoutTotals {
  let subtotalMinor = 0
  let itemCount = 0

  for (const item of items) {
    const qty = Math.max(0, Math.trunc(item.quantity))
    subtotalMinor += Math.max(0, Math.trunc(item.unitPriceMinor)) * qty
    itemCount += qty
  }

  const qualifiesForFreeShipping = subtotalMinor >= FREE_SHIPPING_THRESHOLD_USD_CENTS
  const shippingMinor =
    itemCount === 0 ? 0 : qualifiesForFreeShipping ? 0 : ZONE_SHIPPING_USD_CENTS[zone].fee
  const codFeeMinor =
    itemCount === 0 || paymentMethod !== "cod" || zone === "inside_city" ? 0 : 200
  const totalMinor = subtotalMinor + shippingMinor + codFeeMinor

  return {
    subtotalMinor,
    shippingMinor,
    codFeeMinor,
    totalMinor,
    itemCount,
    qualifiesForFreeShipping,
  }
}

const PHONE_REGEX = /^[+\d][\d\s-]{7,17}$/

export function validateCheckoutForm(input: CheckoutFormInput): CheckoutValidationResult {
  const errors: CheckoutValidationResult["errors"] = {}

  if (input.fullName.trim().length < 2) {
    errors.fullName = "Enter your full name."
  }
  if (!PHONE_REGEX.test(input.phone.trim())) {
    errors.phone = "Enter a valid phone number for delivery updates."
  }
  if (input.address.trim().length < 8) {
    errors.address = "Enter your street address, apartment, and area."
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors,
  }
}

export function createOrderReceipt(params: {
  tenantId: string
  input: CheckoutFormInput
  totals: CheckoutTotals
  currency: StoreCurrency
  sequenceSeed?: number
}): OrderReceipt {
  const code = String(params.sequenceSeed ?? (1040 + (params.totals.totalMinor % 890)))
  return {
    orderNumber: `#VND-${code}`,
    tenantId: params.tenantId,
    customerName: params.input.fullName.trim(),
    phone: params.input.phone.trim(),
    zoneLabel: ZONE_SHIPPING_USD_CENTS[params.input.zone].label,
    paymentLabel: PAYMENT_LABELS[params.input.paymentMethod],
    totalFormatted: formatMinorPrice(params.totals.totalMinor, params.currency),
    itemCount: params.totals.itemCount,
    createdAtIso: new Date().toISOString(),
  }
}
