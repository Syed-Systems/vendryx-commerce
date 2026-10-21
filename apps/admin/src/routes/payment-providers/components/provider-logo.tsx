import React, { useState } from "react"

interface ProviderLogoProps {
  id: string
  size?: "small" | "medium" | "large"
}

export const formatProviderTitle = (id?: string, name?: string): string => {
  const norm = (id || "").toLowerCase()
  if (norm.includes("bkash")) return "bKash"
  if (norm.includes("nagad")) return "Nagad"
  if (norm.includes("sslcommerz")) return "SSLCommerz"
  if (norm.includes("stripe")) return "Stripe"
  if (norm.includes("system_default") || norm.includes("offline")) return "Cash on Delivery"
  return name || id || "Payment Gateway"
}

export const ProviderLogo: React.FC<ProviderLogoProps> = ({ id, size = "medium" }) => {
  const [hasError, setHasError] = useState(false)
  const norm = id.toLowerCase()
  
  const dim =
    size === "small"
      ? "w-9 h-9 rounded-lg p-1"
      : size === "large"
      ? "w-16 h-16 rounded-2xl p-2.5"
      : "w-12 h-12 rounded-xl p-1.5"

  // 1. bKash — Official Vector Logo
  if (norm.includes("bkash")) {
    if (!hasError) {
      return (
        <div
          className={`${dim} flex items-center justify-center bg-white border border-ui-border-base shadow-2xs shrink-0 overflow-hidden transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:scale-105 hover:shadow-xs`}
          title="bKash Payment Gateway"
        >
          <img
            src="/bkash.svg"
            alt="bKash"
            className="w-full h-full object-contain"
            onError={() => setHasError(true)}
          />
        </div>
      )
    }
    return (
      <div
        className={`${dim} flex items-center justify-center shadow-xs shrink-0 overflow-hidden transition-transform hover:scale-105`}
        style={{ background: "linear-gradient(135deg, #E2136E 0%, #C40E5E 100%)" }}
        title="bKash Payment Gateway"
      >
        <svg viewBox="0 0 100 100" className="w-[72%] h-[72%]" fill="none">
          <polygon points="18,34 38,14 34,42" fill="#FFFFFF" opacity="0.95" />
          <polygon points="34,42 38,14 82,18" fill="#FFFFFF" />
          <polygon points="34,42 82,18 64,52" fill="#FFFFFF" opacity="0.9" />
          <polygon points="34,42 64,52 46,74" fill="#FFFFFF" opacity="0.85" />
          <polygon points="46,74 64,52 86,64" fill="#FFFFFF" opacity="0.95" />
          <polygon points="46,74 86,64 68,88" fill="#FFFFFF" />
          <polygon points="46,74 68,88 38,82" fill="#FFFFFF" opacity="0.8" />
        </svg>
      </div>
    )
  }

  // 2. Nagad — Official Vector Logo
  if (norm.includes("nagad")) {
    if (!hasError) {
      return (
        <div
          className={`${dim} flex items-center justify-center bg-white border border-ui-border-base shadow-2xs shrink-0 overflow-hidden transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:scale-105 hover:shadow-xs`}
          title="Nagad Mobile Financial Service"
        >
          <img
            src="/nagad.svg"
            alt="Nagad"
            className="w-full h-full object-contain"
            onError={() => setHasError(true)}
          />
        </div>
      )
    }
    return (
      <div
        className={`${dim} flex items-center justify-center shadow-xs shrink-0 overflow-hidden transition-transform hover:scale-105`}
        style={{ background: "linear-gradient(135deg, #FF6F00 0%, #E52D27 100%)" }}
        title="Nagad Mobile Financial Service"
      >
        <svg viewBox="0 0 100 100" className="w-[70%] h-[70%]" fill="none">
          <path
            d="M50 12C38 24 24 38 24 56C24 72 36 84 50 88C38 78 38 60 48 48C54 40 64 36 68 26C72 40 82 52 74 68C70 76 62 82 50 88C68 84 80 72 80 54C80 34 64 22 50 12Z"
            fill="#FFFFFF"
          />
          <path
            d="M50 42C44 50 44 62 50 72C54 66 56 58 52 50C51 47 50 44 50 42Z"
            fill="#FFD200"
            opacity="0.9"
          />
        </svg>
      </div>
    )
  }

  // 3. SSLCommerz — Official Brand Logo
  if (norm.includes("sslcommerz")) {
    if (!hasError) {
      return (
        <div
          className={`${dim} flex items-center justify-center bg-white border border-ui-border-base shadow-2xs shrink-0 overflow-hidden transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:scale-105 hover:shadow-xs`}
          title="SSLCommerz Payment Gateway"
        >
          <img
            src="/sslcommerz.png"
            alt="SSLCommerz"
            className="w-full h-full object-contain"
            onError={() => setHasError(true)}
          />
        </div>
      )
    }
    return (
      <div
        className={`${dim} flex items-center justify-center shadow-xs shrink-0 overflow-hidden bg-[#0A2540] border border-[#1A3B60] transition-transform hover:scale-105 p-1`}
        title="SSLCommerz Payment Gateway"
      >
        <svg viewBox="0 0 120 120" className="w-[85%] h-[85%]" fill="none">
          <path d="M60 10L18 28V62C18 86 36 104 60 110C84 104 102 86 102 62V28L60 10Z" fill="#E31B23" />
          <rect x="36" y="52" width="48" height="36" rx="6" fill="#FFFFFF" />
          <path d="M45 52V40C45 31.7 51.7 25 60 25C68.3 25 75 31.7 75 40V52" stroke="#FFFFFF" strokeWidth="8" strokeLinecap="round" />
          <circle cx="60" cy="68" r="4.5" fill="#E31B23" />
          <path d="M60 72.5V80" stroke="#E31B23" strokeWidth="3" strokeLinecap="round" />
        </svg>
      </div>
    )
  }

  // 4. Stripe — Official Vector Logo
  if (norm.includes("stripe")) {
    if (!hasError) {
      return (
        <div
          className={`${dim} flex items-center justify-center bg-white border border-ui-border-base shadow-2xs shrink-0 overflow-hidden transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:scale-105 hover:shadow-xs`}
          title="Stripe Global Payment Gateway"
        >
          <img
            src="/stripe.svg"
            alt="Stripe"
            className="w-full h-full object-contain"
            onError={() => setHasError(true)}
          />
        </div>
      )
    }
    return (
      <div
        className={`${dim} flex items-center justify-center shadow-xs shrink-0 overflow-hidden transition-transform hover:scale-105`}
        style={{ background: "#635BFF" }}
        title="Stripe Global Payment Gateway"
      >
        <svg viewBox="0 0 32 32" className="w-[62%] h-[62%]" fill="none">
          <path
            fillRule="evenodd"
            clipRule="evenodd"
            d="M14.6 11.2C14.6 9.8 15.8 9.2 17.6 9.2C20.3 9.2 23.4 10.1 26 11.5V5.5C23.2 4.4 20.3 3.8 17.6 3.8C10.7 3.8 6 7.4 6 13.4C6 22.8 19 21.3 19 25.4C19 27 17.6 27.6 15.8 27.6C12.8 27.6 9.2 26.3 6.2 24.8V31C9.4 32.4 12.8 33 15.8 33C23 33 27.8 29.5 27.8 23.3C27.8 13.2 14.6 14.9 14.6 11.2Z"
            transform="scale(0.8) translate(4, 2)"
            fill="#FFFFFF"
          />
        </svg>
      </div>
    )
  }

  // 5. System Default / Cash on Delivery / Offline Payment
  return (
    <div
      className={`${dim} flex items-center justify-center shadow-xs shrink-0 overflow-hidden bg-gradient-to-br from-emerald-500 to-teal-700 transition-transform hover:scale-105`}
      title="Cash on Delivery & Parcel Collection"
    >
      <svg
        viewBox="0 0 24 24"
        className="w-[60%] h-[60%] text-white"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <rect x="2" y="6" width="20" height="12" rx="2" />
        <circle cx="12" cy="12" r="3" />
        <path d="M6 12h.01M18 12h.01" />
      </svg>
    </div>
  )
}
