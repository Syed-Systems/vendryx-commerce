"use client"

import React, { useEffect, useActionState } from "react"
import { HttpTypes } from "@medusajs/types"
import { updateCustomer } from "@stheno/storefront-core"
import Input from "@modules/common/components/input"
import AccountInfo from "../account-info"

export type ProfilePhoneProps = {
  customer: HttpTypes.StoreCustomer
}

const ProfilePhone: React.FC<ProfilePhoneProps> = ({ customer }) => {
  const [successState, setSuccessState] = React.useState(false)

  const updateCustomerPhone = async (
    _currentState: Record<string, unknown>,
    formData: FormData
  ) => {
    const payload = {
      phone: formData.get("phone") as string,
    }

    try {
      await updateCustomer(payload)
      return { success: true, error: null }
    } catch (error: any) {
      return { success: false, error: error.toString() }
    }
  }

  const [state, formAction] = useActionState(updateCustomerPhone, {
    error: false,
    success: false,
  })

  const clearState = () => {
    setSuccessState(false)
  }

  useEffect(() => {
    setSuccessState(state.success)
  }, [state])

  return (
    <form action={formAction} className="w-full">
      <AccountInfo
        label="Telephone"
        currentInfo={customer.phone || "No phone linked"}
        isSuccess={successState}
        isError={!!state.error}
        errorMessage={state.error}
        clearState={clearState}
        data-testid="account-phone-editor"
      >
        <div className="space-y-3">
          <Input
            label="Phone Number"
            name="phone"
            type="tel"
            defaultValue={customer.phone ?? ""}
            required
            data-testid="phone-input"
          />
        </div>
      </AccountInfo>
    </form>
  )
}

export default ProfilePhone
