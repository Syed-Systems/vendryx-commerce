"use client"

import React, { useEffect, useState, useActionState } from "react"
import { HttpTypes } from "@medusajs/types"
import {
  useToggleState,
  deleteCustomerAddress,
  updateCustomerAddress,
} from "@stheno/storefront-core"
import CountrySelect from "@modules/common/components/country-select"
import Input from "@modules/common/components/input"
import Modal from "@modules/common/components/modal"
import Spinner from "@modules/common/icons/spinner"
import { Trash } from "@phosphor-icons/react/dist/ssr"
import { SubmitButton } from "@modules/common/components/submit-button"

export type EditAddressProps = {
  region: HttpTypes.StoreRegion
  address: HttpTypes.StoreCustomerAddress
  isActive?: boolean
}

const EditAddress: React.FC<EditAddressProps> = ({
  region,
  address,
  isActive = false,
}) => {
  const [removing, setRemoving] = useState(false)
  const [successState, setSuccessState] = useState(false)
  const { state, open, close: closeModal } = useToggleState(false)

  const [formState, formAction] = useActionState(updateCustomerAddress, {
    success: false,
    error: null,
    addressId: address.id,
  })

  const close = () => {
    setSuccessState(false)
    closeModal()
  }

  useEffect(() => {
    if (successState) {
      close()
    }
  }, [successState])

  useEffect(() => {
    if (formState.success) {
      setSuccessState(true)
    }
  }, [formState])

  const removeAddress = async () => {
    setRemoving(true)
    await deleteCustomerAddress(address.id)
    setRemoving(false)
  }

  return (
    <>
      <div
        className="flex min-h-[200px] flex-col justify-between rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 text-xs transition"
        data-testid="address-container"
      >
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-sm font-semibold text-[var(--text-primary)]">
              {address.first_name} {address.last_name}
            </span>
            {isActive && (
              <span className="rounded-full bg-[var(--accent-sapphire)]/10 px-2.5 py-0.5 text-[10px] font-semibold text-[var(--accent-sapphire)]">
                Default
              </span>
            )}
          </div>

          <div className="text-[var(--text-secondary)] space-y-0.5">
            {address.company && <p className="font-medium text-[var(--text-primary)]">{address.company}</p>}
            <p>{address.address_1}{address.address_2 ? `, ${address.address_2}` : ""}</p>
            <p>{address.city}, {address.postal_code}</p>
            <p className="uppercase">{address.country_code}</p>
            {address.phone && <p className="pt-1 text-[11px]">{address.phone}</p>}
          </div>
        </div>

        <div className="mt-4 flex items-center justify-between border-t border-[var(--hairline)] pt-3">
          <button
            type="button"
            onClick={open}
            className="font-semibold text-[var(--accent-sapphire)] hover:underline lustre-btn-press"
            data-testid="address-edit-button"
          >
            Edit Address
          </button>

          <button
            type="button"
            onClick={removeAddress}
            disabled={removing}
            className="flex items-center gap-1.5 text-[var(--text-secondary)] hover:text-rose-500 transition lustre-btn-press"
            data-testid="address-delete-button"
          >
            {removing ? (
              <Spinner className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Trash size={14} />
            )}
            <span>Remove</span>
          </button>
        </div>
      </div>

      <Modal isOpen={state} close={close} data-testid="edit-address-modal">
        <Modal.Title>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">
            Edit Address
          </h2>
        </Modal.Title>
        <form action={formAction}>
          <Modal.Body>
            <div className="space-y-3.5 text-xs">
              <div className="grid grid-cols-2 gap-3">
                <Input
                  label="First name"
                  name="first_name"
                  required
                  defaultValue={address.first_name || ""}
                  autoComplete="given-name"
                  data-testid="first-name-input"
                />
                <Input
                  label="Last name"
                  name="last_name"
                  required
                  defaultValue={address.last_name || ""}
                  autoComplete="family-name"
                  data-testid="last-name-input"
                />
              </div>

              <Input
                label="Company (optional)"
                name="company"
                defaultValue={address.company || ""}
                autoComplete="organization"
                data-testid="company-input"
              />

              <Input
                label="Address line 1"
                name="address_1"
                required
                defaultValue={address.address_1 || ""}
                autoComplete="address-line1"
                data-testid="address-1-input"
              />

              <Input
                label="Apartment, suite, etc. (optional)"
                name="address_2"
                defaultValue={address.address_2 || ""}
                autoComplete="address-line2"
                data-testid="address-2-input"
              />

              <div className="grid grid-cols-2 gap-3">
                <Input
                  label="City"
                  name="city"
                  required
                  defaultValue={address.city || ""}
                  autoComplete="address-level2"
                  data-testid="city-input"
                />
                <Input
                  label="Postal code"
                  name="postal_code"
                  required
                  defaultValue={address.postal_code || ""}
                  autoComplete="postal-code"
                  data-testid="postal-code-input"
                />
              </div>

              <Input
                label="Province / State (optional)"
                name="province"
                defaultValue={address.province || ""}
                autoComplete="address-level1"
                data-testid="state-input"
              />

              <CountrySelect
                region={region}
                name="country_code"
                required
                defaultValue={address.country_code || undefined}
                autoComplete="country"
                data-testid="country-select"
              />

              <Input
                label="Phone number"
                name="phone"
                type="tel"
                defaultValue={address.phone || ""}
                autoComplete="phone"
                data-testid="phone-input"
              />

              {formState.error && (
                <p className="text-red-500 font-medium" data-testid="address-error">
                  {formState.error}
                </p>
              )}
            </div>
          </Modal.Body>
          <Modal.Footer>
            <div className="flex justify-end gap-3 pt-4">
              <button
                type="button"
                onClick={close}
                className="rounded-full border border-[var(--hairline)] bg-[var(--bg-surface)] px-5 py-2 text-xs font-semibold text-[var(--text-primary)] hover:border-[var(--text-secondary)]"
              >
                Cancel
              </button>
              <SubmitButton
                data-testid="save-button"
                className="rounded-full bg-[var(--accent-sapphire)] px-6 py-2 text-xs font-semibold text-white shadow-sm hover:bg-[var(--accent-sapphire-hover)]"
              >
                Update Destination
              </SubmitButton>
            </div>
          </Modal.Footer>
        </form>
      </Modal>
    </>
  )
}

export default EditAddress
