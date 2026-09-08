"use client"

import { transferCart } from "@stheno/storefront-core"
import { StoreCart, StoreCustomer } from "@medusajs/types"
import { useState } from "react"
import { Info } from "@phosphor-icons/react/dist/ssr"

export type CartMismatchBannerProps = {
  customer?: StoreCustomer | null
  cart?: StoreCart | null
}

/**
 * Lustre CartMismatchBanner.
 * Persistent contextual warning when a cart belongs to an unassociated session.
 * Conforms to contract slot 40 without @medusajs/ui dependencies.
 */
function CartMismatchBanner({ customer, cart }: CartMismatchBannerProps) {
  const [isPending, setIsPending] = useState(false)
  const [actionText, setActionText] = useState("Transfer to your account")

  if (!customer || !cart || Boolean(cart.customer_id)) {
    return null
  }

  const handleSubmit = async () => {
    try {
      setIsPending(true)
      setActionText("Transferring...")
      await transferCart()
    } catch {
      setActionText("Retry cart transfer")
      setIsPending(false)
    }
  }

  return (
    <aside
      role="alert"
      className="w-full bg-[var(--bg-raised)] border-b border-[var(--hairline)] py-2.5 px-4 text-xs text-[var(--text-primary)]"
    >
      <div className="max-w-[1440px] mx-auto flex flex-col sm:flex-row items-center justify-center gap-2 text-center">
        <span className="flex items-center gap-1.5 font-medium">
          <Info size={16} className="shrink-0 text-[var(--accent-sapphire)]" aria-hidden="true" />
          Cart items were created before signing in.
        </span>

        <span className="hidden sm:inline text-[var(--text-muted)]" aria-hidden="true">
          &middot;
        </span>

        <button
          type="button"
          disabled={isPending}
          onClick={handleSubmit}
          className="font-medium text-[var(--accent-sapphire)] hover:text-[var(--accent-sapphire-hover)] underline underline-offset-2 transition-colors disabled:opacity-50"
        >
          {actionText}
        </button>
      </div>
    </aside>
  )
}

export default CartMismatchBanner
