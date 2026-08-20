import { ArrowPath, CheckCircle, ShieldCheck, XCircle } from "@medusajs/icons"
import { Badge, Button, Container, Heading, StatusBadge, Table, Text } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { SingleColumnPage } from "../../components/layout/pages"
import { backendUrl, sdk } from "../../lib/client"

// See analytics-dashboard: @medusajs/ui omits colSpan from Table.Cell's prop type.
const spanAllColumns = { colSpan: 4 } as React.TdHTMLAttributes<HTMLTableCellElement>

interface QuotaItem {
  key: string
  /** Absent means unlimited — the server omits the field rather than sending 0. */
  limit?: number
  used: number
  /** `ok` or `grace`; `grace` is the grandfathered over-limit state. */
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

interface FeatureFlagsData {
  feature_flags: Record<string, boolean>
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

const humanise = (key: string) => key.replace(/_/g, " ")

const formatDate = (iso: string | null) =>
  iso ? new Date(iso).toLocaleDateString(undefined, { dateStyle: "medium" }) : null

export const EntitlementsDetail = () => {
  const {
    data: entitlements,
    isLoading: isEntitlementsLoading,
    refetch: refetchEntitlements,
  } = useQuery<EntitlementsData>({
    queryKey: ["admin", "entitlements"],
    // Through the SDK, not a bare same-origin fetch: the dashboard is served from its own
    // origin in development, where a relative /admin/entitlements resolves to the Vite
    // dev server and this whole page reads as "no plan".
    queryFn: () => sdk.client.fetch<EntitlementsData>("/admin/entitlements"),
  })

  // Which screens this build ships at all, as opposed to what the plan allows. The two
  // are different questions and the page says so rather than merging them into one list.
  const { data: flags, refetch: refetchFlags } = useQuery<FeatureFlagsData>({
    queryKey: ["admin", "feature-flags"],
    queryFn: () => sdk.client.fetch<FeatureFlagsData>("/admin/feature-flags"),
  })

  const { data: health, refetch: refetchHealth } = useQuery<HealthData>({
    queryKey: ["health"],
    queryFn: async () => {
      // /health is deliberately outside the tenant layer and outside the SDK's /admin
      // prefix, so it is fetched directly against the same backend origin.
      const res = await fetch(`${backendUrl.replace(/\/$/, "")}/health`)
      if (!res.ok) throw new Error("Failed to fetch health status")
      return res.json()
    },
  })

  const handleRefresh = () => {
    refetchEntitlements()
    refetchFlags()
    refetchHealth()
  }

  const tier = entitlements?.tier ?? "—"
  const isHealthy = health?.status === "healthy"
  const trialEnds = formatDate(entitlements?.trial_ends_at ?? null)
  const termEnds = formatDate(entitlements?.term_ends_at ?? null)
  const inGrace = entitlements?.quotas?.some((q) => q.state === "grace") ?? false

  // Everything the engine can gate, on or off. Showing only what is enabled hides the
  // upgrade path, which is the one thing a merchant on a limited plan wants to see.
  const gateable = Object.entries(flags?.feature_flags ?? {})
    .filter(([, enabled]) => typeof enabled === "boolean")
    .sort(([a], [b]) => a.localeCompare(b))

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
                Plan &amp; Entitlements
              </Heading>
              <Badge size="small" color={tier === "pro" ? "purple" : "blue"} className="uppercase">
                {tier}
              </Badge>
            </div>
            <Text size="small" className="text-ui-fg-subtle mt-1">
              What this store's plan includes, how much of it is in use, and the state of
              the engine serving it.
            </Text>
          </div>
          <Button variant="secondary" size="small" onClick={handleRefresh}>
            <ArrowPath className="text-ui-fg-subtle me-1.5" />
            Refresh
          </Button>
        </div>

        {/* Grace banner (ENTITLEMENTS.md 6.4): existing data stays live, creates stop. */}
        {inGrace && (
          <Container className="border-ui-tag-orange-border bg-ui-tag-orange-bg p-4">
            <Text size="small" weight="plus" className="text-ui-tag-orange-text">
              This store is over one of its plan limits.
            </Text>
            <Text size="small" className="text-ui-tag-orange-text mt-1">
              Nothing has been removed and nothing has been hidden — everything you already
              have stays live and editable. Creating more of the resource below is blocked
              until you upgrade or delete some.
            </Text>
          </Container>
        )}

        {/* Engine health */}
        <Container className="p-6">
          <div className="border-ui-border-base flex items-center justify-between border-b pb-4">
            <div className="flex items-center gap-x-3">
              <div className="bg-ui-bg-subtle text-ui-fg-subtle flex h-10 w-10 items-center justify-center rounded-lg">
                <ShieldCheck className="text-ui-tag-green-icon h-6 w-6" />
              </div>
              <div>
                <Heading level="h2" className="text-ui-fg-base text-base">
                  {health?.engine || "Stheno Commerce Engine"}
                </Heading>
                <Text size="small" className="text-ui-fg-subtle">
                  {health?.architecture || "Modular Monolith (Rust / Axum)"} · v
                  {health?.version || "0.1.0"}
                </Text>
              </div>
            </div>
            <StatusBadge color={isHealthy ? "green" : "red"}>
              {isHealthy ? "Engine Operational" : "Degraded"}
            </StatusBadge>
          </div>

          <div className="grid grid-cols-1 gap-6 pt-5 md:grid-cols-3">
            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Database Pool
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {health?.database.pool_available ?? 0} / {health?.database.pool_max_size ?? 0}{" "}
                connections free
              </Text>
              <div className="mt-2 flex items-center gap-x-2">
                <div className="bg-ui-bg-base-hover h-1.5 flex-1 overflow-hidden rounded-full">
                  <div
                    className="bg-ui-tag-green-icon h-full rounded-full"
                    style={{
                      width: `${Math.min(
                        100,
                        Math.round((health?.database.pool_saturation || 0) * 100)
                      )}%`,
                    }}
                  />
                </div>
                <Text size="xsmall" className="text-ui-fg-muted font-mono">
                  {Math.round((health?.database.pool_saturation || 0) * 100)}%
                </Text>
              </div>
            </div>

            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Trial
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {trialEnds ? `Ends ${trialEnds}` : "Not on trial"}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                When a trial ends nothing is deleted — the store moves to the free plan and
                keeps everything it has.
              </Text>
            </div>

            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Term
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {termEnds ? `Renews ${termEnds}` : "No fixed term"}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                {health?.compatibility || "Commerce REST v2"} API
              </Text>
            </div>
          </div>
        </Container>

        {/* Quotas & capacity */}
        <Container className="overflow-hidden p-0">
          <div className="border-ui-border-base border-b p-6">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Limits &amp; usage
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Enforced across catalogue size, staff seats, and how far back reporting looks.
            </Text>
          </div>

          <Table>
            <Table.Header>
              <Table.Row>
                <Table.HeaderCell>Resource</Table.HeaderCell>
                <Table.HeaderCell>Used</Table.HeaderCell>
                <Table.HeaderCell>Limit</Table.HeaderCell>
                <Table.HeaderCell>State</Table.HeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {isEntitlementsLoading ? (
                <Table.Row>
                  <Table.Cell {...spanAllColumns} className="text-ui-fg-muted py-6 text-center">
                    Loading limits…
                  </Table.Cell>
                </Table.Row>
              ) : (
                <>
                  {(entitlements?.quotas ?? []).map((q) => (
                    <Table.Row key={q.key}>
                      <Table.Cell className="text-ui-fg-base font-medium capitalize">
                        {humanise(q.key)}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-base font-mono font-semibold">
                        {q.used}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-subtle font-mono">
                        {q.limit !== undefined ? q.limit : "Unlimited"}
                      </Table.Cell>
                      <Table.Cell>
                        <Badge size="xsmall" color={q.state === "grace" ? "orange" : "green"}>
                          {q.state === "grace" ? "Over limit" : "Within limit"}
                        </Badge>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                  {(entitlements?.windows ?? []).map((w) => (
                    <Table.Row key={w.key}>
                      <Table.Cell className="text-ui-fg-base font-medium capitalize">
                        {humanise(w.key)}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-muted font-mono">—</Table.Cell>
                      <Table.Cell className="text-ui-fg-subtle font-mono">
                        {w.limit !== undefined ? `${w.limit} days` : "Unlimited"}
                      </Table.Cell>
                      <Table.Cell>
                        <Badge size="xsmall" color="green">
                          Within limit
                        </Badge>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </>
              )}
            </Table.Body>
          </Table>
        </Container>

        {/* Feature matrix: on AND off, so the upgrade path is visible. */}
        <Container className="p-6">
          <div className="border-ui-border-base border-b pb-4">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Features
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              A feature that is off is not installed for this store — its screens and routes
              are not mounted at all.
            </Text>
          </div>

          <div className="grid grid-cols-1 gap-3 pt-5 sm:grid-cols-2 md:grid-cols-3">
            {gateable.map(([feature, enabled]) => (
              <div
                key={feature}
                className="border-ui-border-base bg-ui-bg-subtle flex items-center gap-x-2.5 rounded-lg border p-3"
              >
                {enabled ? (
                  <CheckCircle className="text-ui-tag-green-icon shrink-0" />
                ) : (
                  <XCircle className="text-ui-fg-muted shrink-0" />
                )}
                <Text
                  size="small"
                  weight="plus"
                  className={
                    enabled ? "text-ui-fg-base capitalize" : "text-ui-fg-muted capitalize"
                  }
                >
                  {humanise(feature)}
                </Text>
              </div>
            ))}
          </div>
        </Container>
      </div>
    </SingleColumnPage>
  )
}
