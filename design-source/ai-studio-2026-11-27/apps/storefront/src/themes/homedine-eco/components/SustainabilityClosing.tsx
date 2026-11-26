"use client"

import { ASSETS } from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

export function SustainabilityClosing() {
  return (
    <section
      aria-label="Homedine sustainability commitment"
      className="mx-auto max-w-[1100px] px-4 pt-20 pb-16 sm:px-6 sm:pt-28 sm:pb-24 lg:px-8 text-center"
    >
      {/* Layered 3-Image Editorial Fan-Out Collage */}
      <div
        data-gsap="collage-fan"
        className="relative mx-auto flex items-center justify-center max-w-[680px] h-[220px] sm:h-[290px]"
      >
        {/* Left Tilted Photo */}
        <div
          data-gsap-collage="left"
          className="w-[34%] sm:w-[32%] aspect-[4/3.4] overflow-hidden rounded-[18px] shadow-lg border-4 border-[var(--color-canvas)] -mr-8 sm:-mr-12 -rotate-6 transition-transform duration-500 hover:-rotate-3 hover:scale-[1.03] bg-[#cac8be] z-10"
        >
          <ResilientImage
            src={ASSETS.ceramicPots}
            alt="Non-toxic olive and terracotta ceramic cooking pots"
            className="w-full h-full object-cover"
          />
        </div>

        {/* Center Hero Collage Photo */}
        <div
          data-gsap-collage="center"
          className="relative z-20 w-[52%] sm:w-[48%] aspect-[4/3] overflow-hidden rounded-[22px] shadow-2xl border-4 border-[var(--color-canvas)] transition-transform duration-500 hover:scale-[1.03] bg-[#1c2e28]"
        >
          <ResilientImage
            src={ASSETS.gallerySkilletToast}
            alt="Deep teal non-toxic ceramic frying pan with artisan sourdough toast in morning sunlight"
            className="w-full h-full object-cover"
          />
        </div>

        {/* Right Tilted Photo */}
        <div
          data-gsap-collage="right"
          className="w-[34%] sm:w-[32%] aspect-[4/3.4] overflow-hidden rounded-[18px] shadow-lg border-4 border-[var(--color-canvas)] -ml-8 sm:-ml-12 rotate-6 transition-transform duration-500 hover:rotate-3 hover:scale-[1.03] bg-[#cac8be] z-10"
        >
          <ResilientImage
            src={ASSETS.storageJars}
            alt="Ribbed glass pantry storage jars with bamboo lids"
            className="w-full h-full object-cover"
          />
        </div>
      </div>

      {/* Editorial Manifesto Statement with Inline Miniature Thumbnails */}
      <div data-gsap="reveal" className="mt-12 sm:mt-14 max-w-[820px] mx-auto">
        <p className="text-[22px] sm:text-[28px] lg:text-[31px] leading-[1.48] tracking-[-0.015em] text-[#222824] font-normal">
          Discover our commitment to{" "}
          <span className="inline-flex align-middle mx-1 w-12 h-7 sm:w-14 sm:h-8 rounded-full overflow-hidden border border-black/10 shadow-sm">
            <ResilientImage
              src={ASSETS.galleryWoodenBoards}
              alt="Handcrafted olivewood grain texture"
              className="w-full h-full object-cover"
            />
          </span>{" "}
          <span className={`${styles.fontEditorial} font-medium text-[#1a1f1c]`}>
            sustainable
          </span>{" "}
          materials, low-impact production, and{" "}
          <span className={`${styles.fontEditorial} font-medium text-[#1a1f1c]`}>
            ethical sourcing
          </span>{" "}
          partnerships — all crafted to support a healthier planet and a{" "}
          <span className="inline-flex align-middle mx-1 w-12 h-7 sm:w-14 sm:h-8 rounded-full overflow-hidden border border-black/10 shadow-sm">
            <ResilientImage
              src={ASSETS.lifestyleKitchen}
              alt="Sunlit green eco-friendly kitchen"
              className="w-full h-full object-cover"
            />
          </span>{" "}
          <span className={`${styles.fontEditorial} font-medium text-[#0e5437]`}>
            greener kitchen.
          </span>
        </p>
      </div>
    </section>
  )
}
