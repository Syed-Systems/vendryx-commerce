import { CheckCircle, DocumentText, ShieldCheck, XCircle } from "@medusajs/icons"
import { Badge, Button, FocusModal, Heading, Input, Label, Switch, Text, toast } from "@medusajs/ui"
import { useState, useEffect } from "react"
import { CourierLogo } from "./courier-logo"
import {
  CourierField,
  CourierProvider,
  useProbeCourier,
  useUpdateCourierCredentials,
} from "../../../hooks/api/couriers"

export type { CourierField, CourierProvider }

interface CourierCredentialModalProps {
  courier: CourierProvider | null
  open: boolean
  onClose: () => void
  onSave?: (id: string, credentials: Record<string, any>) => void
}

export const CourierCredentialModal = ({
  courier,
  open,
  onClose,
  onSave,
}: CourierCredentialModalProps) => {
  const [formValues, setFormValues] = useState<Record<string, any>>({})
  const [initialValues, setInitialValues] = useState<Record<string, any>>({})
  const [verdict, setVerdict] = useState<{
    status: "healthy" | "failed"
    message: string
  } | null>(null)
  const [copiedUrl, setCopiedUrl] = useState(false)

  const updateCredentials = useUpdateCourierCredentials()
  const probeCourier = useProbeCourier()
  const isVerifying = probeCourier.isPending || updateCredentials.isPending

  useEffect(() => {
    if (courier) {
      const initial = courier.credentials || {}
      setFormValues(initial)
      setInitialValues(initial)
      if (courier.last_probe) {
        setVerdict({
          status: courier.last_probe.verified ? "healthy" : "failed",
          message: `${courier.last_probe.message} (${courier.last_probe.latency_ms}ms)`,
        })
      } else {
        setVerdict(null)
      }
      setCopiedUrl(false)
    }
  }, [courier, open])

  if (!courier) return null

  const hasChanged = JSON.stringify(formValues) !== JSON.stringify(initialValues)

  const handleFieldChange = (key: string, value: any) => {
    setFormValues((prev) => ({ ...prev, [key]: value }))
  }

  const handleCopyCallback = () => {
    navigator.clipboard.writeText(courier.webhookUrl)
    setCopiedUrl(true)
    toast.success("Webhook callback URL copied to clipboard")
    const timer = setTimeout(() => setCopiedUrl(false), 2500)
    return () => clearTimeout(timer)
  }

  const handleVerify = async () => {
    setVerdict(null)

    // Check required fields
    const missing = courier.fields.filter(
      (f) => f.required && (formValues[f.key] === undefined || formValues[f.key] === "")
    )

    if (missing.length > 0) {
      setVerdict({
        status: "failed",
        message: `Missing required authentication credentials: ${missing
          .map((m) => m.label)
          .join(", ")}`,
      })
      toast.error("Verification probe failed. Missing required fields.")
      return
    }

    try {
      if (hasChanged) {
        await updateCredentials.mutateAsync({
          id: courier.id,
          credentials: formValues,
        })
      }

      const res = await probeCourier.mutateAsync(courier.id)
      if (res.verified) {
        setVerdict({
          status: "healthy",
          message: `${res.message} (${res.latency_ms}ms)`,
        })
        toast.success(res.message)
      } else {
        setVerdict({
          status: "failed",
          message: `${res.message} (${res.latency_ms}ms)`,
        })
        toast.error(res.message)
      }
    } catch (err: any) {
      setVerdict({
        status: "failed",
        message: err.message || "Failed to reach courier probe gateway",
      })
      toast.error("Courier connection probe failed")
    }
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault()

    for (const field of courier.fields) {
      if (
        field.required &&
        (formValues[field.key] === undefined || formValues[field.key] === "")
      ) {
        toast.error(`Please provide a value for ${field.label}`)
        return
      }
    }

    try {
      await updateCredentials.mutateAsync({
        id: courier.id,
        credentials: formValues,
      })
      if (onSave) {
        onSave(courier.id, formValues)
      }
      toast.success(`Credentials encrypted and saved for ${courier.name}`)
      onClose()
    } catch (err: any) {
      toast.error(err.message || `Failed to save credentials for ${courier.name}`)
    }
  }

  return (
    <FocusModal open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <FocusModal.Content className="inset-auto left-1/2 top-1/2 max-h-[calc(100vh-32px)] w-[calc(100vw-32px)] max-w-[660px] -translate-x-1/2 -translate-y-1/2 rounded-2xl shadow-2xl border border-ui-border-base bg-ui-bg-base">
        <FocusModal.Header className="border-b border-ui-border-base px-6 py-4 bg-ui-bg-subtle/50">
          <div className="flex items-center justify-between w-full min-w-0 gap-x-4 pl-2">
            <div className="flex items-center gap-x-3.5 min-w-0">
              <CourierLogo id={courier.id} size="medium" />
              <div className="flex flex-col min-w-0">
                <div className="flex items-center gap-x-2.5">
                  <FocusModal.Title asChild>
                    <Heading level="h2" className="text-base font-bold text-ui-fg-base whitespace-nowrap">
                      {courier.name}
                    </Heading>
                  </FocusModal.Title>
                  <Badge
                    size="small"
                    className="shrink-0 whitespace-nowrap"
                    color={
                      courier.status === "connected"
                        ? "green"
                        : courier.status === "needs_verification"
                        ? "orange"
                        : "grey"
                    }
                  >
                    {courier.status === "connected"
                      ? "Connected"
                      : courier.status === "needs_verification"
                      ? "Needs Verification"
                      : "Unconfigured"}
                  </Badge>
                </div>
                <span className="text-[11px] text-ui-fg-muted font-mono leading-none mt-1">
                  {courier.id}
                </span>
              </div>
            </div>

            <div className="flex items-center gap-x-1.5 rounded-full border border-ui-border-base bg-ui-bg-subtle px-3 py-1 text-xs font-medium shrink-0 whitespace-nowrap text-ui-fg-subtle">
              <span className="h-2 w-2 rounded-full bg-emerald-500 shrink-0" />
              <span>AES-256-GCM Vault</span>
            </div>
          </div>
        </FocusModal.Header>

        <form onSubmit={handleSave} className="flex flex-1 flex-col overflow-hidden">
          <FocusModal.Body className="flex flex-col gap-y-5 overflow-y-auto p-6">
            {/* Vault Notice */}
            <div className="flex items-start gap-x-3 rounded-xl border border-ui-border-base bg-ui-bg-subtle/40 p-3.5 text-xs text-ui-fg-subtle">
              <ShieldCheck className="text-emerald-600 mt-0.5 shrink-0" />
              <div className="flex flex-col gap-y-0.5">
                <span className="font-semibold text-ui-fg-base">Encrypted Dispatch Ledger</span>
                <span>
                  Courier API secrets are encrypted via AES-256-GCM before dispatch calls. Webhook updates trigger idempotent consignment tracking across orders.
                </span>
              </div>
            </div>

            {/* Verification Verdict Banner */}
            {verdict && (
              <div
                className={`flex items-start gap-x-3 rounded-xl border p-4 text-xs ${
                  verdict.status === "healthy"
                    ? "border-emerald-200 bg-emerald-50/80 text-emerald-900"
                    : "border-rose-200 bg-rose-50/80 text-rose-900"
                }`}
              >
                {verdict.status === "healthy" ? (
                  <CheckCircle className="text-emerald-600 mt-0.5 shrink-0" />
                ) : (
                  <XCircle className="text-rose-600 mt-0.5 shrink-0" />
                )}
                <div className="flex flex-col gap-y-1">
                  <span className="font-semibold">
                    {verdict.status === "healthy"
                      ? "Courier API Handshake Successful"
                      : "Handshake Probe Failed"}
                  </span>
                  <span>{verdict.message}</span>
                </div>
              </div>
            )}

            {/* Webhook Callback Helper */}
            <div className="flex flex-col gap-y-1.5 rounded-xl border p-3.5 bg-ui-bg-subtle/30">
              <Label size="xsmall" weight="plus" className="text-ui-fg-muted uppercase tracking-wider">
                Consignment Status Webhook URL
              </Label>
              <div className="flex items-center justify-between gap-x-2">
                <code className="text-xs text-ui-fg-base truncate font-mono bg-ui-bg-field px-2 py-1 rounded border flex-1">
                  {courier.webhookUrl}
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
                Paste this endpoint into your {courier.name} merchant portal to receive automated dispatch, in-transit, and delivery events.
              </Text>
            </div>

            {/* Credential Fields */}
            <div className="flex flex-col gap-y-4">
              {courier.fields.map((field) => {
                const isSecret = field.kind === "secret"

                if (field.kind === "bool") {
                  return (
                    <div
                      key={field.key}
                      className="flex items-center justify-between rounded-xl border p-3.5 bg-ui-bg-subtle/20"
                    >
                      <div className="flex flex-col gap-y-0.5">
                        <Label size="small" weight="plus">
                          {field.label}
                        </Label>
                        <Text size="xsmall" className="text-ui-fg-subtle">
                          Route requests through courier sandbox test environment
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
                        <span className="text-ui-fg-muted text-[11px] font-normal ml-1.5">(optional)</span>
                      )}
                    </Label>
                    <div className="w-full">
                      <Input
                        type={isSecret ? "password" : "text"}
                        placeholder={field.placeholder || `Enter ${field.label.toLowerCase()}`}
                        value={formValues[field.key] || ""}
                        onChange={(e) => handleFieldChange(field.key, e.target.value)}
                        className={`w-full ${isSecret ? "font-mono text-sm" : "text-sm"}`}
                      />
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
            >
              Verify API Handshake
            </Button>

            <div className="flex items-center gap-x-2">
              <Button type="button" variant="secondary" onClick={onClose} disabled={isVerifying}>
                Cancel
              </Button>
              <Button type="submit" variant="primary" disabled={!hasChanged || isVerifying}>
                Save Credentials
              </Button>
            </div>
          </FocusModal.Footer>
        </form>
      </FocusModal.Content>
    </FocusModal>
  )
}
