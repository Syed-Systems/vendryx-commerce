"use client"

import React, { useCallback, useEffect, useState } from "react"
import { HttpTypes } from "@medusajs/types"
import useEmblaCarousel from "embla-carousel-react"
import { CaretLeft, CaretRight } from "@phosphor-icons/react/dist/ssr"
import ProductPreview from "../product-preview"

export default function RelatedProducts({
  region,
  products,
}: {
  region: HttpTypes.StoreRegion
  products: HttpTypes.StoreProduct[]
}) {
  const [emblaRef, emblaApi] = useEmblaCarousel({
    loop: false,
    align: "start",
    containScroll: "trimSnaps",
  })
  const [prevBtnDisabled, setPrevBtnDisabled] = useState(true)
  const [nextBtnDisabled, setNextBtnDisabled] = useState(true)

  const scrollPrev = useCallback(() => {
    if (emblaApi) emblaApi.scrollPrev()
  }, [emblaApi])

  const scrollNext = useCallback(() => {
    if (emblaApi) emblaApi.scrollNext()
  }, [emblaApi])

  const onSelect = useCallback(() => {
    if (!emblaApi) return
    setPrevBtnDisabled(!emblaApi.canScrollPrev())
    setNextBtnDisabled(!emblaApi.canScrollNext())
  }, [emblaApi])

  useEffect(() => {
    if (!emblaApi) return
    onSelect()
    emblaApi.on("select", onSelect)
    emblaApi.on("reInit", onSelect)
    return () => {
      emblaApi.off("select", onSelect)
      emblaApi.off("reInit", onSelect)
    }
  }, [emblaApi, onSelect])

  if (!products || products.length === 0) {
    return null
  }

  return (
    <div className="w-full relative">
      {/* Header with Navigation Controls */}
      <div className="flex items-center justify-between mb-6 pb-2 border-b border-[var(--hairline)]">
        <div>
          <span className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--accent-sapphire)] block mb-1">
            Complementary Pieces
          </span>
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">
            Related Curations
          </h3>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={scrollPrev}
            disabled={prevBtnDisabled}
            aria-label="Previous related products"
            className="h-8 w-8 rounded-full border border-[var(--hairline)] bg-[var(--bg-surface)] flex items-center justify-center text-[var(--text-primary)] hover:border-[var(--accent-sapphire)] disabled:opacity-30 disabled:hover:border-[var(--hairline)] lustre-btn-press transition"
          >
            <CaretLeft size={16} />
          </button>
          <button
            type="button"
            onClick={scrollNext}
            disabled={nextBtnDisabled}
            aria-label="Next related products"
            className="h-8 w-8 rounded-full border border-[var(--hairline)] bg-[var(--bg-surface)] flex items-center justify-center text-[var(--text-primary)] hover:border-[var(--accent-sapphire)] disabled:opacity-30 disabled:hover:border-[var(--hairline)] lustre-btn-press transition"
          >
            <CaretRight size={16} />
          </button>
        </div>
      </div>

      {/* Embla Rail with Next Item Peek Affordance */}
      <div className="overflow-hidden" ref={emblaRef}>
        <div className="flex gap-6 touch-pan-y">
          {products.map((product) => (
            <div
              key={product.id}
              className="min-w-0 flex-[0_0_75%] sm:flex-[0_0_42%] lg:flex-[0_0_28%]"
            >
              <ProductPreview region={region} product={product} />
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
