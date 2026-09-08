import { HttpTypes } from "@medusajs/types"
import React from "react"

export type OptionSelectProps = {
  option: HttpTypes.StoreProductOption
  current: string | undefined
  updateOption: (title: string, value: string) => void
  title: string
  disabled: boolean
  "data-testid"?: string
}

/**
 * Lustre OptionSelect.
 * Pill-shaped variant option selectors with high contrast and accessible focus.
 */
const OptionSelect: React.FC<OptionSelectProps> = ({
  option,
  current,
  updateOption,
  title,
  disabled,
  "data-testid": dataTestId,
}) => {
  const filteredOptions = (option.values ?? []).map((v) => v.value)

  return (
    <div className="flex flex-col space-y-2 text-left">
      <div className="flex items-baseline justify-between text-xs">
        <span className="font-semibold uppercase tracking-wider text-[var(--text-primary)]">
          {title}
        </span>
        {current && (
          <span className="text-[var(--text-secondary)] font-medium">
            {current}
          </span>
        )}
      </div>

      <div className="flex flex-wrap gap-2" data-testid={dataTestId}>
        {filteredOptions.map((value) => {
          const isSelected = value === current
          return (
            <button
              key={value}
              type="button"
              disabled={disabled}
              onClick={() => updateOption(option.id, value)}
              data-testid="option-button"
              className={`min-h-[44px] min-w-[44px] px-4 py-2 rounded-full text-xs font-medium transition-all duration-150 border disabled:opacity-40 disabled:cursor-not-allowed ${
                isSelected
                  ? "bg-[var(--accent-sapphire)] text-white border-[var(--accent-sapphire)] shadow-sm font-semibold"
                  : "bg-[var(--bg-surface)] text-[var(--text-primary)] border-[var(--hairline)] hover:border-[var(--text-secondary)]"
              }`}
            >
              {value}
            </button>
          )
        })}
      </div>
    </div>
  )
}

export default OptionSelect
