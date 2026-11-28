"use client"

import { ArrowRight } from "lucide-react"
import type { HomedineSpotlight } from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

interface EditorialSpotlightsProps {
  spotlights: HomedineSpotlight[]
  onShopSpotlight: (productId: string) => void
}

export function EditorialSpotlights({
  spotlights,
  onShopSpotlight,
}: EditorialSpotlightsProps) {
  return (
    <section
      id="spotlights"
      aria-label="Featured seasonal kitchenware collections"
      className="mx-auto max-w-[1360px] px-4 pt-16 sm:px-6 sm:pt-24 lg:px-8 space-y-12 sm:space-y-16"
    >
      {spotlights.map((item) => (
        <div
          key={item.id}
          data-gsap="spotlight-row"
          className="grid grid-cols-1 lg:grid-cols-12 gap-8 lg:gap-12 items-center"
        >
          {/* Editorial Copy Column */}
          <div
            data-gsap-copy
            className={`lg:col-span-5 flex flex-col items-start ${
              item.reverse ? "lg:order-2 lg:pl-6" : "lg:order-1 lg:pr-6"
            }`}
          >
            <h2 className="text-[32px] sm:text-[40px] leading-[1.15] tracking-[-0.02em] text-[#1a1f1c] font-medium">
              <span>{item.headingPlain} </span>
              <span className={`${styles.fontEditorial} font-normal`}>{item.headingItalic}</span>
            </h2>
            <p className="mt-4 text-[15px] leading-[1.68] text-[#4e5651] max-w-[420px]">
              {item.description}
            </p>
            <button
              type="button"
              onClick={() => onShopSpotlight(item.productId)}
              data-magnetic
              data-magnetic-strength="0.2"
              className={`${styles.btnArrowGlide} mt-7 inline-flex items-center gap-2.5 h-11 px-6 rounded-full bg-[var(--color-brand-forest)] hover:bg-[var(--color-brand-forest-hover)] text-white text-[14px] font-medium transition-colors whitespace-nowrap shrink-0 cursor-pointer`}
            >
              <span>Shop now</span>
              <ArrowRight className={`${styles.arrowIcon} w-4 h-4`} strokeWidth={2} />
            </button>
          </div>

          {/* Editorial Photo Column */}
          <div
            data-gsap-media
            className={`lg:col-span-7 ${item.reverse ? "lg:order-1" : "lg:order-2"}`}
          >
            <div className="group relative overflow-hidden rounded-[var(--radius-banner)] aspect-[4/3] bg-[#d6d4cb]">
              <ResilientImage
                src={item.image}
                alt={item.alt}
                fallbackTitle={`${item.headingPlain} ${item.headingItalic}`}
                className="w-full h-full object-cover object-center transition-transform duration-700 ease-out group-hover:scale-[1.045]"
              />
            </div>
          </div>
        </div>
      ))}
    </section>
  )
}
