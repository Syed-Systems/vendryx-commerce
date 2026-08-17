import {
  ArrowPath,
  CheckCircle,
  Clock,
  Key,
  ShieldCheck,
  Sparkles,
  SquaresPlus,
  Users,
} from "@medusajs/icons"
import { Badge, Button, Container, Heading, StatusBadge, Table, Text, clx } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useTranslation } from "react-i18next"
import { SingleColumnPage } from "../../components/layout/pages"

interface QuotaItem {
  key: string
  limit?: number
  used: number
  state: string
}

interface WindowItem {
  key: string
  limit?: number
}

interface EntitlementsData {
  tier: string
  trial_ends_at: string | null
  term_ends_at: string | null
  features: string[]
  quotas: QuotaItem[]
  windows: WindowItem[]
}

interface HealthData {
  status: string
  engine: string
  version: string
  architecture: string
  compatibility: string
  database: {
    pool_size: number
    pool_max_size: number
    pool_available: number
    pool_saturation: number
  }
}

export const EntitlementsDetail = () => {
  const { t } = useTranslation()

  const {
    data: entitlements,
    isLoading: isEntitlementsLoading,
    refetch: refetchEntitlements,
  } = useQuery<EntitlementsData>({
    queryKey: ["admin", "entitlements"],
    queryFn: async () => {
      const res = await fetch("/admin/entitlements")
      if (!res.ok) throw new Error("Failed to fetch entitlements")
      return res.json()
    },
  })

  const {
    data: health,
    isLoading: isHealthLoading,
    refetch: refetchHealth,
  } = useQuery<HealthData>({
    queryKey: ["health"],
    queryFn: async () => {
      const res = await fetch("/health")
      if (!res.ok) throw new Error("Failed to fetch health status")
      return res.json()
    },
  })

  const handleRefresh = () => {
    refetchEntitlements()
    refetchHealth()
  }

  const tier = entitlements?.tier?.toUpperCase() || "COMMUNITY"
  const isHealthy = health?.status === "healthy"

  return (
    <SingleColumnPage
      widgets={{
        after: [],
        before: [],
      }}
    >
      <div className="flex flex-col gap-y-6">
        {/* Header */}
        <div className="flex flex-col gap-y-4 md:flex-row md:items-center md:justify-between">
          <div>
            <div className="flex items-center gap-x-2.5">
              <Heading level="h1" className="text-ui-fg-base">
                Engine Platform & Entitlements
              </Heading>
              <Badge size="small" color={tier === "ENTERPRISE" ? "purple" : "blue"}>
                {tier} TIER
              </Badge>
            </div>
            <Text size="small" className="text-ui-fg-subtle mt-1">
              Active plan quotas, hardware engine telemetry, and mounted domain capabilities.
            </Text>
          </div>
          <Button variant="secondary" size="small" onClick={handleRefresh}>
            <ArrowPath className="text-ui-fg-subtle me-1.5" />
            Refresh State
          </Button>
        </div>

        {/* Engine Health & Core Infrastructure Card */}
        <Container className="p-6">
          <div className="flex items-center justify-between pb-4 border-b border-ui-border-base">
            <div className="flex items-center gap-x-3">
              <div className="bg-ui-bg-subtle text-ui-fg-subtle flex h-10 w-10 items-center justify-center rounded-lg">
                <ShieldCheck className="h-6 w-6 text-ui-tag-green-icon" />
              </div>
              <div>
                <Heading level="h2" className="text-ui-fg-base text-base">
                  {health?.engine || "Stheno Commerce Engine"}
                </Heading>
                <Text size="small" className="text-ui-fg-subtle">
                  {health?.architecture || "Modular Monolith (Rust / Axum)"} · v{health?.version || "0.1.0"}
                </Text>
              </div>
            </div>
            <StatusBadge color={isHealthy ? "green" : "red"}>
              {isHealthy ? "Engine Operational" : "Degraded"}
            </StatusBadge>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 pt-5">
            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Database Pool
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                PostgreSQL Connection Pool ({health?.database.pool_available || 10} / {health?.database.pool_max_size || 10} avail)
              </Text>
              <div className="mt-2 flex items-center gap-x-2">
                <div className="h-1.5 flex-1 bg-ui-bg-base-hover rounded-full overflow-hidden">
                  <div
                    className="h-full bg-ui-tag-green-icon rounded-full"
                    style={{ width: `${Math.min(100, Math.round((health?.database.pool_saturation || 0) * 100))}%` }}
                  />
                </div>
                <Text size="xsmall" className="text-ui-fg-muted font-mono">
                  {Math.round((health?.database.pool_saturation || 0) * 100)}%
                </Text>
              </div>
            </div>

            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                API Compatibility
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {health?.compatibility || "Medusa v2 Strict Parity"}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                Drop-in compatible with standard Medusa Storefronts and SDKs
              </Text>
            </div>

            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Subscription Status
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                Active Lifetime / Self-Hosted
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                Zero telemetry leakage · Native compiled execution
              </Text>
            </div>
          </div>
        </Container>

        {/* Quotas & Capacity Limits */}
        <Container className="p-0 overflow-hidden">
          <div className="p-6 border-b border-ui-border-base">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Resource Quotas & Utilization
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Enforced limits across catalog scale, staff permissions, and analytical data retention.
            </Text>
          </div>

          <Table>
            <Table.Header>
              <Table.Row>
                <Table.HeaderCell>Quota Resource</Table.HeaderCell>
                <Table.HeaderCell>Current Usage</Table.HeaderCell>
                <Table.HeaderCell>Capacity Limit</Table.HeaderCell>
                <Table.HeaderCell>Status State</Table.HeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {isEntitlementsLoading ? (
                <Table.Row>
                  <Table.Cell colSpan={4} className="text-center py-6 text-ui-fg-muted">
                    Loading quotas...
                  </Table.Cell>
                </Table.Row>
              ) : entitlements?.quotas?.length === 0 ? (
                <Table.Row>
                  <Table.Cell colSpan={4} className="text-center py-6 text-ui-fg-muted">
                    No resource quotas configured (Unlimited).
                  </Table.Cell>
                </Table.Row>
              ) : (
                entitlements?.quotas.map((q) => (
                  <Table.Row key={q.key}>
                    <Table.Cell className="font-medium text-ui-fg-base capitalize">
                      {q.key.replace(/_/g, " ")}
                    </Table.Cell>
                    <Table.Cell className="font-mono text-ui-fg-base font-semibold">
                      {q.used}
                    </Table.Cell>
                    <Table.Cell className="font-mono text-ui-fg-subtle">
                      {q.limit !== undefined ? q.limit : "Unlimited"}
                    </Table.Cell>
                    <Table.Cell>
                      <Badge
                        size="xsmall"
                        color={q.state === "WithinQuota" ? "green" : q.state === "Grace" ? "orange" : "red"}
                      >
                        {q.state}
                      </Badge>
                    </Table.Cell>
                  </Table.Row>
                ))
              )}
            </Table.Body>
          </Table>
        </Container>

        {/* Enabled Features Matrix */}
        <Container className="p-6">
          <div className="pb-4 border-b border-ui-border-base">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Mounted Features & Extensions
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Modules and functional capabilities available in your current license tier.
            </Text>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3 pt-5">
            {(entitlements?.features || []).map((feat) => (
              <div
                key={feat}
                className="flex items-center gap-x-2.5 p-3 rounded-lg border border-ui-border-base bg-ui-bg-subtle"
              >
                <CheckCircle className="text-ui-tag-green-icon shrink-0" />
                <Text size="small" weight="plus" className="text-ui-fg-base capitalize">
                  {feat.replace(/_/g, " ")}
                </Text>
              </div>
            ))}
          </div>
        </Container>
      </div>
    </SingleColumnPage>
  )
}
