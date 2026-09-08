"use client"

import React, { useEffect, useRef, useState } from "react"

type SectionRevealProps = {
  children: React.ReactNode
  className?: string
  stagger?: boolean
}

/**
 * Client-side section reveal using IntersectionObserver (§6 Motion system).
 * Fades in and slides up by 16px over 360ms with standard easing.
 * Respects prefers-reduced-motion.
 */
export default function SectionReveal({
  children,
  className = "",
  stagger = false,
}: SectionRevealProps) {
  const ref = useRef<HTMLDivElement>(null)
  const [isVisible, setIsVisible] = useState(false)

  useEffect(() => {
    const el = ref.current
    if (!el) return

    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setIsVisible(true)
      return
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true)
          observer.unobserve(el)
        }
      },
      { threshold: 0.1, rootMargin: "0px 0px -40px 0px" }
    )

    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  return (
    <div
      ref={ref}
      className={`lustre-reveal ${isVisible ? "visible" : ""} ${
        stagger ? "lustre-reveal-stagger" : ""
      } ${className}`}
    >
      {children}
    </div>
  )
}
