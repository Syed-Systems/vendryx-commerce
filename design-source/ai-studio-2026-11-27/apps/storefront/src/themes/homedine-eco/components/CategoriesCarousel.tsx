"use client"

import { useState, useEffect, useRef } from "react"
import gsap from "gsap"
import { ArrowLeft, ArrowRight } from "lucide-react"
import {
  getNextCarouselIndex,
  getPrevCarouselIndex,
  getVisibleWindow,
  type HomedineCategory,
} from "../catalog"
import { ResilientImage } from "./ResilientImage"
import styles from "../homedine-eco.module.css"

interface CategoriesCarouselProps {
  categories: HomedineCategory[]
  activeCategoryId: string | null
  onSelectCategory: (categoryId: string, label: string) => void
}

export function CategoriesCarousel({
  categories,
  activeCategoryId,
  onSelectCategory,
}: CategoriesCarouselProps) {
  const [startIndex, setStartIndex] = useState(0)
  const gridRef = useRef<HTMLDivElement | null>(null)
  const hasInteracted = useRef(false)

  const visibleCount = 4
  const visibleCategories = getVisibleWindow(categories, startIndex, visibleCount)

  useEffect(() => {
    if (!hasInteracted.current || !gridRef.current) return
    const cards = gridRef.current.querySelectorAll("article")
    if (cards.length === 0) return
    gsap.fromTo(
      cards,
      { opacity: 0.45, y: 12 },
      {
        opacity: 1,
        y: 0,
        duration: 0.36,
        stagger: 0.04,
        ease: "power3.out",
        clearProps: "transform,opacity",
      }
    )
  }, [startIndex])

  const handlePrev = () => {
    hasInteracted.current = true
    setStartIndex((prev) => getPrevCarouselIndex(prev, categories.length, visibleCount))
  }

  const handleNext = () => {
    hasInteracted.current = true
    setStartIndex((prev) => getNextCarouselIndex(prev, categories.length, visibleCount))
  }

  return (
    <section
      id="categories"
      aria-labelledby="categories-heading"
      className="mx-auto max-w-[1360px] px-4 pt-16 sm:px-6 sm:pt-24 lg:px-8"
    >
      {/* Header Row */}
      <div
        data-gsap="reveal"
        className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 mb-9"
      >
        <div>
          <h2
            id="categories-heading"
            className="text-[28px] sm:text-[34px] leading-[1.15] tracking-[-0.02em] text-[#1a1f1c] font-medium"
          >
            <span className={`${styles.fontEditorial} font-normal mr-1.5`}>Shop</span>
            <span>by Categories</span>
          </h2>
          <p className="mt-1.5 text-[14px] text-[#4e5651]">
            Curated zero-waste collections for every corner of your kitchen.
          </p>
        </div>

        <div className="hidden lg:flex items-center gap-2">
          <button
            type="button"
            onClick={handlePrev}
            aria-label="Previous categories"
            className="w-10 h-10 rounded-full bg-white border border-black/10 text-[#1a1f1c] hover:bg-[#0e5437] hover:text-white hover:border-[#0e5437] flex items-center justify-center transition-colors cursor-pointer"
          >
            <ArrowLeft className="w-4 h-4" strokeWidth={1.8} />
          </button>
          <button
            type="button"
            onClick={handleNext}
            aria-label="Next categories"
            className="w-10 h-10 rounded-full bg-white border border-black/10 text-[#1a1f1c] hover:bg-[#0e5437] hover:text-white hover:border-[#0e5437] flex items-center justify-center transition-colors cursor-pointer"
          >
            <ArrowRight className="w-4 h-4" strokeWidth={1.8} />
          </button>
        </div>
      </div>

      {/* Category Cards Grid */}
      <div>
        <div
          ref={gridRef}
          data-gsap="stagger-group"
          className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5"
        >
          {visibleCategories.map((category) => {
            const isSelected = activeCategoryId === category.filterCategoryId
            return (
              <article
                key={category.id}
                data-gsap-item
                className={`${styles.categoryCard} group relative overflow-hidden rounded-[var(--radius-card)] aspect-[4/5.1] bg-[#1c2620] ${
                  isSelected ? "ring-2 ring-offset-2 ring-[#0e5437]" : ""
                }`}
              >
                <ResilientImage
                  src={category.image}
                  alt={category.alt}
                  fallbackTitle={category.title}
                  className={`${styles.categoryCardImg} w-full h-full object-cover object-center`}
                />

                {/* Bottom contrast gradient scrim */}
                <div className="absolute inset-0 bg-gradient-to-t from-black/[0.82] via-black/[0.26] to-black/10 group-hover:from-black/[0.88] transition-colors duration-500" />

                {/* Bottom Centered Label & Cream Shop Button */}
                <div className="absolute inset-x-0 bottom-0 p-6 flex flex-col items-center text-center">
                  <p className="text-[14px] text-white/85 font-normal">{category.prefix}</p>
                  <h3
                    className={`${styles.fontEditorial} text-[26px] leading-[1.15] text-white font-normal mt-0.5`}
                  >
                    {category.title}
                  </h3>
                  <p className="mt-1.5 text-[12px] text-white/75 line-clamp-1">
                    {category.subtitle}
                  </p>
                  <button
                    type="button"
                    onClick={() =>
                      onSelectCategory(category.filterCategoryId, category.title)
                    }
                    className={`${styles.btnArrowGlide} mt-4 inline-flex items-center gap-1.5 h-9 px-5 rounded-full bg-[var(--color-brand-cream)] hover:bg-[var(--color-brand-cream-hover)] text-[#1a1f1c] text-[13px] font-semibold transition-all shadow-md whitespace-nowrap shrink-0 cursor-pointer`}
                  >
                    <span>Shop</span>
                    <ArrowRight className={`${styles.arrowIcon} w-3.5 h-3.5`} strokeWidth={2.2} />
                  </button>
                </div>
              </article>
            )
          })}
        </div>

        {/* Mobile/Tablet arrows */}
        <div className="flex lg:hidden items-center justify-end gap-2 mt-5">
          <button
            type="button"
            onClick={handlePrev}
            aria-label="Previous categories"
            className="w-10 h-10 rounded-full bg-white border border-black/10 flex items-center justify-center text-[#1a1f1c] cursor-pointer"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <button
            type="button"
            onClick={handleNext}
            aria-label="Next categories"
            className="w-10 h-10 rounded-full bg-white border border-black/10 flex items-center justify-center text-[#1a1f1c] cursor-pointer"
          >
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>
      </div>
    </section>
  )
}
