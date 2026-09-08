import React from "react"
import InteractiveLink from "@modules/common/components/interactive-link"
import { ShoppingBag } from "@phosphor-icons/react/dist/ssr"

const EmptyCartMessage: React.FC = () => {
  return (
    <div
      className="flex flex-col items-center justify-center py-28 text-center"
      data-testid="empty-cart-message"
    >
      <div className="mb-4 inline-flex h-12 w-12 items-center justify-center rounded-full bg-[var(--hairline)]/50 text-[var(--accent-sapphire)]">
        <ShoppingBag size={24} aria-hidden="true" />
      </div>

      <h2 className="lustre-section-title text-[var(--text-primary)]">
        Your Shopping Bag is Empty
      </h2>

      <p className="mt-3 max-w-md text-xs text-[var(--text-secondary)] leading-relaxed">
        Products you add to the bag will appear here.
      </p>

      <div className="mt-8">
        <InteractiveLink href="/store">Browse catalog</InteractiveLink>
      </div>
    </div>
  )
}

export default EmptyCartMessage
