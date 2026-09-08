import { Heading, Text, clx } from "@medusajs/ui"
import React from "react"

export interface SettingsSectionProps {
  title?: string
  description?: string
  footer?: string
  children: React.ReactNode
  className?: string
}

export const SettingsSection = ({
  title,
  description,
  footer,
  children,
  className,
}: SettingsSectionProps) => {
  return (
    <div className={clx("flex flex-col gap-y-2", className)}>
      {(title || description) && (
        <div className="px-1">
          {title && (
            <Heading level="h3" className="text-xs uppercase font-semibold tracking-wider text-ui-fg-muted">
              {title}
            </Heading>
          )}
          {description && (
            <Text size="xsmall" className="text-ui-fg-subtle mt-0.5">
              {description}
            </Text>
          )}
        </div>
      )}

      <div className="rounded-xl border border-ui-border-base bg-ui-bg-base overflow-hidden shadow-xs divide-y divide-ui-border-base">
        {children}
      </div>

      {footer && (
        <Text size="xsmall" className="text-ui-fg-muted px-2">
          {footer}
        </Text>
      )}
    </div>
  )
}
