"use client"

import { convertToLocale } from "@stheno/storefront-core"
import {
  HttpTypes,
  StoreCart,
  StoreCartShippingOption,
  StorePrice,
} from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { X } from "@phosphor-icons/react/dist/ssr"
import { useState } from "react"

const computeTarget = (
  cart: HttpTypes.StoreCart,
  price: HttpTypes.StorePrice
) => {
  const priceRule = (price.price_rules || []).find(
    (pr) => pr.attribute === "item_total"
  )

  if (!priceRule) {
    return {
      current_amount: cart.item_total,
      target_amount: 0,
      target_reached: true,
      target_remaining: 0,
      remaining_percentage: 100,
    }
  }

  const currentAmount = cart.item_total
  const targetAmount = parseFloat(priceRule.value)

  if (priceRule.operator === "gt" || priceRule.operator === "gte") {
    const reached = currentAmount >= targetAmount
    const remaining = reached ? 0 : targetAmount - currentAmount
    return {
      current_amount: currentAmount,
      target_amount: targetAmount,
      target_reached: reached,
      target_remaining: remaining,
      remaining_percentage: Math.min(100, (currentAmount / targetAmount) * 100),
    }
  }

  return {
    current_amount: currentAmount,
    target_amount: targetAmount,
    target_reached: currentAmount >= targetAmount,
    target_remaining: Math.max(0, targetAmount - currentAmount),
    remaining_percentage: Math.min(100, (currentAmount / targetAmount) * 100),
  }
}

export type ShippingPriceNudgeProps = {
  variant?: "popup" | "inline"
  cart: StoreCart
  shippingOptions: StoreCartShippingOption[]
}

/**
 * Lustre FreeShippingPriceNudge.
 * Calculates dynamic shipping incentive thresholds without @medusajs/ui.
 * Conforms to contract slot 48.
 */
export default function ShippingPriceNudge({
  variant = "inline",
  cart,
  shippingOptions,
}: ShippingPriceNudgeProps) {
  if (!cart || !shippingOptions?.length) {
    return null
  }

  const freeShippingPrice = shippingOptions
    .map((shippingOption) => {
      const calculatedPrice = shippingOption.calculated_price
      if (!calculatedPrice) {
        return null
      }

      const validCurrencyPrices = shippingOption.prices.filter(
        (price) =>
          price.currency_code === cart.currency_code &&
          (price.price_rules || []).some(
            (priceRule) => priceRule.attribute === "item_total"
          )
      )

      return validCurrencyPrices.map((price) => ({
        ...price,
        shipping_option_id: shippingOption.id,
        ...computeTarget(cart, price),
      }))
    })
    .flat()
    .filter((price): price is NonNullable<typeof price> => Boolean(price))
    .find((price) => price.amount === 0)

  if (!freeShippingPrice) {
    return null
  }

  if (variant === "popup") {
    return <FreeShippingPopup cart={cart} price={freeShippingPrice} />
  }

  return <FreeShippingInline cart={cart} price={freeShippingPrice} />
}

function FreeShippingInline({
  cart,
  price,
}: {
  cart: StoreCart
  price: StorePrice & {
    target_reached: boolean
    target_remaining: number
    remaining_percentage: number
  }
}) {
  return (
    <div className="p-3 my-2 rounded-[12px] bg-[var(--bg-raised)] border border-[var(--hairline)] text-xs text-[var(--text-primary)]">
      <div className="flex items-center justify-between font-medium">
        <span>
          {price.target_reached
            ? "Free delivery eligible."
            : "Free delivery threshold:"}
        </span>
        {!price.target_reached && (
          <span className="text-[var(--accent-sapphire)] tabular-nums font-semibold">
            {convertToLocale({
              amount: price.target_remaining,
              currency_code: cart.currency_code,
            })}{" "}
            remaining
          </span>
        )}
      </div>

      <div className="mt-2 w-full h-1 rounded-full bg-[var(--hairline)] overflow-hidden">
        <div
          className={`h-full transition-all duration-300 ${
            price.target_reached ? "bg-[var(--status-success)]" : "bg-[var(--accent-sapphire)]"
          }`}
          style={{ width: `${price.remaining_percentage}%` }}
        />
      </div>
    </div>
  )
}

function FreeShippingPopup({
  cart,
  price,
}: {
  cart: StoreCart
  price: any
}) {
  const [isDismissed, setIsDismissed] = useState(false)

  if (price.target_reached || isDismissed) {
    return null
  }

  return (
    <div className="fixed bottom-6 right-6 z-40 max-w-sm rounded-[16px] bg-[var(--bg-surface)] border border-[var(--hairline)] shadow-[var(--elevation-modal)] p-4 text-xs text-[var(--text-primary)]">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-semibold text-sm">Free delivery</p>
          <p className="text-[var(--text-secondary)] mt-1">
            Add{" "}
            <span className="font-semibold text-[var(--accent-sapphire)] tabular-nums">
              {convertToLocale({
                amount: price.target_remaining,
                currency_code: cart.currency_code,
              })}
            </span>{" "}
            more to qualify.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setIsDismissed(true)}
          aria-label="Dismiss notification"
          className="p-1 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
        >
          <X size="14" />
        </button>
      </div>

      <div className="mt-3 flex gap-2">
        <LocalizedClientLink
          href="/store"
          className="flex-1 py-2 text-center rounded-full text-xs font-medium bg-[var(--accent-sapphire)] text-white hover:bg-[var(--accent-sapphire-hover)] transition-colors"
        >
          Explore Catalog
        </LocalizedClientLink>
      </div>
    </div>
  )
}
