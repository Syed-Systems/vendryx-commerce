"use client"

import React, { useState } from "react"
import { deleteLineItem } from "@stheno/storefront-core"
import Spinner from "@modules/common/icons/spinner"
import { Trash } from "@phosphor-icons/react/dist/ssr"

export type DeleteButtonProps = {
  id: string
  children?: React.ReactNode
  className?: string
}

/**
 * Lustre DeleteButton.
 * Async delete button with in-flight feedback and error handling.
 */
const DeleteButton: React.FC<DeleteButtonProps> = ({
  id,
  children,
  className = "",
}) => {
  const [isDeleting, setIsDeleting] = useState(false)

  const handleDelete = async () => {
    setIsDeleting(true)
    try {
      await deleteLineItem(id)
    } catch {
      setIsDeleting(false)
    }
  }

  return (
    <button
      type="button"
      disabled={isDeleting}
      onClick={handleDelete}
      className={`inline-flex items-center gap-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--status-error)] transition-colors disabled:opacity-50 lustre-btn-press ${className}`}
    >
      {isDeleting ? <Spinner size="14" /> : <Trash size={14} />}
      {children && <span>{children}</span>}
    </button>
  )
}

export default DeleteButton
