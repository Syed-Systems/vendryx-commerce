"use client"

import { useState, type FormEvent } from "react"
import { ArrowRight, Check } from "lucide-react"
import styles from "../homedine-eco.module.css"

interface FooterProps {
  footerNote?: string
}

export function Footer({ footerNote }: FooterProps) {
  const [email, setEmail] = useState("")
  const [subscribed, setSubscribed] = useState(false)
  const [error, setError] = useState("")

  const handleSubscribe = (e: FormEvent) => {
    e.preventDefault()
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.trim())) {
      setError("Please enter a valid email address.")
      return
    }
    setError("")
    setSubscribed(true)
    setEmail("")
  }

  return (
    <footer className="border-t border-[var(--color-border-hairline)] bg-[#f4f5ee] text-[#1a1f1c]">
      <div className="mx-auto max-w-[1360px] px-4 py-14 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 md:grid-cols-12 gap-10 pb-12 border-b border-black/[0.08]">
          {/* Brand Column */}
          <div className="md:col-span-5">
            <a
              href="#top"
              className="inline-block text-[26px] leading-none tracking-[-0.02em] text-[#1a1f1c]"
            >
              <span className="font-semibold">Home</span>
              <span className={`${styles.fontEditorial} font-normal ml-[1px]`}>dine</span>
            </a>
            <p className="mt-4 text-[14px] leading-[1.65] text-[#4e5651] max-w-[340px]">
              Non-toxic, planet-first kitchenware designed to bring enduring craft and
              calm rituals into everyday cooking.
            </p>
          </div>

          {/* Shop Links */}
          <div className="md:col-span-2">
            <h3 className="text-[13px] font-semibold text-[#1a1f1c] mb-3.5">Collection</h3>
            <ul className="space-y-2.5 text-[14px] text-[#4e5651]">
              <li>
                <a href="#bestsellers" className="hover:text-[#0e5437] transition-colors">
                  Bestsellers
                </a>
              </li>
              <li>
                <a href="#categories" className="hover:text-[#0e5437] transition-colors">
                  Categories
                </a>
              </li>
              <li>
                <a href="#spotlights" className="hover:text-[#0e5437] transition-colors">
                  New Arrivals
                </a>
              </li>
              <li>
                <a href="#gallery" className="hover:text-[#0e5437] transition-colors">
                  Gallery
                </a>
              </li>
            </ul>
          </div>

          {/* Standards Links */}
          <div className="md:col-span-2">
            <h3 className="text-[13px] font-semibold text-[#1a1f1c] mb-3.5">Standards</h3>
            <ul className="space-y-2.5 text-[14px] text-[#4e5651]">
              <li>
                <a href="#about" className="hover:text-[#0e5437] transition-colors">
                  Materials
                </a>
              </li>
              <li>
                <a href="#about" className="hover:text-[#0e5437] transition-colors">
                  Zero-PFAS Pledge
                </a>
              </li>
              <li>
                <a href="#about" className="hover:text-[#0e5437] transition-colors">
                  Carbon-Neutral Shipping
                </a>
              </li>
            </ul>
          </div>

          {/* Newsletter Column */}
          <div className="md:col-span-3">
            <h3 className="text-[13px] font-semibold text-[#1a1f1c] mb-3.5">
              Seasonal Dispatches
            </h3>
            <p className="text-[13px] leading-[1.55] text-[#4e5651] mb-4">
              Early access to small-batch stoneware releases and zero-waste kitchen guides.
            </p>

            {subscribed ? (
              <div className="flex items-center gap-2 rounded-2xl bg-[#0e5437]/10 px-4 py-3 text-[13px] font-medium text-[#0e5437]">
                <Check className="w-4 h-4 shrink-0" />
                <span>You&apos;re subscribed to Homedine dispatches.</span>
              </div>
            ) : (
              <form onSubmit={handleSubscribe} noValidate className="space-y-2">
                <div className="flex items-center gap-2">
                  <label htmlFor="footer-email" className="sr-only">
                    Email address
                  </label>
                  <input
                    id="footer-email"
                    type="email"
                    value={email}
                    onChange={(e) => {
                      setEmail(e.target.value)
                      if (error) setError("")
                    }}
                    placeholder="you@example.com"
                    className="h-10 flex-1 min-w-0 rounded-full bg-white px-4 text-[13px] text-[#1a1f1c] placeholder:text-[#767e78] border border-black/10 focus:border-[#0e5437] focus:outline-none"
                  />
                  <button
                    type="submit"
                    aria-label="Subscribe to newsletter"
                    className="h-10 w-10 rounded-full bg-[#0e5437] hover:bg-[#0a3f29] text-white flex items-center justify-center shrink-0 transition-colors cursor-pointer"
                  >
                    <ArrowRight className="w-4 h-4" />
                  </button>
                </div>
                {error && <p className="text-[12px] text-[#a33327] pl-2">{error}</p>}
              </form>
            )}
          </div>
        </div>

        {/* Bottom Quiet Copyright Bar */}
        <div className="pt-7 flex flex-col sm:flex-row items-center justify-between gap-3 text-[12px] text-[#767e78]">
          <p>© {new Date().getFullYear()} Homedine Kitchenware. Crafted for greener homes.</p>
          <p>{footerNote ?? "Plastic-free shipping · Certified non-toxic materials"}</p>
        </div>
      </div>
    </footer>
  )
}
