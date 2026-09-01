import {
  ArrowPath,
  CheckCircle,
  CurrencyDollar,
  Eye,
  EyeSlash,
  InformationCircle,
  LockClosedSolid,
} from "@medusajs/icons"
import {
  Badge,
  Button,
  Container,
  Drawer,
  Heading,
  Input,
  Label,
  StatusBadge,
  Switch,
  Table,
  Text,
  toast,
} from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"

import {
  listPlatformPaymentProviders,
  upsertPlatformPaymentProviderCredentials,
  type CredentialField,
  type CredentialHealth,
  type PlatformPaymentProvider,
} from "./api"

/**
 * `Table.Cell` renders a <td> and forwards every prop to it, but @medusajs/ui types it as
 * `HTMLAttributes<HTMLTableCellElement>` — which omits `colSpan`, because colSpan lives on
 * `TdHTMLAttributes`. Widened to the attributes the element actually accepts rather than
 * cast to `any`, so the rest of the props stay type-checked.
 */
const SpanningCell = Table.Cell as React.FC<
  React.PropsWithChildren<React.TdHTMLAttributes<HTMLTableCellElement>>
>


function healthBadge(health: CredentialHealth) {
  switch (health) {
    case "valid":
      return <StatusBadge color="green">Valid</StatusBadge>
    case "expiring":
      return <StatusBadge color="orange">Expiring</StatusBadge>
    case "failing_auth":
      return <StatusBadge color="red">Failing Auth</StatusBadge>
    case "unknown":
      return <StatusBadge color="grey">Unknown</StatusBadge>
    case "unconfigured":
    default:
      return <StatusBadge color="grey">Unconfigured</StatusBadge>
  }
}

interface CredentialDrawerProps {
  provider: PlatformPaymentProvider | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

function CredentialDrawer({ provider, open, onOpenChange }: CredentialDrawerProps) {
  const queryClient = useQueryClient()
  const [formValues, setFormValues] = useState<Record<string, unknown>>({})
  const [revealed, setRevealed] = useState<Record<string, boolean>>({})

  const handleOpen = (isOpen: boolean) => {
    if (isOpen && provider) {
      const initial: Record<string, unknown> = {}
      provider.credential_schema.forEach((field) => {
        if (provider.credential?.fields?.[field.key] !== undefined) {
          initial[field.key] = provider.credential.fields[field.key]
        } else if (field.kind === "bool") {
          initial[field.key] = false
        } else {
          initial[field.key] = ""
        }
      })
      setFormValues(initial)
      setRevealed({})
    }
    onOpenChange(isOpen)
  }

  const mutation = useMutation({
    mutationFn: () => {
      if (!provider) throw new Error("No provider selected")
      return upsertPlatformPaymentProviderCredentials(provider.id, formValues)
    },
    onSuccess: (data) => {
      toast.success(`${data.payment_provider.name} credentials saved successfully`)
      queryClient.invalidateQueries({ queryKey: ["platform", "payment-providers"] })
      onOpenChange(false)
    },
    onError: (err: Error) => {
      toast.error(err.message || "Failed to save credentials")
    },
  })

  if (!provider) return null

  return (
    <Drawer open={open} onOpenChange={handleOpen}>
      <Drawer.Content className="max-w-[560px]">
        <Drawer.Header>
          <div className="flex items-center gap-x-2">
            <LockClosedSolid className="text-ui-fg-subtle" />
            <Drawer.Title>Configure {provider.name}</Drawer.Title>
          </div>
          <Drawer.Description>
            Platform-level corporate credentials for collecting subscription invoices across merchants.
          </Drawer.Description>
        </Drawer.Header>

        <Drawer.Body className="flex flex-col gap-y-4 p-6 overflow-y-auto">
          <div className="rounded-lg border border-ui-border-base bg-ui-bg-subtle p-3 flex gap-x-3 items-start">
            <InformationCircle className="text-ui-fg-interactive shrink-0 mt-0.5" />
            <Text size="small" className="text-ui-fg-subtle">
              These credentials are encrypted using <strong>AES-256-GCM</strong> and saved in the platform
              vault. They are used for automated subscription billing runs, invoice retries, and dunning collections.
            </Text>
          </div>

          <form
            id="provider-cred-form"
            onSubmit={(e) => {
              e.preventDefault()
              mutation.mutate()
            }}
            className="flex flex-col gap-y-4"
          >
            {provider.credential_schema.map((field: CredentialField) => {
              const isSecret = field.kind === "secret"
              const isBool = field.kind === "bool"
              const isShown = revealed[field.key]
              const val = formValues[field.key]

              if (isBool) {
                return (
                  <div
                    key={field.key}
                    className="flex items-center justify-between rounded-lg border border-ui-border-base p-3"
                  >
                    <div className="flex flex-col">
                      <Label size="small" className="font-medium">
                        {field.label}
                      </Label>
                      <Text size="xsmall" className="text-ui-fg-muted">
                        {field.key === "is_sandbox"
                          ? "Enable sandbox/test mode for simulation"
                          : `Toggle ${field.label}`}
                      </Text>
                    </div>
                    <Switch
                      checked={Boolean(val)}
                      onCheckedChange={(checked) =>
                        setFormValues((prev) => ({ ...prev, [field.key]: checked }))
                      }
                    />
                  </div>
                )
              }

              return (
                <div key={field.key} className="flex flex-col gap-y-1.5">
                  <div className="flex items-center justify-between">
                    <Label size="small" htmlFor={`field-${field.key}`}>
                      {field.label} {field.required && <span className="text-ui-fg-error">*</span>}
                    </Label>
                    <Badge size="2xsmall" color="grey">
                      {field.kind}
                    </Badge>
                  </div>

                  <div className="relative flex items-center">
                    <Input
                      id={`field-${field.key}`}
                      type={isSecret && !isShown ? "password" : "text"}
                      value={(val as string) || ""}
                      placeholder={`Enter ${field.label.toLowerCase()}`}
                      autoComplete="off"
                      onChange={(e) =>
                        setFormValues((prev) => ({ ...prev, [field.key]: e.target.value }))
                      }
                      className={isSecret ? "pr-10 font-mono text-xs" : "font-mono text-xs"}
                      required={field.required}
                    />
                    {isSecret && (
                      <Button
                        type="button"
                        variant="transparent"
                        size="small"
                        className="absolute right-1 text-ui-fg-muted hover:text-ui-fg-base"
                        onClick={() =>
                          setRevealed((prev) => ({ ...prev, [field.key]: !prev[field.key] }))
                        }
                        aria-label={isShown ? "Hide secret" : "Show secret"}
                      >
                        {isShown ? <EyeSlash className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                      </Button>
                    )}
                  </div>
                </div>
              )
            })}
          </form>
        </Drawer.Body>

        <Drawer.Footer>
          <Button variant="secondary" size="small" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            size="small"
            type="submit"
            form="provider-cred-form"
            isLoading={mutation.isPending}
          >
            Save Credentials
          </Button>
        </Drawer.Footer>
      </Drawer.Content>
    </Drawer>
  )
}

export function Gateways() {
  const [selectedProvider, setSelectedProvider] = useState<PlatformPaymentProvider | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)

  const { data, isLoading, refetch, isFetching } = useQuery({
    queryKey: ["platform", "payment-providers"],
    queryFn: listPlatformPaymentProviders,
  })

  const providers = data?.payment_providers ?? []
  const configuredCount = providers.filter((p) => p.configured).length

  return (
    <div className="flex flex-col gap-y-5">
      <div className="flex flex-col gap-y-1">
        <div className="flex items-center justify-between">
          <Heading level="h2">Corporate Payment Gateways</Heading>
          <Button
            variant="secondary"
            size="small"
            onClick={() => refetch()}
            disabled={isFetching}
          >
            <ArrowPath className={isFetching ? "animate-spin" : ""} />
            Refresh
          </Button>
        </div>
        <Text size="small" className="text-ui-fg-subtle">
          Configure platform-level payment provider credentials to collect subscription invoices,
          usage fees, and dunning attempts from merchants.
        </Text>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Container className="p-4">
          <Text size="xsmall" className="text-ui-fg-muted font-medium uppercase tracking-wider">
            Available Gateways
          </Text>
          <Heading level="h2" className="mt-1">
            {providers.length}
          </Heading>
          <Text size="xsmall" className="text-ui-fg-subtle mt-1">
            bKash, SSLCommerz, Stripe
          </Text>
        </Container>

        <Container className="p-4">
          <Text size="xsmall" className="text-ui-fg-muted font-medium uppercase tracking-wider">
            Configured for Billing
          </Text>
          <Heading level="h2" className="mt-1 text-ui-fg-interactive">
            {configuredCount} / {providers.length}
          </Heading>
          <Text size="xsmall" className="text-ui-fg-subtle mt-1">
            Ready for automated billing runs
          </Text>
        </Container>

        <Container className="p-4">
          <Text size="xsmall" className="text-ui-fg-muted font-medium uppercase tracking-wider">
            Encryption & Security
          </Text>
          <div className="mt-1 flex items-center gap-x-1.5 text-ui-fg-success font-semibold text-base">
            <CheckCircle className="h-5 w-5" /> AES-256-GCM Vault
          </div>
          <Text size="xsmall" className="text-ui-fg-subtle mt-1">
            Isolated in platform schema
          </Text>
        </Container>
      </div>

      <Container className="p-0 overflow-hidden">
        <Table>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Provider</Table.HeaderCell>
              <Table.HeaderCell>Provider ID</Table.HeaderCell>
              <Table.HeaderCell>Status</Table.HeaderCell>
              <Table.HeaderCell>Health</Table.HeaderCell>
              <Table.HeaderCell>Version</Table.HeaderCell>
              <Table.HeaderCell>Last Updated</Table.HeaderCell>
              <Table.HeaderCell className="text-right">Action</Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {isLoading ? (
              <Table.Row>
                <SpanningCell colSpan={7} className="text-center py-8 text-ui-fg-muted">
                  Loading payment gateways...
                </SpanningCell>
              </Table.Row>
            ) : providers.length === 0 ? (
              <Table.Row>
                <SpanningCell colSpan={7} className="text-center py-8 text-ui-fg-muted">
                  No payment gateways registered in server.
                </SpanningCell>
              </Table.Row>
            ) : (
              providers.map((p) => (
                <Table.Row key={p.id}>
                  <Table.Cell className="font-medium text-ui-fg-base">
                    <div className="flex items-center gap-x-2">
                      <CurrencyDollar className="text-ui-fg-interactive h-4 w-4" />
                      <span>{p.name}</span>
                    </div>
                  </Table.Cell>
                  <Table.Cell className="font-mono text-xs text-ui-fg-subtle">
                    {p.id}
                  </Table.Cell>
                  <Table.Cell>
                    {p.configured ? (
                      <Badge color="green" size="small">
                        Configured
                      </Badge>
                    ) : (
                      <Badge color="grey" size="small">
                        Not Configured
                      </Badge>
                    )}
                  </Table.Cell>
                  <Table.Cell>{healthBadge(p.credential_health)}</Table.Cell>
                  <Table.Cell className="text-ui-fg-subtle">
                    {p.credential ? `v${p.credential.credential_version}` : "—"}
                  </Table.Cell>
                  <Table.Cell className="text-ui-fg-subtle text-xs">
                    {p.credential?.updated_at
                      ? new Date(p.credential.updated_at).toLocaleString()
                      : "—"}
                  </Table.Cell>
                  <Table.Cell className="text-right">
                    <Button
                      variant={p.configured ? "secondary" : "primary"}
                      size="small"
                      onClick={() => {
                        setSelectedProvider(p)
                        setDrawerOpen(true)
                      }}
                    >
                      {p.configured ? "Edit Credentials" : "Configure"}
                    </Button>
                  </Table.Cell>
                </Table.Row>
              ))
            )}
          </Table.Body>
        </Table>
      </Container>

      <CredentialDrawer
        provider={selectedProvider}
        open={drawerOpen}
        onOpenChange={setDrawerOpen}
      />
    </div>
  )
}
