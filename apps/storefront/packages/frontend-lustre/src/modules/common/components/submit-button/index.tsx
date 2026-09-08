"use client"

import React from "react"
import { useFormStatus } from "react-dom"
import Spinner from "@modules/common/icons/spinner"

export type SubmitButtonProps = {
  children: React.ReactNode
  variant?: "primary" | "secondary" | "transparent" | "danger" | null
  className?: string
  "data-testid"?: string
}

/**
 * Lustre SubmitButton.
 * Full-pill action button with immediate feedback and pending spinner.
 * Native button implementation without @medusajs/ui dependency.
 */
export function SubmitButton({
  children,
  variant = "primary",
  className = "",
  "data-testid": dataTestId,
}: SubmitButtonProps) {
  const { pending } = useFormStatus()

  const variantStyles = {
    primary:
      "bg-[var(--accent-sapphire)] text-white hover:bg-[var(--accent-sapphire-hover)] border-transparent shadow-sm",
    secondary:
      "bg-[var(--bg-surface)] text-[var(--text-primary)] border-[var(--hairline)] hover:border-[var(--text-secondary)]",
    transparent:
      "bg-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)] border-transparent",
    danger:
      "bg-[var(--status-error)] text-white hover:opacity-90 border-transparent",
  }[variant || "primary"]

  return (
    <button
      type="submit"
      disabled={pending}
      data-testid={dataTestId}
      className={`relative inline-flex items-center justify-center h-12 px-6 rounded-full text-sm font-medium tracking-wide transition-all duration-150 ease-out border disabled:opacity-50 disabled:cursor-not-allowed lustre-btn-press ${variantStyles} ${className}`}
    >
      {pending ? (
        <span className="inline-flex items-center gap-2">
          <Spinner size="16" />
          <span className="opacity-80">Processing...</span>
        </span>
      ) : (
        children
      )}
    </button>
  )
}

export default SubmitButton
