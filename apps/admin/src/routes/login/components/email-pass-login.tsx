import { zodResolver } from "@hookform/resolvers/zod"
import type { AuthTypes } from "@medusajs/types"
import { Alert, Button, Hint, Input } from "@medusajs/ui"
import { useForm } from "react-hook-form"
import { useTranslation } from "react-i18next"
import { useLocation, useNavigate } from "react-router-dom"
import * as z from "zod"

import { Form } from "../../../components/common/form"
import { useSignInWithEmailPass } from "../../../hooks/api"
import { isFetchError } from "../../../lib/is-fetch-error"
import {
  AVAILABLE_TENANTS,
  resolveTenantHostFromEmail,
  setActiveTenantHost,
} from "../../../lib/tenant"

const EmailPassSchema = z.object({
  email: z.string().email(),
  password: z.string(),
})

type EmailPassLoginProps = {
  onMfaChallenge?: (
    challenge: AuthTypes.AuthMfaChallengeDTO,
    onSuccess: (token: string) => void | Promise<void>
  ) => void
}

export const EmailPassLogin = ({ onMfaChallenge }: EmailPassLoginProps) => {
  const { t } = useTranslation()
  const location = useLocation()
  const navigate = useNavigate()

  const from = location.state?.from?.pathname || "/orders"

  const form = useForm<z.infer<typeof EmailPassSchema>>({
    resolver: zodResolver(EmailPassSchema),
    defaultValues: {
      email: AVAILABLE_TENANTS[0].defaultEmail,
      password: "Sup3rSecret!",
    },
  })

  const { mutateAsync, isPending } = useSignInWithEmailPass()

  const handleSubmit = form.handleSubmit(async ({ email, password }) => {
    // Automatically route to the store matching the entered credentials
    const targetHost = resolveTenantHostFromEmail(email)
    setActiveTenantHost(targetHost)

    await mutateAsync(
      {
        email,
        password,
      },
      {
        onError: (error) => {
          if (isFetchError(error)) {
            if (error.status === 401) {
              form.setError("email", {
                type: "manual",
                message: error.message,
              })

              return
            }
          }

          form.setError("root.serverError", {
            type: "manual",
            message: error.message,
          })
        },
        onSuccess: (result) => {
          if (typeof result === "object" && "mfa_challenge" in result) {
            onMfaChallenge?.(result.mfa_challenge, () => {
              navigate(from, { replace: true })
            })
            return
          }

          navigate(from, { replace: true })
        },
      }
    )
  })

  const serverError = form.formState.errors?.root?.serverError?.message
  const validationError =
    form.formState.errors.email?.message ||
    form.formState.errors.password?.message

  const currentEmail = form.watch("email")

  return (
    <Form {...form}>
      <form onSubmit={handleSubmit} className="flex w-full flex-col gap-y-4">
        <div className="flex flex-col gap-y-1.5 rounded-lg border border-ui-border-base/70 bg-ui-bg-subtle/50 p-2.5">
          <span className="text-xs font-medium text-ui-fg-muted">
            Select store dashboard:
          </span>
          <div className="grid grid-cols-3 gap-1.5">
            {AVAILABLE_TENANTS.map((tenant) => {
              const active = currentEmail === tenant.defaultEmail
              return (
                <button
                  key={tenant.id}
                  type="button"
                  onClick={() => {
                    form.setValue("email", tenant.defaultEmail)
                    form.setValue("password", "Sup3rSecret!")
                    setActiveTenantHost(tenant.host)
                  }}
                  className={`flex flex-col p-1.5 rounded-md text-start border transition-[color,background-color,border-color,box-shadow,transform,opacity] text-xs ${
                    active
                      ? "border-ui-border-interactive bg-ui-bg-base text-ui-fg-base shadow-xs"
                      : "border-ui-border-base/60 bg-ui-bg-base/50 text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-base"
                  }`}
                >
                  <span className="font-semibold truncate">
                    {tenant.genre === "appliance"
                      ? "Appliances"
                      : tenant.genre === "clothing"
                      ? "Clothing"
                      : "Tech"}
                  </span>
                  <span className="text-xs text-ui-fg-muted truncate">
                    {tenant.tierLabel}
                  </span>
                </button>
              )
            })}
          </div>
        </div>

        <div className="flex flex-col gap-y-1">
          <Form.Field
            control={form.control}
            name="email"
            render={({ field }) => {
              return (
                <Form.Item>
                  <Form.Control>
                    <Input
                      autoComplete="email"
                      {...field}
                      className="bg-ui-bg-field-component"
                      placeholder={t("fields.email")}
                    />
                  </Form.Control>
                </Form.Item>
              )
            }}
          />
          <Form.Field
            control={form.control}
            name="password"
            render={({ field }) => {
              return (
                <Form.Item>
                  <Form.Label>{}</Form.Label>
                  <Form.Control>
                    <Input
                      type="password"
                      autoComplete="current-password"
                      {...field}
                      className="bg-ui-bg-field-component"
                      placeholder={t("fields.password")}
                    />
                  </Form.Control>
                </Form.Item>
              )
            }}
          />
        </div>
        {validationError && (
          <div className="text-center">
            <Hint className="inline-flex" variant={"error"}>
              {validationError}
            </Hint>
          </div>
        )}
        {serverError && (
          <Alert
            className="bg-ui-bg-base items-center p-2"
            dismissible
            variant="error"
          >
            {serverError}
          </Alert>
        )}
        <Button className="w-full" type="submit" isLoading={isPending}>
          {t("actions.continueWithEmail")}
        </Button>
      </form>
    </Form>
  )
}
