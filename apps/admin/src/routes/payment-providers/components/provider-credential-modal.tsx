import { Eye, EyeSlash } from "@medusajs/icons"
import { Button, FocusModal, Heading, IconButton, Input, Label, Switch, Text, toast } from "@medusajs/ui"
import { useState } from "react"
import { sdk } from "../../../lib/client"

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
  credential_health: "healthy" | "degraded" | "failed" | "unconfigured"
  credential_schema: CredentialField[]
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

export const ProviderCredentialModal = ({
  provider,
  open,
  onClose,
  onSuccess,
}: ProviderCredentialModalProps) => {
  const [formValues, setFormValues] = useState<Record<string, any>>({})
  const [visibleSecrets, setVisibleSecrets] = useState<Record<string, boolean>>({})
  const [isSubmitting, setIsSubmitting] = useState(false)

  if (!provider) return null

  const handleFieldChange = (key: string, value: any) => {
    setFormValues((prev) => ({ ...prev, [key]: value }))
  }

  const toggleSecretVisibility = (key: string) => {
    setVisibleSecrets((prev) => ({ ...prev, [key]: !prev[key] }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    // Validate required fields
    for (const field of provider.credential_schema) {
      if (field.required && (formValues[field.key] === undefined || formValues[field.key] === "")) {
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
      toast.success(`Credentials saved for ${provider.name}`)
      onSuccess()
      onClose()
    } catch (err: any) {
      toast.error(err.message || "Failed to update provider credentials")
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <FocusModal open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <FocusModal.Content className="inset-auto left-1/2 top-1/2 max-h-[calc(100vh-32px)] w-[calc(100vw-32px)] max-w-[560px] -translate-x-1/2 -translate-y-1/2">
        <FocusModal.Header>
          <div className="flex flex-col">
            <FocusModal.Title asChild>
              <Heading level="h2">Configure {provider.name}</Heading>
            </FocusModal.Title>
            <Text size="small" className="text-ui-fg-subtle">
              Provider ID: {provider.id}
            </Text>
          </div>
        </FocusModal.Header>

        <form onSubmit={handleSubmit} className="flex flex-1 flex-col overflow-hidden">
          <FocusModal.Body className="flex flex-col gap-y-4 overflow-y-auto p-6">
            <Text size="small" className="text-ui-fg-subtle">
              Enter the merchant API credentials below. Credentials are encrypted with AES-256-GCM before storage.
            </Text>

            {provider.credential_schema.map((field) => {
              const isSecret = field.kind === "secret"
              const isVisible = visibleSecrets[field.key]

              if (field.kind === "bool") {
                return (
                  <div key={field.key} className="flex items-center justify-between rounded-lg border p-3">
                    <div className="flex flex-col gap-y-0.5">
                      <Label size="small" weight="plus">
                        {field.label} {field.required && <span className="text-ui-fg-error">*</span>}
                      </Label>
                      <Text size="xsmall" className="text-ui-fg-subtle">
                        {field.key === "is_sandbox"
                          ? "Enable sandbox/test mode for simulation"
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
                <div key={field.key} className="flex flex-col gap-y-1.5">
                  <Label size="small" weight="plus">
                    {field.label} {field.required && <span className="text-ui-fg-error">*</span>}
                  </Label>
                  <div className="relative flex items-center">
                    <Input
                      type={isSecret && !isVisible ? "password" : "text"}
                      placeholder={field.kind === "url" ? "https://..." : `Enter ${field.label}`}
                      value={formValues[field.key] || ""}
                      onChange={(e) => handleFieldChange(field.key, e.target.value)}
                      className={isSecret ? "pr-10" : ""}
                    />
                    {isSecret && (
                      <IconButton
                        type="button"
                        size="small"
                        variant="transparent"
                        className="absolute right-2"
                        onClick={() => toggleSecretVisibility(field.key)}
                      >
                        {isVisible ? <EyeSlash className="text-ui-fg-subtle" /> : <Eye className="text-ui-fg-subtle" />}
                      </IconButton>
                    )}
                  </div>
                </div>
              )
            })}
          </FocusModal.Body>

          <FocusModal.Footer>
            <div className="flex items-center justify-end gap-x-2">
              <Button type="button" variant="secondary" onClick={onClose} disabled={isSubmitting}>
                Cancel
              </Button>
              <Button type="submit" variant="primary" isLoading={isSubmitting}>
                Save Credentials
              </Button>
            </div>
          </FocusModal.Footer>
        </form>
      </FocusModal.Content>
    </FocusModal>
  )
}
