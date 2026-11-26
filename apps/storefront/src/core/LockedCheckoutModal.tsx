"use client"

import React, { useMemo, useState } from "react"
import {
  calculateLockedCheckout,
  createOrderReceipt,
  formatMinorPrice,
  validateCheckoutForm,
  type CartLineItem,
  type DeliveryZone,
  type OrderReceipt,
  type PaymentMethod,
} from "./checkout-engine"
import type { ResolvedTenantContext } from "./tenant-resolver"

interface LockedCheckoutModalProps {
  open: boolean
  tenant: ResolvedTenantContext
  items: CartLineItem[]
  onClose: () => void
  onOrderComplete: (receipt: OrderReceipt) => void
}

const DELIVERY_ZONES: { id: DeliveryZone; label: string; eta: string; feeMinor: number }[] = [
  { id: "inside_city", label: "Standard Metro Express", eta: "24 hours", feeMinor: 600 },
  { id: "suburb", label: "Suburban Courier", eta: "24–48 hours", feeMinor: 900 },
  { id: "nationwide", label: "Nationwide Insured Courier", eta: "2–3 business days", feeMinor: 1400 },
]

const PAYMENT_OPTIONS: { id: PaymentMethod; label: string; note: string }[] = [
  { id: "cod", label: "Cash on Delivery", note: "Pay courier at doorstep" },
  { id: "bkash", label: "bKash Direct", note: "Instant mobile wallet" },
  { id: "sslcommerz", label: "SSLCommerz", note: "Visa, Mastercard & local cards" },
  { id: "stripe", label: "Stripe Card", note: "Global Apple Pay & cards" },
]

export function LockedCheckoutModal({
  open,
  tenant,
  items,
  onClose,
  onOrderComplete,
}: LockedCheckoutModalProps) {
  const [fullName, setFullName] = useState("")
  const [phone, setPhone] = useState("")
  const [address, setAddress] = useState("")
  const [zone, setZone] = useState<DeliveryZone>("inside_city")
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>("stripe")
  const [errors, setErrors] = useState<Partial<Record<"fullName" | "phone" | "address", string>>>({})
  const [receipt, setReceipt] = useState<OrderReceipt | null>(null)

  const totals = useMemo(
    () => calculateLockedCheckout(items, zone, paymentMethod),
    [items, zone, paymentMethod]
  )

  if (!open) return null

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const validation = validateCheckoutForm({
      fullName,
      phone,
      zone,
      address,
      paymentMethod,
    })

    if (!validation.valid) {
      setErrors(validation.errors)
      return
    }

    setErrors({})
    const created = createOrderReceipt({
      tenantId: tenant.tenantId,
      input: { fullName, phone, zone, address, paymentMethod },
      totals,
      currency: tenant.currency,
    })
    setReceipt(created)
    onOrderComplete(created)
  }

  const handleClose = () => {
    setReceipt(null)
    setErrors({})
    onClose()
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/55 p-4 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-label="Locked Core Checkout"
      data-testid="locked-checkout-modal"
    >
      <div className="relative max-h-[92vh] w-full max-w-4xl overflow-y-auto border border-[var(--theme-border)] bg-[var(--theme-canvas)] text-[var(--theme-ink)] shadow-2xl rounded-[var(--theme-radius-card)]">
        {/* Top bar */}
        <div className="flex items-center justify-between border-b border-[var(--theme-border)] bg-[var(--theme-surface)] px-6 py-4">
          <div className="flex items-center gap-3">
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.75"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <rect x="4" y="11" width="16" height="10" rx="2" />
              <path d="M8 11V7a4 4 0 0 1 8 0v4" />
            </svg>
            <div>
              <h2 className="text-base font-semibold tracking-tight">{tenant.brandName} · Checkout</h2>
              <p className="text-xs text-[var(--theme-ink-muted)]">
                Protected Commerce Core · Host-isolated session ({tenant.resolvedHost})
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={handleClose}
            aria-label="Close checkout"
            className="flex h-9 w-9 items-center justify-center border border-[var(--theme-border)] text-[var(--theme-ink)] transition hover:bg-[var(--theme-surface-alt)] rounded-[var(--theme-radius-button)]"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8">
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        {receipt ? (
          <div className="p-8 md:p-10" data-testid="checkout-order-receipt">
            <div className="mx-auto max-w-lg border border-[var(--theme-border)] bg-[var(--theme-surface)] p-8 text-center rounded-[var(--theme-radius-card)]">
              <div className="mx-auto mb-5 flex h-12 w-12 items-center justify-center bg-[var(--theme-accent)] text-[var(--theme-accent-ink)] rounded-[var(--theme-radius-button)]">
                <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
                  <path d="M20 6 9 17l-5-5" />
                </svg>
              </div>
              <h3 className="text-2xl font-semibold tracking-tight">Order Confirmed</h3>
              <p className="mt-2 text-sm text-[var(--theme-ink-muted)]">
                Reference <span className="font-mono font-semibold text-[var(--theme-ink)]" data-testid="order-number">{receipt.orderNumber}</span>
              </p>

              <dl className="mt-6 space-y-2.5 border-y border-[var(--theme-border)] py-5 text-left text-sm">
                <div className="flex justify-between gap-4">
                  <dt className="text-[var(--theme-ink-muted)]">Recipient</dt>
                  <dd className="font-medium">{receipt.customerName}</dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="text-[var(--theme-ink-muted)]">Phone</dt>
                  <dd className="font-mono text-xs">{receipt.phone}</dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="text-[var(--theme-ink-muted)]">Courier</dt>
                  <dd className="font-medium">{receipt.zoneLabel}</dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="text-[var(--theme-ink-muted)]">Payment</dt>
                  <dd className="font-medium">{receipt.paymentLabel}</dd>
                </div>
                <div className="flex justify-between gap-4 pt-1 text-base font-semibold">
                  <dt>Total Charged</dt>
                  <dd className="font-mono">{receipt.totalFormatted}</dd>
                </div>
              </dl>

              <button
                type="button"
                onClick={handleClose}
                className="mt-6 w-full bg-[var(--theme-accent)] px-6 py-3.5 text-sm font-medium text-[var(--theme-accent-ink)] transition hover:opacity-90 rounded-[var(--theme-radius-button)]"
              >
                Continue Shopping
              </button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="grid grid-cols-1 gap-8 p-6 md:grid-cols-12 md:p-8" noValidate>
            {/* Left: Customer & Shipping */}
            <div className="space-y-6 md:col-span-7">
              <div>
                <h3 className="text-sm font-semibold uppercase tracking-wider text-[var(--theme-ink-muted)]">
                  1. Contact & Delivery
                </h3>
                <div className="mt-4 space-y-4">
                  <div>
                    <label htmlFor="checkout-name" className="block text-xs font-medium text-[var(--theme-ink)]">
                      Full Name
                    </label>
                    <input
                      id="checkout-name"
                      name="fullName"
                      type="text"
                      value={fullName}
                      onChange={(e) => setFullName(e.target.value)}
                      placeholder="Nadia Rahman"
                      className="mt-1.5 w-full border border-[var(--theme-border)] bg-[var(--theme-surface)] px-3.5 py-2.5 text-sm text-[var(--theme-ink)] outline-none focus:border-[var(--theme-accent)] rounded-[var(--theme-radius-card)]"
                    />
                    {errors.fullName && (
                      <p className="mt-1 text-xs text-red-600">{errors.fullName}</p>
                    )}
                  </div>

                  <div>
                    <label htmlFor="checkout-phone" className="block text-xs font-medium text-[var(--theme-ink)]">
                      Phone Number
                    </label>
                    <input
                      id="checkout-phone"
                      name="phone"
                      type="tel"
                      value={phone}
                      onChange={(e) => setPhone(e.target.value)}
                      placeholder="+880 1711-000000"
                      className="mt-1.5 w-full border border-[var(--theme-border)] bg-[var(--theme-surface)] px-3.5 py-2.5 text-sm text-[var(--theme-ink)] outline-none focus:border-[var(--theme-accent)] rounded-[var(--theme-radius-card)]"
                    />
                    {errors.phone && (
                      <p className="mt-1 text-xs text-red-600">{errors.phone}</p>
                    )}
                  </div>

                  <div>
                    <label htmlFor="checkout-address" className="block text-xs font-medium text-[var(--theme-ink)]">
                      Street Address & Area
                    </label>
                    <textarea
                      id="checkout-address"
                      name="address"
                      rows={2}
                      value={address}
                      onChange={(e) => setAddress(e.target.value)}
                      placeholder="House 14, Road 7, Banani, Dhaka"
                      className="mt-1.5 w-full border border-[var(--theme-border)] bg-[var(--theme-surface)] px-3.5 py-2.5 text-sm text-[var(--theme-ink)] outline-none focus:border-[var(--theme-accent)] rounded-[var(--theme-radius-card)]"
                    />
                    {errors.address && (
                      <p className="mt-1 text-xs text-red-600">{errors.address}</p>
                    )}
                  </div>
                </div>
              </div>

              {/* Delivery Zone */}
              <div>
                <h3 className="text-sm font-semibold uppercase tracking-wider text-[var(--theme-ink-muted)]">
                  2. Delivery Zone
                </h3>
                <div className="mt-3 grid grid-cols-1 gap-2.5">
                  {DELIVERY_ZONES.map((z) => {
                    const selected = zone === z.id
                    return (
                      <button
                        key={z.id}
                        type="button"
                        onClick={() => setZone(z.id)}
                        className={`flex items-center justify-between border px-4 py-3 text-left text-sm transition rounded-[var(--theme-radius-card)] ${
                          selected
                            ? "border-[var(--theme-accent)] bg-[var(--theme-surface)]"
                            : "border-[var(--theme-border)] bg-transparent hover:bg-[var(--theme-surface)]"
                        }`}
                      >
                        <div>
                          <div className="font-medium">{z.label}</div>
                          <div className="text-xs text-[var(--theme-ink-muted)]">{z.eta}</div>
                        </div>
                        <span className="font-mono text-xs font-semibold">
                          {totals.qualifiesForFreeShipping
                            ? "Complimentary"
                            : formatMinorPrice(z.feeMinor, tenant.currency)}
                        </span>
                      </button>
                    )
                  })}
                </div>
              </div>

              {/* Payment Method */}
              <div>
                <h3 className="text-sm font-semibold uppercase tracking-wider text-[var(--theme-ink-muted)]">
                  3. Payment Method
                </h3>
                <div className="mt-3 grid grid-cols-1 gap-2.5 sm:grid-cols-2">
                  {PAYMENT_OPTIONS.map((p) => {
                    const selected = paymentMethod === p.id
                    return (
                      <button
                        key={p.id}
                        type="button"
                        onClick={() => setPaymentMethod(p.id)}
                        className={`border px-3.5 py-3 text-left transition rounded-[var(--theme-radius-card)] ${
                          selected
                            ? "border-[var(--theme-accent)] bg-[var(--theme-surface)]"
                            : "border-[var(--theme-border)] bg-transparent hover:bg-[var(--theme-surface)]"
                        }`}
                      >
                        <div className="text-sm font-medium">{p.label}</div>
                        <div className="text-xs text-[var(--theme-ink-muted)]">{p.note}</div>
                      </button>
                    )
                  })}
                </div>
              </div>
            </div>

            {/* Right: Locked Order Summary */}
            <div className="md:col-span-5">
              <div className="border border-[var(--theme-border)] bg-[var(--theme-surface)] p-5 rounded-[var(--theme-radius-card)]">
                <h3 className="text-sm font-semibold uppercase tracking-wider text-[var(--theme-ink-muted)]">
                  Order Summary ({totals.itemCount})
                </h3>

                <ul className="mt-4 max-h-64 space-y-3 overflow-y-auto pr-1">
                  {items.map((item) => (
                    <li key={item.variantId} className="flex items-center gap-3">
                      <img
                        src={item.imagePath}
                        alt={item.title}
                        className="h-14 w-14 shrink-0 border border-[var(--theme-border)] object-cover rounded-[var(--theme-radius-card)]"
                      />
                      <div className="min-w-0 flex-1">
                        <p className="truncate text-sm font-medium">{item.title}</p>
                        <p className="truncate text-xs text-[var(--theme-ink-muted)]">
                          {item.variantLabel} × {item.quantity}
                        </p>
                      </div>
                      <span className="font-mono text-xs font-medium">
                        {formatMinorPrice(item.unitPriceMinor * item.quantity, tenant.currency)}
                      </span>
                    </li>
                  ))}
                </ul>

                <dl className="mt-5 space-y-2 border-t border-[var(--theme-border)] pt-4 text-sm">
                  <div className="flex justify-between">
                    <dt className="text-[var(--theme-ink-muted)]">Subtotal</dt>
                    <dd className="font-mono">{formatMinorPrice(totals.subtotalMinor, tenant.currency)}</dd>
                  </div>
                  <div className="flex justify-between">
                    <dt className="text-[var(--theme-ink-muted)]">Shipping</dt>
                    <dd className="font-mono">
                      {totals.shippingMinor === 0
                        ? "Complimentary"
                        : formatMinorPrice(totals.shippingMinor, tenant.currency)}
                    </dd>
                  </div>
                  {totals.codFeeMinor > 0 && (
                    <div className="flex justify-between">
                      <dt className="text-[var(--theme-ink-muted)]">COD Handling</dt>
                      <dd className="font-mono">{formatMinorPrice(totals.codFeeMinor, tenant.currency)}</dd>
                    </div>
                  )}
                  <div className="flex justify-between border-t border-[var(--theme-border)] pt-3 text-base font-semibold">
                    <dt>Total</dt>
                    <dd className="font-mono">{formatMinorPrice(totals.totalMinor, tenant.currency)}</dd>
                  </div>
                </dl>

                <button
                  type="submit"
                  disabled={items.length === 0}
                  data-testid="place-order-button"
                  className="mt-6 w-full bg-[var(--theme-accent)] px-5 py-3.5 text-sm font-medium text-[var(--theme-accent-ink)] transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-40 rounded-[var(--theme-radius-button)]"
                >
                  Place Order · {formatMinorPrice(totals.totalMinor, tenant.currency)}
                </button>

                <p className="mt-3 text-center text-[11px] leading-relaxed text-[var(--theme-ink-muted)]">
                  Calculated server-authoritatively in minor integer units. Themes cannot alter checkout totals or payment fields.
                </p>
              </div>
            </div>
          </form>
        )}
      </div>
    </div>
  )
}
