import { ArrowPath, CheckCircle, XCircle } from "@medusajs/icons"
import { Badge, Button, Container, Heading, Table, Text } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { SingleColumnPage } from "../../components/layout/pages"
import { sdk } from "../../lib/client"

const spanAllColumns = { colSpan: 4 } as React.TdHTMLAttributes<HTMLTableCellElement>

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
  quotas: QuotaItem[]
  windows: WindowItem[]
}

interface FeatureFlagsData {
  feature_flags: Record<string, boolean>
}

const humanise = (key: string) => key.replace(/_/g, " ")
const formatDate = (iso: string | null) =>
  iso ? new Date(iso).toLocaleDateString(undefined, { dateStyle: "medium" }) : null

export const EntitlementsDetail = () => {
  const {
    data: entitlements,
    isLoading: isEntitlementsLoading,
    isError: isEntitlementsError,
    refetch: refetchEntitlements,
  } = useQuery<EntitlementsData>({
    queryKey: ["admin", "entitlements"],
    queryFn: () => sdk.client.fetch<EntitlementsData>("/admin/entitlements"),
  })
  const {
    data: flags,
    isError: isFlagsError,
    refetch: refetchFlags,
  } = useQuery<FeatureFlagsData>({
    queryKey: ["admin", "feature-flags"],
    queryFn: () => sdk.client.fetch<FeatureFlagsData>("/admin/feature-flags"),
  })

  const tier = entitlements?.tier ?? "Not available"
  const trialEnds = formatDate(entitlements?.trial_ends_at ?? null)
  const termEnds = formatDate(entitlements?.term_ends_at ?? null)
  const inGrace = entitlements?.quotas?.some((q) => q.state === "grace") ?? false
  const gateable = Object.entries(flags?.feature_flags ?? {})
    .filter(([, enabled]) => typeof enabled === "boolean")
    .sort(([a], [b]) => a.localeCompare(b))

  return (
    <SingleColumnPage widgets={{ after: [], before: [] }}>
      <div className="flex flex-col gap-y-6">
        <div className="flex flex-col gap-y-4 md:flex-row md:items-center md:justify-between">
          <div>
            <Text size="xsmall" weight="plus" className="text-ui-fg-muted">
              Platform settings
            </Text>
            <div className="mt-1 flex items-center gap-x-2.5">
              <Heading level="h1" className="text-ui-fg-base">
                Plan &amp; access
              </Heading>
              <Badge size="small" color={tier === "pro" ? "green" : "blue"} className="uppercase">
                {tier}
              </Badge>
            </div>
            <Text size="small" className="text-ui-fg-subtle mt-1">
              Review this store&apos;s effective plan, enforced limits, and available services.
            </Text>
          </div>
          <Button
            variant="secondary"
            size="small"
            onClick={() => {
              refetchEntitlements()
              refetchFlags()
            }}
            aria-label="Refresh plan and access details"
          >
            <ArrowPath className="text-ui-fg-subtle me-1.5" />
            Refresh
          </Button>
        </div>

        {(isEntitlementsError || isFlagsError) && (
          <Container role="alert" className="border-ui-tag-red-border bg-ui-tag-red-bg p-4">
            <Text size="small" weight="plus" className="text-ui-tag-red-text">
              Some plan details could not be loaded.
            </Text>
            <Text size="small" className="text-ui-tag-red-text mt-1">
              Refresh to try again. No store settings have been changed.
            </Text>
          </Container>
        )}

        {inGrace && (
          <Container className="border-ui-tag-orange-border bg-ui-tag-orange-bg p-4">
            <Text size="small" weight="plus" className="text-ui-tag-orange-text">
              This store is over one of its plan limits.
            </Text>
            <Text size="small" className="text-ui-tag-orange-text mt-1">
              Existing work stays live and editable. New items in the affected resource are
              blocked until the limit is resolved.
            </Text>
          </Container>
        )}

        <Container className="p-6">
          <Heading level="h2" className="text-ui-fg-base text-base">
            Current plan
          </Heading>
          <Text size="small" className="text-ui-fg-subtle mt-1">
            The values below come from the active platform subscription, not browser settings.
          </Text>
          <div className="grid grid-cols-1 gap-6 pt-5 md:grid-cols-3">
            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Plan
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1 capitalize">
                {tier}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                The plan currently applied to this store.
              </Text>
            </div>
            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Trial
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {trialEnds ? `Ends ${trialEnds}` : "Not on trial"}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                A completed trial moves to Free without removing existing store data.
              </Text>
            </div>
            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Commitment
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {termEnds ? `Renews ${termEnds}` : "No fixed term"}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                Contact the platform team for billing changes.
              </Text>
            </div>
          </div>
        </Container>

        <Container className="overflow-hidden p-0">
          <div className="border-ui-border-base border-b p-6">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Limits &amp; usage
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Limits apply to new work. Existing products and staff stay available after a plan
              change.
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
                    Loading limits...
                  </Table.Cell>
                </Table.Row>
              ) : (
                <>
                  {(entitlements?.quotas ?? []).map((quota) => (
                    <Table.Row key={quota.key}>
                      <Table.Cell className="text-ui-fg-base font-medium capitalize">
                        {humanise(quota.key)}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-base font-mono font-semibold">
                        {quota.used}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-subtle font-mono">
                        {quota.limit !== undefined ? quota.limit : "Unlimited"}
                      </Table.Cell>
                      <Table.Cell>
                        <Badge size="xsmall" color={quota.state === "grace" ? "orange" : "green"}>
                          {quota.state === "grace" ? "Over limit" : "Within limit"}
                        </Badge>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                  {(entitlements?.windows ?? []).map((window) => (
                    <Table.Row key={window.key}>
                      <Table.Cell className="text-ui-fg-base font-medium capitalize">
                        {humanise(window.key)}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-muted">Not measured</Table.Cell>
                      <Table.Cell className="text-ui-fg-subtle font-mono">
                        {window.limit !== undefined ? `${window.limit} days` : "Unlimited"}
                      </Table.Cell>
                      <Table.Cell>
                        <Badge size="xsmall" color="green">
                          Included
                        </Badge>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </>
              )}
            </Table.Body>
          </Table>
        </Container>

        <Container className="p-6">
          <div className="border-ui-border-base border-b pb-4">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Services available to this store
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Included services are ready to use. Services outside the plan return not found at
              the server, even if a browser link is saved.
            </Text>
          </div>
          <div className="grid grid-cols-1 gap-3 pt-5 sm:grid-cols-2 md:grid-cols-3">
            {gateable.map(([feature, enabled]) => (
              <div
                key={feature}
                className="border-ui-border-base bg-ui-bg-subtle flex items-center gap-x-2.5 rounded-lg border p-3"
              >
                {enabled ? (
                  <CheckCircle className="text-ui-tag-green-icon shrink-0" aria-hidden="true" />
                ) : (
                  <XCircle className="text-ui-fg-muted shrink-0" aria-hidden="true" />
                )}
                <Text
                  size="small"
                  weight="plus"
                  className={enabled ? "text-ui-fg-base capitalize" : "text-ui-fg-muted capitalize"}
                >
                  {humanise(feature)}
                </Text>
                <Badge size="xsmall" color={enabled ? "green" : "grey"} className="ms-auto">
                  {enabled ? "Included" : "Not in plan"}
                </Badge>
              </div>
            ))}
          </div>
        </Container>
      </div>
    </SingleColumnPage>
  )
}
