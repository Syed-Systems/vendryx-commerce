"use client"

import { Dialog, Transition } from "@headlessui/react"
import React, { Fragment, useMemo, useState } from "react"
import { CaretDown, X } from "@phosphor-icons/react/dist/ssr"
import Spinner from "@modules/common/icons/spinner"
import { getProductPrice, isSimpleProduct } from "@stheno/storefront-core"
import OptionSelect from "./option-select"
import { HttpTypes } from "@medusajs/types"

export type MobileActionsProps = {
  product: HttpTypes.StoreProduct
  variant?: HttpTypes.StoreProductVariant
  options: Record<string, string | undefined>
  updateOptions: (title: string, value: string) => void
  inStock?: boolean
  handleAddToCart: () => void
  isAdding?: boolean
  show: boolean
  optionsDisabled: boolean
}

/**
 * Lustre MobileActions.
 * Restrained frosted mobile purchase bar with accessible variant sheet.
 */
const MobileActions: React.FC<MobileActionsProps> = ({
  product,
  variant,
  options,
  updateOptions,
  inStock,
  handleAddToCart,
  isAdding,
  show,
  optionsDisabled,
}) => {
  const [isOpen, setIsOpen] = useState(false)

  const price = getProductPrice({
    product,
    variantId: variant?.id,
  })

  const selectedPrice = useMemo(() => {
    if (!price) {
      return null
    }
    return price.variantPrice || price.cheapestPrice || null
  }, [price])

  const isSimple = isSimpleProduct(product)

  return (
    <>
      <div
        className={`lg:hidden fixed inset-x-0 bottom-0 z-40 px-4 pb-4 pt-2 pointer-events-none transition-opacity duration-200 ${
          show ? "opacity-100" : "opacity-0"
        }`}
      >
        <div
          data-testid="mobile-actions"
          className="pointer-events-auto max-w-md mx-auto p-3 rounded-[16px] bg-[var(--frosted-bg)] border border-[var(--frosted-border)] shadow-[var(--elevation-modal)] backdrop-blur-md lustre-frosted flex flex-col gap-2.5"
        >
          <div className="flex items-center justify-between text-xs px-1">
            <span
              data-testid="mobile-title"
              className="font-medium text-[var(--text-primary)] truncate max-w-[200px]"
            >
              {product.title}
            </span>
            {selectedPrice && (
              <span className="font-semibold tabular-nums text-[var(--text-primary)]">
                {selectedPrice.calculated_price}
              </span>
            )}
          </div>

          <div className={`grid gap-2 ${isSimple ? "grid-cols-1" : "grid-cols-2"}`}>
            {!isSimple && (
              <button
                type="button"
                onClick={() => setIsOpen(true)}
                data-testid="mobile-actions-button"
                className="h-11 px-3.5 rounded-full text-xs font-medium bg-[var(--bg-surface)] border border-[var(--hairline)] text-[var(--text-primary)] flex items-center justify-between lustre-btn-press"
              >
                <span className="truncate">
                  {variant
                    ? Object.values(options).filter(Boolean).join(" / ") || "Selected"
                    : "Select variant"}
                </span>
                <CaretDown size={14} />
              </button>
            )}

            <button
              type="button"
              onClick={handleAddToCart}
              disabled={!inStock || !variant || isAdding}
              data-testid="mobile-cart-button"
              className="h-11 px-4 rounded-full text-xs font-semibold uppercase tracking-wider bg-[var(--accent-sapphire)] text-white hover:bg-[var(--accent-sapphire-hover)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-2 shadow-sm lustre-btn-press"
            >
              {isAdding ? (
                <Spinner size="14" />
              ) : !variant ? (
                "Choose Variant"
              ) : !inStock ? (
                "Unavailable"
              ) : (
                "Add to Bag"
              )}
            </button>
          </div>
        </div>
      </div>

      <Transition appear show={isOpen} as={Fragment}>
        <Dialog as="div" className="relative z-50" onClose={() => setIsOpen(false)}>
          <Transition.Child
            as={Fragment}
            enter="ease-out duration-200"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="ease-in duration-150"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div className="fixed inset-0 bg-black/40 backdrop-blur-sm" />
          </Transition.Child>

          <div className="fixed inset-x-0 bottom-0 max-h-[85vh] overflow-y-auto">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-260"
              enterFrom="translate-y-full"
              enterTo="translate-y-0"
              leave="ease-in duration-200"
              leaveFrom="translate-y-0"
              leaveTo="translate-y-full"
            >
              <Dialog.Panel
                data-testid="mobile-actions-modal"
                className="w-full rounded-t-[24px] bg-[var(--bg-surface)] border-t border-[var(--hairline)] shadow-2xl p-6 text-left"
              >
                <div className="flex items-center justify-between pb-4 border-b border-[var(--hairline)]">
                  <h3 className="text-sm font-semibold uppercase tracking-wider text-[var(--text-primary)]">
                    Select Reference
                  </h3>
                  <button
                    type="button"
                    onClick={() => setIsOpen(false)}
                    data-testid="close-modal-button"
                    aria-label="Close options dialog"
                    className="p-1.5 rounded-full text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-raised)] lustre-btn-press"
                  >
                    <X size={18} />
                  </button>
                </div>

                <div className="py-6 space-y-6">
                  {(product.options || []).map((opt) => (
                    <OptionSelect
                      key={opt.id}
                      option={opt}
                      current={options[opt.id]}
                      updateOption={updateOptions}
                      title={opt.title ?? ""}
                      disabled={optionsDisabled}
                    />
                  ))}
                </div>

                <button
                  type="button"
                  onClick={() => setIsOpen(false)}
                  className="w-full h-12 rounded-full bg-[var(--accent-sapphire)] text-white text-xs font-semibold uppercase tracking-wider lustre-btn-press"
                >
                  Confirm Selection
                </button>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </Dialog>
      </Transition>
    </>
  )
}

export default MobileActions
