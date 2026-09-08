"use client"

import React, { useActionState } from "react"
import { signup } from "@stheno/storefront-core"
import { LOGIN_VIEW } from "@modules/account/templates/login-template"
import ErrorMessage from "@modules/common/components/error-message"
import { SubmitButton } from "@modules/common/components/submit-button"
import Input from "@modules/common/components/input"

export type RegisterProps = {
  setCurrentView: (view: LOGIN_VIEW) => void
}

const Register: React.FC<RegisterProps> = ({ setCurrentView }) => {
  const [message, formAction] = useActionState(signup, null)

  return (
    <div
      className="mx-auto w-full max-w-sm rounded-[var(--radius-lg)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-8 shadow-sm backdrop-blur-md"
      data-testid="register-page"
    >
      <div className="mb-6 text-center">
        <span className="text-xs font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
          Client Registration
        </span>
        <h1 className="mt-1 text-2xl font-semibold text-[var(--text-primary)]">
          Create account
        </h1>
        <p className="mt-2 text-xs text-[var(--text-secondary)]">
          Save your details for checkout and order access.
        </p>
      </div>

      <form action={formAction} className="space-y-3.5">
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
          label="Email address"
          name="email"
          required
          type="email"
          autoComplete="email"
          data-testid="email-input"
        />

        <Input
          label="Phone number"
          name="phone"
          type="tel"
          autoComplete="tel"
          data-testid="phone-input"
        />

        <Input
          label="Password"
          name="password"
          required
          type="password"
          autoComplete="new-password"
          data-testid="password-input"
        />

        <ErrorMessage error={message} data-testid="register-error" />

        <div className="pt-2">
          <SubmitButton
            data-testid="register-button"
            className="h-11 w-full rounded-full bg-[var(--accent-sapphire)] text-xs font-semibold uppercase tracking-wider text-white shadow-sm hover:bg-[var(--accent-sapphire-hover)]"
          >
            Create Profile
          </SubmitButton>
        </div>
      </form>

      <div className="mt-6 border-t border-[var(--hairline)] pt-4 text-center text-xs text-[var(--text-secondary)]">
        <span>Already have an account? </span>
        <button
          type="button"
          onClick={() => setCurrentView(LOGIN_VIEW.SIGN_IN)}
          className="font-semibold text-[var(--accent-sapphire)] hover:underline"
        >
          Sign in
        </button>
      </div>
    </div>
  )
}

export default Register
