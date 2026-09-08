import React from "react"
import { HttpTypes } from "@medusajs/types"
import ItemsTemplate from "./items"
import Summary from "./summary"
import EmptyCartMessage from "../components/empty-cart-message"
import SignInPrompt from "../components/sign-in-prompt"

export type CartTemplateProps = {
  cart: HttpTypes.StoreCart | null
  customer: HttpTypes.StoreCustomer | null
}

/**
 * Lustre CartTemplate (Slot 19).
 * Full shopping bag experience with two-column editorial view, sign-in prompt, and sticky checkout panel.
 */
const CartTemplate: React.FC<CartTemplateProps> = ({ cart, customer }) => {
  const hasItems = (cart?.items?.length ?? 0) > 0

  return (
    <main className="min-h-screen bg-[var(--bg-primary)] py-10 sm:py-16">
      <div className="content-container" data-testid="cart-container">
        {/* Header Section */}
        <header className="mb-10 border-b border-[var(--hairline)] pb-6">
          <span className="text-xs font-semibold uppercase tracking-[0.25em] text-[var(--accent-sapphire)]">
            Selection Summary
          </span>
          <h1 className="mt-1 lustre-section-title text-[var(--text-primary)]">
            Shopping Bag
          </h1>
        </header>

        {hasItems ? (
          <div className="grid grid-cols-1 gap-12 lg:grid-cols-12 lg:gap-14 items-start">
            {/* Left Items Column */}
            <section aria-label="Cart Items" className="space-y-8 lg:col-span-8">
              {!customer && <SignInPrompt />}
              <div className="rounded-[var(--radius-lg)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 shadow-sm">
                <ItemsTemplate cart={cart!} />
              </div>
            </section>

            {/* Right Summary Panel */}
            <div className="lg:col-span-4 lg:sticky lg:top-24">
              {cart && cart.region && <Summary cart={cart as any} />}
            </div>
          </div>
        ) : (
          <EmptyCartMessage />
        )}
      </div>
    </main>
  )
}

export default CartTemplate
