import { ArrowPath, BuildingStorefront, CurrencyDollar, Sun, Moon } from "@medusajs/icons"
import {
  Badge,
  Button,
  Container,
  Heading,
  Input,
  Label,
  Select,
  StatusBadge,
  Table,
  Tabs,
  Text,
  Toaster,
  toast,
} from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useState } from "react"

import {
  clearToken,
  createTenant,
  getToken,
  listTenants,
  listTiers,
  setTier,
  setToken,
  type Tenant,
} from "./api"
import { Plans } from "./plans"
import { SubscriptionDrawer } from "./subscription"

/**
 * The plans the engine actually has, read from the control plane rather than written out
 * here. A tier added in Rust turns up in every picker on this page without a matching
 * edit in TypeScript that would otherwise drift out of step with it.
 */
function useTierNames() {
  const { data } = useQuery({
    queryKey: ["platform", "tiers"],
    queryFn: listTiers,
    retry: false,
  })
  return data?.tiers.map((t) => t.tier) ?? []
}

/** Matches the admin's ThemeProvider contract, so both apps flip the same way. */
function useTheme() {
  const [theme, setThemeState] = useState<"light" | "dark">(() => {
    const stored = localStorage.getItem("stheno_platform_theme")
    if (stored === "light" || stored === "dark") return stored
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
  })

  useEffect(() => {
    const root = document.documentElement
    root.classList.remove(theme === "dark" ? "light" : "dark")
    root.classList.add(theme)
    root.style.colorScheme = theme
    localStorage.setItem("stheno_platform_theme", theme)
  }, [theme])

  return { theme, toggle: () => setThemeState((t) => (t === "dark" ? "light" : "dark")) }
}

function TokenGate({ onAuthenticated }: { onAuthenticated: () => void }) {
  const [value, setValue] = useState("")

  return (
    <div className="bg-ui-bg-subtle flex min-h-dvh w-dvw items-center justify-center">
      <Container className="m-4 w-full max-w-[380px] p-6">
        <Heading level="h1" className="mb-1">
          Stheno Platform
        </Heading>
        <Text size="small" className="text-ui-fg-subtle mb-5">
          The control plane for every merchant on this deployment.
        </Text>

        <form
          className="flex flex-col gap-y-3"
          onSubmit={(event) => {
            event.preventDefault()
            setToken(value.trim())
            onAuthenticated()
          }}
        >
          <div className="flex flex-col gap-y-1.5">
            <Label size="small" htmlFor="token">
              Operator token
            </Label>
            <Input
              id="token"
              type="password"
              autoComplete="off"
              value={value}
              onChange={(event) => setValue(event.target.value)}
              placeholder="STHENO_PLATFORM_TOKEN"
            />
          </div>
          <Button type="submit" disabled={!value.trim()}>
            Continue
          </Button>
        </form>
      </Container>
    </div>
  )
}

function CreateTenantForm({ onCreated }: { onCreated: () => void }) {
  const [open, setOpen] = useState(false)
  const [form, setForm] = useState({ id: "", name: "", tier: "free", domain: "" })
  const tiers = useTierNames()

  const mutation = useMutation({
    mutationFn: () =>
      createTenant({
        id: form.id.trim(),
        name: form.name.trim(),
        tier: form.tier,
        domain: form.domain.trim() || undefined,
      }),
    onSuccess: () => {
      toast.success(`Created ${form.id}`)
      setForm({ id: "", name: "", tier: "free", domain: "" })
      setOpen(false)
      onCreated()
    },
    onError: (error: Error) => toast.error(error.message),
  })

  if (!open) {
    return (
      <Button variant="secondary" size="small" onClick={() => setOpen(true)}>
        Add merchant
      </Button>
    )
  }

  return (
    <Container className="p-4">
      <Heading level="h3" className="mb-3">
        New merchant
      </Heading>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Tenant id</Label>
          <Input
            value={form.id}
            placeholder="ten_acme"
            onChange={(e) => setForm({ ...form, id: e.target.value })}
          />
        </div>
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Name</Label>
          <Input
            value={form.name}
            placeholder="Acme"
            onChange={(e) => setForm({ ...form, name: e.target.value })}
          />
        </div>
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Plan</Label>
          <Select value={form.tier} onValueChange={(v) => setForm({ ...form, tier: v })}>
            <Select.Trigger>
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              {tiers.map((tier) => (
                <Select.Item key={tier} value={tier}>
                  {tier}
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
      <Text size="xsmall" className="text-ui-fg-muted mt-3">
        The domain is how requests are routed to this merchant. Without one it can only be
        reached by the single-tenant fallback.
      </Text>
    </Container>
  )
}

function TierCell({ tenant, onChanged }: { tenant: Tenant; onChanged: () => void }) {
  const tiers = useTierNames()
  const mutation = useMutation({
    mutationFn: (tier: string) => setTier(tenant.id, tier),
    onSuccess: (_, tier) => {
      // The change is live on the tenant's very next request — entitlements resolve per
      // request from the subscription row, not at boot.
      toast.success(`${tenant.name} moved to ${tier}`)
      onChanged()
    },
    onError: (error: Error) => toast.error(error.message),
  })

  if (tenant.tier === "unknown") {
    // No subscription row. A provisioning bug, not a plan — say so rather than offering
    // a picker that would write a second row.
    return <Badge color="red">no subscription</Badge>
  }

  return (
    <Select
      value={tenant.tier}
      onValueChange={(tier) => mutation.mutate(tier)}
      disabled={mutation.isPending}
      size="small"
    >
      <Select.Trigger className="w-[110px]">
        <Select.Value />
      </Select.Trigger>
      <Select.Content>
        {tiers.map((tier) => (
          <Select.Item key={tier} value={tier}>
            {tier}
          </Select.Item>
        ))}
      </Select.Content>
    </Select>
  )
}

function Merchants() {
  const queryClient = useQueryClient()
  const tiers = useTierNames()
  const [editing, setEditing] = useState<Tenant | null>(null)

  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["platform", "tenants"],
    queryFn: listTenants,
    retry: false,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["platform"] })

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
            Every tenant on this deployment, and the plan each one is on.
          </Text>
        </div>
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

      <CreateTenantForm onCreated={invalidate} />

      <Container className="overflow-hidden p-0">
        <Table>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Merchant</Table.HeaderCell>
              <Table.HeaderCell>Tenant</Table.HeaderCell>
              <Table.HeaderCell>Plan</Table.HeaderCell>
              <Table.HeaderCell>Subscription</Table.HeaderCell>
              <Table.HeaderCell>Created</Table.HeaderCell>
              <Table.HeaderCell />
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {isLoading ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center">
                  Loading merchants…
                </Table.Cell>
              </Table.Row>
            ) : tenants.length === 0 ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center">
                  No merchants yet.
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
                    <TierCell tenant={tenant} onChanged={invalidate} />
                  </Table.Cell>
                  <Table.Cell>
                    <StatusBadge
                      color={tenant.subscription_status === "active" ? "green" : "orange"}
                    >
                      {tenant.subscription_status}
                    </StatusBadge>
                  </Table.Cell>
                  <Table.Cell className="text-ui-fg-subtle text-xs tabular-nums">
                    {new Date(tenant.created_at).toLocaleDateString()}
                  </Table.Cell>
                  <Table.Cell className="text-right">
                    <Button
                      variant="secondary"
                      size="small"
                      disabled={tenant.tier === "unknown"}
                      onClick={() => setEditing(tenant)}
                    >
                      Manage
                    </Button>
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
    </div>
  )
}

function Dashboard({ onSignOut }: { onSignOut: () => void }) {
  const { theme, toggle } = useTheme()
  const [tab, setTab] = useState("merchants")

  // One query stands in for the whole surface: if the token is wrong every call 404s, so
  // failing here once beats six panels each reporting the same thing.
  const { isError, error } = useQuery({
    queryKey: ["platform", "tiers"],
    queryFn: listTiers,
    retry: false,
  })

  if (isError) {
    return (
      <div className="bg-ui-bg-subtle flex min-h-dvh items-center justify-center">
        <Container className="m-4 max-w-[420px] p-6">
          <Heading level="h2" className="mb-2">
            Could not read the control plane
          </Heading>
          <Text size="small" className="text-ui-fg-subtle mb-4">
            {(error as Error).message}
          </Text>
          <Button variant="secondary" size="small" onClick={onSignOut}>
            Use a different token
          </Button>
        </Container>
      </div>
    )
  }

  return (
    <div className="bg-ui-bg-subtle min-h-dvh w-dvw">
      <div className="mx-auto flex max-w-[1200px] flex-col gap-y-5 p-6">
        <Tabs value={tab} onValueChange={setTab}>
          <div className="flex items-center justify-between">
            <Tabs.List>
              <Tabs.Trigger value="merchants">Merchants</Tabs.Trigger>
              <Tabs.Trigger value="plans">Plans</Tabs.Trigger>
            </Tabs.List>
            <div className="flex items-center gap-x-2">
              <Button
                variant="transparent"
                size="small"
                onClick={toggle}
                aria-label="Toggle theme"
              >
                {theme === "dark" ? <Sun /> : <Moon />}
              </Button>
              <Button variant="secondary" size="small" onClick={onSignOut}>
                Sign out
              </Button>
            </div>
          </div>

          <Tabs.Content value="merchants" className="pt-5">
            <Merchants />
          </Tabs.Content>
          <Tabs.Content value="plans" className="pt-5">
            <Plans />
          </Tabs.Content>
        </Tabs>
      </div>
      <Toaster />
    </div>
  )
}

export default function App() {
  const [authenticated, setAuthenticated] = useState(() => Boolean(getToken()))

  if (!authenticated) {
    return <TokenGate onAuthenticated={() => setAuthenticated(true)} />
  }

  return (
    <Dashboard
      onSignOut={() => {
        clearToken()
        setAuthenticated(false)
      }}
    />
  )
}
