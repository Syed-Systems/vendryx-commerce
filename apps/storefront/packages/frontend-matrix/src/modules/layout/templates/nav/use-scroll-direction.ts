"use client"

import { useEffect, useState } from "react"

/**
 * PORT of The Matrix Hours `useScrollDirection`, unchanged in behaviour:
 * rAF-throttled, a 10px dead zone so a trackpad twitch does not toggle the bar,
 * and no hiding until 80px down so the header never flickers at the top.
 */
export default function useScrollDirection() {
  const [scrollDirection, setScrollDirection] = useState<"up" | "down">("up")
  const [lastScrollY, setLastScrollY] = useState(0)

  useEffect(() => {
    let ticking = false

    const updateScrollDirection = () => {
      const scrollY = window.scrollY

      if (Math.abs(scrollY - lastScrollY) < 10) {
        ticking = false
        return
      }

      setScrollDirection(scrollY > lastScrollY && scrollY > 80 ? "down" : "up")
      setLastScrollY(scrollY <= 0 ? 0 : scrollY)
      ticking = false
    }

    const onScroll = () => {
      if (!ticking) {
        window.requestAnimationFrame(updateScrollDirection)
        ticking = true
      }
    }

    window.addEventListener("scroll", onScroll, { passive: true })
    return () => window.removeEventListener("scroll", onScroll)
  }, [lastScrollY])

  return scrollDirection
}
