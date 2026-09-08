"use client"

import React, {
  SelectHTMLAttributes,
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react"
import { CaretDown } from "@phosphor-icons/react/dist/ssr"

export type CartItemSelectProps = {
  placeholder?: string
  errors?: Record<string, unknown>
  touched?: Record<string, unknown>
} & Omit<SelectHTMLAttributes<HTMLSelectElement>, "size">

const CartItemSelect = forwardRef<HTMLSelectElement, CartItemSelectProps>(
  ({ placeholder = "Select...", className = "", children, ...props }, ref) => {
    const innerRef = useRef<HTMLSelectElement>(null)
    const [isPlaceholder, setIsPlaceholder] = useState(false)

    useImperativeHandle<HTMLSelectElement | null, HTMLSelectElement | null>(
      ref,
      () => innerRef.current
    )

    useEffect(() => {
      if (innerRef.current && innerRef.current.value === "") {
        setIsPlaceholder(true)
      } else {
        setIsPlaceholder(false)
      }
    }, [innerRef.current?.value])

    return (
      <div className="relative inline-flex items-center">
        <select
          ref={innerRef}
          {...props}
          className={`h-9 appearance-none rounded-full border border-[var(--hairline)] bg-[var(--bg-surface)] pl-3 pr-8 text-xs font-semibold text-[var(--text-primary)] transition hover:border-[var(--accent-sapphire)] focus:border-[var(--accent-sapphire)] focus:outline-none focus:ring-1 focus:ring-[var(--accent-sapphire)] cursor-pointer ${
            isPlaceholder ? "text-[var(--text-secondary)]" : ""
          } ${className}`}
        >
          {placeholder && (
            <option disabled value="">
              {placeholder}
            </option>
          )}
          {children}
        </select>
        <span className="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-[var(--text-secondary)]">
          <CaretDown size={12} />
        </span>
      </div>
    )
  }
)

CartItemSelect.displayName = "CartItemSelect"

export default CartItemSelect
