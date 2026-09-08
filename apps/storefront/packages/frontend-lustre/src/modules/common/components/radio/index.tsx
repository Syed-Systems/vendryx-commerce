import React from "react"

export type RadioProps = {
  checked: boolean
  "data-testid"?: string
}

/**
 * Lustre RadioIndicator component.
 * Minimal concentric circular indicator with sapphire selection dot.
 */
const Radio: React.FC<RadioProps> = ({ checked, "data-testid": dataTestId }) => {
  return (
    <span
      role="radio"
      aria-checked={checked}
      data-testid={dataTestId || "radio-button"}
      className="relative flex h-5 w-5 items-center justify-center shrink-0"
    >
      <span
        className={`flex h-4 w-4 items-center justify-center rounded-full border transition-colors duration-150 ${
          checked
            ? "border-[var(--accent-sapphire)] bg-[var(--bg-surface)]"
            : "border-[var(--hairline)] bg-[var(--bg-surface)]"
        }`}
      >
        {checked && (
          <span className="h-2 w-2 rounded-full bg-[var(--accent-sapphire)] transition-transform duration-150 transform scale-100" />
        )}
      </span>
    </span>
  )
}

export default Radio
