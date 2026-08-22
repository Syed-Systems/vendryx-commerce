"use client"

import { convertToLocale, deleteLineItem } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import Thumbnail from "@modules/products/components/thumbnail"
import { useCallback, useEffect, useRef, useState } from "react"
import drawer from "./cart-drawer.module.css"
import row from "@modules/cart/components/item/cart-item.module.css"
import navStyles from "@modules/layout/templates/nav/nav.module.css"

/**
 * PORT of The Matrix Hours `CartDrawer` + `CartItem` — their stylesheets, their
 * slide-in panel, their overlay, their focus trap, their body-scroll lock, their
 * empty state, their subtotal footer.
 *
 * This is a behaviour change, not a restyle, and that is the point: Medusa's
 * cart-dropdown is a `@headlessui/react` Popover that opens on hover and
 * auto-closes on a 5s timer. The source design is a right-hand drawer opened by
 * click and dismissed by overlay, Escape or the close button. Repainting the
 * popover would have kept Medusa's interaction under Matrix colours — which is
 * exactly the failure this port exists to undo.
 *
 * Cart mutation still goes through core (`deleteLineItem`); the drawer owns
 * only the surface. Quantity steppers are not wired — see
 * docs/FRONTEND_MATRIX_SLOTS.md.
 */
const CartDropdown = ({
  cart: cartState,
}: {
  cart?: HttpTypes.StoreCart | null
}) => {
  const [isOpen, setIsOpen] = useState(false)
  const [removing, setRemoving] = useState<string | null>(null)
  const panelRef = useRef<HTMLDivElement>(null)

  const items = cartState?.items ?? []
  const totalItems = items.reduce((acc, item) => acc + item.quantity, 0)
  const currencyCode = cartState?.currency_code ?? "usd"

  const close = useCallback(() => setIsOpen(false), [])

  useEffect(() => {
    document.body.style.overflow = isOpen ? "hidden" : ""
    return () => {
      document.body.style.overflow = ""
    }
  }, [isOpen])

  // Focus trap, ported from the source drawer.
  useEffect(() => {
    if (!isOpen || !panelRef.current) return
    const panel = panelRef.current
    const focusable = panel.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    )
    focusable[0]?.focus()

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        close()
        return
      }
      if (e.key !== "Tab" || !focusable.length) return
      const first = focusable[0]
      const last = focusable[focusable.length - 1]
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault()
        last.focus()
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault()
        first.focus()
      }
    }

    document.addEventListener("keydown", onKeyDown)
    return () => document.removeEventListener("keydown", onKeyDown)
  }, [isOpen, close])

  const handleRemove = async (id: string) => {
    setRemoving(id)
    await deleteLineItem(id).catch(() => undefined)
    setRemoving(null)
  }

  return (
    <>
      <button
        type="button"
        className={navStyles.iconButton}
        aria-label={`Cart (${totalItems})`}
        aria-expanded={isOpen}
        onClick={() => setIsOpen(true)}
        data-testid="nav-cart-link"
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="square"
          aria-hidden
        >
          <path d="M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4Z" />
          <path d="M3 6h18M16 10a4 4 0 0 1-8 0" />
        </svg>
        {totalItems > 0 && (
          <span className={navStyles.cartBadge}>{totalItems}</span>
        )}
      </button>

      <div
        className={`${drawer.overlay} ${isOpen ? drawer.overlayOpen : ""}`}
        onClick={close}
        aria-hidden="true"
      />

      <div
        ref={panelRef}
        className={`${drawer.drawer} ${isOpen ? drawer.drawerOpen : ""}`}
        role="dialog"
        aria-modal={isOpen}
        aria-label="Shopping cart"
        data-testid="nav-cart-dropdown"
      >
        <div className={drawer.header}>
          <h2 className={drawer.title}>Your Cart ({totalItems})</h2>
          <button
            onClick={close}
            className={drawer.closeButton}
            aria-label="Close cart"
            tabIndex={isOpen ? 0 : -1}
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="square"
              aria-hidden
            >
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className={drawer.body}>
          {items.length === 0 ? (
            <div className={drawer.emptyState}>
              <p className={drawer.emptyText}>Your cart is empty</p>
              <LocalizedClientLink
                href="/store"
                onClick={close}
                className={drawer.startShoppingBtn}
                tabIndex={isOpen ? 0 : -1}
              >
                Start Shopping
              </LocalizedClientLink>
            </div>
          ) : (
            <div className={drawer.cartList}>
              {items
                .sort((a, b) =>
                  (a.created_at ?? "") > (b.created_at ?? "") ? -1 : 1
                )
                .map((item) => (
                  <div
                    className={row.cartItem}
                    key={item.id}
                    data-testid="cart-item"
                  >
                    <LocalizedClientLink
                      href={`/products/${item.product_handle}`}
                      className={row.imageWrapper}
                      onClick={close}
                      tabIndex={isOpen ? 0 : -1}
                    >
                      <Thumbnail thumbnail={item.thumbnail} size="square" />
                    </LocalizedClientLink>

                    <div className={row.content}>
                      <div>
                        <p className={row.brand}>
                          {item.variant?.title ?? "Reference"}
                        </p>
                        <h4 className={row.name} data-testid="cart-item-name">
                          {item.title}
                        </h4>
                      </div>
                      <p className={row.price}>
                        {convertToLocale({
                          amount: item.total ?? 0,
                          currency_code: currencyCode,
                        })}
                      </p>
                    </div>

                    <div className={row.actions}>
                      <div className={row.qtyControl}>
                        <span className={row.qtyText}>{item.quantity}</span>
                      </div>
                      <button
                        className={row.removeBtn}
                        onClick={() => handleRemove(item.id)}
                        disabled={removing === item.id}
                        aria-label="Remove item"
                        tabIndex={isOpen ? 0 : -1}
                        data-testid="cart-item-remove-button"
                      >
                        <svg
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="square"
                          aria-hidden
                        >
                          <path d="M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14" />
                        </svg>
                      </button>
                    </div>
                  </div>
                ))}
            </div>
          )}
        </div>

        {items.length > 0 && (
          <div className={drawer.footer}>
            <div className={drawer.subtotalRow}>
              <span className={drawer.subtotalLabel}>Subtotal</span>
              <span
                className={drawer.subtotalValue}
                data-testid="cart-subtotal"
                data-value={cartState?.subtotal ?? 0}
              >
                {convertToLocale({
                  amount: cartState?.subtotal ?? 0,
                  currency_code: currencyCode,
                })}
              </span>
            </div>
            <p className={drawer.shippingNote}>
              Shipping calculated at checkout
            </p>
            <LocalizedClientLink
              href="/cart"
              onClick={close}
              tabIndex={isOpen ? 0 : -1}
            >
              <span
                className={drawer.startShoppingBtn}
                data-testid="go-to-cart-button"
              >
                Go to Cart
              </span>
            </LocalizedClientLink>
            <button onClick={close} className={drawer.continueShopping}>
              Continue Shopping
            </button>
          </div>
        )}
      </div>
    </>
  )
}

export default CartDropdown
