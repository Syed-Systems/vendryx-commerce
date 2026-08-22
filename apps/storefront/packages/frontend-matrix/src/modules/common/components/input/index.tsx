"use client"

import React, { useEffect, useId, useImperativeHandle, useState } from "react"
import styles from "../../styles/ui.module.css"

type InputProps = Omit<
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
 * Matrix field: mono label above the box, square 1px border, gold on focus.
 *
 * Replaces @medusajs/ui's Input and Label, and drops the floating-label trick
 * the Medusa template used — that pattern needs `placeholder=" "` plus a global
 * `input:not(:placeholder-shown)` rule reaching in from core's globals.css to
 * move the label. A theme that depends on core's stylesheet to position its own
 * label is not an independent theme. The label sits above the field and stays
 * there, which is also what MASTER.md's "rigid and mechanical" asks for.
 */
const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ type, name, label, touched, required, topLabel, ...props }, ref) => {
    const inputRef = React.useRef<HTMLInputElement>(null)
    const [showPassword, setShowPassword] = useState(false)
    const [inputType, setInputType] = useState(type)
    const id = useId()

    useEffect(() => {
      if (type === "password") {
        setInputType(showPassword ? "text" : "password")
      }
    }, [type, showPassword])

    useImperativeHandle(ref, () => inputRef.current!)

    return (
      <div className={styles.field}>
        {topLabel && <span className={styles.label}>{topLabel}</span>}

        <label htmlFor={id} className={styles.label}>
          {label}
          {required && (
            <span className={styles.required} aria-hidden>
              *
            </span>
          )}
        </label>

        <div className={styles.inputWrapper}>
          <input
            id={id}
            type={inputType}
            name={name}
            required={required}
            className={styles.input}
            {...props}
            ref={inputRef}
          />

          {type === "password" && (
            <button
              type="button"
              className={styles.revealBtn}
              onClick={() => setShowPassword(!showPassword)}
              aria-label={showPassword ? "Hide password" : "Show password"}
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.75"
                strokeLinecap="square"
                aria-hidden
              >
                <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z" />
                <circle cx="12" cy="12" r="3" />
                {!showPassword && <path d="m3 3 18 18" />}
              </svg>
            </button>
          )}
        </div>
      </div>
    )
  }
)

Input.displayName = "Input"

export default Input
