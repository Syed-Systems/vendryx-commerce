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
import { Audit } from "./audit"
import { Billing } from "./billing"
import { Gateways } from "./gateways"
import { Operators } from "./operators"
import { MerchantsView } from "./tenants"
import { AnalyticsView } from "./analytics"
import { SupportView } from "./support"
import { ConfigurationView } from "./configuration"
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
              placeholder="op_..."
            />
          </div>
          <Button type="submit" disabled={!value.trim()} size="small">
            Authenticate
          </Button>
        </form>

        <Text size="xsmall" className="text-ui-fg-muted mt-5">
          Pass STHENO_PLATFORM_TOKEN here to bootstrap the first operator credential, then
          issue individual named keys in the Operators tab.
        </Text>
      </Container>
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
              <Tabs.Trigger value="analytics">Analytics</Tabs.Trigger>
              <Tabs.Trigger value="plans">Plans</Tabs.Trigger>
              <Tabs.Trigger value="billing">Billing</Tabs.Trigger>
              <Tabs.Trigger value="gateways">Gateways</Tabs.Trigger>
              <Tabs.Trigger value="support">Support</Tabs.Trigger>
              <Tabs.Trigger value="config">Config & Themes</Tabs.Trigger>
              <Tabs.Trigger value="operators">Operators</Tabs.Trigger>
              <Tabs.Trigger value="audit">Audit</Tabs.Trigger>
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
            <MerchantsView />
          </Tabs.Content>
          <Tabs.Content value="analytics" className="pt-5">
            <AnalyticsView />
          </Tabs.Content>
          <Tabs.Content value="plans" className="pt-5">
            <Plans />
          </Tabs.Content>
          <Tabs.Content value="billing" className="pt-5">
            <Billing />
          </Tabs.Content>
          <Tabs.Content value="gateways" className="pt-5">
            <Gateways />
          </Tabs.Content>
          <Tabs.Content value="support" className="pt-5">
            <SupportView />
          </Tabs.Content>
          <Tabs.Content value="config" className="pt-5">
            <ConfigurationView />
          </Tabs.Content>
          <Tabs.Content value="operators" className="pt-5">
            <Operators />
          </Tabs.Content>
          <Tabs.Content value="audit" className="pt-5">
            <Audit />
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
