import React from "react"
import { WarningCircle } from "@phosphor-icons/react/dist/ssr"

export type ErrorMessageProps = {
  error?: string | null
  title?: string
  action?: {
    label: string
    onClick: () => void
  }
  "data-testid"?: string
}

/**
 * Lustre ErrorMessage.
 * Accessible alert banner with cool-red left edge border, optional title and recovery action.
 * Renders null when no error is present.
 */
const ErrorMessage: React.FC<ErrorMessageProps> = ({
  error,
  title,
  action,
  "data-testid": dataTestId,
}) => {
  if (!error) {
    return null
  }

  return (
    <div
      role="alert"
      data-testid={dataTestId}
      className="flex items-start gap-2.5 p-3.5 my-2 text-xs font-medium rounded-[12px] bg-[var(--status-error-bg)] text-[var(--status-error)] border border-[var(--status-error)]/20 border-l-4 border-l-[var(--status-error)] shadow-sm"
    >
      <WarningCircle size={16} className="mt-0.5 shrink-0" aria-hidden="true" />
      <div className="flex-1 min-w-0">
        {title && (
          <h4 className="font-semibold text-xs text-[var(--status-error)] mb-0.5">
            {title}
          </h4>
        )}
        <p className="leading-relaxed">{error}</p>
        {action && (
          <button
            type="button"
            onClick={action.onClick}
            className="mt-2 text-xs font-semibold underline underline-offset-2 hover:opacity-80 transition lustre-btn-press block"
          >
            {action.label}
          </button>
        )}
      </div>
    </div>
  )
}

export default ErrorMessage
