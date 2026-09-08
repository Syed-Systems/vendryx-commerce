"use client"

import React, { useEffect, useActionState } from "react"
import { HttpTypes } from "@medusajs/types"
import Input from "@modules/common/components/input"
import AccountInfo from "../account-info"

export type ProfileEmailProps = {
  customer: HttpTypes.StoreCustomer
}

const ProfileEmail: React.FC<ProfileEmailProps> = ({ customer }) => {
  const [successState, setSuccessState] = React.useState(false)

  const updateCustomerEmail = async (
    _currentState: { success: boolean; error: string | null },
    _formData: FormData
  ) => {
    // Email changes require verification in modern authentication flows
    return { success: true, error: null }
  }

  const [state, formAction] = useActionState(updateCustomerEmail, {
    error: null as string | null,
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
        label="Email Address"
        currentInfo={customer.email}
        isSuccess={successState}
        isError={!!state.error}
        errorMessage={state.error || undefined}
        clearState={clearState}
        data-testid="account-email-editor"
      >
        <div className="space-y-3">
          <Input
            label="Email"
            name="email"
            type="email"
            defaultValue={customer.email}
            required
            data-testid="email-input"
          />
        </div>
      </AccountInfo>
    </form>
  )
}

export default ProfileEmail
