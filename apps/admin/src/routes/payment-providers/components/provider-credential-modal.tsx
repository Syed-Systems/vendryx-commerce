import { CheckCircle, DocumentText, InformationCircleSolid, ShieldCheck, XCircle } from "@medusajs/icons"
import { Badge, Button, FocusModal, Heading, Input, Label, Switch, Text, toast } from "@medusajs/ui"
import { useState, useEffect } from "react"
import { sdk } from "../../../lib/client"
import { formatProviderTitle, ProviderLogo } from "./provider-logo"

export interface CredentialField {
  key: string
  label: string
  kind: "secret" | "text" | "url" | "bool" | "number"
  required: boolean
}

export interface PaymentProvider {
  id: string
  name: string
  is_enabled: boolean
  enabled_regions: string[]
  configured: boolean
  credential_health: "healthy" | "degraded" | "failed" | "unconfigured" | "configured" | "configured_unverified" | "invalid"
  credential_schema: CredentialField[]
  callback_url?: string
  webhook_slug?: string
  last_verified_at?: string
  credential?: {
    provider_id: string
    updated_at: string
    health: string
    last_tested_at?: string
  } | null
}

interface ProviderCredentialModalProps {
  provider: PaymentProvider | null
  open: boolean
  onClose: () => void
  onSuccess: () => void
}

interface VerificationVerdict {
  status: "healthy" | "failed" | "configured_unverified" | "degraded"
  verified: boolean
  message: string
  checked_at?: string
}

export const ProviderCredentialModal = ({
  provider,
  open,
  onClose,
  onSuccess,
}: ProviderCredentialModalProps) => {
  const [formValues, setFormValues] = useState<Record<string, any>>({})
  const [initialValues, setInitialValues] = useState<Record<string, any>>({})
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [isVerifying, setIsVerifying] = useState(false)
  const [verdict, setVerdict] = useState<VerificationVerdict | null>(null)
  const [copiedUrl, setCopiedUrl] = useState(false)

  // Initialize form when provider opens
  useEffect(() => {
    if (provider) {
      const defaults: Record<string, any> = {}
      if (provider.id.includes("sslcommerz")) {
        defaults.is_sandbox = true
      } else if (provider.id.includes("stripe")) {
        defaults.livemode = false
      }
      setFormValues(defaults)
      setInitialValues(defaults)
      setVerdict(null)
      setCopiedUrl(false)
    }
  }, [provider, open])

  if (!provider) return null

  const callbackUrl =
    provider.callback_url || `https://api.vendryx.com/hooks/payment/${provider.id}`

  const hasChanged = JSON.stringify(formValues) !== JSON.stringify(initialValues)

  const handleFieldChange = (key: string, value: any) => {
    setFormValues((prev) => ({ ...prev, [key]: value }))
  }

  const handleCopyCallback = () => {
    navigator.clipboard.writeText(callbackUrl)
    setCopiedUrl(true)
    toast.success("Webhook callback URL copied to clipboard")
    setTimeout(() => setCopiedUrl(false), 2500)
  }

  // Live Verification Probe
  const handleVerify = async () => {
    setIsVerifying(true)
    setVerdict(null)

    try {
      const res = await sdk.client.fetch<VerificationVerdict>(
        `/admin/payment-providers/${provider.id}/verify`,
        {
          method: "POST",
          body: {
            credentials: formValues,
          },
        }
      )
      setVerdict(res)
      if (res.status === "healthy") {
        toast.success(`Connection verified: ${res.message}`)
      } else if (res.status === "configured_unverified") {
        toast.info(res.message)
      } else {
        toast.error(`Verification probe failed: ${res.message}`)
      }
    } catch (err: any) {
      setVerdict({
        status: "failed",
        verified: false,
        message: err.message || "Failed to reach provider endpoint",
      })
      toast.error(err.message || "Provider verification failed")
    } finally {
      setIsVerifying(false)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    // Validate required fields
    for (const field of provider.credential_schema) {
      if (
        field.required &&
        (formValues[field.key] === undefined || formValues[field.key] === "")
      ) {
        toast.error(`Please provide a value for ${field.label}`)
        return
      }
    }

    setIsSubmitting(true)
    try {
      await sdk.client.fetch(`/admin/payment-providers/${provider.id}/credentials`, {
        method: "POST",
        body: {
          credentials: formValues,
        },
      })
      toast.success(`Encrypted credentials saved for ${formatProviderTitle(provider.id, provider.name)}`)
      onSuccess()
      onClose()
    } catch (err: any) {
      toast.error(err.message || "Failed to update provider credentials")
    } finally {
      setIsSubmitting(false)
    }
  }

  const isStripe = provider.id.includes("stripe")
  const isSslcommerz = provider.id.includes("sslcommerz")
  const isBkash = provider.id.includes("bkash")

  return (
    <FocusModal open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <FocusModal.Content className="inset-auto left-1/2 top-1/2 max-h-[calc(100vh-32px)] w-[calc(100vw-32px)] max-w-[660px] -translate-x-1/2 -translate-y-1/2 rounded-2xl shadow-2xl border border-ui-border-base bg-ui-bg-base">
        <FocusModal.Header className="border-b border-ui-border-base px-6 py-4 bg-ui-bg-subtle/50">
          <div className="flex items-center justify-between w-full min-w-0 gap-x-4 pl-2">
            <div className="flex items-center gap-x-3.5 min-w-0">
              <ProviderLogo id={provider.id} size="medium" />
              <div className="flex flex-col min-w-0">
                <div className="flex items-center gap-x-2.5">
                  <FocusModal.Title asChild>
                    <Heading level="h2" className="text-base font-bold text-ui-fg-base whitespace-nowrap">
                      {formatProviderTitle(provider.id, provider.name)}
                    </Heading>
                  </FocusModal.Title>
                  <Badge size="small" className="shrink-0 whitespace-nowrap" color={provider.configured ? "green" : "grey"}>
                    {provider.configured ? "Configured" : "Unconfigured"}
                  </Badge>
                </div>
                <span className="text-xs text-ui-fg-muted font-mono leading-none mt-1">
                  {provider.id}
                </span>
              </div>
            </div>

            {/* Environment Indicator */}
            {(isStripe || isSslcommerz || isBkash) && (
              <div className="flex items-center gap-x-1.5 rounded-full border border-ui-border-base bg-ui-bg-subtle px-3 py-1 text-xs font-medium shrink-0 whitespace-nowrap text-ui-fg-subtle">
                <span className="h-2 w-2 rounded-full bg-emerald-500 shrink-0" />
                <span>AES-256-GCM Vault</span>
              </div>
            )}
          </div>
        </FocusModal.Header>

        <form onSubmit={handleSubmit} className="flex flex-1 flex-col overflow-hidden">
          <FocusModal.Body className="flex flex-col gap-y-5 overflow-y-auto p-6">
            {/* Vault Notice */}
            <div className="flex items-start gap-x-3 rounded-xl border border-ui-border-base bg-ui-bg-subtle/40 p-3.5 text-xs text-ui-fg-subtle">
              <ShieldCheck className="text-emerald-600 mt-0.5 shrink-0" />
              <div className="flex flex-col gap-y-0.5">
                <span className="font-semibold text-ui-fg-base">Zero-Knowledge Hardware Encryption</span>
                <span>
                  All secret keys are encrypted client-isolated via AES-256-GCM before database commit. Stored fields remain permanently opaque to platform logs and operators.
                </span>
              </div>
            </div>

            {/* Space Control Plane Governance Notice */}
            {isStripe && (
              <div className="flex items-center justify-between rounded-xl border border-blue-200 bg-blue-50/60 p-3.5 text-xs text-blue-950 dark:border-blue-900/50 dark:bg-blue-950/20 dark:text-blue-200">
                <div className="flex items-center gap-x-2.5">
                  <Badge size="small" color="blue">Space Governed</Badge>
                  <span className="font-medium text-ui-fg-subtle">
                    Live billing subscriptions, quotas, and merchant platform fees are governed by Space Control Plane.
                  </span>
                </div>
                <a
                  href="http://127.0.0.1:5175/space/billing"
                  target="_blank"
                  rel="noreferrer"
                  className="inline-flex items-center gap-x-1 font-semibold text-blue-600 hover:text-blue-700 underline text-xs shrink-0 ml-3"
                >
                  Open Space Console &rarr;
                </a>
              </div>
            )}

            {/* Verification Verdict Banner */}
            {verdict && (
              <div
                className={`flex items-start gap-x-3 rounded-xl border p-4 text-xs ${
                  verdict.status === "healthy"
                    ? "border-emerald-200 bg-emerald-50/80 text-emerald-900"
                    : verdict.status === "configured_unverified"
                    ? "border-amber-200 bg-amber-50/80 text-amber-900"
                    : "border-rose-200 bg-rose-50/80 text-rose-900"
                }`}
              >
                {verdict.status === "healthy" ? (
                  <CheckCircle className="text-emerald-600 mt-0.5 shrink-0" />
                ) : verdict.status === "configured_unverified" ? (
                  <InformationCircleSolid className="text-amber-600 mt-0.5 shrink-0" />
                ) : (
                  <XCircle className="text-rose-600 mt-0.5 shrink-0" />
                )}
                <div className="flex flex-col gap-y-1">
                  <span className="font-semibold">
                    {verdict.status === "healthy"
                      ? "Connection Verified Successfully"
                      : verdict.status === "configured_unverified"
                      ? "Configuration Recorded (Pending Verification)"
                      : "Probe Verification Failed"}
                  </span>
                  <span>{verdict.message}</span>
                </div>
              </div>
            )}

            {/* Webhook Callback Helper */}
            <div className="flex flex-col gap-y-1.5 rounded-xl border p-3.5 bg-ui-bg-subtle/30">
              <Label size="xsmall" weight="plus" className="text-ui-fg-muted uppercase tracking-wider">
                Webhook / IPN Callback URL
              </Label>
              <div className="flex items-center justify-between gap-x-2">
                <code className="text-xs text-ui-fg-base truncate font-mono bg-ui-bg-field px-2 py-1 rounded border flex-1">
                  {callbackUrl}
                </code>
                <Button
                  type="button"
                  size="small"
                  variant="secondary"
                  onClick={handleCopyCallback}
                  className="shrink-0"
                >
                  <DocumentText className="mr-1 h-3.5 w-3.5" />
                  {copiedUrl ? "Copied" : "Copy URL"}
                </Button>
              </div>
              <Text size="xsmall" className="text-ui-fg-subtle">
                Provide this callback address in your merchant dashboard to receive real-time webhook confirmations.
              </Text>
            </div>

            {/* Credential Fields */}
            <div className="flex flex-col gap-y-4">
              {provider.credential_schema.map((field) => {
                const isSecret = field.kind === "secret"

                if (field.kind === "bool") {
                  return (
                    <div
                      key={field.key}
                      className="flex items-center justify-between rounded-xl border p-3.5 bg-ui-bg-subtle/20"
                    >
                      <div className="flex flex-col gap-y-0.5">
                        <Label size="small" weight="plus">
                          {field.label} {field.required && <span className="text-rose-500">*</span>}
                        </Label>
                        <Text size="xsmall" className="text-ui-fg-subtle">
                          {field.key === "is_sandbox"
                            ? "Route payments through sandbox test endpoints instead of live merchant acquiring"
                            : field.key === "livemode"
                            ? "Enable Stripe live production processing mode"
                            : `Enable ${field.label}`}
                        </Text>
                      </div>
                      <Switch
                        checked={!!formValues[field.key]}
                        onCheckedChange={(checked) => handleFieldChange(field.key, checked)}
                      />
                    </div>
                  )
                }

                return (
                  <div key={field.key} className="flex flex-col gap-y-1.5 w-full">
                    <Label size="small" weight="plus" className="text-ui-fg-base">
                      {field.label}
                      {field.required ? (
                        <span className="text-rose-500 ml-1 font-semibold">*</span>
                      ) : (
                        <span className="text-ui-fg-muted text-xs font-normal ml-1.5">(optional)</span>
                      )}
                    </Label>
                    <div className="w-full">
                      {field.key === "payment_instructions" ? (
                        <textarea
                          rows={3}
                          maxLength={500}
                          value={formValues[field.key] || ""}
                          onChange={(e) => handleFieldChange(field.key, e.target.value)}
                          placeholder="Explain how customers should transfer and identify the payment"
                          className="w-full rounded-md border border-ui-border-base bg-ui-bg-field px-3 py-2 text-sm"
                        />
                      ) : (
                        <Input
                          type={isSecret ? "password" : field.kind === "number" ? "number" : "text"}
                          maxLength={field.key === "payment_account" ? 80 : undefined}
                          placeholder={
                            field.kind === "url"
                              ? "https://..."
                              : `Enter ${field.label.toLowerCase()}`
                          }
                          value={formValues[field.key] || ""}
                          onChange={(e) => handleFieldChange(field.key, e.target.value)}
                          className={`w-full ${isSecret ? "font-mono text-sm" : "text-sm"}`}
                        />
                      )}
                    </div>
                  </div>
                )
              })}
            </div>
          </FocusModal.Body>

          <FocusModal.Footer className="border-t p-4 flex items-center justify-between bg-ui-bg-subtle/20">
            <Button
              type="button"
              variant="secondary"
              onClick={handleVerify}
              isLoading={isVerifying}
              disabled={isSubmitting}
            >
              Verify Connection
            </Button>

            <div className="flex items-center gap-x-2">
              <Button
                type="button"
                variant="secondary"
                onClick={onClose}
                disabled={isSubmitting || isVerifying}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                variant="primary"
                isLoading={isSubmitting}
                disabled={!hasChanged || isVerifying}
              >
                Save Credentials
              </Button>
            </div>
          </FocusModal.Footer>
        </form>
      </FocusModal.Content>
    </FocusModal>
  )
}
