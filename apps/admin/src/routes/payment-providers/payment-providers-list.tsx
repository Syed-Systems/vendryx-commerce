import { ArrowPath, DocumentText } from "@medusajs/icons"
import { Badge, Button, Container, Heading, StatusBadge, Text, toast } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useState } from "react"
import { SingleColumnPage } from "../../components/layout/pages"
import { sdk } from "../../lib/client"
import { CodRulesCard } from "./components/cod-rules-card"
import { PaymentProvider, ProviderCredentialModal } from "./components/provider-credential-modal"
import { formatProviderTitle, ProviderLogo } from "./components/provider-logo"
import { ProviderRegionModal } from "./components/provider-region-modal"

interface PaymentProvidersResponse {
  payment_providers: PaymentProvider[]
  count: number
}

interface VerificationVerdict {
  status: "healthy" | "failed" | "configured_unverified" | "degraded"
  verified: boolean
  message: string
}

export const PaymentProvidersList = () => {
  const [selectedForCredentials, setSelectedForCredentials] = useState<PaymentProvider | null>(null)
  const [selectedForRegions, setSelectedForRegions] = useState<PaymentProvider | null>(null)
  const [probingId, setProbingId] = useState<string | null>(null)

  const { data, isLoading, refetch, isFetching } = useQuery<PaymentProvidersResponse>({
    queryKey: ["admin", "payment-providers"],
    queryFn: () => sdk.client.fetch<PaymentProvidersResponse>("/admin/payment-providers"),
  })

  const providers = data?.payment_providers || []

  const handleQuickProbe = async (provider: PaymentProvider) => {
    setProbingId(provider.id)
    try {
      const res = await sdk.client.fetch<VerificationVerdict>(
        `/admin/payment-providers/${provider.id}/verify`,
        {
          method: "POST",
          body: {},
        }
      )
      if (res.status === "healthy") {
        toast.success(`${provider.name} verified: ${res.message}`)
      } else if (res.status === "configured_unverified") {
        toast.info(res.message)
      } else {
        toast.error(`Verification probe failed: ${res.message}`)
      }
      refetch()
    } catch (err: any) {
      toast.error(err.message || `Failed to probe ${provider.name}`)
    } finally {
      setProbingId(null)
    }
  }

  const handleCopyWebhook = (url: string) => {
    navigator.clipboard.writeText(url)
    toast.success("Webhook callback URL copied to clipboard")
  }

  const renderHealthBadge = (provider: PaymentProvider) => {
    if (!provider.configured) {
      return <StatusBadge color="grey">Unconfigured</StatusBadge>
    }

    switch (provider.credential_health) {
      case "healthy":
      case "configured":
        return <StatusBadge color="green">Configured</StatusBadge>
      case "configured_unverified":
        return <StatusBadge color="orange">Needs Verification</StatusBadge>
      case "degraded":
        return <StatusBadge color="orange">Degraded</StatusBadge>
      case "failed":
      case "invalid":
        return <StatusBadge color="red">Invalid / Failed</StatusBadge>
      default:
        return <StatusBadge color="grey">{provider.credential_health}</StatusBadge>
    }
  }

  const getProviderDescription = (id: string) => {
    if (id === "pp_manual") {
      return "Accept bKash, Nagad, Rocket, or bank transfers using the account and instructions configured by your store."
    }
    if (id === "pp_system_default") {
      return "Cash on delivery. Customers do not submit a transfer reference for this method."
    }
    if (id.includes("bkash")) {
      return "Direct mobile wallet tokenized checkout with automated refunds for Bangladesh consumers."
    }
    if (id.includes("stripe")) {
      return "Global credit & debit cards, Apple Pay, Google Pay with automated 3D Secure authentication."
    }
    if (id.includes("sslcommerz")) {
      return "All-in-one Bangladesh payment aggregator supporting Visa, Mastercard, and internet banking."
    }
    return "Built-in offline payment and Cash on Delivery with customizable parcel collection instructions."
  }

  return (
    <SingleColumnPage
      widgets={{
        after: [],
        before: [],
      }}
    >
      <div className="flex flex-col gap-y-8">
        {/* Main Online Payment Gateways Container */}
        <Container className="p-0 border border-ui-border-base rounded-2xl overflow-hidden shadow-xs bg-ui-bg-base">
          {/* Header */}
          <div className="flex flex-col sm:flex-row sm:items-center justify-between border-b border-ui-border-base p-6 gap-4 bg-ui-bg-subtle/50">
            <div className="flex flex-col gap-y-1">
              <div className="flex items-center gap-x-3">
                <Heading level="h1" className="text-xl font-bold tracking-tight text-ui-fg-base">
                  Payment Gateways
                </Heading>
                <Badge size="small" color="blue">
                  {providers.length} registered
                </Badge>
              </div>
              <Text size="small" className="text-ui-fg-subtle">
                Configure merchant credentials, automated health probes, and regional availability across checkout channels.
              </Text>
            </div>

            <Button
              size="small"
              variant="secondary"
              onClick={() => refetch()}
              isLoading={isFetching}
            >
              <ArrowPath className="mr-1" />
              Refresh
            </Button>
          </div>

          {/* Provider Cards Grid */}
          <div className="p-6">
            {isLoading ? (
              <div className="py-16 text-center text-ui-fg-subtle">
                <Text size="small">Loading payment providers...</Text>
              </div>
            ) : providers.length === 0 ? (
              <div className="py-16 text-center text-ui-fg-subtle">
                <Text size="small">No payment providers registered in server factory registry.</Text>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
                {providers.map((p) => {
                  const isConfigured = p.configured
                  const regionCount = p.enabled_regions?.length || 0
                  const isProbing = probingId === p.id
                  const callbackUrl =
                    p.callback_url || `https://api.vendryx.com/hooks/payment/${p.id}`

                  return (
                    <div
                      key={p.id}
                      className="flex flex-col justify-between rounded-xl border border-ui-border-base bg-ui-bg-subtle/30 dark:bg-ui-bg-base p-5 shadow-2xs transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:border-ui-border-interactive/60 hover:shadow-xs"
                    >
                      {/* Top identity bar */}
                      <div className="flex flex-col gap-y-3.5">
                        <div className="flex items-start justify-between gap-x-3">
                          <div className="flex items-center gap-x-3">
                            <ProviderLogo id={p.id} size="medium" />
                            <div className="flex flex-col">
                              <Text size="small" weight="plus" className="text-ui-fg-base font-semibold">
                                {formatProviderTitle(p.id, p.name)}
                              </Text>
                              <code className="text-xs text-ui-fg-muted font-mono">
                                {p.id}
                              </code>
                            </div>
                          </div>

                          <div className="flex items-center gap-x-1.5">
                            {renderHealthBadge(p)}
                          </div>
                        </div>

                        {/* Description */}
                        <Text size="xsmall" className="text-ui-fg-subtle leading-relaxed">
                          {getProviderDescription(p.id)}
                        </Text>

                        {/* Webhook Callback box */}
                        <div className="flex items-center justify-between rounded-lg border bg-ui-bg-subtle/40 px-3 py-1.5 text-xs">
                          <span className="text-xs text-ui-fg-muted font-medium truncate mr-2">
                            Webhook: <span className="font-mono text-ui-fg-subtle">{callbackUrl}</span>
                          </span>
                          <button
                            type="button"
                            onClick={() => handleCopyWebhook(callbackUrl)}
                            className="text-ui-fg-interactive hover:text-ui-fg-interactive-hover shrink-0 font-medium text-xs flex items-center gap-x-1"
                            title="Copy webhook callback URL"
                          >
                            <DocumentText className="h-3 w-3" />
                            Copy
                          </button>
                        </div>
                      </div>

                      {/* Region & Action Bar */}
                      <div className="mt-5 pt-4 border-t border-ui-border-base flex items-center justify-between gap-2">
                        {/* Region pill */}
                        <div>
                          {regionCount > 0 ? (
                            <Badge size="small" color="green">
                              {regionCount} region{regionCount > 1 ? "s" : ""} active
                            </Badge>
                          ) : (
                            <Badge size="small" color="grey">
                              Not active in any region
                            </Badge>
                          )}
                        </div>

                        {/* Action buttons */}
                        <div className="flex items-center gap-x-2">
                          <Button
                            size="small"
                            variant="secondary"
                            onClick={() => handleQuickProbe(p)}
                            isLoading={isProbing}
                            title="Test connectivity against provider API"
                          >
                            Verify API
                          </Button>
                          <Button
                            size="small"
                            variant="secondary"
                            onClick={() => setSelectedForRegions(p)}
                            title="Assign to sales regions"
                          >
                            Assign Regions
                          </Button>
                          <Button
                            size="small"
                            variant={isConfigured ? "secondary" : "primary"}
                            onClick={() => setSelectedForCredentials(p)}
                            title={isConfigured ? "Edit stored credentials" : "Configure API credentials"}
                          >
                            {isConfigured ? "Edit Credentials" : "Configure Credentials"}
                          </Button>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        </Container>

        {/* Cash on Delivery Rules Section */}
        <CodRulesCard />
      </div>

      {/* Credential Modal */}
      <ProviderCredentialModal
        provider={selectedForCredentials}
        open={!!selectedForCredentials}
        onClose={() => setSelectedForCredentials(null)}
        onSuccess={() => refetch()}
      />

      {/* Region Assignment Modal */}
      <ProviderRegionModal
        provider={selectedForRegions}
        open={!!selectedForRegions}
        onClose={() => setSelectedForRegions(null)}
        onSuccess={() => refetch()}
      />
    </SingleColumnPage>
  )
}
