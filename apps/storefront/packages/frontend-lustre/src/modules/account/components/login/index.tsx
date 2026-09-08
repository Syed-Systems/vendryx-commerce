"use client"

import React, { useActionState } from "react"
import { login } from "@stheno/storefront-core"
import { LOGIN_VIEW } from "@modules/account/templates/login-template"
import ErrorMessage from "@modules/common/components/error-message"
import { SubmitButton } from "@modules/common/components/submit-button"
import Input from "@modules/common/components/input"

export type LoginProps = {
  setCurrentView: (view: LOGIN_VIEW) => void
}

const Login: React.FC<LoginProps> = ({ setCurrentView }) => {
  const [message, formAction] = useActionState(login, null)

  return (
    <div
      className="mx-auto w-full max-w-sm rounded-[var(--radius-lg)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-8 shadow-sm backdrop-blur-md"
      data-testid="login-page"
    >
      <div className="mb-6 text-center">
        <span className="text-xs font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
          Account
        </span>
        <h1 className="mt-1 text-2xl font-semibold text-[var(--text-primary)]">
          Sign in
        </h1>
        <p className="mt-2 text-xs text-[var(--text-secondary)]">
          Access your orders and saved addresses.
        </p>
      </div>

      <form action={formAction} className="space-y-4">
        <Input
          label="Email Address"
          name="email"
          type="email"
          title="Enter a valid email address."
          autoComplete="email"
          required
          data-testid="email-input"
        />

        <Input
          label="Password"
          name="password"
          type="password"
          autoComplete="current-password"
          required
          data-testid="password-input"
        />

        <ErrorMessage error={message} data-testid="login-error-message" />

        <div className="pt-2">
          <SubmitButton
            data-testid="sign-in-button"
            className="h-11 w-full rounded-full bg-[var(--accent-sapphire)] text-xs font-semibold uppercase tracking-wider text-white shadow-sm hover:bg-[var(--accent-sapphire-hover)]"
          >
            Sign in
          </SubmitButton>
        </div>
      </form>

      <div className="mt-6 border-t border-[var(--hairline)] pt-4 text-center text-xs text-[var(--text-secondary)]">
        <span>Not yet registered? </span>
        <button
          type="button"
          onClick={() => setCurrentView(LOGIN_VIEW.REGISTER)}
          className="font-semibold text-[var(--accent-sapphire)] hover:underline"
          data-testid="register-button"
        >
          Create account
        </button>
      </div>
    </div>
  )
}

export default Login
