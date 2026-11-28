"use client"

import { Button } from "@medusajs/ui"
import { useActionState } from "react"

import { revokeCustomerWhatsAppConsent } from "@stheno/storefront-core"

type ActionState = {
  success: boolean
  error: string | null
}

const initialState: ActionState = { success: false, error: null }

const WhatsAppNotifications = () => {
  const revokeConsent = async (): Promise<ActionState> => {
    try {
      await revokeCustomerWhatsAppConsent()
      return { success: true, error: null }
    } catch {
      return {
        success: false,
        error: "We couldn't update your WhatsApp notification preference. Please try again.",
      }
    }
  }

  const [state, formAction, pending] = useActionState(revokeConsent, initialState)

  return (
    <section className="flex flex-col gap-y-4" aria-labelledby="whatsapp-notifications-heading">
      <div className="flex flex-col gap-y-2">
        <h2 id="whatsapp-notifications-heading" className="text-base-semi">
          WhatsApp preferences
        </h2>
        <p className="text-small-regular text-ui-fg-subtle">
          Withdraw transactional WhatsApp updates and any separate anonymous activity
          aggregation consent for every phone number linked to your account. Email notifications
          are not affected. You can choose each option again during checkout.
        </p>
      </div>
      <form action={formAction}>
        <Button type="submit" variant="secondary" disabled={pending}>
          {pending ? "Saving…" : "Withdraw WhatsApp consent"}
        </Button>
      </form>
      {state.success && (
        <p role="status" className="text-small-regular text-ui-fg-subtle">
          WhatsApp update and analytics consent has been withdrawn for this account.
        </p>
      )}
      {state.error && (
        <p role="alert" className="text-small-regular text-ui-fg-error">
          {state.error}
        </p>
      )}
    </section>
  )
}

export default WhatsAppNotifications
