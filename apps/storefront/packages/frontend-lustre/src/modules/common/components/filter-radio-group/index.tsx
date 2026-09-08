import React from "react"

export type FilterRadioGroupProps = {
  title: string
  items: {
    value: string
    label: string
  }[]
  value: any
  handleChange: (...args: any[]) => void
  "data-testid"?: string
}

/**
 * Lustre FilterRadioGroup.
 * Pure accessible radio list for catalog refinement.
 */
const FilterRadioGroup: React.FC<FilterRadioGroupProps> = ({
  title,
  items,
  value,
  handleChange,
  "data-testid": dataTestId,
}) => {
  return (
    <div className="flex flex-col gap-2.5 text-left">
      <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">
        {title}
      </h3>
      <div role="radiogroup" data-testid={dataTestId} className="flex flex-col gap-2">
        {items?.map((item) => {
          const isSelected = item.value === value
          return (
            <label
              key={item.value}
              className="flex items-center justify-between py-1 cursor-pointer select-none group"
            >
              <span
                data-testid="radio-label"
                data-active={isSelected}
                className={`text-sm transition-colors ${
                  isSelected
                    ? "font-medium text-[var(--accent-sapphire)]"
                    : "text-[var(--text-secondary)] group-hover:text-[var(--text-primary)]"
                }`}
              >
                {item.label}
              </span>
              <input
                type="radio"
                name={title}
                value={item.value}
                checked={isSelected}
                onChange={() => handleChange(item.value)}
                className="sr-only"
              />
              {isSelected && (
                <span className="w-1.5 h-1.5 rounded-full bg-[var(--accent-sapphire)]" aria-hidden="true" />
              )}
            </label>
          )
        })}
      </div>
    </div>
  )
}

export default FilterRadioGroup
