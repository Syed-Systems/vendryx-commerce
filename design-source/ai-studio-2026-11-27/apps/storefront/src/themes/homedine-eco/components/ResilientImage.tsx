"use client"

import { useState, type CSSProperties } from "react"
import { Leaf } from "lucide-react"

interface ResilientImageProps {
  src: string
  alt: string
  className?: string
  style?: CSSProperties
  loading?: "eager" | "lazy"
  fallbackTitle?: string
}

export function ResilientImage({
  src,
  alt,
  className = "",
  style,
  loading = "lazy",
  fallbackTitle,
}: ResilientImageProps) {
  const [failed, setFailed] = useState(false)

  if (failed) {
    return (
      <div
        role="img"
        aria-label={alt}
        className={`flex flex-col items-center justify-center bg-[#dfe3d5] text-[#0e5437] p-6 text-center ${className}`}
        style={style}
      >
        <Leaf className="w-7 h-7 mb-2 opacity-60" strokeWidth={1.5} />
        <span className="text-xs font-medium tracking-wide opacity-75">
          {fallbackTitle ?? "Homedine Craft"}
        </span>
      </div>
    )
  }

  return (
    <img
      src={src}
      alt={alt}
      loading={loading}
      decoding="async"
      onError={() => setFailed(true)}
      className={className}
      style={style}
    />
  )
}
