import React, {
  SelectHTMLAttributes,
  forwardRef,
  useImperativeHandle,
  useRef,
} from "react"
import { CaretDown } from "@phosphor-icons/react/dist/ssr"

export type NativeSelectProps = {
  placeholder?: string
  errors?: Record<string, unknown>
  touched?: Record<string, unknown>
} & SelectHTMLAttributes<HTMLSelectElement>

/**
 * Lustre NativeSelect primitive.
 * Accessible custom-styled select with 12px radius, hairline borders, and sapphire focus state.
 */
const NativeSelect = forwardRef<HTMLSelectElement, NativeSelectProps>(
  ({ placeholder = "Select...", defaultValue, className = "", children, ...props }, ref) => {
    const innerRef = useRef<HTMLSelectElement>(null)

    useImperativeHandle<HTMLSelectElement | null, HTMLSelectElement | null>(
      ref,
      () => innerRef.current
    )

    return (
      <div className="relative w-full">
        <select
          ref={innerRef}
          defaultValue={defaultValue}
          className={`w-full h-11 pl-3.5 pr-10 text-sm bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--hairline)] rounded-[12px] appearance-none transition-colors duration-150 ease-out hover:border-[var(--text-secondary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent-sapphire)] focus:border-transparent ${className}`}
          {...props}
        >
          {placeholder && (
            <option disabled value="">
              {placeholder}
            </option>
          )}
          {children}
        </select>
        <span className="absolute right-3.5 top-1/2 -translate-y-1/2 pointer-events-none text-[var(--text-muted)]">
          <CaretDown size={16} />
        </span>
      </div>
    )
  }
)

NativeSelect.displayName = "NativeSelect"

export default NativeSelect
