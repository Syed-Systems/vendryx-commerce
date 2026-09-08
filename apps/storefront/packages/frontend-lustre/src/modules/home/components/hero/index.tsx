"use client"

import { useRef } from "react"
import Image from "next/image"
import { useGSAP } from "@gsap/react"
import gsap from "gsap"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import heroStillLife from "../../../../assets/lustre-hero-still-life.png"

/**
 * A theme-owned editorial image, never a representation of a catalog SKU.
 * Product imagery and product claims continue to come from core data.
 */
export default function Hero() {
  const scope = useRef<HTMLElement>(null)

  useGSAP(
    () => {
      if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
        return
      }

      gsap
        .timeline({ defaults: { ease: "power3.out" } })
        .from("[data-hero-copy] > *", {
          opacity: 0,
          y: 18,
          duration: 0.55,
          stagger: 0.07,
        })
        .from(
          "[data-hero-media]",
          { opacity: 0, y: 24, duration: 0.7 },
          0.08
        )
    },
    { scope }
  )

  return (
    <section
      ref={scope}
      aria-labelledby="lustre-hero-title"
      className="bg-[var(--bg-canvas)] px-6 pb-16 pt-10 sm:px-8 sm:pb-24 lg:px-12 lg:pb-28 lg:pt-14"
    >
      <div className="mx-auto grid min-h-[calc(100dvh-7rem)] max-w-[1440px] grid-cols-1 items-center gap-10 lg:grid-cols-12 lg:gap-16">
        <div data-hero-copy className="lg:col-span-5">
          <p className="mb-5 text-xs font-semibold uppercase tracking-[0.18em] text-[var(--accent-sapphire)]">
            Watches and ornaments
          </p>
          <h1
            id="lustre-hero-title"
            className="max-w-[10ch] text-[clamp(3rem,7vw,6.5rem)] font-medium leading-[0.94] tracking-[-0.055em] text-[var(--text-primary)]"
          >
            Objects worth keeping.
          </h1>
          <p className="mt-7 max-w-[34rem] text-base leading-7 text-[var(--text-secondary)] sm:text-lg">
            Explore watches and ornaments through material, proportion, and presence.
          </p>
          <div className="mt-9">
            <LocalizedClientLink
              href="/store"
              className="lustre-btn-press inline-flex min-h-12 items-center justify-center rounded-full bg-[var(--accent-sapphire)] px-7 text-sm font-semibold text-white hover:bg-[var(--accent-sapphire-hover)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--hairline-focus)]"
            >
              View collection
            </LocalizedClientLink>
          </div>
        </div>

        <figure data-hero-media className="lg:col-span-7">
          <div className="relative aspect-[4/5] overflow-hidden rounded-[var(--radius-surface)] bg-[var(--bg-raised)] sm:aspect-[5/4] lg:aspect-[4/5]">
            <Image
              src={heroStillLife}
              alt=""
              priority
              sizes="(min-width: 1024px) 56vw, 100vw"
              className="h-full w-full object-cover"
            />
          </div>
          <figcaption className="mt-3 text-xs text-[var(--text-muted)]">
            Editorial still life. Catalog items are shown separately.
          </figcaption>
        </figure>
      </div>
    </section>
  )
}
