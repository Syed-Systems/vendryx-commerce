import React from "react"

export type DividerProps = {
  className?: string
}

/**
 * Lustre hairline divider.
 * Sparse, clean separator following 8px grid.
 */
const Divider: React.FC<DividerProps> = ({ className = "" }) => (
  <hr
    className={`w-full border-0 border-t border-[var(--hairline)] my-4 opacity-70 ${className}`}
    aria-hidden="true"
  />
)

export default Divider
