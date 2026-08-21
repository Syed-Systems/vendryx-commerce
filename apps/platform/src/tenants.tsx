import {
  ArrowPath,
  BuildingStorefront,
  CurrencyDollar,
  CheckCircle,
  ExclamationCircle,
  Trash,
  Plus,
  ArrowDownTray,
} from "@medusajs/icons"
import {
  Badge,
  Button,
  Container,
  Drawer,
  Heading,
  Input,
  Label,
  Select,
  StatusBadge,
  Table,
  Text,
  toast,
} from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"

import {
  addTenantDomain,
  closeTenant,
  createTenant,
  exportTenantData,
  getDomainVerification,
  getTenantHealth,
  getTenantOnboarding,
  listTenantDomains,
  listTenants,
  listTiers,
  purgeClosedTenants,
  reactivateTenant,
  removeTenantDomain,
  setPrimaryDomain,
  setTier,
  suspendTenant,
  type DomainVerification,
  type MerchantHealth,
  type OnboardingState,
  type Tenant,
  type TenantDomain,
} from "./api"
import { SubscriptionDrawer } from "./subscription"

function useTierNames() {
  const { data } = useQuery({
    queryKey: ["platform", "tiers"],
    queryFn: listTiers,
    retry: false,
  })
  return data?.tiers.map((t) => t.tier) ?? []
}

function CreateTenantForm({ onCreated }: { onCreated: () => void }) {
  const tiers = useTierNames()
  const [open, setOpen] = useState(false)
  const [form, setForm] = useState({ id: "", name: "", tier: "free", domain: "" })

  const mutation = useMutation({
    mutationFn: () =>
      createTenant({
        id: form.id.trim(),
        name: form.name.trim(),
        tier: form.tier,
        domain: form.domain.trim() || undefined,
      }),
    onSuccess: () => {
      toast.success(`Created tenant ${form.name}`)
      setForm({ id: "", name: "", tier: "free", domain: "" })
      setOpen(false)
      onCreated()
    },
    onError: (error: Error) => toast.error(error.message),
  })

  if (!open) {
    return (
      <div className="flex justify-between items-center">
        <Button size="small" onClick={() => setOpen(true)}>
          New merchant
        </Button>
      </div>
    )
  }

  return (
    <Container className="p-6 border border-ui-border-base">
      <Heading level="h2" className="mb-4">
        New merchant
      </Heading>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Tenant ID</Label>
          <Input
            value={form.id}
            placeholder="acme_corp"
            onChange={(e) => setForm({ ...form, id: e.target.value })}
          />
        </div>
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Store name</Label>
          <Input
            value={form.name}
            placeholder="Acme Corporation"
            onChange={(e) => setForm({ ...form, name: e.target.value })}
          />
        </div>
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Plan</Label>
          <Select value={form.tier} onValueChange={(tier) => setForm({ ...form, tier })}>
            <Select.Trigger>
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              {tiers.map((t) => (
                <Select.Item key={t} value={t}>
                  {t}
                </Select.Item>
              ))}
            </Select.Content>
          </Select>
        </div>
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Domain</Label>
          <Input
            value={form.domain}
            placeholder="acme.localhost"
            onChange={(e) => setForm({ ...form, domain: e.target.value })}
          />
        </div>
      </div>
      <div className="mt-4 flex gap-x-2">
        <Button
          size="small"
          isLoading={mutation.isPending}
          disabled={!form.id.trim() || !form.name.trim()}
          onClick={() => mutation.mutate()}
        >
          Create
        </Button>
        <Button variant="secondary" size="small" onClick={() => setOpen(false)}>
          Cancel
        </Button>
      </div>
    </Container>
  )
}

function TenantDomainDrawer({
  tenant,
  open,
  onOpenChange,
}: {
  tenant: Tenant
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()
  const [newDomain, setNewDomain] = useState("")
  const [isPrimary, setIsPrimary] = useState(false)

  const { data, isLoading } = useQuery({
    queryKey: ["platform", "tenants", tenant.id, "domains"],
    queryFn: () => listTenantDomains(tenant.id),
    enabled: open,
  })

  const { data: healthData } = useQuery({
    queryKey: ["platform", "tenants", tenant.id, "health"],
    queryFn: () => getTenantHealth(tenant.id),
    enabled: open,
  })

  const { data: onboardingData } = useQuery({
    queryKey: ["platform", "tenants", tenant.id, "onboarding"],
    queryFn: () => getTenantOnboarding(tenant.id),
    enabled: open,
  })

  const invalidate = () => {
    queryClient.invalidateQueries({
      queryKey: ["platform", "tenants", tenant.id, "domains"],
    })
  }

  const addMutation = useMutation({
    mutationFn: () => addTenantDomain(tenant.id, newDomain.trim(), isPrimary),
    onSuccess: () => {
      toast.success(`Added domain ${newDomain}`)
      setNewDomain("")
      setIsPrimary(false)
      invalidate()
    },
    onError: (err: Error) => toast.error(err.message),
  })

  const removeMutation = useMutation({
    mutationFn: (domain: string) => removeTenantDomain(tenant.id, domain),
    onSuccess: () => {
      toast.success("Domain removed")
      invalidate()
    },
    onError: (err: Error) => toast.error(err.message),
  })

  const primaryMutation = useMutation({
    mutationFn: (domain: string) => setPrimaryDomain(tenant.id, domain),
    onSuccess: () => {
      toast.success("Primary domain updated")
      invalidate()
    },
    onError: (err: Error) => toast.error(err.message),
  })

  const exportMutation = useMutation({
    mutationFn: () => exportTenantData(tenant.id),
    onSuccess: (res) => {
      const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(res.export, null, 2))
      const downloadAnchor = document.createElement("a")
      downloadAnchor.setAttribute("href", dataStr)
      downloadAnchor.setAttribute("download", `tenant_${tenant.id}_export.json`)
      document.body.appendChild(downloadAnchor)
      downloadAnchor.click()
      downloadAnchor.remove()
      toast.success("Tenant data exported")
    },
    onError: (err: Error) => toast.error(err.message),
  })

  const domains = data?.domains ?? []
  const health = healthData?.health
  const onboarding = onboardingData?.onboarding

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <Drawer.Content className="w-full max-w-[540px]">
        <Drawer.Header>
          <Drawer.Title>{tenant.name} — Merchant Operations</Drawer.Title>
          <Drawer.Description>
            Custom domains, merchant health telemetry, and data compliance exports.
          </Drawer.Description>
        </Drawer.Header>

        <Drawer.Body className="flex flex-col gap-y-6 overflow-y-auto p-6">
          {/* Health & Telemetry */}
          {health && (
            <Container className="p-4 bg-ui-bg-subtle border border-ui-border-base">
              <div className="flex items-center justify-between mb-3">
                <Text size="small" weight="plus">
                  Merchant Health (7-day)
                </Text>
                <Badge
                  color={
                    health.health_verdict === "healthy"
                      ? "green"
                      : health.health_verdict === "critical"
                      ? "red"
                      : "blue"
                  }
                >
                  {health.health_verdict}
                </Badge>
              </div>
              <div className="grid grid-cols-2 gap-4 text-xs">
                <div>
                  <Text size="xsmall" className="text-ui-fg-muted">
                    7d Order Volume
                  </Text>
                  <Text size="small" weight="plus">
                    {health.orders_last_7d} orders
                  </Text>
                </div>
                <div>
                  <Text size="xsmall" className="text-ui-fg-muted">
                    7d GMV
                  </Text>
                  <Text size="small" weight="plus">
                    {health.gmv_last_7d.amount} {health.gmv_last_7d.currency}
                  </Text>
                </div>
              </div>
            </Container>
          )}

          {/* Onboarding Funnel */}
          {onboarding && (
            <Container className="p-4 bg-ui-bg-subtle border border-ui-border-base">
              <Text size="small" weight="plus" className="mb-2">
                Onboarding Funnel: <span className="capitalize">{onboarding.stage.replace(/_/g, " ")}</span>
              </Text>
              <div className="flex items-center gap-x-2 text-xs text-ui-fg-subtle">
                <Badge color={onboarding.created ? "green" : "grey"}>Created</Badge>
                <span>→</span>
                <Badge color={onboarding.product_added ? "green" : "grey"}>Products</Badge>
                <span>→</span>
                <Badge color={onboarding.payment_connected ? "green" : "grey"}>Domain/Gateway</Badge>
                <span>→</span>
                <Badge color={onboarding.first_order_received ? "green" : "grey"}>First Order</Badge>
              </div>
            </Container>
          )}

          {/* Custom Domains */}
          <div>
            <Heading level="h3" className="mb-2">
              Assigned Hostnames & Domains
            </Heading>
            <div className="flex gap-x-2 mb-4">
              <Input
                placeholder="store.custombrand.com"
                value={newDomain}
                onChange={(e) => setNewDomain(e.target.value)}
                size="small"
              />
              <Button
                size="small"
                onClick={() => addMutation.mutate()}
                isLoading={addMutation.isPending}
                disabled={!newDomain.trim()}
              >
                <Plus className="mr-1" /> Add
              </Button>
            </div>

            <div className="flex flex-col gap-y-2">
              {isLoading ? (
                <Text size="small" className="text-ui-fg-muted">
                  Loading domains...
                </Text>
              ) : domains.length === 0 ? (
                <Text size="small" className="text-ui-fg-muted">
                  No domains configured for this merchant.
                </Text>
              ) : (
                domains.map((d) => (
                  <div
                    key={d.domain}
                    className="flex items-center justify-between p-3 rounded-lg border border-ui-border-base bg-ui-bg-base"
                  >
                    <div>
                      <div className="flex items-center gap-x-2">
                        <span className="font-mono text-sm font-medium">{d.domain}</span>
                        {d.is_primary && <Badge color="purple">Primary</Badge>}
                        <Badge color="green">Verified (TLS active)</Badge>
                      </div>
                      <Text size="xsmall" className="text-ui-fg-muted mt-1">
                        Bound to tenant: {d.tenant_id}
                      </Text>
                    </div>
                    <div className="flex items-center gap-x-2">
                      {!d.is_primary && (
                        <Button
                          variant="secondary"
                          size="small"
                          onClick={() => primaryMutation.mutate(d.domain)}
                          isLoading={primaryMutation.isPending}
                        >
                          Make Primary
                        </Button>
                      )}
                      <Button
                        variant="danger"
                        size="small"
                        onClick={() => removeMutation.mutate(d.domain)}
                        isLoading={removeMutation.isPending}
                      >
                        <Trash />
                      </Button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          {/* Export Tenant Bundle */}
          <div className="pt-4 border-t border-ui-border-base">
            <Heading level="h3" className="mb-1">
              Data Privacy & Export
            </Heading>
            <Text size="small" className="text-ui-fg-subtle mb-3">
              Export an aggregated JSON bundle containing subscriptions, invoices, collection attempts, and host bindings.
            </Text>
            <Button
              variant="secondary"
              size="small"
              onClick={() => exportMutation.mutate()}
              isLoading={exportMutation.isPending}
            >
              <ArrowDownTray className="mr-1" /> Export Tenant Archive
            </Button>
          </div>
        </Drawer.Body>
      </Drawer.Content>
    </Drawer>
  )
}

function PurgeModal({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const queryClient = useQueryClient()
  const [retentionDays, setRetentionDays] = useState(30)

  const mutation = useMutation({
    mutationFn: () => purgeClosedTenants(retentionDays),
    onSuccess: (data) => {
      toast.success(`Purge complete: ${data.purge.purged_count} tenant(s) hard purged`)
      queryClient.invalidateQueries({ queryKey: ["platform", "tenants"] })
      onOpenChange(false)
    },
    onError: (err: Error) => toast.error(err.message),
  })

  if (!open) return null

  return (
    <div className="fixed inset-0 bg-ui-bg-overlay/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
      <Container className="max-w-[460px] p-6 bg-ui-bg-base border border-ui-border-base shadow-elevation-modal">
        <Heading level="h2" className="flex items-center gap-x-2 text-ui-fg-destructive mb-2">
          <ExclamationCircle /> Purge Closed Tenants
        </Heading>
        <Text size="small" className="text-ui-fg-subtle mb-4">
          Permanently deletes all rows, domains, and subscription records for closed tenants beyond the retention window.
        </Text>
        <div className="flex flex-col gap-y-2 mb-6">
          <Label size="xsmall">Retention Window (days)</Label>
          <Input
            type="number"
            value={retentionDays}
            onChange={(e) => setRetentionDays(Number(e.target.value))}
          />
        </div>
        <div className="flex justify-end gap-x-2">
          <Button variant="secondary" size="small" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            variant="danger"
            size="small"
            onClick={() => mutation.mutate()}
            isLoading={mutation.isPending}
          >
            Confirm Hard Purge
          </Button>
        </div>
      </Container>
    </div>
  )
}

export function MerchantsView() {
  const queryClient = useQueryClient()
  const tiers = useTierNames()
  const [search, setSearch] = useState("")
  const [tierFilter, setTierFilter] = useState("all")
  const [statusFilter, setStatusFilter] = useState("all")
  const [editing, setEditing] = useState<Tenant | null>(null)
  const [inspectingDomain, setInspectingDomain] = useState<Tenant | null>(null)
  const [purgeOpen, setPurgeOpen] = useState(false)

  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["platform", "tenants", { search, tierFilter, statusFilter }],
    queryFn: () =>
      listTenants({
        search: search.trim() || undefined,
        tier: tierFilter !== "all" ? tierFilter : undefined,
        status: statusFilter !== "all" ? statusFilter : undefined,
      }),
    retry: false,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["platform", "tenants"] })

  const suspendMutation = useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) => suspendTenant(id, reason),
    onSuccess: () => {
      toast.success("Merchant suspended")
      invalidate()
    },
    onError: (err: Error) => toast.error(err.message),
  })

  const reactivateMutation = useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) => reactivateTenant(id, reason),
    onSuccess: () => {
      toast.success("Merchant reactivated")
      invalidate()
    },
    onError: (err: Error) => toast.error(err.message),
  })

  const closeMutation = useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) => closeTenant(id, id, reason),
    onSuccess: () => {
      toast.success("Merchant closed")
      invalidate()
    },
    onError: (err: Error) => toast.error(err.message),
  })

  if (isError) {
    return (
      <Container className="p-6">
        <Heading level="h2" className="mb-2">
          Could not read the registry
        </Heading>
        <Text size="small" className="text-ui-fg-subtle">
          {(error as Error).message}
        </Text>
      </Container>
    )
  }

  const tenants = data?.tenants ?? []
  const byTier = tiers.map((tier) => ({
    tier,
    count: tenants.filter((t) => t.tier === tier).length,
  }))

  return (
    <div className="flex flex-col gap-y-5">
      <div className="flex items-start justify-between">
        <div>
          <Heading level="h1">Merchants</Heading>
          <Text size="small" className="text-ui-fg-subtle">
            Every tenant on this deployment, domain bindings, lifecycle control, and telemetry.
          </Text>
        </div>
        <div className="flex gap-x-2">
          <Button variant="secondary" size="small" onClick={() => setPurgeOpen(true)}>
            Purge Closed
          </Button>
          <Button
            variant="secondary"
            size="small"
            onClick={() => refetch()}
            isLoading={isFetching}
            aria-label="Refresh"
          >
            <ArrowPath />
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Container className="p-4">
          <div className="flex items-center justify-between">
            <Text size="small" weight="plus" className="text-ui-fg-subtle">
              Merchants
            </Text>
            <BuildingStorefront className="text-ui-fg-muted" />
          </div>
          <Text size="xlarge" weight="plus" className="mt-2 tabular-nums">
            {tenants.length}
          </Text>
        </Container>
        {byTier.map(({ tier, count }) => (
          <Container key={tier} className="p-4">
            <div className="flex items-center justify-between">
              <Text size="small" weight="plus" className="text-ui-fg-subtle capitalize">
                {tier}
              </Text>
              <CurrencyDollar className="text-ui-fg-muted" />
            </div>
            <Text size="xlarge" weight="plus" className="mt-2 tabular-nums">
              {count}
            </Text>
          </Container>
        ))}
      </div>

      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-x-3">
          <Input
            placeholder="Search merchants..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-[240px]"
            size="small"
          />
          <Select value={tierFilter} onValueChange={setTierFilter} size="small">
            <Select.Trigger className="w-[130px]">
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="all">All Plans</Select.Item>
              {tiers.map((t) => (
                <Select.Item key={t} value={t}>
                  {t}
                </Select.Item>
              ))}
            </Select.Content>
          </Select>
          <Select value={statusFilter} onValueChange={setStatusFilter} size="small">
            <Select.Trigger className="w-[140px]">
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="all">All Statuses</Select.Item>
              <Select.Item value="active">Active</Select.Item>
              <Select.Item value="suspended">Suspended</Select.Item>
              <Select.Item value="closed">Closed</Select.Item>
            </Select.Content>
          </Select>
        </div>
      </div>

      <CreateTenantForm onCreated={invalidate} />

      <Container className="overflow-hidden p-0">
        <Table>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Merchant</Table.HeaderCell>
              <Table.HeaderCell>Tenant ID</Table.HeaderCell>
              <Table.HeaderCell>Plan</Table.HeaderCell>
              <Table.HeaderCell>Status</Table.HeaderCell>
              <Table.HeaderCell>Created</Table.HeaderCell>
              <Table.HeaderCell className="text-right">Actions</Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {isLoading ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center" colSpan={6}>
                  Loading merchants…
                </Table.Cell>
              </Table.Row>
            ) : tenants.length === 0 ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center" colSpan={6}>
                  No merchants match the current filters.
                </Table.Cell>
              </Table.Row>
            ) : (
              tenants.map((tenant) => (
                <Table.Row key={tenant.id}>
                  <Table.Cell className="text-ui-fg-base font-medium">
                    {tenant.name}
                  </Table.Cell>
                  <Table.Cell className="text-ui-fg-subtle font-mono text-xs">
                    {tenant.id}
                  </Table.Cell>
                  <Table.Cell>
                    <Badge color="blue">{tenant.tier}</Badge>
                  </Table.Cell>
                  <Table.Cell>
                    <StatusBadge
                      color={
                        tenant.status === "active"
                          ? "green"
                          : tenant.status === "suspended"
                          ? "orange"
                          : "red"
                      }
                    >
                      {tenant.status}
                    </StatusBadge>
                  </Table.Cell>
                  <Table.Cell className="text-ui-fg-subtle text-xs tabular-nums">
                    {new Date(tenant.created_at).toLocaleDateString()}
                  </Table.Cell>
                  <Table.Cell className="text-right">
                    <div className="flex items-center justify-end gap-x-2">
                      <Button
                        variant="secondary"
                        size="small"
                        onClick={() => setInspectingDomain(tenant)}
                      >
                        Domains & Telemetry
                      </Button>
                      <Button
                        variant="secondary"
                        size="small"
                        disabled={tenant.tier === "unknown"}
                        onClick={() => setEditing(tenant)}
                      >
                        Subscription
                      </Button>
                      {tenant.status === "active" ? (
                        <Button
                          variant="secondary"
                          size="small"
                          onClick={() => {
                            const reason = prompt("Enter suspension reason:")
                            if (reason) suspendMutation.mutate({ id: tenant.id, reason })
                          }}
                        >
                          Suspend
                        </Button>
                      ) : tenant.status === "suspended" ? (
                        <Button
                          variant="secondary"
                          size="small"
                          onClick={() => {
                            const reason = prompt("Enter reactivation reason:")
                            if (reason) reactivateMutation.mutate({ id: tenant.id, reason })
                          }}
                        >
                          Reactivate
                        </Button>
                      ) : null}
                      {tenant.status !== "closed" && (
                        <Button
                          variant="danger"
                          size="small"
                          onClick={() => {
                            const confirmation = prompt(`Type '${tenant.id}' to confirm closure:`)
                            if (confirmation === tenant.id) {
                              const reason = prompt("Enter closure reason:") || "Account closed by operator"
                              closeMutation.mutate({ id: tenant.id, reason })
                            }
                          }}
                        >
                          Close
                        </Button>
                      )}
                    </div>
                  </Table.Cell>
                </Table.Row>
              ))
            )}
          </Table.Body>
        </Table>
      </Container>

      {editing && (
        <SubscriptionDrawer
          key={editing.id}
          tenantId={editing.id}
          tenantName={editing.name}
          open
          onOpenChange={(open) => {
            if (!open) setEditing(null)
          }}
        />
      )}

      {inspectingDomain && (
        <TenantDomainDrawer
          key={inspectingDomain.id}
          tenant={inspectingDomain}
          open
          onOpenChange={(open) => {
            if (!open) setInspectingDomain(null)
          }}
        />
      )}

      <PurgeModal open={purgeOpen} onOpenChange={setPurgeOpen} />
    </div>
  )
}
