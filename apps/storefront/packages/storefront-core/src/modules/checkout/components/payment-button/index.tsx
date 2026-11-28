"use client"

import { isManual, isStripeLike } from "@lib/constants"
import { placeOrder } from "@lib/data/cart"
import { HttpTypes } from "@medusajs/types"
import { Button } from "@medusajs/ui"
import { useElements, useStripe } from "@stripe/react-stripe-js"
import React, { useState } from "react"
import ErrorMessage from "@frontend/common/components/error-message"

type PaymentButtonProps = {
  cart: HttpTypes.StoreCart
  "data-testid": string
}

const PaymentButton: React.FC<PaymentButtonProps> = ({
  cart,
  "data-testid": dataTestId,
}) => {
  const [whatsappOptIn, setWhatsappOptIn] = useState(false)
  const [whatsappAnalyticsOptIn, setWhatsappAnalyticsOptIn] = useState(false)
  const notReady =
    !cart ||
    !cart.shipping_address ||
    !cart.billing_address ||
    !cart.email ||
    (cart.shipping_methods?.length ?? 0) < 1

  const paymentSession = cart.payment_collection?.payment_sessions?.[0]

  let paymentButton: React.ReactNode
  switch (true) {
    case isStripeLike(paymentSession?.provider_id):
      paymentButton = (
        <StripePaymentButton
          notReady={notReady}
          cart={cart}
          whatsappOptIn={whatsappOptIn}
          whatsappAnalyticsOptIn={whatsappAnalyticsOptIn}
          data-testid={dataTestId}
        />
      )
      break
    case isManual(paymentSession?.provider_id):
      paymentButton = (
        <ManualTestPaymentButton
          notReady={notReady}
          whatsappOptIn={whatsappOptIn}
          whatsappAnalyticsOptIn={whatsappAnalyticsOptIn}
          data-testid={dataTestId}
        />
      )
      break
    default:
      paymentButton = <Button disabled>Select a payment method</Button>
  }

  return (
    <div className="flex flex-col gap-3">
      {cart.shipping_address?.phone ? (
        <div className="flex flex-col gap-3">
          <label className="flex items-start gap-2 text-sm text-ui-fg-subtle">
            <input
              type="checkbox"
              checked={whatsappOptIn}
              onChange={(event) => setWhatsappOptIn(event.target.checked)}
              aria-label="Receive order updates on WhatsApp"
              className="mt-0.5"
            />
            <span>
              Send my order updates on WhatsApp. I agree to receive transactional
              messages at the phone number above.
            </span>
          </label>
          <label className="flex items-start gap-2 text-sm text-ui-fg-subtle">
            <input
              type="checkbox"
              checked={whatsappAnalyticsOptIn}
              onChange={(event) => setWhatsappAnalyticsOptIn(event.target.checked)}
              aria-label="Allow anonymous WhatsApp activity aggregates"
              className="mt-0.5"
            />
            <span>
              Allow anonymous aggregate counts of my WhatsApp messages with this store to appear
              in its insights. This is optional and separate from order updates. You can withdraw
              this consent in your account settings or by sending STOP on WhatsApp.
            </span>
          </label>
        </div>
      ) : null}
      {paymentButton}
    </div>
  )
}

const StripePaymentButton = ({
  cart,
  notReady,
  whatsappOptIn,
  whatsappAnalyticsOptIn,
  "data-testid": dataTestId,
}: {
  cart: HttpTypes.StoreCart
  notReady: boolean
  whatsappOptIn: boolean
  whatsappAnalyticsOptIn: boolean
  "data-testid"?: string
}) => {
  const [submitting, setSubmitting] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const onPaymentCompleted = async () => {
    await placeOrder(undefined, whatsappOptIn, whatsappAnalyticsOptIn)
      .catch((err) => {
        setErrorMessage(err.message)
      })
      .finally(() => {
        setSubmitting(false)
      })
  }

  const stripe = useStripe()
  const elements = useElements()
  const card = elements?.getElement("card")

  const session = cart.payment_collection?.payment_sessions?.find(
    (s) => s.status === "pending"
  )

  const disabled = !stripe || !elements ? true : false

  const handlePayment = async () => {
    setSubmitting(true)

    if (!stripe || !elements || !card || !cart) {
      setSubmitting(false)
      return
    }

    await stripe
      .confirmCardPayment(session?.data.client_secret as string, {
        payment_method: {
          card: card,
          billing_details: {
            name:
              cart.billing_address?.first_name +
              " " +
              cart.billing_address?.last_name,
            address: {
              city: cart.billing_address?.city ?? undefined,
              country: cart.billing_address?.country_code ?? undefined,
              line1: cart.billing_address?.address_1 ?? undefined,
              line2: cart.billing_address?.address_2 ?? undefined,
              postal_code: cart.billing_address?.postal_code ?? undefined,
              state: cart.billing_address?.province ?? undefined,
            },
            email: cart.email,
            phone: cart.billing_address?.phone ?? undefined,
          },
        },
      })
      .then(({ error, paymentIntent }) => {
        if (error) {
          const pi = error.payment_intent

          if (
            (pi && pi.status === "requires_capture") ||
            (pi && pi.status === "succeeded")
          ) {
            onPaymentCompleted()
          }

          setErrorMessage(error.message || null)
          return
        }

        if (
          (paymentIntent && paymentIntent.status === "requires_capture") ||
          paymentIntent.status === "succeeded"
        ) {
          return onPaymentCompleted()
        }

        return
      })
  }

  return (
    <>
      <Button
        disabled={disabled || notReady}
        onClick={handlePayment}
        size="large"
        isLoading={submitting}
        data-testid={dataTestId}
      >
        Place order
      </Button>
      <ErrorMessage
        error={errorMessage}
        data-testid="stripe-payment-error-message"
      />
    </>
  )
}

const ManualTestPaymentButton = ({
  notReady,
  whatsappOptIn,
  whatsappAnalyticsOptIn,
  "data-testid": dataTestId,
}: {
  notReady: boolean
  whatsappOptIn: boolean
  whatsappAnalyticsOptIn: boolean
  "data-testid"?: string
}) => {
  const [submitting, setSubmitting] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const onPaymentCompleted = async () => {
    await placeOrder(undefined, whatsappOptIn, whatsappAnalyticsOptIn)
      .catch((err) => {
        setErrorMessage(err.message)
      })
      .finally(() => {
        setSubmitting(false)
      })
  }

  const handlePayment = () => {
    setSubmitting(true)

    onPaymentCompleted()
  }

  return (
    <>
      <Button
        disabled={notReady}
        isLoading={submitting}
        onClick={handlePayment}
        size="large"
        data-testid={dataTestId ?? "submit-order-button"}
      >
        Place order
      </Button>
      <ErrorMessage
        error={errorMessage}
        data-testid="manual-payment-error-message"
      />
    </>
  )
}

export default PaymentButton
