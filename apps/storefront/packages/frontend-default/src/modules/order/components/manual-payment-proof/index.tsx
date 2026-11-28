"use client"

import { Button, Heading, Text } from "@medusajs/ui"
import { isManual, OrderPaymentProof, submitManualOrderPaymentProof } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import { useActionState } from "react"

type SubmitState = {
  success: boolean
  error: string | null
  status?: string
}

const initialState: SubmitState = { success: false, error: null }

export default function ManualPaymentProof({
  order,
  proof,
}: {
  order: HttpTypes.StoreOrder
  proof: OrderPaymentProof | null
}) {
  const collections = (order.payment_collections ?? []) as Array<{
    payment_sessions?: Array<{
      provider_id?: string
      data?: {
        amount?: string | number
        currency_code?: string
        payment_account?: string
        payment_instructions?: string
      }
    }>
  }>
  const manualSession = collections
    .flatMap((collection) => collection.payment_sessions ?? [])
    .find((session) => session.provider_id?.startsWith("pp_manual"))
  const manualPayment = Boolean(manualSession && isManual(manualSession.provider_id))
  const isPaid = ["captured", "partially_captured", "paid"].includes(
    order.payment_status
  )

  if (!manualPayment || !order.customer_id || (isPaid && proof?.status !== "verified")) {
    return null
  }

  const submit = async (_state: SubmitState, formData: FormData) =>
    submitManualOrderPaymentProof(order.id, formData)
  const [state, formAction, isPending] = useActionState(submit, initialState)
  const hasPendingProof = proof?.status === "pending_verification" || state.success

  return (
    <section className="mt-8 rounded-xl border border-ui-border-base p-5" aria-labelledby="manual-payment-proof-title">
      <Heading id="manual-payment-proof-title" level="h2" className="text-xl">
        Manual payment verification
      </Heading>
      {manualSession?.data?.payment_account && (
        <Text className="mt-3">
          Send {manualSession.data.amount ?? "the order total"} {manualSession.data.currency_code?.toUpperCase()} using your selected transfer method to {manualSession.data.payment_account}.
          {manualSession.data.payment_instructions ? ` ${manualSession.data.payment_instructions}` : ""}
        </Text>
      )}
      {hasPendingProof ? (
        <Text className="mt-3" role="status" aria-live="polite">
          Payment Under Verification — your transaction reference has been submitted. We’ll
          update the order after the store verifies the payment.
        </Text>
      ) : proof?.status === "verified" ? (
        <Text className="mt-3" role="status">The store has verified this payment.</Text>
      ) : (
        <>
        <form action={formAction} className="mt-4 grid gap-4">
          {proof?.status === "rejected" && (
            <Text role="alert">
              The previous payment reference was not accepted. Check your transaction details
              and submit the correct reference.
            </Text>
          )}
          <label className="grid gap-1" htmlFor="payment_method">
            <span>Payment method</span>
            <select
              id="payment_method"
              name="payment_method"
              required
              className="rounded-md border border-ui-border-base px-3 py-2"
              defaultValue="bkash"
            >
              <option value="bkash">bKash</option>
              <option value="nagad">Nagad</option>
              <option value="rocket">Rocket</option>
              <option value="bank_transfer">Bank transfer</option>
            </select>
          </label>
          <label className="grid gap-1" htmlFor="transaction_id">
            <span>Transaction ID</span>
            <input
              id="transaction_id"
              name="transaction_id"
              required
              minLength={1}
              maxLength={128}
              autoComplete="off"
              className="rounded-md border border-ui-border-base px-3 py-2"
            />
          </label>
          {state.error && <Text role="alert">{state.error}</Text>}
          <Text className="text-ui-fg-subtle">
            Submitting a reference does not mark the order as paid. Never share your PIN or
            one-time password.
          </Text>
          <Button type="submit" isLoading={isPending} disabled={isPending}>
            Submit for verification
          </Button>
        </form>
        </>
      )}
    </section>
  )
}
