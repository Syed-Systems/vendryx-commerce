import { Moon, Sun } from "@medusajs/icons"
import {
  Button,
  Container,
  Heading,
  Input,
  Label,
  Tabs,
  Text,
  Toaster,
} from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useEffect, useState } from "react"

import { clearToken, getToken, listTiers, setToken } from "./api"
import { Audit } from "./audit"
import { Billing } from "./billing"
import { Gateways } from "./gateways"
import { Operators } from "./operators"
import { MerchantsView } from "./tenants"
import { AnalyticsView } from "./analytics"
import { SupportView } from "./support"
import { ConfigurationView } from "./configuration"
import { Plans } from "./plans"
import { LaunchRequests } from "./launch-requests"

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
  const isDark = document.documentElement.classList.contains("dark")

  return (
    <div className="bg-ui-bg-subtle flex min-h-dvh w-dvw items-center justify-center">
      <Container className="m-4 w-full max-w-[380px] p-6">
        <div className="mb-4">
          <img
            src={isDark ? "/brand/vendryx/vendryx-primary-dark.svg" : "/brand/vendryx/vendryx-primary-light.svg"}
            alt="Vendryx by Syed Systems"
            className="h-8 max-w-full object-contain"
          />
        </div>
        <Heading level="h1" className="mb-1">
          Vendryx Platform
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

function Dashboard({
  onSignOut,
  theme,
  toggle,
}: {
  onSignOut: () => void
  theme: "light" | "dark"
  toggle: () => void
}) {
  const [tab, setTab] = useState("launch-requests")

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
      <div className="mx-auto flex max-w-[1320px] flex-col gap-y-5 p-6">
        <div className="flex items-center justify-between border-b border-ui-border-base pb-3">
          <div className="flex items-center gap-x-3">
            <img
              src={theme === "dark" ? "/brand/vendryx/vendryx-primary-dark.svg" : "/brand/vendryx/vendryx-primary-light.svg"}
              alt="Vendryx by Syed Systems"
              className="h-7 object-contain"
            />
            <span className="text-ui-border-strong hidden h-4 w-px bg-ui-border-base sm:inline-block" />
            <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase tracking-wider hidden sm:inline-block">
              Platform Control Plane
            </Text>
          </div>
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

        <Tabs value={tab} onValueChange={setTab}>
          <div className="flex flex-col gap-y-4 border-b border-ui-border-base pb-4">
            <div className="grid gap-x-6 gap-y-3 sm:grid-cols-2 lg:grid-cols-4">
              <div>
                <Text size="xsmall" weight="plus" className="text-ui-fg-muted mb-1">Launch</Text>
                <Tabs.List className="[&>*]:whitespace-nowrap">
                  <Tabs.Trigger value="launch-requests">Requests</Tabs.Trigger>
                  <Tabs.Trigger value="merchants">Merchants</Tabs.Trigger>
                </Tabs.List>
              </div>
              <div>
                <Text size="xsmall" weight="plus" className="text-ui-fg-muted mb-1">Commercial</Text>
                <Tabs.List className="[&>*]:whitespace-nowrap">
                  <Tabs.Trigger value="plans">Plans</Tabs.Trigger>
                  <Tabs.Trigger value="billing">Billing</Tabs.Trigger>
                  <Tabs.Trigger value="analytics">Revenue</Tabs.Trigger>
                </Tabs.List>
              </div>
              <div>
                <Text size="xsmall" weight="plus" className="text-ui-fg-muted mb-1">Operations</Text>
                <Tabs.List className="[&>*]:whitespace-nowrap">
                  <Tabs.Trigger value="gateways">Gateways</Tabs.Trigger>
                  <Tabs.Trigger value="support">Support</Tabs.Trigger>
                </Tabs.List>
              </div>
              <div>
                <Text size="xsmall" weight="plus" className="text-ui-fg-muted mb-1">System</Text>
                <Tabs.List className="[&>*]:whitespace-nowrap">
                  <Tabs.Trigger value="config">Configuration</Tabs.Trigger>
                  <Tabs.Trigger value="operators">Operators</Tabs.Trigger>
                  <Tabs.Trigger value="audit">Audit</Tabs.Trigger>
                </Tabs.List>
              </div>
            </div>
          </div>

          <Tabs.Content value="launch-requests" className="pt-5">
            <LaunchRequests />
          </Tabs.Content>
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
  // Called here, above the TokenGate branch, not inside Dashboard. useTheme is what puts
  // the light/dark class on <html>, and Dashboard never mounts until an operator is
  // authenticated — so an operator on dark got a white login screen every time, because
  // nothing had applied their saved theme yet.
  const { theme, toggle } = useTheme()

  if (!authenticated) {
    return <TokenGate onAuthenticated={() => setAuthenticated(true)} />
  }

  return (
    <Dashboard
      theme={theme}
      toggle={toggle}
      onSignOut={() => {
        clearToken()
        setAuthenticated(false)
      }}
    />
  )
}
