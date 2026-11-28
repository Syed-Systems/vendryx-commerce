import React from "react"

interface IconProps {
  className?: string
}

/**
 * Bespoke architectural icon pack exclusive to the hydro-vessel (KOVA) theme.
 * Zero shared icons with maison-cuir or any other storefront theme.
 */
export function AlpinePeakMark({ className = "h-4 w-6" }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 28 18" fill="none" aria-hidden="true">
      <path
        d="M2 16L10.5 3L16 11.5L19.5 6.5L26 16H2Z"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinejoin="round"
      />
    </svg>
  )
}

export function SunModeIcon({ className = "h-[18px] w-[18px]" }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 256 256" fill="none" aria-hidden="true">
      <circle cx="128" cy="128" r="54" stroke="currentColor" strokeWidth="18" />
      <line x1="128" y1="24" x2="128" y2="44" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
      <line x1="128" y1="212" x2="128" y2="232" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
      <line x1="24" y1="128" x2="44" y2="128" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
      <line x1="212" y1="128" x2="232" y2="128" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
      <line x1="54" y1="54" x2="68" y2="68" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
      <line x1="188" y1="188" x2="202" y2="202" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
      <line x1="54" y1="202" x2="68" y2="188" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
      <line x1="188" y1="68" x2="202" y2="54" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
    </svg>
  )
}

export function MoonModeIcon({ className = "h-[18px] w-[18px]" }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 256 256" fill="none" aria-hidden="true">
      <path
        d="M210.69 158.18A96.78 96.78 0 0 1 192 160 96.11 96.11 0 0 1 96 64a96.78 96.78 0 0 1 1.82-18.69A96.04 96.04 0 1 0 210.69 158.18Z"
        stroke="currentColor"
        strokeWidth="18"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

export function VesselBagIcon({ className = "h-[18px] w-[18px]" }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 256 256" fill="none" aria-hidden="true">
      <rect x="32" y="72" width="192" height="144" rx="16" stroke="currentColor" strokeWidth="18" />
      <path d="M88 72V64a40 40 0 0 1 80 0v8" stroke="currentColor" strokeWidth="18" strokeLinecap="round" />
    </svg>
  )
}

export function DiagonalExploreIcon({ className = "h-4 w-4" }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 256 256" fill="none" aria-hidden="true">
      <line x1="64" y1="64" x2="192" y2="192" stroke="currentColor" strokeWidth="20" strokeLinecap="round" />
      <polyline
        points="88 192 192 192 192 88"
        stroke="currentColor"
        strokeWidth="20"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

export function CompareDragIcon({ className = "h-5 w-5" }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="M8 7L3 12L8 17" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M16 7L21 12L16 17" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

export function VesselCloseIcon({ className = "h-[18px] w-[18px]" }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 256 256" fill="none" aria-hidden="true">
      <line x1="200" y1="56" x2="56" y2="200" stroke="currentColor" strokeWidth="20" strokeLinecap="round" />
      <line x1="200" y1="200" x2="56" y2="56" stroke="currentColor" strokeWidth="20" strokeLinecap="round" />
    </svg>
  )
}
