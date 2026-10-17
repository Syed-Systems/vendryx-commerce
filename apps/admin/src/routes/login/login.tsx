import { Spinner } from "@medusajs/icons"
import type { AuthTypes } from "@medusajs/types"
import { Heading, InlineTip, Text } from "@medusajs/ui"
import { useMemo, useState } from "react"
import { Trans, useTranslation } from "react-i18next"
import { Link, useLocation, useNavigate } from "react-router-dom"

import AvatarBox from "../../components/common/logo-box/avatar-box"
import { useAuthProviders } from "../../hooks/api"
import { useExtension } from "../../providers/extension-provider"
import { useClerkAuth } from "../../providers/clerk-provider"
import { ClerkLogin } from "./components/clerk-login"
import { EmailPassLogin } from "./components/email-pass-login"
import { MfaChallengeCard } from "./components/mfa-challenge-card"
import { SsoLogin } from "./components/sso-login"
import { hasEmailPassProvider } from "./utils"

export const Login = () => {
  const { t } = useTranslation()
  const location = useLocation()
  const navigate = useNavigate()
  const { getWidgets } = useExtension()
  const { isClerkEnabled } = useClerkAuth()
  const [authMode, setAuthMode] = useState<"clerk" | "email">(
    isClerkEnabled ? "clerk" : "email"
  )
  const [mfaChallenge, setMfaChallenge] =
    useState<AuthTypes.AuthMfaChallengeDTO | null>(null)
  const [mfaSuccessHandler, setMfaSuccessHandler] = useState<
    ((token: string) => void | Promise<void>) | null
  >(null)

  const from = location.state?.from?.pathname || "/orders"

  const { providers, isLoading: isProvidersLoading } = useAuthProviders()

  const providerList = useMemo(() => providers ?? [], [providers])
  const isEmailPassInstalled = useMemo(
    () => hasEmailPassProvider(providerList),
    [providerList]
  )

  const handleMfaChallenge = (
    challenge: AuthTypes.AuthMfaChallengeDTO,
    onSuccess: (token: string) => void | Promise<void>
  ) => {
    setMfaChallenge(challenge)
    setMfaSuccessHandler(() => onSuccess)
  }

  if (mfaChallenge) {
    return (
      <div className="bg-ui-bg-subtle flex min-h-dvh w-dvw items-center justify-center">
        <MfaChallengeCard
          challenge={mfaChallenge}
          onSuccess={(token) => {
            if (mfaSuccessHandler) {
              return mfaSuccessHandler(token)
            }

            navigate(from, { replace: true })
          }}
          onBack={() => {
            setMfaChallenge(null)
            setMfaSuccessHandler(null)
          }}
        />
      </div>
    )
  }

  return (
    <div className="bg-ui-bg-subtle flex min-h-dvh w-dvw items-center justify-center p-4">
      <div className="m-4 flex w-full max-w-[340px] flex-col items-center">
        <AvatarBox />
        <div className="mb-4 flex flex-col items-center text-center">
          <Heading>{t("login.title")}</Heading>
          <Text size="small" className="text-ui-fg-subtle text-center">
            {t("login.hint")}
          </Text>
        </div>

        {isClerkEnabled && (
          <div className="bg-ui-bg-base border-ui-border-base mb-4 flex w-full rounded-lg border p-1">
            <button
              type="button"
              onClick={() => setAuthMode("clerk")}
              className={`flex-1 rounded-md py-1.5 text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] ${
                authMode === "clerk"
                  ? "bg-ui-bg-base-pressed text-ui-fg-base shadow-sm"
                  : "text-ui-fg-subtle hover:text-ui-fg-base"
              }`}
            >
              Storefront Owner (Clerk)
            </button>
            <button
              type="button"
              onClick={() => setAuthMode("email")}
              className={`flex-1 rounded-md py-1.5 text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] ${
                authMode === "email"
                  ? "bg-ui-bg-base-pressed text-ui-fg-base shadow-sm"
                  : "text-ui-fg-subtle hover:text-ui-fg-base"
              }`}
            >
              Staff / Email
            </button>
          </div>
        )}

        {isClerkEnabled && authMode === "clerk" ? (
          <div className="flex w-full flex-col items-center gap-y-4">
            <ClerkLogin redirectTo={from} />
            <div className="text-center text-xs text-ui-fg-muted">
              New merchant?{" "}
              <Link
                to="/signup"
                className="text-ui-fg-interactive font-medium hover:underline"
              >
                Create Storefront Owner Account
              </Link>
            </div>
          </div>
        ) : (
          <div className="flex w-full flex-col gap-y-3">
            {getWidgets("login.before").map((Component, i) => {
              return <Component key={i} />
            })}
            {isProvidersLoading ? (
              <div className="flex w-full items-center justify-center py-6">
                <Spinner className="text-ui-fg-subtle animate-spin" />
              </div>
            ) : !providerList.length ? (
              <div>
                <InlineTip label={t("general.tip")}>
                  {t("auth.login.noProviders")}
                </InlineTip>
              </div>
            ) : (
              isEmailPassInstalled && (
                <EmailPassLogin onMfaChallenge={handleMfaChallenge} />
              )
            )}
            {getWidgets("login.after").map((Component, i) => {
              return <Component key={i} />
            })}
            <SsoLogin
              providers={providerList}
              onMfaChallenge={handleMfaChallenge}
            />

            {isEmailPassInstalled && (
              <span className="text-ui-fg-muted txt-small my-4 text-center">
                <Trans
                  i18nKey="login.forgotPassword"
                  components={[
                    <Link
                      key="reset-password-link"
                      to="/reset-password"
                      className="text-ui-fg-interactive transition-fg hover:text-ui-fg-interactive-hover focus-visible:text-ui-fg-interactive-hover font-medium outline-none"
                    />,
                  ]}
                />
              </span>
            )}

            {!isClerkEnabled && (
              <div className="border-ui-border-base mt-2 rounded-lg border border-dashed p-2.5 text-center text-xs text-ui-fg-muted">
                Storefront Owner Clerk auth available. Set{" "}
                <code className="text-ui-fg-base font-mono">
                  VITE_CLERK_PUBLISHABLE_KEY
                </code>{" "}
                to activate.
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
