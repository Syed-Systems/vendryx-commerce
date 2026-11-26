"use client"

import { Globe, Leaf, Sprout } from "lucide-react"
import { ASSETS, PILLARS } from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

const PILLAR_ICONS = [Leaf, Globe, Sprout] as const

export function LifestyleBanner() {
  return (
    <section
      id="about"
      aria-label="Homedine craftsmanship and sustainability pillars"
      className="mx-auto max-w-[1360px] px-4 pt-14 sm:px-6 sm:pt-20 lg:px-8"
    >
      {/* Full-Width Architectural Kitchen Manifesto Banner */}
      <div
        data-gsap="reveal"
        className="relative overflow-hidden rounded-[var(--radius-banner)] min-h-[360px] sm:min-h-[440px] lg:min-h-[500px] flex items-end bg-[#28382c]"
      >
        <div className="absolute inset-0 overflow-hidden pointer-events-none">
          <div data-gsap="parallax-media" className="w-full h-[116%] -mt-[8%]">
            <ResilientImage
              src={ASSETS.lifestyleKitchen}
              alt="Serene contemporary sage-green architectural kitchen with white stone island"
              className="w-full h-full object-cover object-center"
            />
          </div>
          {/* Bottom-left contrast scrim */}
          <div className="absolute inset-0 bg-gradient-to-t from-[#08170f]/[0.82] via-[#08170f]/[0.28] to-transparent" />
        </div>

        <div className="relative z-10 p-6 sm:p-10 lg:p-12 max-w-[680px]">
          <p className="text-[20px] sm:text-[26px] lg:text-[28px] leading-[1.34] tracking-[-0.015em] text-white font-normal">
            We craft{" "}
            <span className={`${styles.fontEditorial} font-normal`}>kitchenware</span> you can
            trust for years to come — through everyday meals and{" "}
            <span className={`${styles.fontEditorial} font-normal`}>evolving lifestyles</span>.
            Each piece is thoughtfully made with sustainable materials.
          </p>
        </div>
      </div>

      {/* 3-Column Value Pillars Strip */}
      <div
        data-gsap="stagger-group"
        className="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4 sm:gap-5"
      >
        {PILLARS.map((pillar, idx) => {
          const IconComponent = PILLAR_ICONS[idx]
          const isCenterPill = idx === 1
          return (
            <div
              key={pillar.id}
              data-gsap-item
              className={`${styles.pillarCard} group bg-[var(--color-surface-sage)] px-7 py-7 sm:py-8 flex items-center justify-center gap-4 border border-black/[0.04] ${
                isCenterPill
                  ? "rounded-[var(--radius-pillar-pill)]"
                  : "rounded-[var(--radius-card)]"
              }`}
            >
              <div className="w-12 h-12 rounded-full bg-white flex items-center justify-center shrink-0 shadow-sm transition-transform duration-300 group-hover:scale-110">
                <IconComponent
                  className="w-5 h-5 text-[#0e5437]"
                  strokeWidth={1.75}
                  aria-hidden="true"
                />
              </div>
              <div className="text-left">
                <p className="text-[19px] sm:text-[21px] leading-[1.15] text-[#1a1f1c]">
                  <span
                    className={`${styles.fontEditorial} block text-[21px] sm:text-[23px] font-normal`}
                  >
                    {pillar.italicLabel}
                  </span>
                  <span className="font-semibold tracking-[-0.015em]">
                    {pillar.boldLabel}
                  </span>
                </p>
                <p className="mt-1 text-[12px] leading-[1.4] text-[#4e5651] max-w-[200px]">
                  {pillar.detail}
                </p>
              </div>
            </div>
          )
        })}
      </div>
    </section>
  )
}
