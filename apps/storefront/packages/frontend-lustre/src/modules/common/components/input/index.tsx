import React, { useEffect, useId, useImperativeHandle, useState } from "react"
import { Eye, EyeSlash } from "@phosphor-icons/react/dist/ssr"

export type InputProps = Omit<
  Omit<React.InputHTMLAttributes<HTMLInputElement>, "size">,
  "placeholder"
> & {
  label: string
  errors?: Record<string, unknown>
  touched?: Record<string, unknown>
  name: string
  topLabel?: string
}

/**
 * Lustre Input primitive.
 * Clean, accessible input field with 12px radius, hairline borders, and sapphire focus ring.
 * Conforms to WCAG 2.2 AA and rejects @medusajs/ui dependency.
 */
const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ type, name, label, errors, touched, required, topLabel, className, ...props }, ref) => {
    const inputRef = React.useRef<HTMLInputElement>(null)
    const generatedId = useId()
    const inputId = props.id || generatedId
    const [showPassword, setShowPassword] = useState(false)
    const [inputType, setInputType] = useState(type)

    useEffect(() => {
      if (type === "password") {
        setInputType(showPassword ? "text" : "password")
      }
    }, [type, showPassword])

    useImperativeHandle(ref, () => inputRef.current!)

    const hasError = Boolean(errors && name && errors[name] && touched && touched[name])
    const errorMessage = hasError ? (errors![name] as string) : undefined

    return (
      <div className="flex flex-col w-full text-left">
        {topLabel && (
          <label
            htmlFor={inputId}
            className="mb-1.5 text-xs font-medium tracking-wide text-[var(--text-secondary)]"
          >
            {topLabel}
            {required && <span className="text-[var(--status-error)] ml-1" aria-hidden="true">*</span>}
          </label>
        )}

        <div className="relative flex items-center w-full">
          <input
            id={inputId}
            name={name}
            type={inputType}
            ref={inputRef}
            required={required}
            aria-invalid={hasError ? "true" : undefined}
            aria-describedby={hasError ? `${inputId}-error` : undefined}
            placeholder={label}
            className={`w-full h-11 px-3.5 text-sm bg-[var(--bg-surface)] text-[var(--text-primary)] border rounded-[12px] transition-colors duration-150 ease-out focus:outline-none focus:ring-2 focus:ring-[var(--accent-sapphire)] focus:border-transparent placeholder:text-[var(--text-muted)] ${
              hasError
                ? "border-[var(--status-error)] ring-1 ring-[var(--status-error)]"
                : "border-[var(--hairline)] hover:border-[var(--text-secondary)]"
            } ${type === "password" ? "pr-11" : ""} ${className || ""}`}
            {...props}
          />

          {type === "password" && (
            <button
              type="button"
              onClick={() => setShowPassword((prev) => !prev)}
              aria-label={showPassword ? "Hide password" : "Show password"}
              className="absolute right-3 p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] focus:outline-none transition-colors lustre-btn-press"
            >
              {showPassword ? <Eye size={16} /> : <EyeSlash size={16} />}
            </button>
          )}
        </div>

        {hasError && errorMessage && (
          <span
            id={`${inputId}-error`}
            role="alert"
            className="mt-1.5 text-xs text-[var(--status-error)] font-medium"
          >
            {errorMessage}
          </span>
        )}
      </div>
    )
  }
)

Input.displayName = "Input"

export default Input
