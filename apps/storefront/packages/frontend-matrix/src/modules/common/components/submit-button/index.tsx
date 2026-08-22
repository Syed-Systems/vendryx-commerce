"use client"

import { clx } from "@medusajs/ui"
import React from "react"
import { useFormStatus } from "react-dom"
import styles from "./checkout-button.module.css"

/**
 * PORT of The Matrix Hours `CheckoutButton` — its stylesheet, its full-width
 * gold block, its disabled treatment — generalised to every submit in the store.
 *
 * The source button navigated to `/checkout` on click. This one submits the
 * form it sits in and takes its pending state from `useFormStatus`, because the
 * action belongs to core (ADR-013 decision 4). Same surface, core's behaviour.
 */
export function SubmitButton({
  children,
  variant = "primary",
  className,
  "data-testid": dataTestId,
}: {
  children: React.ReactNode
  variant?: "primary" | "secondary" | "transparent" | "danger" | null
  className?: string
  "data-testid"?: string
}) {
  const { pending } = useFormStatus()

  return (
    <button
      type="submit"
      disabled={pending}
      aria-busy={pending}
      className={clx(styles.checkoutBtn, className)}
      data-variant={variant ?? "primary"}
      data-testid={dataTestId}
    >
      {children}
    </button>
  )
}
