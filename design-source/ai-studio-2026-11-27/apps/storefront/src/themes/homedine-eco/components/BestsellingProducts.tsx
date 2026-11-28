"use client"

import { useState, useEffect, useRef } from "react"
import gsap from "gsap"
import { ArrowLeft, ArrowRight, Check, Heart, Plus, X } from "lucide-react"
import {
  formatUsd,
  getActiveSwatch,
  getNextCarouselIndex,
  getPrevCarouselIndex,
  getVisibleWindow,
  type HomedineProduct,
} from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

interface BestsellingProductsProps {
  products: HomedineProduct[]
  activeCategoryLabel: string | null
  searchQuery: string
  savedIds: string[]
  onClearFilters: () => void
  onSelectProduct: (product: HomedineProduct, swatchId: string) => void
  onAddToCart: (product: HomedineProduct, swatchId: string) => void
  onToggleSave: (productId: string) => void
}

export function BestsellingProducts({
  products,
  activeCategoryLabel,
  searchQuery,
  savedIds,
  onClearFilters,
  onSelectProduct,
  onAddToCart,
  onToggleSave,
}: BestsellingProductsProps) {
  const [startIndex, setStartIndex] = useState(0)
  const [selectedSwatches, setSelectedSwatches] = useState<Record<string, string>>({})
  const [addedKey, setAddedKey] = useState<string | null>(null)
  const gridRef = useRef<HTMLDivElement | null>(null)
  const hasInteracted = useRef(false)

  const visibleCount = 4
  const visibleProducts = getVisibleWindow(products, startIndex, visibleCount)
  const canCarousel = products.length > visibleCount

  useEffect(() => {
    setStartIndex(0)
  }, [activeCategoryLabel, searchQuery, products.length])

  useEffect(() => {
    if (!hasInteracted.current || !gridRef.current) return
    const cards = gridRef.current.querySelectorAll("article")
    if (cards.length === 0) return
    gsap.fromTo(
      cards,
      { opacity: 0.4, y: 14 },
      {
        opacity: 1,
        y: 0,
        duration: 0.38,
        stagger: 0.045,
        ease: "power3.out",
        clearProps: "transform,opacity",
      }
    )
  }, [startIndex])

  const handleSelectSwatch = (productId: string, swatchId: string) => {
    setSelectedSwatches((prev) => ({ ...prev, [productId]: swatchId }))
  }

  const handleAdd = (product: HomedineProduct) => {
    const activeSwatch = getActiveSwatch(product, selectedSwatches[product.id])
    onAddToCart(product, activeSwatch.id)
    const feedbackKey = `${product.id}::${activeSwatch.id}`
    setAddedKey(feedbackKey)
    window.setTimeout(() => {
      setAddedKey((current) => (current === feedbackKey ? null : current))
    }, 1150)
  }

  const handlePrev = () => {
    hasInteracted.current = true
    setStartIndex((prev) => getPrevCarouselIndex(prev, products.length, visibleCount))
  }

  const handleNext = () => {
    hasInteracted.current = true
    setStartIndex((prev) => getNextCarouselIndex(prev, products.length, visibleCount))
  }

  return (
    <section
      id="bestsellers"
      aria-labelledby="bestsellers-heading"
      className="mx-auto max-w-[1360px] px-4 pt-16 sm:px-6 sm:pt-20 lg:px-8"
    >
      {/* Header Row */}
      <div
        data-gsap="reveal"
        className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 mb-9"
      >
        <div>
          <h2
            id="bestsellers-heading"
            className="text-[28px] sm:text-[34px] leading-[1.15] tracking-[-0.02em] text-[#1a1f1c] font-medium"
          >
            <span className={`${styles.fontEditorial} font-normal mr-1.5`}>Bestselling</span>
            <span>Products</span>
          </h2>
          {(activeCategoryLabel || searchQuery.trim()) && (
            <div className="mt-2.5 flex flex-wrap items-center gap-2 text-[13px] text-[#4e5651]">
              <span>
                Showing {products.length} {products.length === 1 ? "item" : "items"}
                {activeCategoryLabel ? ` in ${activeCategoryLabel}` : ""}
                {searchQuery.trim() ? ` matching "${searchQuery.trim()}"` : ""}
              </span>
              <button
                type="button"
                onClick={onClearFilters}
                className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full bg-[#eff1e7] hover:bg-[#e3e7d6] text-[#0e5437] text-[12px] font-medium transition-colors cursor-pointer"
              >
                <X className="w-3.5 h-3.5" />
                <span>Reset filter</span>
              </button>
            </div>
          )}
        </div>

        {/* Desktop Carousel Arrows */}
        {canCarousel && (
          <div className="hidden lg:flex items-center gap-2">
            <button
              type="button"
              onClick={handlePrev}
              aria-label="Previous bestselling products"
              className="w-10 h-10 rounded-full bg-white border border-black/[0.1] text-[#1a1f1c] hover:bg-[#0e5437] hover:text-white hover:border-[#0e5437] flex items-center justify-center transition-colors cursor-pointer"
            >
              <ArrowLeft className="w-4 h-4" strokeWidth={1.8} />
            </button>
            <button
              type="button"
              onClick={handleNext}
              aria-label="Next bestselling products"
              className="w-10 h-10 rounded-full bg-white border border-black/[0.1] text-[#1a1f1c] hover:bg-[#0e5437] hover:text-white hover:border-[#0e5437] flex items-center justify-center transition-colors cursor-pointer"
            >
              <ArrowRight className="w-4 h-4" strokeWidth={1.8} />
            </button>
          </div>
        )}
      </div>

      {/* Empty Filter State */}
      {products.length === 0 ? (
        <div className="rounded-[var(--radius-card)] bg-[#eff1e7] py-16 px-6 text-center">
          <p className="text-[17px] font-medium text-[#1a1f1c]">
            No sustainable kitchenware matched your current filter.
          </p>
          <p className="mt-1.5 text-[14px] text-[#4e5651]">
            Try clearing your search or exploring the full collection.
          </p>
          <button
            type="button"
            onClick={onClearFilters}
            className="mt-5 inline-flex items-center gap-2 h-10 px-5 rounded-full bg-[var(--color-brand-forest)] hover:bg-[var(--color-brand-forest-hover)] text-white text-[13px] font-medium transition-colors cursor-pointer"
          >
            View all products
          </button>
        </div>
      ) : (
        <div>
          <div
            ref={gridRef}
            data-gsap="stagger-group"
            className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5 lg:gap-6"
          >
            {visibleProducts.map((product) => {
              const activeSwatch = getActiveSwatch(product, selectedSwatches[product.id])
              const isSaved = savedIds.includes(product.id)
              const isAdded = addedKey === `${product.id}::${activeSwatch.id}`

              return (
                <article
                  key={product.id}
                  data-gsap-item
                  className={`${styles.productCard} group flex flex-col`}
                >
                  {/* Studio Product Image Surface */}
                  <div
                    className={`${styles.productCardMedia} relative aspect-[4/4.35] overflow-hidden rounded-[var(--radius-card)] bg-[#cac8be]`}
                  >
                    <button
                      type="button"
                      onClick={() => onSelectProduct(product, activeSwatch.id)}
                      aria-label={`Inspect ${product.shortName}`}
                      className="block w-full h-full cursor-pointer"
                    >
                      <ResilientImage
                        src={product.image}
                        alt={product.alt}
                        fallbackTitle={product.shortName}
                        className="w-full h-full object-cover object-center"
                        style={
                          activeSwatch.tintFilter ? { filter: activeSwatch.tintFilter } : undefined
                        }
                      />
                    </button>

                    {/* Subtle Floating Glass Tag */}
                    {product.tag && (
                      <span
                        className={`${styles.glassBadge} absolute top-3.5 left-3.5 px-3 py-1 rounded-full text-[11px] font-medium text-white pointer-events-none`}
                      >
                        {product.tag}
                      </span>
                    )}

                    {/* Save / Wishlist Heart */}
                    <button
                      type="button"
                      onClick={() => onToggleSave(product.id)}
                      aria-label={
                        isSaved
                          ? `Remove ${product.shortName} from saved`
                          : `Save ${product.shortName}`
                      }
                      aria-pressed={isSaved}
                      className={`absolute top-3.5 right-3.5 w-8 h-8 rounded-full flex items-center justify-center transition-all cursor-pointer ${
                        isSaved
                          ? "bg-[#0e5437] text-white shadow-md"
                          : "bg-white/85 text-[#1a1f1c] hover:bg-white shadow-sm"
                      }`}
                    >
                      <Heart
                        className="w-4 h-4"
                        strokeWidth={1.8}
                        fill={isSaved ? "currentColor" : "none"}
                      />
                    </button>
                  </div>

                  {/* Finish Swatches Row */}
                  <div className="mt-3.5 flex items-center justify-between gap-2">
                    <div
                      className="flex items-center gap-2"
                      role="radiogroup"
                      aria-label={`Color finishes for ${product.shortName}`}
                    >
                      {product.swatches.map((swatch) => {
                        const isSelected = activeSwatch.id === swatch.id
                        return (
                          <button
                            key={swatch.id}
                            type="button"
                            role="radio"
                            aria-checked={isSelected}
                            aria-label={`${swatch.name} finish for ${product.shortName}`}
                            title={swatch.name}
                            onClick={() => handleSelectSwatch(product.id, swatch.id)}
                            className={`w-4 h-4 rounded-full transition-transform cursor-pointer ${
                              isSelected
                                ? "ring-2 ring-offset-2 ring-[#1a1f1c] scale-105"
                                : "ring-1 ring-black/15 hover:scale-110"
                            }`}
                            style={{ backgroundColor: swatch.hex }}
                          />
                        )
                      })}
                    </div>
                    <span className="text-[11px] text-[#767e78] truncate">
                      {activeSwatch.name}
                    </span>
                  </div>

                  {/* Product Title */}
                  <h3 className="mt-2.5 text-[15px] leading-[1.35] font-medium text-[#1a1f1c] min-h-[42px]">
                    <button
                      type="button"
                      onClick={() => onSelectProduct(product, activeSwatch.id)}
                      className="text-left hover:text-[#0e5437] transition-colors cursor-pointer"
                    >
                      {product.name}
                    </button>
                  </h3>

                  {/* Price & Add to Cart Row */}
                  <div className="mt-3 pt-1 flex items-center justify-between gap-3">
                    <span className="text-[16px] font-bold text-[#1a1f1c] tabular-nums">
                      {formatUsd(product.price)}
                    </span>
                    <button
                      type="button"
                      onClick={() => handleAdd(product)}
                      data-testid={`homedine-add-${product.id}`}
                      aria-label={`Add ${product.shortName} (${activeSwatch.name}) to cart`}
                      className={`inline-flex items-center gap-1.5 h-9 px-4 rounded-full text-[13px] font-medium transition-all whitespace-nowrap shrink-0 cursor-pointer ${
                        isAdded
                          ? "bg-[#1a6b49] text-white scale-[0.98]"
                          : "bg-[var(--color-brand-forest)] hover:bg-[var(--color-brand-forest-hover)] text-white"
                      }`}
                    >
                      {isAdded ? (
                        <>
                          <Check className="w-3.5 h-3.5" strokeWidth={2.4} />
                          <span>Added</span>
                        </>
                      ) : (
                        <>
                          <Plus className="w-3.5 h-3.5" strokeWidth={2.2} />
                          <span>Cart</span>
                        </>
                      )}
                    </button>
                  </div>
                </article>
              )
            })}
          </div>

          {/* Mobile/Tablet Carousel Arrows */}
          {canCarousel && (
            <div className="flex lg:hidden items-center justify-end gap-2 mt-6">
              <button
                type="button"
                onClick={handlePrev}
                aria-label="Previous products"
                className="w-10 h-10 rounded-full bg-white border border-black/10 flex items-center justify-center text-[#1a1f1c] hover:bg-[#0e5437] hover:text-white transition-colors cursor-pointer"
              >
                <ArrowLeft className="w-4 h-4" />
              </button>
              <button
                type="button"
                onClick={handleNext}
                aria-label="Next products"
                className="w-10 h-10 rounded-full bg-white border border-black/10 flex items-center justify-center text-[#1a1f1c] hover:bg-[#0e5437] hover:text-white transition-colors cursor-pointer"
              >
                <ArrowRight className="w-4 h-4" />
              </button>
            </div>
          )}
        </div>
      )}
    </section>
  )
}
