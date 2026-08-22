"use client"

import { Button, clx } from "@medusajs/ui"
import React from "react"
import { useFormStatus } from "react-dom"

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
    <Button
      size="large"
      // The site's primary action is a dark pill; the checkout was still using the
      // template's square button, so the one flow that matters looked like a different
      // product from the pages leading into it.
      className={clx(
        "rounded-full px-6 py-3 text-xs font-semibold uppercase tracking-widest shadow-md transition-all active:scale-[0.99]",
        className
      )}
      type="submit"
      isLoading={pending}
      variant={variant || "primary"}
      data-testid={dataTestId}
    >
      {children}
    </Button>
  )
}
