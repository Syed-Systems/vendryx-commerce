"use client"

import { useState, useEffect } from "react"
import { Check, Heart, Leaf, Plus, Star, X } from "lucide-react"
import { formatUsd, getActiveSwatch, type HomedineProduct } from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

interface ProductModalProps {
  product: HomedineProduct | null
  initialSwatchId?: string
  contextNote?: string
  isSaved: boolean
  onClose: () => void
  onAddToCart: (product: HomedineProduct, swatchId: string) => void
  onToggleSave: (productId: string) => void
}

export function ProductModal({
  product,
  initialSwatchId,
  contextNote,
  isSaved,
  onClose,
  onAddToCart,
  onToggleSave,
}: ProductModalProps) {
  const [selectedSwatchId, setSelectedSwatchId] = useState<string>("")
  const [addedFeedback, setAddedFeedback] = useState(false)

  useEffect(() => {
    if (product) {
      setSelectedSwatchId(initialSwatchId ?? product.swatches[0].id)
      setAddedFeedback(false)
    }
  }, [product, initialSwatchId])

  useEffect(() => {
    if (!product) return
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose()
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [product, onClose])

  if (!product) return null

  const activeSwatch = getActiveSwatch(product, selectedSwatchId)

  const handleAdd = () => {
    onAddToCart(product, activeSwatch.id)
    setAddedFeedback(true)
    window.setTimeout(() => setAddedFeedback(false), 1200)
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6"
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-product-title"
      data-testid="homedine-product-modal"
    >
      {/* Backdrop */}
      <button
        type="button"
        aria-label="Close product quick view"
        onClick={onClose}
        className="fixed inset-0 bg-black/50 backdrop-blur-[3px] cursor-pointer"
      />

      {/* Modal Container */}
      <div
        data-lenis-prevent
        className="relative z-10 w-full max-w-[820px] overflow-hidden rounded-[24px] bg-[var(--color-canvas)] shadow-2xl border border-black/[0.08] grid grid-cols-1 md:grid-cols-12 max-h-[90vh] overflow-y-auto"
      >
        <button
          type="button"
          onClick={onClose}
          aria-label="Close product dialog"
          className="absolute top-4 right-4 z-20 w-9 h-9 rounded-full bg-white/90 hover:bg-white text-[#1a1f1c] flex items-center justify-center shadow-sm cursor-pointer"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Left Image Pane */}
        <div className="md:col-span-6 bg-[#cac8be] aspect-[4/4.2] md:aspect-auto relative">
          <ResilientImage
            src={product.image}
            alt={product.alt}
            fallbackTitle={product.shortName}
            className="w-full h-full object-cover object-center transition-[filter] duration-300"
            style={activeSwatch.tintFilter ? { filter: activeSwatch.tintFilter } : undefined}
          />
          {product.tag && (
            <span
              className={`${styles.glassBadge} absolute top-4 left-4 px-3 py-1 rounded-full text-[11px] font-medium text-white`}
            >
              {product.tag}
            </span>
          )}
        </div>

        {/* Right Purchase Module */}
        <div className="md:col-span-6 p-6 sm:p-8 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 text-[12px] text-[#4e5651]">
              <span>{product.category}</span>
              <span aria-hidden="true">·</span>
              <span className="inline-flex items-center gap-1 tabular-nums">
                <Star className="w-3.5 h-3.5 fill-[#0e5437] text-[#0e5437]" />
                {product.rating.toFixed(1)} ({product.reviewCount} reviews)
              </span>
            </div>

            <h2
              id="modal-product-title"
              className="mt-2 text-[24px] leading-[1.2] font-semibold tracking-[-0.02em] text-[#1a1f1c]"
            >
              {product.shortName}
            </h2>

            {contextNote && (
              <p className="mt-2 text-[12px] text-[#0e5437] font-medium">
                Featured in gallery: {contextNote}
              </p>
            )}

            <p className="mt-3 text-[22px] font-bold text-[#1a1f1c] tabular-nums">
              {formatUsd(product.price)}
            </p>

            <p className="mt-4 text-[14px] leading-[1.65] text-[#4e5651]">
              {product.description}
            </p>

            {/* Color Variant Picker */}
            <div className="mt-6">
              <p className="text-[12px] font-semibold text-[#1a1f1c] mb-2.5">
                Finish: <span className="font-normal text-[#4e5651]">{activeSwatch.name}</span>
              </p>
              <div className="flex items-center gap-2.5" role="radiogroup" aria-label="Choose finish">
                {product.swatches.map((swatch) => {
                  const isSelected = activeSwatch.id === swatch.id
                  return (
                    <button
                      key={swatch.id}
                      type="button"
                      role="radio"
                      aria-checked={isSelected}
                      aria-label={swatch.name}
                      onClick={() => setSelectedSwatchId(swatch.id)}
                      className={`w-7 h-7 rounded-full transition-transform cursor-pointer ${
                        isSelected
                          ? "ring-2 ring-offset-2 ring-[#0e5437] scale-105"
                          : "ring-1 ring-black/15 hover:scale-105"
                      }`}
                      style={{ backgroundColor: swatch.hex }}
                    />
                  )
                })}
              </div>
            </div>

            {/* Sustainability Specs */}
            <div className="mt-6 pt-5 border-t border-black/[0.08]">
              <p className="text-[12px] font-semibold text-[#1a1f1c] mb-2.5 flex items-center gap-1.5">
                <Leaf className="w-3.5 h-3.5 text-[#0e5437]" />
                <span>Material transparency</span>
              </p>
              <ul className="space-y-1.5 text-[13px] text-[#4e5651]">
                {product.sustainabilitySpecs.map((spec) => (
                  <li key={spec} className="flex items-center gap-2">
                    <Check className="w-3.5 h-3.5 text-[#0e5437] shrink-0" />
                    <span>{spec}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>

          {/* Contiguous Purchase Actions */}
          <div className="mt-7 flex items-center gap-3">
            <button
              type="button"
              onClick={handleAdd}
              className="flex-1 h-11 px-5 rounded-full bg-[var(--color-brand-forest)] hover:bg-[var(--color-brand-forest-hover)] text-white text-[14px] font-semibold inline-flex items-center justify-center gap-2 transition-colors cursor-pointer"
            >
              {addedFeedback ? (
                <>
                  <Check className="w-4 h-4" />
                  <span>Added to Bag</span>
                </>
              ) : (
                <>
                  <Plus className="w-4 h-4" />
                  <span>Add to Cart · {formatUsd(product.price)}</span>
                </>
              )}
            </button>

            <button
              type="button"
              onClick={() => onToggleSave(product.id)}
              aria-label={isSaved ? "Remove from saved items" : "Save item"}
              aria-pressed={isSaved}
              className={`w-11 h-11 rounded-full border flex items-center justify-center transition-colors shrink-0 cursor-pointer ${
                isSaved
                  ? "bg-[#0e5437]/10 border-[#0e5437] text-[#0e5437]"
                  : "border-black/15 text-[#1a1f1c] hover:bg-black/5"
              }`}
            >
              <Heart className="w-4 h-4" fill={isSaved ? "currentColor" : "none"} />
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
