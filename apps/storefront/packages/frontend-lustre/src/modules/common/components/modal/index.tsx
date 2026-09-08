"use client"

import { Dialog, Transition } from "@headlessui/react"
import React, { Fragment } from "react"
import { ModalProvider, useModal } from "@stheno/storefront-core"
import { X } from "@phosphor-icons/react/dist/ssr"

export type ModalProps = {
  isOpen: boolean
  close: () => void
  size?: "small" | "medium" | "large"
  search?: boolean
  children: React.ReactNode
  "data-testid"?: string
}

/**
 * Lustre Modal.
 * Accessible dialog popup with 16px surface radius and frosted backdrop.
 */
const Modal = ({
  isOpen,
  close,
  size = "medium",
  search = false,
  children,
  "data-testid": dataTestId,
}: ModalProps) => {
  return (
    <Transition appear show={isOpen} as={Fragment}>
      <Dialog as="div" className="relative z-50" onClose={close}>
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-200"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-150"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-black/40 backdrop-blur-sm" />
        </Transition.Child>

        <div className="fixed inset-0 overflow-y-auto">
          <div
            className={`flex min-h-full justify-center p-4 text-center ${
              search ? "items-start pt-16" : "items-center"
            }`}
          >
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-200"
              enterFrom="opacity-0 scale-98 -translate-y-1"
              enterTo="opacity-100 scale-100 translate-y-0"
              leave="ease-in duration-150"
              leaveFrom="opacity-100 scale-100 translate-y-0"
              leaveTo="opacity-0 scale-98 -translate-y-1"
            >
              <Dialog.Panel
                data-testid={dataTestId}
                className={`w-full text-left align-middle transform transition-all p-6 rounded-[16px] border ${
                  search
                    ? "bg-transparent border-transparent shadow-none"
                    : "bg-[var(--bg-surface)] border-[var(--hairline)] shadow-2xl"
                } ${
                  size === "small"
                    ? "max-w-md"
                    : size === "large"
                    ? "max-w-3xl"
                    : "max-w-xl"
                }`}
              >
                <ModalProvider close={close}>{children}</ModalProvider>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </Dialog>
    </Transition>
  )
}

const Title: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { close } = useModal()

  return (
    <div className="flex items-center justify-between pb-3 border-b border-[var(--hairline)]">
      <Dialog.Title className="text-base font-semibold text-[var(--text-primary)]">
        {children}
      </Dialog.Title>
      <button
        onClick={close}
        data-testid="close-modal-button"
        aria-label="Close dialog"
        className="p-1 rounded-full text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-raised)] transition-colors lustre-btn-press"
      >
        <X size={18} />
      </button>
    </div>
  )
}

const Description: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <Dialog.Description className="py-2 text-xs text-[var(--text-secondary)]">
    {children}
  </Dialog.Description>
)

const Body: React.FC<{ children: React.ReactNode; className?: string }> = ({
  children,
  className = "",
}) => <div className={`pt-4 ${className}`}>{children}</div>

const Footer: React.FC<{ children: React.ReactNode; className?: string }> = ({
  children,
  className = "",
}) => (
  <div className={`mt-6 pt-3 border-t border-[var(--hairline)] flex items-center justify-end gap-3 ${className}`}>
    {children}
  </div>
)

Modal.Title = Title
Modal.Description = Description
Modal.Body = Body
Modal.Footer = Footer

export default Modal
