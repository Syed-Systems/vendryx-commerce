"use client"

import React, { useState } from "react"
import { acceptTransferRequest, declineTransferRequest } from "@stheno/storefront-core"
import Spinner from "@modules/common/icons/spinner"

export type TransferStatus = "pending" | "success" | "error"

const TransferActions: React.FC<{ id: string; token: string }> = ({ id, token }) => {
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [status, setStatus] = useState<{
    accept: TransferStatus | null
    decline: TransferStatus | null
  } | null>({
    accept: null,
    decline: null,
  })

  const acceptTransfer = async () => {
    setStatus({ accept: "pending", decline: null })
    setErrorMessage(null)

    const { success, error } = await acceptTransferRequest(id, token)
    if (error) setErrorMessage(error)
    setStatus({ accept: success ? "success" : "error", decline: null })
  }

  const declineTransfer = async () => {
    setStatus({ accept: null, decline: "pending" })
    setErrorMessage(null)

    const { success, error } = await declineTransferRequest(id, token)
    if (error) setErrorMessage(error)
    setStatus({ accept: null, decline: success ? "success" : "error" })
  }

  return (
    <div className="space-y-4">
      {status?.accept === "success" && (
        <p className="rounded-[var(--radius-sm)] bg-emerald-500/10 p-3 text-xs font-semibold text-emerald-600 dark:text-emerald-400">
          Order custody transferred successfully.
        </p>
      )}

      {status?.decline === "success" && (
        <p className="rounded-[var(--radius-sm)] bg-[var(--hairline)] p-3 text-xs font-semibold text-[var(--text-secondary)]">
          Order transfer request has been declined.
        </p>
      )}

      {status?.accept !== "success" && status?.decline !== "success" && (
        <div className="flex flex-wrap gap-3">
          <button
            type="button"
            onClick={acceptTransfer}
            disabled={status?.accept === "pending" || status?.decline === "pending"}
            className="inline-flex h-11 items-center justify-center rounded-full bg-[var(--accent-sapphire)] px-6 text-xs font-semibold text-white shadow-sm transition hover:bg-[var(--accent-sapphire-hover)] disabled:opacity-50"
          >
            {status?.accept === "pending" ? (
              <span className="flex items-center gap-2">
                <Spinner className="h-4 w-4 animate-spin text-white" />
                <span>Accepting...</span>
              </span>
            ) : (
              "Accept Transfer"
            )}
          </button>

          <button
            type="button"
            onClick={declineTransfer}
            disabled={status?.accept === "pending" || status?.decline === "pending"}
            className="inline-flex h-11 items-center justify-center rounded-full border border-[var(--hairline)] bg-[var(--bg-surface)] px-6 text-xs font-semibold text-[var(--text-primary)] transition hover:border-[var(--text-secondary)] disabled:opacity-50"
          >
            {status?.decline === "pending" ? (
              <span className="flex items-center gap-2">
                <Spinner className="h-4 w-4 animate-spin text-[var(--text-secondary)]" />
                <span>Declining...</span>
              </span>
            ) : (
              "Decline Transfer"
            )}
          </button>
        </div>
      )}

      {errorMessage && (
        <p className="rounded-[var(--radius-sm)] bg-red-500/10 p-3 text-xs font-medium text-red-500">
          {errorMessage}
        </p>
      )}
    </div>
  )
}

export default TransferActions
