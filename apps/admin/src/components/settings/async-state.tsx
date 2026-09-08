import { ArrowPath, ExclamationCircle } from "@medusajs/icons"
import { Button, Text, clx } from "@medusajs/ui"
import React from "react"

export interface AsyncStateProps {
  isLoading?: boolean
  error?: Error | null
  onRetry?: () => void
  loadingMessage?: string
  children: React.ReactNode
  className?: string
}

export const AsyncState = ({
  isLoading = false,
  error = null,
  onRetry,
  loadingMessage = "Loading...",
  children,
  className,
}: AsyncStateProps) => {
  if (isLoading) {
    return (
      <div className={clx("flex flex-col items-center justify-center p-8 gap-y-3 text-center", className)}>
        <ArrowPath className="h-5 w-5 animate-spin text-ui-fg-muted" />
        <Text size="small" className="text-ui-fg-subtle">
          {loadingMessage}
        </Text>
      </div>
    )
  }

  if (error) {
    return (
      <div className={clx("flex flex-col items-center justify-center p-8 gap-y-3 text-center border rounded-xl bg-ui-bg-subtle", className)}>
        <ExclamationCircle className="h-6 w-6 text-ui-fg-error" />
        <Text size="small" weight="plus" className="text-ui-fg-base">
          Failed to load data
        </Text>
        <Text size="xsmall" className="text-ui-fg-subtle max-w-sm">
          {error.message || "An unexpected error occurred while communicating with the server."}
        </Text>
        {onRetry && (
          <Button size="small" variant="secondary" onClick={onRetry} className="mt-1">
            <ArrowPath className="me-1.5 h-3.5 w-3.5" />
            Retry
          </Button>
        )}
      </div>
    )
  }

  return <>{children}</>
}
