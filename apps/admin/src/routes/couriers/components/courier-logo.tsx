import React, { useState } from "react"

interface CourierLogoProps {
  id: string
  size?: "small" | "medium" | "large"
}

export const CourierLogo: React.FC<CourierLogoProps> = ({ id, size = "medium" }) => {
  const [hasError, setHasError] = useState(false)
  const dim =
    size === "small"
      ? "w-9 h-9 rounded-lg p-1 text-xs"
      : size === "large"
      ? "w-14 h-14 rounded-2xl p-2 text-base"
      : "w-11 h-11 rounded-xl p-1.5 text-sm"

  const normalized = id.toLowerCase()

  // 1. Pathao Courier — Official Logo
  if (normalized.includes("pathao")) {
    return (
      <div
        className={`${dim} flex items-center justify-center rounded-xl bg-white border border-ui-border-base shadow-2xs shrink-0 overflow-hidden transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:scale-105 hover:shadow-xs`}
        title="Pathao Courier Logistics"
      >
        {!hasError ? (
          <img
            src="/pathao.svg"
            alt="Pathao"
            className="w-full h-full object-contain"
            onError={() => setHasError(true)}
          />
        ) : (
          <div
            className="w-full h-full flex items-center justify-center rounded-lg font-bold text-white shadow-xs"
            style={{ background: "linear-gradient(135deg, #E82933 0%, #C0141D 100%)" }}
          >
            <span className="tracking-tighter font-black">P</span>
          </div>
        )}
        <span className="sr-only">P</span>
      </div>
    )
  }

  // 2. Steadfast Courier — Official Logo
  if (normalized.includes("steadfast")) {
    return (
      <div
        className={`${dim} flex items-center justify-center rounded-xl bg-white border border-ui-border-base shadow-2xs shrink-0 overflow-hidden transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:scale-105 hover:shadow-xs`}
        title="Steadfast Courier Services"
      >
        {!hasError ? (
          <img
            src="/steadfast.svg"
            alt="Steadfast"
            className="w-full h-full object-contain"
            onError={() => setHasError(true)}
          />
        ) : (
          <div
            className="w-full h-full flex items-center justify-center rounded-lg font-bold text-white shadow-xs"
            style={{ background: "linear-gradient(135deg, #00A65A 0%, #008749 100%)" }}
          >
            <span className="font-extrabold tracking-tight text-xs">SF</span>
          </div>
        )}
        <span className="sr-only">SF</span>
      </div>
    )
  }

  // 3. RedX Logistics — Official Logo
  if (normalized.includes("redx")) {
    return (
      <div
        className={`${dim} flex items-center justify-center rounded-xl bg-white border border-ui-border-base shadow-2xs shrink-0 overflow-hidden transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:scale-105 hover:shadow-xs`}
        title="RedX Parcel Delivery"
      >
        {!hasError ? (
          <img
            src="/redx.svg"
            alt="RedX"
            className="w-full h-full object-contain"
            onError={() => setHasError(true)}
          />
        ) : (
          <div
            className="w-full h-full flex items-center justify-center rounded-lg font-bold text-white shadow-xs"
            style={{ background: "linear-gradient(135deg, #E61C24 0%, #9E1016 100%)" }}
          >
            <span className="font-black tracking-tight text-xs">RX</span>
          </div>
        )}
        <span className="sr-only">RX</span>
      </div>
    )
  }

  // Generic courier / In-House / Manual fulfillment
  return (
    <div
      className={`${dim} flex items-center justify-center rounded-xl font-bold text-ui-fg-base bg-ui-bg-subtle border border-ui-border-base shadow-xs transition-transform`}
      title="In-House / Custom Dispatch"
    >
      <span className="font-bold">🚚</span>
    </div>
  )
}
