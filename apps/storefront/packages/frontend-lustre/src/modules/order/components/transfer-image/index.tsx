import React, { SVGProps } from "react"

const TransferImage: React.FC<SVGProps<SVGSVGElement>> = (props) => (
  <svg
    width="280"
    height="180"
    viewBox="0 0 280 180"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    {...props}
  >
    {/* Base plate with frosted luxury styling */}
    <rect
      x="30"
      y="30"
      width="220"
      height="120"
      rx="16"
      className="fill-[var(--bg-surface)] stroke-[var(--hairline)]"
      strokeWidth="1.5"
    />

    {/* Primary card outline */}
    <rect
      x="50"
      y="50"
      width="180"
      height="80"
      rx="10"
      className="stroke-[var(--accent-sapphire)]/40"
      strokeWidth="1.5"
      strokeDasharray="4 4"
    />

    {/* Center transfer glyph */}
    <circle
      cx="140"
      cy="90"
      r="24"
      className="fill-[var(--accent-sapphire)]/10 stroke-[var(--accent-sapphire)]"
      strokeWidth="1.5"
    />

    {/* Transfer arrows */}
    <path
      d="M132 86H148M148 86L144 82M148 86L144 90"
      className="stroke-[var(--accent-sapphire)]"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
    <path
      d="M148 94H132M132 94L136 90M132 94L136 98"
      className="stroke-[var(--accent-sapphire)]"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
)

export default TransferImage
