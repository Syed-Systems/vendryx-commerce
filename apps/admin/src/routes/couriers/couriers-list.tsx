import { ArrowPath, DocumentText } from "@medusajs/icons"
import { Badge, Button, Container, Heading, StatusBadge, Text, toast } from "@medusajs/ui"
import { useState } from "react"
import { SingleColumnPage } from "../../components/layout/pages"
import { CourierCredentialModal } from "./components/courier-credential-modal"
import { CourierLogo } from "./components/courier-logo"
import { CourierZonesModal } from "./components/courier-zones-modal"
import {
  CourierProvider,
  useCouriers,
  useProbeCourier,
  useUpdateCourierRates,
} from "../../hooks/api/couriers"

const INITIAL_COURIERS: CourierProvider[] = [
  {
    id: "pathao",
    name: "Pathao Courier",
    status: "not_configured",
    is_enabled: true,
    webhookUrl: "https://api.vendryx.com/hooks/courier/pathao",
    rates: {
      insideDhaka: 60,
      subDhaka: 100,
      outsideDhaka: 130,
    },
    credentials: {},
    fields: [
      { key: "client_id", label: "Client ID", kind: "text", required: true, placeholder: "From Pathao Merchant Portal" },
      { key: "client_secret", label: "Client Secret", kind: "secret", required: true },
      { key: "username", label: "Merchant Email / Username", kind: "text", required: true },
      { key: "password", label: "Password", kind: "secret", required: true },
      { key: "store_id", label: "Store ID", kind: "text", required: true, placeholder: "e.g. 10243" },
      { key: "sandbox_mode", label: "Sandbox Test Mode", kind: "bool", required: false },
    ],
  },
  {
    id: "steadfast",
    name: "Steadfast Courier",
    status: "not_configured",
    is_enabled: true,
    webhookUrl: "https://api.vendryx.com/hooks/courier/steadfast",
    rates: {
      insideDhaka: 70,
      subDhaka: 100,
      outsideDhaka: 130,
    },
    credentials: {},
    fields: [
      { key: "api_key", label: "API Key", kind: "secret", required: true, placeholder: "From Steadfast Merchant Portal" },
      { key: "secret_key", label: "Secret Key", kind: "secret", required: true },
    ],
  },
  {
    id: "redx",
    name: "RedX Logistics",
    status: "not_configured",
    is_enabled: false,
    webhookUrl: "https://api.vendryx.com/hooks/courier/redx",
    rates: {
      insideDhaka: 60,
      subDhaka: 100,
      outsideDhaka: 140,
    },
    credentials: {},
    fields: [
      { key: "api_key", label: "Bearer Token / API Key", kind: "secret", required: true, placeholder: "RedX Authorization Token" },
      { key: "sandbox_mode", label: "Sandbox Test Mode", kind: "bool", required: false },
    ],
  },
]

export const CouriersList = () => {
  const { data: couriersData, refetch, isRefetching } = useCouriers()
  const couriers = couriersData && couriersData.length > 0 ? couriersData : INITIAL_COURIERS
  const probeMutation = useProbeCourier()
  const updateRatesMutation = useUpdateCourierRates()

  const [selectedForCredentials, setSelectedForCredentials] = useState<CourierProvider | null>(null)
  const [selectedForZones, setSelectedForZones] = useState<CourierProvider | null>(null)
  const [probingId, setProbingId] = useState<string | null>(null)

  const handleSaveRates = async (
    id: string,
    rates: { insideDhaka: number; subDhaka: number; outsideDhaka: number }
  ) => {
    try {
      await updateRatesMutation.mutateAsync({ id, rates })
      toast.success("Zone delivery charges updated successfully")
    } catch (err: any) {
      toast.error(err.message || "Failed to update rates")
    }
  }

  const handleQuickProbe = async (courier: CourierProvider) => {
    setProbingId(courier.id)
    try {
      const res = await probeMutation.mutateAsync(courier.id)
      if (res.verified) {
        toast.success(`Handshake probe successful (${res.latency_ms}ms): ${courier.name} API gateway responded.`)
      } else {
        toast.error(res.message || `${courier.name} probe failed`)
      }
    } catch (err: any) {
      toast.error(err.message || `Failed to probe ${courier.name}`)
    } finally {
      setProbingId(null)
    }
  }

  const handleCopyWebhook = (url: string) => {
    navigator.clipboard.writeText(url)
    toast.success("Webhook callback URL copied to clipboard")
  }

  const renderStatusBadge = (status: CourierProvider["status"]) => {
    switch (status) {
      case "connected":
        return <StatusBadge color="green">Connected</StatusBadge>
      case "needs_verification":
        return <StatusBadge color="orange">Needs Verification</StatusBadge>
      case "failed":
        return <StatusBadge color="red">Failed</StatusBadge>
      default:
        return <StatusBadge color="grey">Not Configured</StatusBadge>
    }
  }

  return (
    <SingleColumnPage
      widgets={{
        after: [],
        before: [],
      }}
    >
      <Container className="p-0 border border-ui-border-base rounded-2xl overflow-hidden shadow-xs bg-ui-bg-base">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between border-b border-ui-border-base p-6 gap-4 bg-ui-bg-subtle/50">
          <div className="flex flex-col gap-y-1">
            <div className="flex items-center gap-x-3">
              <Heading level="h1" className="text-xl font-bold tracking-tight text-ui-fg-base">
                Courier Logistics
              </Heading>
              <Badge size="small" color="blue">
                {couriers.length} services
              </Badge>
            </div>
            <Text size="small" className="text-ui-fg-subtle">
              Manage automated consignment booking, delivery-charge zone tiers, and real-time tracking webhooks across Bangladesh.
            </Text>
          </div>

          <Button
            size="small"
            variant="secondary"
            isLoading={isRefetching}
            onClick={() => refetch().then(() => toast.success("Refreshed courier service status"))}
          >
            <ArrowPath className={`mr-1 ${isRefetching ? "animate-spin" : ""}`} />
            Refresh
          </Button>
        </div>

        {/* Courier Cards List */}
        <div className="p-6">
          <div className="grid grid-cols-1 gap-5">
            {couriers.map((c) => {
              const isConnected = c.status === "connected"
              const isProbing = probingId === c.id

              return (
                <div
                  key={c.id}
                  className="flex flex-col md:flex-row md:items-center justify-between rounded-xl border border-ui-border-base bg-ui-bg-subtle/30 dark:bg-ui-bg-base p-5 shadow-2xs transition-all hover:border-ui-border-interactive/60 hover:shadow-xs gap-5"
                >
                  {/* Left Identity & Info */}
                  <div className="flex items-start gap-x-4 flex-1">
                    <CourierLogo id={c.id} size="large" />
                    <div className="flex flex-col gap-y-1.5 flex-1">
                      <div className="flex items-center gap-x-3">
                        <Text size="base" weight="plus" className="text-ui-fg-base font-semibold">
                          {c.name}
                        </Text>
                        <code className="text-xs text-ui-fg-muted font-mono">
                          {c.id}
                        </code>
                        {renderStatusBadge(c.status)}
                      </div>

                      {/* Zone Charges Summary */}
                      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-ui-fg-subtle">
                        <span>
                          Inside Dhaka: <strong className="text-ui-fg-base font-semibold">৳{c.rates.insideDhaka}</strong>
                        </span>
                        <span className="text-ui-fg-muted">·</span>
                        <span>
                          Suburbs: <strong className="text-ui-fg-base font-semibold">৳{c.rates.subDhaka}</strong>
                        </span>
                        <span className="text-ui-fg-muted">·</span>
                        <span>
                          Outside Dhaka: <strong className="text-ui-fg-base font-semibold">৳{c.rates.outsideDhaka}</strong>
                        </span>
                      </div>

                      {/* Webhook bar */}
                      <div className="flex items-center gap-x-2 pt-1 text-xs">
                        <span className="text-ui-fg-muted font-mono truncate max-w-sm">
                          Webhook: {c.webhookUrl}
                        </span>
                        <button
                          type="button"
                          onClick={() => handleCopyWebhook(c.webhookUrl)}
                          className="text-ui-fg-interactive hover:text-ui-fg-interactive-hover font-medium text-[11px] flex items-center gap-x-0.5"
                          title="Copy webhook URL"
                        >
                          <DocumentText className="h-3 w-3" />
                          Copy
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Right Actions */}
                  <div className="flex items-center gap-x-2 shrink-0 self-end md:self-center">
                    <Button
                      size="small"
                      variant="secondary"
                      onClick={() => handleQuickProbe(c)}
                      isLoading={isProbing}
                      title="Test API connectivity"
                    >
                      Probe
                    </Button>
                    <Button
                      size="small"
                      variant="secondary"
                      onClick={() => setSelectedForZones(c)}
                    >
                      Zone Rates
                    </Button>
                    <Button
                      size="small"
                      variant={isConnected ? "secondary" : "primary"}
                      onClick={() => setSelectedForCredentials(c)}
                    >
                      {isConnected ? "Update Keys" : "Connect"}
                    </Button>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      </Container>

      {/* Credentials Modal */}
      <CourierCredentialModal
        courier={selectedForCredentials}
        open={!!selectedForCredentials}
        onClose={() => setSelectedForCredentials(null)}
        onSave={() => refetch()}
      />

      {/* Zone Rates Modal */}
      <CourierZonesModal
        courier={selectedForZones}
        open={!!selectedForZones}
        onClose={() => setSelectedForZones(null)}
        onSaveRates={handleSaveRates}
      />
    </SingleColumnPage>
  )
}
