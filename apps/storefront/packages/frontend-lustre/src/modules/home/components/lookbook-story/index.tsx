"use client"

import { useRef } from "react"
import Image from "next/image"
import gsap from "gsap"
import { ScrollTrigger } from "gsap/ScrollTrigger"
import { useGSAP } from "@gsap/react"

if (typeof window !== "undefined") {
  gsap.registerPlugin(ScrollTrigger)
}

export type LookbookStoryPassage = {
  title: string
  body: string
}

export type LookbookStoryProps = {
  image?: string
  alt?: string
  eyebrow?: string
  passages?: LookbookStoryPassage[]
}

/**
 * Material story: A single pinned macro image with three short, approved passages (§4.2 #4).
 * This is the only GSAP scroll narrative in Lustre.
 * Content-gated: Returns null if no editorial media and passages are provided.
 */
export default function LookbookStory({
  image,
  alt = "Material and craftsmanship detail",
  eyebrow = "Material Story",
  passages,
}: LookbookStoryProps) {
  const containerRef = useRef<HTMLElement>(null)
  const imageRef = useRef<HTMLDivElement>(null)
  const passagesRef = useRef<HTMLDivElement>(null)

  // Content-gated per plan §3.4 and §4.2: omit if no approved assets/copy are provided
  if (!image && (!passages || passages.length === 0)) {
    return null
  }

  useGSAP(
    () => {
      if (!containerRef.current || !imageRef.current || !passagesRef.current) return

      const isReducedMotion = window.matchMedia(
        "(prefers-reduced-motion: reduce)"
      ).matches

      // Under reduced motion: remove pinning and scrubbed transforms; retain simple static layout
      if (isReducedMotion) {
        return
      }

      // GSAP ScrollTrigger pinned macro image + scrubbed passage detail
      const passageElements = gsap.utils.toArray<HTMLElement>("[data-story-passage]")

      const trigger = ScrollTrigger.create({
        trigger: containerRef.current,
        start: "top top",
        end: "bottom bottom",
        pin: imageRef.current,
        pinSpacing: false,
      })

      passageElements.forEach((el) => {
        gsap.fromTo(
          el,
          { opacity: 0.25, y: 20 },
          {
            opacity: 1,
            y: 0,
            duration: 0.6,
            ease: "power2.out",
            scrollTrigger: {
              trigger: el,
              start: "top 60%",
              end: "bottom 40%",
              toggleActions: "play reverse play reverse",
            },
          }
        )
      })

      return () => {
        trigger.kill()
      }
    },
    { scope: containerRef }
  )

  return (
    <section
      ref={containerRef}
      aria-label={eyebrow}
      className="relative bg-[var(--bg-canvas)] px-6 py-20 sm:px-8 sm:py-28 lg:px-12 lg:py-32"
    >
      <div className="mx-auto grid max-w-[1440px] grid-cols-1 items-start gap-12 lg:grid-cols-12 lg:gap-16">
        {/* Pinned media container */}
        <div className="lg:col-span-6 relative h-full">
          <div
            ref={imageRef}
            className="w-full rounded-[var(--radius-surface)] overflow-hidden border border-[var(--hairline)] bg-[var(--bg-raised)] aspect-[4/5] relative"
          >
            {image ? (
              <Image
                src={image}
                alt={alt}
                fill
                className="object-cover"
                sizes="(max-width: 1024px) 100vw, 50vw"
              />
            ) : (
              <div className="flex h-full w-full items-center justify-center text-[var(--text-muted)] text-sm">
                No media provided
              </div>
            )}
          </div>
        </div>

        {/* Scrubbed passages */}
        <div ref={passagesRef} className="lg:col-span-6 flex flex-col space-y-32 py-8 lg:py-16">
          <div className="space-y-4">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--accent-sapphire)]">
              {eyebrow}
            </p>
          </div>

          {passages?.map((passage, index) => (
            <div
              key={index}
              data-story-passage
              className="space-y-4 max-w-lg transition-opacity duration-300"
            >
              <h2 className="lustre-section-title text-[var(--text-primary)] font-semibold">
                {passage.title}
              </h2>
              <p className="lustre-body text-[var(--text-secondary)]">
                {passage.body}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
