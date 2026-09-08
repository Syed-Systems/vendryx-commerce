"use client"

import React, { useActionState, useEffect, useState } from "react"
import { createTransferRequest } from "@stheno/storefront-core"
import { SubmitButton } from "@modules/common/components/submit-button"

/**
 * Lustre TransferRequestForm (Slot 16).
 * Form allowing registered clients to request transfer of custody for previous unlinked orders.
 */
export default function TransferRequestForm() {
  const [showSuccess, setShowSuccess] = useState(false)

  const [state, formAction] = useActionState(createTransferRequest, {
    success: false,
    error: null,
    order: null,
  })

  useEffect(() => {
    if (state.success && state.order) {
      setShowSuccess(true)
    }
  }, [state.success, state.order])

  return (
    <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 text-xs space-y-4">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
            Ownership Reconciliation
          </span>
          <h3 className="text-base font-semibold text-[var(--text-primary)] mt-0.5">
            Claim Order Ownership
          </h3>
          <p className="text-[var(--text-secondary)] mt-1 max-w-sm">
            Placed an order prior to registering? Enter your order identification code to link custody to this account.
          </p>
        </div>

        <form action={formAction} className="flex flex-col sm:flex-row gap-2.5 sm:items-center">
          <input
            type="text"
            name="order_id"
            placeholder="Order ID"
            required
            className="h-10 rounded-full border border-[var(--hairline)] bg-[var(--bg-primary)] px-4 text-xs text-[var(--text-primary)] placeholder-[var(--text-secondary)] focus:border-[var(--accent-sapphire)] focus:outline-none focus:ring-1 focus:ring-[var(--accent-sapphire)]"
          />
          <SubmitButton
            className="h-10 rounded-full bg-[var(--accent-sapphire)] px-5 text-xs font-semibold text-white shadow-sm hover:bg-[var(--accent-sapphire-hover)] transition"
          >
            Initiate Transfer
          </SubmitButton>
        </form>
      </div>

      {!state.success && state.error && (
        <p className="text-rose-500 font-medium">
          {state.error}
        </p>
      )}

      {showSuccess && (
        <div className="rounded-[var(--radius-sm)] border border-emerald-500/20 bg-emerald-500/10 p-3 text-emerald-600 dark:text-emerald-400">
          Transfer request created.
        </div>
      )}
    </div>
  )
}
