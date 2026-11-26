"use client"

import { useState } from "react"
import { ArrowLeft, ArrowRight, Quote, Star } from "lucide-react"
import type { HomedineReview } from "../catalog"

interface ReviewsSectionProps {
  reviews: HomedineReview[]
}

export function ReviewsSection({ reviews }: ReviewsSectionProps) {
  const [activeIndex, setActiveIndex] = useState(1)

  const handlePrev = () => {
    setActiveIndex((prev) => (prev - 1 + reviews.length) % reviews.length)
  }

  const handleNext = () => {
    setActiveIndex((prev) => (prev + 1) % reviews.length)
  }

  return (
    <section
      id="reviews"
      aria-labelledby="reviews-heading"
      className="mx-auto max-w-[1360px] px-4 pt-16 sm:px-6 sm:pt-24 lg:px-8"
    >
      <div
        data-gsap="reveal"
        className="rounded-[var(--radius-banner)] bg-[var(--color-surface-sage)] px-5 py-10 sm:px-10 sm:py-14 lg:px-12"
      >
        {/* Top Rating Summary Row */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-6 mb-10">
          <div className="flex items-center gap-5">
            <div className="flex items-baseline gap-1 shrink-0">
              <span
                id="reviews-heading"
                className="text-[42px] sm:text-[48px] leading-none font-medium tracking-[-0.03em] text-[#1a1f1c] tabular-nums"
              >
                4.9
              </span>
              <span className="text-[18px] text-[#767e78] font-medium tabular-nums">/5</span>
            </div>
            <div className="h-10 w-px bg-black/10 hidden sm:block" aria-hidden="true" />
            <div>
              <div className="flex items-center gap-1 mb-1" aria-label="Rated 4.9 out of 5 stars">
                {Array.from({ length: 5 }).map((_, i) => (
                  <Star
                    key={i}
                    className="w-3.5 h-3.5 text-[#0e5437] fill-[#0e5437]"
                    aria-hidden="true"
                  />
                ))}
              </div>
              <p className="text-[13px] sm:text-[14px] leading-[1.45] text-[#4e5651] max-w-[280px]">
                More than <strong className="font-semibold text-[#1a1f1c]">25,000 5-Star Reviews</strong>{" "}
                for our award-winning eco kitchenware
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={handlePrev}
              aria-label="Highlight previous review"
              className="w-10 h-10 rounded-full bg-white/80 hover:bg-[#0e5437] text-[#1a1f1c] hover:text-white border border-black/[0.08] flex items-center justify-center transition-colors cursor-pointer"
            >
              <ArrowLeft className="w-4 h-4" strokeWidth={1.8} />
            </button>
            <button
              type="button"
              onClick={handleNext}
              aria-label="Highlight next review"
              className="w-10 h-10 rounded-full bg-white/80 hover:bg-[#0e5437] text-[#1a1f1c] hover:text-white border border-black/[0.08] flex items-center justify-center transition-colors cursor-pointer"
            >
              <ArrowRight className="w-4 h-4" strokeWidth={1.8} />
            </button>
          </div>
        </div>

        {/* Review Cards Grid */}
        <div
          data-gsap="stagger-group"
          className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-5 items-stretch"
        >
          {reviews.map((review, index) => {
            const isSpotlight = index === activeIndex
            return (
              <button
                key={review.id}
                type="button"
                data-gsap-item
                onClick={() => setActiveIndex(index)}
                aria-pressed={isSpotlight}
                className={`text-left rounded-[18px] p-6 flex flex-col justify-between min-h-[245px] transition-all duration-300 cursor-pointer ${
                  isSpotlight
                    ? "bg-white text-[#1a1f1c] shadow-[var(--shadow-card-hover)] lg:-translate-y-2 border border-black/[0.06]"
                    : "bg-white/[0.52] hover:bg-white/80 text-[#3f4742] border border-black/[0.04]"
                }`}
              >
                <div>
                  <Quote
                    className={`w-6 h-6 mb-4 transition-colors ${
                      isSpotlight ? "text-[#0e5437]" : "text-[#8c948e]"
                    }`}
                    strokeWidth={1.6}
                    aria-hidden="true"
                  />
                  <p
                    className={`text-[14px] leading-[1.6] ${
                      isSpotlight ? "text-[#1a1f1c] font-medium" : "text-[#4e5651]"
                    }`}
                  >
                    {review.quote}
                  </p>
                </div>

                <div className="mt-6 pt-4 border-t border-black/[0.06]">
                  <p className="text-[14px] font-semibold text-[#1a1f1c]">{review.author}</p>
                  <p className="text-[12px] text-[#767e78] mt-0.5">
                    {review.role} · {review.verifiedProduct}
                  </p>
                </div>
              </button>
            )
          })}
        </div>
      </div>
    </section>
  )
}
