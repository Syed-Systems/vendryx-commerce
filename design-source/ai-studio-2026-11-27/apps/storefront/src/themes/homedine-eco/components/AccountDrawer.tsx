"use client"

import { useEffect } from "react"
import { Heart, Leaf, Plus, X } from "lucide-react"
import { formatUsd, type HomedineProduct } from "../catalog"
import { ResilientImage } from "./ResilientImage"

interface AccountDrawerProps {
  isOpen: boolean
  savedProducts: HomedineProduct[]
  onClose: () => void
  onToggleSave: (productId: string) => void
  onAddToCart: (product: HomedineProduct) => void
}

export function AccountDrawer({
  isOpen,
  savedProducts,
  onClose,
  onToggleSave,
  onAddToCart,
}: AccountDrawerProps) {
  useEffect(() => {
    if (!isOpen) return
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose()
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [isOpen, onClose])

  if (!isOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex justify-end"
      role="dialog"
      aria-modal="true"
      aria-label="Saved items drawer"
    >
      <button
        type="button"
        aria-label="Close saved items drawer"
        onClick={onClose}
        className="fixed inset-0 bg-black/45 backdrop-blur-[2px] cursor-pointer"
      />

      <div
        data-lenis-prevent
        className="relative z-10 w-full max-w-[420px] h-full bg-[var(--color-canvas)] shadow-[var(--shadow-drawer)] flex flex-col"
      >
        <div className="flex items-center justify-between px-6 py-5 border-b border-black/[0.08]">
          <h2 className="text-[18px] font-semibold text-[#1a1f1c]">Saved &amp; Rituals</h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close saved drawer"
            className="w-9 h-9 rounded-full hover:bg-black/5 flex items-center justify-center text-[#1a1f1c] cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto px-6 py-6 space-y-8">
          <div>
            <h3 className="text-[14px] font-semibold text-[#1a1f1c] flex items-center gap-2 mb-4">
              <Heart className="w-4 h-4 text-[#0e5437]" />
              <span>Saved Kitchenware ({savedProducts.length})</span>
            </h3>

            {savedProducts.length === 0 ? (
              <p className="text-[13px] text-[#4e5651] rounded-2xl bg-[#eff1e7] p-4">
                Tap the heart icon on any product to save your favorite eco pieces here.
              </p>
            ) : (
              <ul className="space-y-3">
                {savedProducts.map((product) => (
                  <li
                    key={product.id}
                    className="flex items-center gap-3 rounded-2xl bg-white border border-black/[0.06] p-3"
                  >
                    <ResilientImage
                      src={product.image}
                      alt={product.alt}
                      className="w-14 h-14 rounded-xl object-cover bg-[#cac8be] shrink-0"
                    />
                    <div className="flex-1 min-w-0">
                      <p className="text-[13px] font-semibold text-[#1a1f1c] truncate">
                        {product.shortName}
                      </p>
                      <p className="text-[12px] text-[#4e5651] tabular-nums">
                        {formatUsd(product.price)}
                      </p>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <button
                        type="button"
                        onClick={() => onAddToCart(product)}
                        aria-label={`Add ${product.shortName} to cart`}
                        className="h-8 px-3 rounded-full bg-[#0e5437] text-white text-[12px] font-medium inline-flex items-center gap-1 hover:bg-[#0a3f29] cursor-pointer"
                      >
                        <Plus className="w-3 h-3" />
                        <span>Bag</span>
                      </button>
                      <button
                        type="button"
                        onClick={() => onToggleSave(product.id)}
                        aria-label={`Remove ${product.shortName} from saved`}
                        className="w-8 h-8 rounded-full text-[#767e78] hover:text-[#1a1f1c] hover:bg-black/5 flex items-center justify-center cursor-pointer"
                      >
                        <X className="w-4 h-4" />
                      </button>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="rounded-2xl bg-[#eff1e7] p-5 border border-black/[0.04]">
            <div className="flex items-center gap-2 text-[13px] font-semibold text-[#0e5437]">
              <Leaf className="w-4 h-4" />
              <span>Compostable Delivery Guarantee</span>
            </div>
            <p className="mt-2 text-[12px] leading-[1.6] text-[#4e5651]">
              Every Homedine order ships in plastic-free molded pulp packaging with verified
              carbon-neutral ground couriers through Vendryx Locked Checkout.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
