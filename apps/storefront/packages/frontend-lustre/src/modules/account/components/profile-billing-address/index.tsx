"use client"

import React, { useEffect, useMemo, useActionState } from "react"
import { HttpTypes } from "@medusajs/types"
import { addCustomerAddress, updateCustomerAddress } from "@stheno/storefront-core"
import Input from "@modules/common/components/input"
import NativeSelect from "@modules/common/components/native-select"
import AccountInfo from "../account-info"

export type ProfileBillingAddressProps = {
  customer: HttpTypes.StoreCustomer
  regions: HttpTypes.StoreRegion[]
}

const ProfileBillingAddress: React.FC<ProfileBillingAddressProps> = ({
  customer,
  regions,
}) => {
  const regionOptions = useMemo(() => {
    return (
      regions
        ?.map((region) =>
          region.countries?.map((country) => ({
            value: country.iso_2,
            label: country.display_name,
          }))
        )
        .flat() || []
    )
  }, [regions])

  const [successState, setSuccessState] = React.useState(false)

  const billingAddress = customer.addresses?.find(
    (addr) => addr.is_default_billing
  )

  const initialState: Record<string, any> = {
    isDefaultBilling: true,
    isDefaultShipping: false,
    error: false,
    success: false,
  }

  if (billingAddress) {
    initialState.addressId = billingAddress.id
  }

  const [state, formAction] = useActionState(
    billingAddress ? updateCustomerAddress : addCustomerAddress,
    initialState
  )

  const clearState = () => {
    setSuccessState(false)
  }

  useEffect(() => {
    setSuccessState(state.success)
  }, [state])

  const currentAddressSummary = billingAddress ? (
    <div className="text-left">
      <p>{billingAddress.first_name} {billingAddress.last_name}</p>
      <p>{billingAddress.address_1}{billingAddress.address_2 ? `, ${billingAddress.address_2}` : ""}</p>
      <p>{billingAddress.city}, {billingAddress.postal_code}</p>
      <p className="uppercase">{billingAddress.country_code}</p>
    </div>
  ) : (
    "No default billing address registered"
  )

  return (
    <form action={formAction} className="w-full">
      <AccountInfo
        label="Billing Address"
        currentInfo={currentAddressSummary}
        isSuccess={successState}
        isError={!!state.error}
        errorMessage={state.error}
        clearState={clearState}
        data-testid="account-billing-address-editor"
      >
        <div className="space-y-3">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <Input
              label="First Name"
              name="first_name"
              defaultValue={billingAddress?.first_name || ""}
              required
              data-testid="billing-first-name-input"
            />
            <Input
              label="Last Name"
              name="last_name"
              defaultValue={billingAddress?.last_name || ""}
              required
              data-testid="billing-last-name-input"
            />
          </div>

          <Input
            label="Street Address"
            name="address_1"
            defaultValue={billingAddress?.address_1 || ""}
            required
            data-testid="billing-address-1-input"
          />

          <Input
            label="Apartment, suite, etc. (optional)"
            name="address_2"
            defaultValue={billingAddress?.address_2 || ""}
            data-testid="billing-address-2-input"
          />

          <div className="grid grid-cols-2 gap-3">
            <Input
              label="City"
              name="city"
              defaultValue={billingAddress?.city || ""}
              required
              data-testid="billing-city-input"
            />
            <Input
              label="Postal Code"
              name="postal_code"
              defaultValue={billingAddress?.postal_code || ""}
              required
              data-testid="billing-postal-code-input"
            />
          </div>

          <NativeSelect
            name="country_code"
            defaultValue={billingAddress?.country_code || undefined}
            data-testid="billing-country-select"
          >
            <option value="">Select country...</option>
            {regionOptions.map((country: any) => (
              <option key={country.value} value={country.value}>
                {country.label}
              </option>
            ))}
          </NativeSelect>
        </div>
      </AccountInfo>
    </form>
  )
}

export default ProfileBillingAddress
