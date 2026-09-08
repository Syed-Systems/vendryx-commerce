"use client"

import React, { useState } from "react"
import { applyPromotions, convertToLocale } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import { Trash } from "@phosphor-icons/react/dist/ssr"
import ErrorMessage from "@modules/common/components/error-message"
import { SubmitButton } from "@modules/common/components/submit-button"

export type DiscountCodeProps = {
  cart: HttpTypes.StoreCart & {
    promotions: HttpTypes.StorePromotion[]
  }
}

/**
 * Lustre DiscountCode.
 * Disclosure form for applying and removing promotion codes.
 * Conforms to Lustre styling without @medusajs/ui dependencies.
 */
const DiscountCode: React.FC<DiscountCodeProps> = ({ cart }) => {
  const [isOpen, setIsOpen] = useState(false)
  const [errorMessage, setErrorMessage] = useState("")

  const { promotions = [] } = cart

  const removePromotionCode = async (code: string) => {
    const validPromotions = promotions.filter(
      (promotion) => promotion.code !== code
    )

    await applyPromotions(
      validPromotions.filter((p) => p.code !== undefined).map((p) => p.code!)
    )
  }

  const addPromotionCode = async (formData: FormData) => {
    setErrorMessage("")

    const code = formData.get("code")
    if (!code) {
      return
    }

    const input = document.getElementById("promotion-input") as HTMLInputElement
    const codes = promotions
      .filter((p) => p.code !== undefined)
      .map((p) => p.code!)
    codes.push(code.toString())

    try {
      await applyPromotions(codes)
      if (input) {
        input.value = ""
      }
    } catch (e: any) {
      setErrorMessage(e.message || "Failed to apply promotion code")
    }
  }

  return (
    <div className="w-full flex flex-col py-3 border-t border-[var(--hairline)]">
      <div className="flex items-center justify-between">
        <button
          type="button"
          onClick={() => setIsOpen((prev) => !prev)}
          data-testid="add-discount-button"
          className="text-xs font-medium text-[var(--accent-sapphire)] hover:text-[var(--accent-sapphire-hover)] transition-colors inline-flex items-center gap-1 lustre-btn-press"
        >
          <span>{isOpen ? "Close Promotion Form" : "Add Promotion Code"}</span>
        </button>
      </div>

      {isOpen && (
        <form
          action={addPromotionCode}
          className="mt-3 flex flex-col gap-2"
        >
          <div className="flex gap-2">
            <input
              id="promotion-input"
              name="code"
              type="text"
              placeholder="Enter code"
              data-testid="discount-input"
              className="flex-1 h-10 px-3 text-xs bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--hairline)] rounded-[12px] focus:outline-none focus:ring-2 focus:ring-[var(--accent-sapphire)] focus:border-transparent placeholder:text-[var(--text-muted)]"
            />
            <SubmitButton
              variant="secondary"
              data-testid="discount-apply-button"
              className="h-10 px-4 text-xs font-medium"
            >
              Apply
            </SubmitButton>
          </div>

          <ErrorMessage
            error={errorMessage}
            data-testid="discount-error-message"
          />
        </form>
      )}

      {promotions.length > 0 && (
        <div className="mt-3 space-y-2">
          <p className="text-xs font-medium text-[var(--text-secondary)]">
            Active promotions:
          </p>
          {promotions.map((promotion) => (
            <div
              key={promotion.id}
              data-testid="discount-row"
              className="flex items-center justify-between p-2 rounded-[8px] bg-[var(--bg-raised)] text-xs"
            >
              <div className="flex items-center gap-2 truncate">
                <span
                  data-testid="discount-code"
                  className="px-2 py-0.5 rounded-full font-mono text-[10px] font-semibold bg-[var(--accent-sapphire-subtle)] text-[var(--accent-sapphire)] border border-[var(--accent-sapphire)]/20"
                >
                  {promotion.code}
                </span>
                {promotion.application_method?.value !== undefined &&
                  promotion.application_method.currency_code !== undefined && (
                    <span className="text-[var(--text-secondary)]">
                      {promotion.application_method.type === "percentage"
                        ? `${promotion.application_method.value}% off`
                        : `${convertToLocale({
                            amount: +promotion.application_method.value,
                            currency_code:
                              promotion.application_method.currency_code,
                          })} off`}
                    </span>
                  )}
              </div>

              {!promotion.is_automatic && (
                <button
                  type="button"
                  onClick={() => promotion.code && removePromotionCode(promotion.code)}
                  data-testid="remove-discount-button"
                  aria-label={`Remove promotion ${promotion.code}`}
                  className="p-1 text-[var(--text-muted)] hover:text-[var(--status-error)] transition-colors lustre-btn-press"
                >
                  <Trash size={14} />
                </button>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default DiscountCode
