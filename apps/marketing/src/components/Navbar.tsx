import { Menu, X } from "lucide-react"
import { useState } from "react"

interface NavbarProps {
  onOpenOnboarding: (plan: "trial" | "free" | "pro") => void
}

const NAV_LINKS = [
  { label: "Dashboard", href: "#cockpit" },
  { label: "AI & Social", href: "#social-okf" },
  { label: "Features", href: "#operations" },
  { label: "Reliability", href: "#engine" },
  { label: "Pricing", href: "#pricing" },
] as const

export function Navbar({ onOpenOnboarding }: NavbarProps) {
  const [mobileOpen, setMobileOpen] = useState(false)

  return (
    <header className="sticky top-0 z-40 border-b border-[#FFFCF8]/[0.08] bg-[#080A0C]/85 backdrop-blur-md">
      <div className="mx-auto flex h-14 max-w-[1200px] items-center justify-between px-4 sm:px-6">
        <a
          href="#"
          className="flex shrink-0 items-center gap-2.5 rounded-md focus:outline-none focus-visible:ring-2 focus-visible:ring-[#FF7A00]"
          aria-label="Vendryx home"
        >
          <img
            src="/brand/vendryx/vendryx-mark.svg"
            alt="Vendryx emblem"
            width={28}
            height={28}
            className="h-7 w-7 shrink-0 rounded-[6px]"
          />
          <span className="text-[15px] font-semibold tracking-[-0.02em] text-[#FFFCF8]">
            Vendryx
          </span>
        </a>

        <nav aria-label="Primary" className="hidden items-center gap-7 md:flex">
          {NAV_LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="text-[13px] font-medium text-[#94A3B8] transition-colors hover:text-[#FFFCF8]"
            >
              {link.label}
            </a>
          ))}
        </nav>

        <div className="flex items-center gap-2.5">
          <a
            href="#pricing"
            className="hidden px-2 py-1.5 text-[13px] font-medium text-[#94A3B8] transition-colors hover:text-[#FFFCF8] sm:inline-block"
          >
            Free plan
          </a>
          <button
            type="button"
            onClick={() => onOpenOnboarding("trial")}
            className="shrink-0 whitespace-nowrap rounded-md bg-[#FF7A00] px-3.5 py-1.5 text-[13px] font-semibold text-[#080A0C] transition-colors hover:bg-[#D95700] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#FFAF24]"
          >
            Start free trial
          </button>
          <button
            type="button"
            onClick={() => setMobileOpen((prev) => !prev)}
            className="inline-flex h-9 w-9 items-center justify-center rounded-md border border-[#FFFCF8]/10 text-[#94A3B8] hover:text-[#FFFCF8] md:hidden"
            aria-label={mobileOpen ? "Close navigation menu" : "Open navigation menu"}
            aria-expanded={mobileOpen}
          >
            {mobileOpen ? <X className="h-4 w-4" /> : <Menu className="h-4 w-4" />}
          </button>
        </div>
      </div>

      {mobileOpen && (
        <div className="border-t border-[#FFFCF8]/[0.08] bg-[#0F1115] px-4 py-3 md:hidden">
          <nav aria-label="Mobile navigation" className="flex flex-col gap-1">
            {NAV_LINKS.map((link) => (
              <a
                key={link.href}
                href={link.href}
                onClick={() => setMobileOpen(false)}
                className="rounded-md px-3 py-2 text-[14px] font-medium text-[#CBD5E1] hover:bg-[#1B1E24] hover:text-[#FFFCF8]"
              >
                {link.label}
              </a>
            ))}
          </nav>
        </div>
      )}
    </header>
  )
}
