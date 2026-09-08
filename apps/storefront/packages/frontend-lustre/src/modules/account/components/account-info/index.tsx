"use client"

import React, { useEffect, useState } from "react"
import { useFormStatus } from "react-dom"
import { SubmitButton } from "@modules/common/components/submit-button"

export type AccountInfoProps = {
  label: string
  currentInfo: string | React.ReactNode
  isSuccess?: boolean
  isError?: boolean
  errorMessage?: string
  clearState: () => void
  children?: React.ReactNode
  "data-testid"?: string
}

const AccountInfo: React.FC<AccountInfoProps> = ({
  label,
  currentInfo,
  isSuccess,
  isError,
  clearState,
  errorMessage = "An error occurred, please try again",
  children,
  "data-testid": dataTestid,
}) => {
  const [isOpen, setIsOpen] = useState(false)
  const { pending } = useFormStatus()

  const handleToggle = () => {
    clearState()
    setIsOpen((prev) => !prev)
  }

  useEffect(() => {
    if (isSuccess) {
      setIsOpen(false)
    }
  }, [isSuccess])

  return (
    <div
      className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-5 text-xs transition"
      data-testid={dataTestid}
    >
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
            {label}
          </span>
          <div className="font-medium text-[var(--text-primary)]" data-testid="current-info">
            {currentInfo}
          </div>
        </div>

        <div>
          <button
            type="button"
            onClick={handleToggle}
            className="rounded-full border border-[var(--hairline)] bg-[var(--bg-primary)] px-4 py-1.5 font-semibold text-[var(--text-primary)] hover:border-[var(--accent-sapphire)] transition"
            data-testid="edit-button"
          >
            {isOpen ? "Cancel" : "Modify"}
          </button>
        </div>
      </div>

      {/* Expandable Form Body */}
      {isOpen && (
        <div className="mt-4 border-t border-[var(--hairline)] pt-4 space-y-4">
          <div>{children}</div>

          {isError && (
            <p className="text-red-500 font-medium" data-testid="error-message">
              {errorMessage}
            </p>
          )}

          <div className="flex justify-end gap-2">
            <SubmitButton
              className="rounded-full bg-[var(--accent-sapphire)] px-5 py-2 text-xs font-semibold text-white shadow-sm hover:bg-[var(--accent-sapphire-hover)]"
              data-testid="save-button"
            >
              Save Changes
            </SubmitButton>
          </div>
        </div>
      )}
    </div>
  )
}

export default AccountInfo
