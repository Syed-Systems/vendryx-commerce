"use client"

import {
  Dialog,
  DialogPanel,
  Transition,
  TransitionChild,
} from "@headlessui/react"
import { convertToLocale } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import DeleteButton from "@modules/common/components/delete-button"
import LineItemOptions from "@modules/common/components/line-item-options"
import LineItemPrice from "@modules/common/components/line-item-price"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import Thumbnail from "@modules/products/components/thumbnail"
import { X } from "@phosphor-icons/react/dist/ssr"
import { Fragment, useState } from "react"

export type CartDropdownProps = {
  cart?: HttpTypes.StoreCart | null
}

/**
 * Lustre Bag Preview (FRONTEND_LUSTRE_PLAN §4.5).
 * Right-side Headless UI dialog with focus trap, Escape dismissal, scroll lock,
 * and focus restoration.
 */
const CartDropdown = ({ cart: cartState }: CartDropdownProps) => {
  const [isOpen, setIsOpen] = useState(false)
  const totalItems =
    cartState?.items?.reduce((acc, item) => acc + item.quantity, 0) || 0
  const subtotal = cartState?.subtotal ?? 0

  return (
    <>
      <button
        type="button"
        onClick={() => setIsOpen(true)}
        data-testid="nav-cart-link"
        aria-label={`Shopping Bag, ${totalItems} items`}
        className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium tracking-wide bg-[var(--bg-surface)] border border-[var(--hairline)] text-[var(--text-primary)] hover:border-[var(--accent-sapphire)] transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--accent-sapphire)]"
      >
        <span>Bag</span>
        <span className="w-5 h-5 rounded-full bg-[var(--accent-sapphire-subtle)] text-[var(--accent-sapphire)] font-semibold flex items-center justify-center text-[10px] tabular-nums">
          {totalItems}
        </span>
      </button>

      <Transition show={isOpen} as={Fragment}>
        <Dialog
          as="div"
          className="relative z-50"
          onClose={() => setIsOpen(false)}
        >
          {/* Backdrop with subtle blur */}
          <TransitionChild
            as={Fragment}
            enter="transition-opacity ease-out duration-200"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="transition-opacity ease-in duration-150"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div
              className="fixed inset-0 bg-black/40 backdrop-blur-sm"
              data-testid="cart-drawer-backdrop"
            />
          </TransitionChild>

          <div className="fixed inset-0 overflow-hidden">
            <div className="absolute inset-0 overflow-hidden">
              <div className="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-10">
                <TransitionChild
                  as={Fragment}
                  enter="transform transition ease-out duration-260"
                  enterFrom="translate-x-full"
                  enterTo="translate-x-0"
                  leave="transform transition ease-in duration-200"
                  leaveFrom="translate-x-0"
                  leaveTo="translate-x-full"
                >
                  <DialogPanel
                    data-testid="nav-cart-dropdown"
                    className="pointer-events-auto w-screen max-w-md bg-[var(--bg-surface)] text-[var(--text-primary)] border-l border-[var(--hairline)] shadow-2xl flex flex-col justify-between p-6 sm:p-8 overflow-y-auto"
                  >
                    {/* Header */}
                    <div className="flex items-center justify-between pb-4 border-b border-[var(--hairline)]">
                      <div className="flex items-center gap-2">
                        <h3 className="text-sm font-semibold tracking-wider uppercase text-[var(--text-primary)]">
                          Shopping Bag
                        </h3>
                        <span className="text-xs text-[var(--text-muted)]">
                          ({totalItems})
                        </span>
                      </div>
                      <button
                        type="button"
                        onClick={() => setIsOpen(false)}
                        aria-label="Close bag"
                        className="p-2 rounded-full text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-raised)] transition-colors lustre-btn-press"
                      >
                        <X size={18} />
                      </button>
                    </div>

                    {/* Content */}
                    {cartState && cartState.items?.length ? (
                      <>
                        <div className="flex-1 overflow-y-auto py-6 space-y-5 no-scrollbar">
                          {cartState.items
                            .sort((a, b) => ((a.created_at ?? "") > (b.created_at ?? "") ? -1 : 1))
                            .map((item) => (
                              <div
                                key={item.id}
                                data-testid="cart-item"
                                className="flex gap-4 text-xs pb-4 border-b border-[var(--hairline)] last:border-0"
                              >
                                <LocalizedClientLink
                                  href={`/products/${item.product_handle}`}
                                  onClick={() => setIsOpen(false)}
                                  className="shrink-0 w-20 aspect-square rounded-[8px] overflow-hidden bg-[var(--bg-raised)]"
                                >
                                  <Thumbnail
                                    thumbnail={item.thumbnail}
                                    images={item.variant?.product?.images}
                                    size="square"
                                  />
                                </LocalizedClientLink>

                                <div className="flex-1 flex flex-col justify-between min-w-0">
                                  <div>
                                    <LocalizedClientLink
                                      href={`/products/${item.product_handle}`}
                                      onClick={() => setIsOpen(false)}
                                      data-testid="product-link"
                                      className="font-medium text-sm text-[var(--text-primary)] hover:text-[var(--accent-sapphire)] truncate block"
                                    >
                                      {item.title}
                                    </LocalizedClientLink>
                                    <LineItemOptions
                                      variant={item.variant}
                                      title={item.variant_title}
                                      data-testid="cart-item-variant"
                                      data-value={item.variant}
                                    />
                                    <span
                                      data-testid="cart-item-quantity"
                                      data-value={item.quantity}
                                      className="text-[var(--text-muted)] text-[11px] block mt-1"
                                    >
                                      Quantity: {item.quantity}
                                    </span>
                                  </div>

                                  <div className="flex items-center justify-between mt-3 pt-2">
                                    <DeleteButton
                                      id={item.id}
                                      data-testid="cart-item-remove-button"
                                    >
                                      Remove
                                    </DeleteButton>
                                    <LineItemPrice
                                      item={item}
                                      style="tight"
                                      currencyCode={cartState.currency_code}
                                    />
                                  </div>
                                </div>
                              </div>
                            ))}
                        </div>

                        {/* Footer & Checkout Call-to-action */}
                        <div className="pt-5 border-t border-[var(--hairline)] space-y-4">
                          <div className="flex items-center justify-between text-sm">
                            <span className="text-[var(--text-secondary)]">Subtotal</span>
                            <span
                              data-testid="cart-subtotal"
                              data-value={subtotal}
                              className="font-semibold tabular-nums text-[var(--text-primary)] text-base"
                            >
                              {convertToLocale({
                                amount: subtotal,
                                currency_code: cartState.currency_code,
                              })}
                            </span>
                          </div>

                          <div className="grid grid-cols-2 gap-3">
                            <LocalizedClientLink
                              href="/cart"
                              onClick={() => setIsOpen(false)}
                              data-testid="go-to-cart-button"
                              className="w-full py-2.5 px-4 rounded-full text-xs font-semibold text-center border border-[var(--hairline)] text-[var(--text-primary)] hover:bg-[var(--bg-raised)] transition-colors block lustre-btn-press"
                            >
                              View Bag
                            </LocalizedClientLink>
                            <LocalizedClientLink
                              href="/checkout"
                              onClick={() => setIsOpen(false)}
                              className="w-full py-2.5 px-4 rounded-full text-xs font-semibold text-center bg-[var(--accent-sapphire)] text-white hover:bg-[var(--accent-sapphire-hover)] transition-colors block lustre-btn-press"
                            >
                              Checkout
                            </LocalizedClientLink>
                          </div>
                        </div>
                      </>
                    ) : (
                      <div className="flex-1 flex flex-col items-center justify-center text-center py-12">
                        <div className="w-12 h-12 rounded-full bg-[var(--bg-raised)] flex items-center justify-center text-[var(--text-muted)] mb-4">
                          <span className="text-xl">○</span>
                        </div>
                        <p className="text-sm font-medium text-[var(--text-primary)] mb-1">
                          Your shopping bag is empty
                        </p>
                        <p className="text-xs text-[var(--text-muted)] max-w-[200px] mb-6">
                          Explore our collection of watches and fine objects.
                        </p>
                        <LocalizedClientLink
                          href="/store"
                          onClick={() => setIsOpen(false)}
                          className="px-5 py-2 rounded-full text-xs font-medium bg-[var(--accent-sapphire)] text-white hover:bg-[var(--accent-sapphire-hover)] transition-colors lustre-btn-press"
                        >
                          Explore Catalog
                        </LocalizedClientLink>
                      </div>
                    )}
                  </DialogPanel>
                </TransitionChild>
              </div>
            </div>
          </div>
        </Dialog>
      </Transition>
    </>
  )
}

export default CartDropdown
