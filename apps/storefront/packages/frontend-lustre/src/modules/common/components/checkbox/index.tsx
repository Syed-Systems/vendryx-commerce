import React, { useId } from "react"
import { Check } from "@phosphor-icons/react/dist/ssr"

export type CheckboxProps = {
  checked?: boolean
  onChange?: () => void
  label: string
  name?: string
  "data-testid"?: string
}

/**
 * Lustre Checkbox with label.
 * Pure accessible HTML checkbox with custom cold-luxury sapphire check state.
 * Generates unique IDs per instance to prevent DOM collisions.
 */
const CheckboxWithLabel: React.FC<CheckboxProps> = ({
  checked = false,
  onChange,
  label,
  name,
  "data-testid": dataTestId,
}) => {
  const id = useId()

  return (
    <label
      htmlFor={id}
      className="inline-flex items-center gap-2.5 cursor-pointer select-none group"
    >
      <div className="relative flex items-center justify-center">
        <input
          id={id}
          type="checkbox"
          name={name}
          checked={checked}
          onChange={onChange}
          data-testid={dataTestId}
          className="peer sr-only"
        />
        <div className="w-4 h-4 rounded-[4px] border border-[var(--hairline)] bg-[var(--bg-surface)] transition-all duration-150 ease-out peer-checked:bg-[var(--accent-sapphire)] peer-checked:border-[var(--accent-sapphire)] peer-focus-visible:ring-2 peer-focus-visible:ring-[var(--accent-sapphire)] peer-focus-visible:ring-offset-2 group-hover:border-[var(--text-secondary)] flex items-center justify-center">
          <Check
            size={10}
            weight="bold"
            className={`text-white transition-opacity duration-150 ${
              checked ? "opacity-100" : "opacity-0"
            }`}
            aria-hidden="true"
          />
        </div>
      </div>
      <span className="text-sm font-normal text-[var(--text-primary)]">
        {label}
      </span>
    </label>
  )
}

export default CheckboxWithLabel
