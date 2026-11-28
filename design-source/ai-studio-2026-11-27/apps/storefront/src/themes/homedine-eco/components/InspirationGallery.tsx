"use client"

import { useEffect, useRef, useState } from "react"
import gsap from "gsap"
import { ArrowLeft, ArrowRight, Eye, Sparkles } from "lucide-react"
import type { HomedineGalleryItem } from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

interface InspirationGalleryProps {
  items: HomedineGalleryItem[]
  onInspectGalleryItem: (item: HomedineGalleryItem) => void
}

export function InspirationGallery({
  items,
  onInspectGalleryItem,
}: InspirationGalleryProps) {
  const [shiftOffset, setShiftOffset] = useState(0)
  const gridRef = useRef<HTMLDivElement | null>(null)
  const hasInteracted = useRef(false)

  const orderedItems = [...items.slice(shiftOffset), ...items.slice(0, shiftOffset)]

  useEffect(() => {
    if (!hasInteracted.current || !gridRef.current) return
    const tiles = gridRef.current.querySelectorAll("button")
    if (tiles.length === 0) return
    gsap.fromTo(
      tiles,
      { opacity: 0.5, scale: 0.97 },
      {
        opacity: 1,
        scale: 1,
        duration: 0.4,
        stagger: 0.04,
        ease: "power3.out",
        clearProps: "transform,opacity",
      }
    )
  }, [shiftOffset])

  const handlePrev = () => {
    hasInteracted.current = true
    setShiftOffset((prev) => (prev - 1 + items.length) % items.length)
  }

  const handleNext = () => {
    hasInteracted.current = true
    setShiftOffset((prev) => (prev + 1) % items.length)
  }

  return (
    <section
      id="gallery"
      aria-labelledby="gallery-heading"
      className="mx-auto max-w-[1360px] px-4 pt-[72px] sm:px-6 sm:pt-24 lg:px-8"
    >
      {/* Header Row */}
      <div
        data-gsap="reveal"
        className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 mb-9"
      >
        <div>
          <p className="text-[18px] sm:text-[22px] leading-[1.3] tracking-[-0.015em] text-[#4e5651]">
            Thoughtful, Planet-Prioritizing Ideas
          </p>
          <h2
            id="gallery-heading"
            className="text-[24px] sm:text-[30px] leading-[1.2] tracking-[-0.02em] text-[#1a1f1c] font-medium flex items-center gap-2.5 flex-wrap"
          >
            <span>and Inspiration</span>
            <Sparkles
              className="w-[19px] h-[19px] text-[#1a1f1c] shrink-0"
              strokeWidth={1.6}
              aria-hidden="true"
            />
            <span className={`${styles.fontEditorial} font-normal`}>Gallery</span>
          </h2>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handlePrev}
            aria-label="Rotate gallery left"
            className="w-10 h-10 rounded-full border border-black/[0.12] bg-white text-[#1a1f1c] hover:bg-[#0e5437] hover:text-white hover:border-[#0e5437] flex items-center justify-center transition-colors cursor-pointer"
          >
            <ArrowLeft className="w-4 h-4" strokeWidth={1.8} />
          </button>
          <button
            type="button"
            onClick={handleNext}
            aria-label="Rotate gallery right"
            className="w-10 h-10 rounded-full border border-black/[0.12] bg-white text-[#1a1f1c] hover:bg-[#0e5437] hover:text-white hover:border-[#0e5437] flex items-center justify-center transition-colors cursor-pointer"
          >
            <ArrowRight className="w-4 h-4" strokeWidth={1.8} />
          </button>
        </div>
      </div>

      {/* 5-Column Asymmetric Editorial Gallery Strip */}
      <div
        ref={gridRef}
        data-gsap="stagger-group"
        className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4 items-start"
      >
        {orderedItems.map((item, idx) => {
          const heightClass = idx === 1 || idx === 3 ? "aspect-[4/5.2]" : "aspect-[4/3.8]"
          const offsetClass = idx === 0 || idx === 2 || idx === 4 ? "lg:mt-0" : "lg:mt-3"

          return (
            <button
              key={item.id}
              type="button"
              data-gsap-item
              onClick={() => onInspectGalleryItem(item)}
              aria-label={`Inspect ${item.title}`}
              className={`${styles.galleryTile} group relative w-full overflow-hidden rounded-[var(--radius-tile)] bg-[#232925] text-left cursor-pointer ${heightClass} ${offsetClass}`}
            >
              <ResilientImage
                src={item.image}
                alt={item.alt}
                fallbackTitle={item.title}
                className={`${styles.galleryTileImg} w-full h-full object-cover object-center`}
              />

              {/* Subtle bottom gradient scrim */}
              <div className="absolute inset-0 bg-gradient-to-t from-black/[0.78] via-black/15 to-transparent group-hover:from-black/[0.86] transition-colors duration-300" />

              {/* Hover quick inspect cue */}
              <span className="absolute top-3 right-3 w-8 h-8 rounded-full bg-white/85 backdrop-blur-sm text-[#1a1f1c] flex items-center justify-center opacity-0 translate-y-1 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-300">
                <Eye className="w-4 h-4" strokeWidth={1.8} />
              </span>

              {/* Bottom Caption */}
              <div className="absolute inset-x-0 bottom-0 p-3.5 sm:p-4">
                <p className="text-[13px] sm:text-[14px] leading-[1.3] font-medium text-white">
                  {item.title}
                </p>
              </div>
            </button>
          )
        })}
      </div>
    </section>
  )
}
