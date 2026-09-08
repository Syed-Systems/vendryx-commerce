import { ChevronRight } from "@medusajs/icons"
import { clx, Text } from "@medusajs/ui"
import React from "react"
import { Link } from "react-router-dom"

export interface SettingsRowProps {
  title: string
  description?: string
  icon?: React.ReactNode
  to?: string
  onClick?: () => void
  accessory?: React.ReactNode
  disabled?: boolean
  className?: string
  destructive?: boolean
}

export const SettingsRow = ({
  title,
  description,
  icon,
  to,
  onClick,
  accessory,
  disabled = false,
  className,
  destructive = false,
}: SettingsRowProps) => {
  const content = (
    <div
      className={clx(
        "flex min-h-[44px] w-full items-center justify-between px-4 py-3 transition-colors",
        !disabled && (to || onClick) && "hover:bg-ui-bg-subtle/50 active:bg-ui-bg-subtle",
        disabled && "opacity-50 pointer-events-none cursor-not-allowed",
        className
      )}
    >
      <div className="flex items-center gap-x-3.5 min-w-0 pr-2">
        {icon && (
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-ui-border-base bg-ui-bg-subtle text-ui-fg-subtle">
            {icon}
          </div>
        )}
        <div className="flex flex-col min-w-0">
          <Text
            size="small"
            weight="plus"
            className={clx(
              "truncate font-medium",
              destructive ? "text-ui-fg-error" : "text-ui-fg-base"
            )}
          >
            {title}
          </Text>
          {description && (
            <Text size="xsmall" className="text-ui-fg-subtle truncate mt-0.5">
              {description}
            </Text>
          )}
        </div>
      </div>

      <div className="flex items-center gap-x-2 shrink-0">
        {accessory}
        {to && !accessory && (
          <ChevronRight className="h-4 w-4 text-ui-fg-muted" />
        )}
      </div>
    </div>
  )

  if (to && !disabled) {
    return (
      <Link to={to} className="block outline-hidden focus-visible:ring-2 focus-visible:ring-ui-fg-interactive rounded-md">
        {content}
      </Link>
    )
  }

  if (onClick && !disabled) {
    return (
      <button
        type="button"
        onClick={onClick}
        className="w-full text-left outline-hidden focus-visible:ring-2 focus-visible:ring-ui-fg-interactive rounded-md"
      >
        {content}
      </button>
    )
  }

  return <div>{content}</div>
}
