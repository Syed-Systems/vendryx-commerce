"use client"

import React, { useState, useEffect, useCallback } from "react"
import { HttpTypes } from "@medusajs/types"
import Image from "next/image"
import useEmblaCarousel from "embla-carousel-react"
import { CaretLeft, CaretRight } from "@phosphor-icons/react/dist/ssr"
import PlaceholderImage from "@modules/common/icons/placeholder-image"

export type ImageGalleryProps = {
  images: HttpTypes.StoreProductImage[]
}

/**
 * Lustre ImageGallery.
 * Desktop: 7-column vertical stack.
 * Mobile: Embla carousel with drag, snap, dot indicators, and manual arrow controls.
 * Aspect 4:5, 16px radius, hairline border.
 */
const ImageGallery = ({ images }: ImageGalleryProps) => {
  const [emblaRef, emblaApi] = useEmblaCarousel({ loop: false })
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [scrollSnaps, setScrollSnaps] = useState<number[]>([])

  const onSelect = useCallback(() => {
    if (!emblaApi) return
    setSelectedIndex(emblaApi.selectedScrollSnap())
  }, [emblaApi])

  useEffect(() => {
    if (!emblaApi) return
    setScrollSnaps(emblaApi.scrollSnapList())
    emblaApi.on("select", onSelect)
    onSelect()
    return () => {
      emblaApi.off("select", onSelect)
    }
  }, [emblaApi, onSelect])

  const scrollPrev = useCallback(() => {
    if (emblaApi) emblaApi.scrollPrev()
  }, [emblaApi])

  const scrollNext = useCallback(() => {
    if (emblaApi) emblaApi.scrollNext()
  }, [emblaApi])

  const scrollTo = useCallback(
    (index: number) => {
      if (emblaApi) emblaApi.scrollTo(index)
    },
    [emblaApi]
  )

  if (!images || images.length === 0) {
    return (
      <div className="aspect-[4/5] w-full rounded-[16px] bg-[var(--bg-raised)] border border-[var(--hairline)] flex items-center justify-center text-[var(--text-muted)]">
        <PlaceholderImage size="48" />
      </div>
    )
  }

  return (
    <div className="w-full">
      {/* Desktop layout: Vertical Stack */}
      <div className="hidden lg:flex flex-col space-y-4">
        {images.map((image, index) => (
          <div
            key={image.id || index}
            id={image.id}
            className="relative aspect-[4/5] w-full rounded-[16px] overflow-hidden bg-[var(--bg-raised)] border border-[var(--hairline)]"
          >
            {image.url ? (
              <Image
                src={image.url}
                alt={`Product angle ${index + 1}`}
                fill
                priority={index === 0}
                sizes="60vw"
                className="object-cover object-center"
                draggable={false}
              />
            ) : (
              <div className="absolute inset-0 flex items-center justify-center text-[var(--text-muted)]">
                <PlaceholderImage size="32" />
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Mobile layout: Embla Carousel */}
      <div className="lg:hidden relative">
        <div className="overflow-hidden rounded-[16px] border border-[var(--hairline)]" ref={emblaRef}>
          <div className="flex touch-pan-y">
            {images.map((image, index) => (
              <div
                key={image.id || index}
                className="relative aspect-[4/5] min-w-0 flex-[0_0_100%] bg-[var(--bg-raised)]"
              >
                {image.url ? (
                  <Image
                    src={image.url}
                    alt={`Product view ${index + 1}`}
                    fill
                    priority={index === 0}
                    sizes="100vw"
                    className="object-cover object-center"
                    draggable={false}
                  />
                ) : (
                  <div className="absolute inset-0 flex items-center justify-center text-[var(--text-muted)]">
                    <PlaceholderImage size="32" />
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Mobile controls: Next/Prev arrows */}
        {images.length > 1 && (
          <>
            <button
              type="button"
              onClick={scrollPrev}
              disabled={selectedIndex === 0}
              aria-label="Previous image"
              className="absolute left-3 top-1/2 -translate-y-1/2 h-8 w-8 rounded-full bg-[var(--bg-surface)]/80 backdrop-blur-sm border border-[var(--hairline)] flex items-center justify-center text-[var(--text-primary)] disabled:opacity-30 lustre-btn-press"
            >
              <CaretLeft size={16} />
            </button>
            <button
              type="button"
              onClick={scrollNext}
              disabled={selectedIndex === images.length - 1}
              aria-label="Next image"
              className="absolute right-3 top-1/2 -translate-y-1/2 h-8 w-8 rounded-full bg-[var(--bg-surface)]/80 backdrop-blur-sm border border-[var(--hairline)] flex items-center justify-center text-[var(--text-primary)] disabled:opacity-30 lustre-btn-press"
            >
              <CaretRight size={16} />
            </button>

            {/* Dot indicators */}
            <div className="flex justify-center items-center gap-1.5 mt-3">
              {scrollSnaps.map((_, index) => (
                <button
                  key={index}
                  type="button"
                  onClick={() => scrollTo(index)}
                  aria-label={`Go to slide ${index + 1}`}
                  className={`h-1.5 rounded-full transition-all duration-200 ${
                    index === selectedIndex
                      ? "w-4 bg-[var(--accent-sapphire)]"
                      : "w-1.5 bg-[var(--hairline)] hover:bg-[var(--text-muted)]"
                  }`}
                />
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  )
}

export default ImageGallery
