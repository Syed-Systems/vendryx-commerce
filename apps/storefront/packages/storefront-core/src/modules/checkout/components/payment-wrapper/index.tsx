"use client"

import { loadStripe } from "@stripe/stripe-js"
import React from "react"
import StripeWrapper from "./stripe-wrapper"
import { HttpTypes } from "@medusajs/types"
import { isStripeLike } from "@lib/constants"

type PaymentWrapperProps = {
  cart: HttpTypes.StoreCart
  children: React.ReactNode
}

// The fork carried a fallback to NEXT_PUBLIC_MEDUSA_PAYMENTS_PUBLISHABLE_KEY — Medusa
// Cloud's hosted Stripe Connect. We are not on Medusa Cloud, so it was a fallback to a
// service that does not exist for us, behind a key that already exists.
const stripeKey = process.env.NEXT_PUBLIC_STRIPE_KEY

// Stripe Connect account, kept: the capability is Stripe's, not Medusa's.
const stripeAccountId = process.env.NEXT_PUBLIC_STRIPE_ACCOUNT_ID
const stripePromise = stripeKey
  ? loadStripe(
      stripeKey,
      stripeAccountId ? { stripeAccount: stripeAccountId } : undefined
    )
  : null

const PaymentWrapper: React.FC<PaymentWrapperProps> = ({ cart, children }) => {
  const paymentSession = cart.payment_collection?.payment_sessions?.find(
    (s) => s.status === "pending"
  )

  if (
    isStripeLike(paymentSession?.provider_id) &&
    paymentSession &&
    stripePromise
  ) {
    return (
      <StripeWrapper
        paymentSession={paymentSession}
        stripeKey={stripeKey}
        stripePromise={stripePromise}
      >
        {children}
      </StripeWrapper>
    )
  }

  return <div>{children}</div>
}

export default PaymentWrapper
