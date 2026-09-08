"use client"

import React, { useEffect, useState } from "react"

export default function NavShell({ children }: { children: React.ReactNode }) {
  const [scrolled, setScrolled] = useState(false)

  useEffect(() => {
    let ticking = false

    const onScroll = () => {
      if (!ticking) {
        window.requestAnimationFrame(() => {
          setScrolled(window.scrollY > 10)
          ticking = false
        })
        ticking = true
      }
    }

    window.addEventListener("scroll", onScroll, { passive: true })
    onScroll()
    return () => window.removeEventListener("scroll", onScroll)
  }, [])

  return (
    <nav
      className={`pointer-events-auto max-w-[1440px] mx-auto h-[72px] px-6 rounded-[16px] border border-[var(--frosted-border)] shadow-[var(--elevation-nav)] backdrop-blur-md flex items-center justify-between transition-[background-color] duration-200 ${
        scrolled
          ? "bg-[rgba(244,246,247,0.95)] dark:bg-[rgba(10,12,16,0.95)]"
          : "bg-[var(--frosted-bg)]"
      }`}
      aria-label="Main navigation"
    >
      {children}
    </nav>
  )
}
