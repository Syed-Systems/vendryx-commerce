"use client"

import React, { useEffect, useState, useActionState } from "react"
import { HttpTypes } from "@medusajs/types"
import { useToggleState, addCustomerAddress } from "@stheno/storefront-core"
import CountrySelect from "@modules/common/components/country-select"
import Input from "@modules/common/components/input"
import Modal from "@modules/common/components/modal"
import { SubmitButton } from "@modules/common/components/submit-button"
import { Plus } from "@phosphor-icons/react/dist/ssr"

export type AddAddressProps = {
  region: HttpTypes.StoreRegion
  addresses: HttpTypes.StoreCustomerAddress[]
}

const AddAddress: React.FC<AddAddressProps> = ({ region, addresses }) => {
  const [successState, setSuccessState] = useState(false)
  const { state, open, close: closeModal } = useToggleState(false)

  const [formState, formAction] = useActionState(addCustomerAddress, {
    isDefaultShipping: addresses.length === 0,
    success: false,
    error: null,
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

  return (
    <>
      <button
        type="button"
        role="button"
        onClick={open}
        className="flex min-h-[220px] w-full flex-col items-center justify-center gap-2 rounded-[var(--radius-md)] border border-dashed border-[var(--hairline)] bg-[var(--bg-surface)] p-6 text-xs transition hover:border-[var(--accent-sapphire)]/60 lustre-btn-press"
        data-testid="add-address-button"
      >
        <span className="flex h-10 w-10 items-center justify-center rounded-full bg-[var(--accent-sapphire)]/10 text-[var(--accent-sapphire)]">
          <Plus size={20} aria-hidden="true" />
        </span>
        <span className="font-semibold text-[var(--text-primary)]">Register New Address</span>
        <span className="text-[var(--text-secondary)]">Add a delivery destination</span>
      </button>

      <Modal isOpen={state} close={close} data-testid="add-address-modal">
        <Modal.Title>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">
            Add Shipping Address
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
                  autoComplete="given-name"
                  data-testid="first-name-input"
                />
                <Input
                  label="Last name"
                  name="last_name"
                  required
                  autoComplete="family-name"
                  data-testid="last-name-input"
                />
              </div>

              <Input
                label="Company (optional)"
                name="company"
                autoComplete="organization"
                data-testid="company-input"
              />

              <Input
                label="Address line 1"
                name="address_1"
                required
                autoComplete="address-line1"
                data-testid="address-1-input"
              />

              <Input
                label="Apartment, suite, etc. (optional)"
                name="address_2"
                autoComplete="address-line2"
                data-testid="address-2-input"
              />

              <div className="grid grid-cols-2 gap-3">
                <Input
                  label="City"
                  name="city"
                  required
                  autoComplete="address-level2"
                  data-testid="city-input"
                />
                <Input
                  label="Postal code"
                  name="postal_code"
                  required
                  autoComplete="postal-code"
                  data-testid="postal-code-input"
                />
              </div>

              <Input
                label="Province / State (optional)"
                name="province"
                autoComplete="address-level1"
                data-testid="state-input"
              />

              <CountrySelect
                region={region}
                name="country_code"
                required
                autoComplete="country"
                data-testid="country-select"
              />

              <Input
                label="Phone number"
                name="phone"
                type="tel"
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
                Save Destination
              </SubmitButton>
            </div>
          </Modal.Footer>
        </form>
      </Modal>
    </>
  )
}

export default AddAddress
