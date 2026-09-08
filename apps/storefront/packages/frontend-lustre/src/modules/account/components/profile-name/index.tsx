"use client"

import React, { useEffect, useActionState } from "react"
import { HttpTypes } from "@medusajs/types"
import { updateCustomer } from "@stheno/storefront-core"
import Input from "@modules/common/components/input"
import AccountInfo from "../account-info"

export type ProfileNameProps = {
  customer: HttpTypes.StoreCustomer
}

const ProfileName: React.FC<ProfileNameProps> = ({ customer }) => {
  const [successState, setSuccessState] = React.useState(false)

  const updateCustomerName = async (
    _currentState: Record<string, unknown>,
    formData: FormData
  ) => {
    const payload = {
      first_name: formData.get("first_name") as string,
      last_name: formData.get("last_name") as string,
    }

    try {
      await updateCustomer(payload)
      return { success: true, error: null }
    } catch (error: any) {
      return { success: false, error: error.toString() }
    }
  }

  const [state, formAction] = useActionState(updateCustomerName, {
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
        label="Full Name"
        currentInfo={`${customer.first_name || ""} ${customer.last_name || ""}`.trim() || "Not configured"}
        isSuccess={successState}
        isError={!!state?.error}
        errorMessage={state?.error || undefined}
        clearState={clearState}
        data-testid="account-name-editor"
      >
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <Input
            label="First Name"
            name="first_name"
            required
            defaultValue={customer.first_name ?? ""}
            data-testid="first-name-input"
          />
          <Input
            label="Last Name"
            name="last_name"
            required
            defaultValue={customer.last_name ?? ""}
            data-testid="last-name-input"
          />
        </div>
      </AccountInfo>
    </form>
  )
}

export default ProfileName
