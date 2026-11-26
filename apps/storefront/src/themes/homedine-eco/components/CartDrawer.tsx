"use client"

import { useEffect } from "react"
import { ArrowRight, Leaf, Minus, Plus, ShoppingBag, Trash2, X } from "lucide-react"
import {
  formatMinorPrice,
  type CartLineItem,
  type CheckoutTotals,
} from "../../../core/checkout-engine"
import type { StoreCurrency } from "../../../core/tenant-resolver"
import { HOMEDINE_STATS } from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

interface CartDrawerProps {
  isOpen: boolean
  items: CartLineItem[]
  totals: CheckoutTotals
  currency: StoreCurrency
  onClose: () => void
  onUpdateQuantity: (variantId: string, delta: number) => void
  onRemoveItem: (variantId: string) => void
  onProceedToCheckout: () => void
}

export function CartDrawer({
  isOpen,
  items,
  totals,
  currency,
  onClose,
  onUpdateQuantity,
  onRemoveItem,
  onProceedToCheckout,
}: CartDrawerProps) {
  useEffect(() => {
    if (!isOpen) return
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose()
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [isOpen, onClose])

  if (!isOpen) return null

  const remainingMinor = Math.max(
    0,
    HOMEDINE_STATS.freeShippingThresholdMinor - totals.subtotalMinor
  )
  const progressPct =
    totals.subtotalMinor <= 0
      ? 0
      : Math.min(
          100,
          Math.round((totals.subtotalMinor / HOMEDINE_STATS.freeShippingThresholdMinor) * 100)
        )

  return (
    <div
      className="fixed inset-0 z-50 flex justify-end"
      role="dialog"
      aria-modal="true"
      aria-label="Shopping cart drawer"
      data-testid="homedine-cart-drawer"
    >
      {/* Backdrop */}
      <button
        type="button"
        aria-label="Close shopping bag"
        onClick={onClose}
        className="fixed inset-0 bg-black/45 backdrop-blur-[2px] cursor-pointer"
      />

      {/* Drawer Panel */}
      <div
        data-lenis-prevent
        className="relative z-10 w-full max-w-[440px] h-full bg-[var(--color-canvas)] shadow-[var(--shadow-drawer)] flex flex-col"
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-5 border-b border-black/[0.08]">
          <div className="flex items-center gap-2.5">
            <ShoppingBag className="w-5 h-5 text-[#0e5437]" strokeWidth={1.8} />
            <h2 className="text-[18px] font-semibold text-[#1a1f1c]">
              Your Bag ({totals.itemCount})
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close cart drawer"
            className="w-9 h-9 rounded-full hover:bg-black/5 flex items-center justify-center text-[#1a1f1c] transition-colors cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Carbon-Neutral Free Shipping Progress Bar */}
        {items.length > 0 && (
          <div className="px-6 py-3.5 bg-[#eff1e7] border-b border-black/[0.06]">
            <div className="flex items-center gap-2 text-[12px] font-medium text-[#1a1f1c]">
              <Leaf className="w-3.5 h-3.5 text-[#0e5437] shrink-0" />
              {totals.qualifiesForFreeShipping ? (
                <span>Unlocked complimentary carbon-neutral express shipping!</span>
              ) : (
                <span>
                  Add{" "}
                  <strong className="tabular-nums text-[#0e5437]">
                    {formatMinorPrice(remainingMinor, currency)}
                  </strong>{" "}
                  more for free carbon-neutral shipping
                </span>
              )}
            </div>
            <div className="mt-2 h-1.5 w-full rounded-full bg-black/10 overflow-hidden">
              <div
                className="h-full rounded-full bg-[#0e5437] transition-all duration-500"
                style={{ width: `${progressPct}%` }}
              />
            </div>
          </div>
        )}

        {/* Drawer Body */}
        <div className="flex-1 overflow-y-auto px-6 py-5">
          {items.length === 0 ? (
            <div className="py-16 text-center">
              <div className="mx-auto w-14 h-14 rounded-full bg-[#eff1e7] text-[#0e5437] flex items-center justify-center mb-4">
                <ShoppingBag className="w-6 h-6" strokeWidth={1.6} />
              </div>
              <p className="text-[16px] font-semibold text-[#1a1f1c]">Your bag is empty</p>
              <p className="mt-1.5 text-[13px] text-[#4e5651] max-w-[260px] mx-auto">
                Explore our non-toxic ceramic cookware, thermal bottles, and bamboo essentials.
              </p>
              <button
                type="button"
                onClick={onClose}
                className="mt-6 h-10 px-6 rounded-full bg-[var(--color-brand-forest)] hover:bg-[var(--color-brand-forest-hover)] text-white text-[13px] font-medium transition-colors cursor-pointer"
              >
                Continue shopping
              </button>
            </div>
          ) : (
            <ul className="divide-y divide-black/[0.08]">
              {items.map((item) => (
                <li key={item.variantId} className="py-4 first:pt-0 flex gap-4">
                  <ResilientImage
                    src={item.imagePath}
                    alt={item.title}
                    className="w-20 h-24 rounded-xl object-cover bg-[#cac8be] shrink-0"
                  />
                  <div className="flex-1 min-w-0 flex flex-col justify-between">
                    <div>
                      <div className="flex items-start justify-between gap-2">
                        <h3 className="text-[14px] font-semibold text-[#1a1f1c] leading-snug">
                          {item.title}
                        </h3>
                        <button
                          type="button"
                          onClick={() => onRemoveItem(item.variantId)}
                          aria-label={`Remove ${item.title} from cart`}
                          className="text-[#767e78] hover:text-[#a33327] p-1 transition-colors cursor-pointer"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                      <p className="mt-1 text-[12px] text-[#4e5651]">{item.variantLabel}</p>
                    </div>

                    <div className="mt-3 flex items-center justify-between">
                      <div className="inline-flex items-center rounded-full border border-black/[0.12] bg-white">
                        <button
                          type="button"
                          onClick={() => onUpdateQuantity(item.variantId, -1)}
                          aria-label={`Decrease quantity of ${item.title}`}
                          className="w-8 h-8 flex items-center justify-center text-[#1a1f1c] hover:bg-black/5 rounded-l-full cursor-pointer"
                        >
                          <Minus className="w-3.5 h-3.5" />
                        </button>
                        <span className="w-8 text-center text-[13px] font-semibold tabular-nums">
                          {item.quantity}
                        </span>
                        <button
                          type="button"
                          onClick={() => onUpdateQuantity(item.variantId, 1)}
                          aria-label={`Increase quantity of ${item.title}`}
                          className="w-8 h-8 flex items-center justify-center text-[#1a1f1c] hover:bg-black/5 rounded-r-full cursor-pointer"
                        >
                          <Plus className="w-3.5 h-3.5" />
                        </button>
                      </div>
                      <span className="text-[14px] font-bold text-[#1a1f1c] tabular-nums">
                        {formatMinorPrice(item.unitPriceMinor * item.quantity, currency)}
                      </span>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>

        {/* Drawer Footer Totals & Locked Core Checkout Action */}
        {items.length > 0 && (
          <div className="border-t border-black/[0.08] bg-white px-6 py-5 space-y-3">
            <div className="space-y-1.5 text-[13px]">
              <div className="flex justify-between text-[#4e5651]">
                <span>Subtotal</span>
                <span className="tabular-nums font-medium text-[#1a1f1c]">
                  {formatMinorPrice(totals.subtotalMinor, currency)}
                </span>
              </div>
              <div className="flex justify-between text-[#4e5651]">
                <span>Carbon-neutral shipping (free over $150)</span>
                <span className="tabular-nums font-medium text-[#0e5437]">
                  {totals.shippingMinor === 0
                    ? "Free"
                    : formatMinorPrice(totals.shippingMinor, currency)}
                </span>
              </div>
              <div className="flex justify-between pt-2 border-t border-black/[0.06] text-[16px] font-bold text-[#1a1f1c]">
                <span>Estimated Total</span>
                <span data-testid="cart-total-amount" className="tabular-nums">
                  {formatMinorPrice(totals.totalMinor, currency)}
                </span>
              </div>
            </div>

            <button
              type="button"
              onClick={onProceedToCheckout}
              data-testid="homedine-proceed-checkout"
              className={`${styles.btnArrowGlide} w-full h-11 rounded-full bg-[var(--color-brand-forest)] hover:bg-[var(--color-brand-forest-hover)] text-white text-[14px] font-semibold flex items-center justify-center gap-2 transition-colors cursor-pointer`}
            >
              <span>Proceed to checkout</span>
              <ArrowRight className={`${styles.arrowIcon} w-4 h-4`} />
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
