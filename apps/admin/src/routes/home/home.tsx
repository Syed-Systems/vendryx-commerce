import { ExclamationCircle } from "@medusajs/icons"
import { Button, Heading, Text } from "@medusajs/ui"
import { useMe } from "../../hooks/api/users"
import { useHomeSummary } from "../../hooks/api/home"
import { useOnboarding } from "../../hooks/api/onboarding"
import { useTodayPulse } from "../../hooks/api/today-pulse"
import { useRecentOrders } from "../../hooks/api/recent-orders"
import { useProducts } from "../../hooks/api/products"
import { useStore } from "../../hooks/api/store"
import { GreetingHeader } from "./components/greeting-header"
import { AttentionStrip } from "./components/attention-strip"
import { TodayPulseCard } from "./components/today-pulse"
import { RecentOrdersCard } from "./components/recent-orders-card"
import { OnboardingSummaryCard } from "./components/onboarding-summary-card"
import { StorefrontPreviewCard } from "./components/storefront-preview-card"
import { QuickOperationsCard } from "./components/quick-operations-card"

// ─────────────────────────────────────────────────────────────────────────────
// Home — Merchant Owner Command Cockpit
// ─────────────────────────────────────────────────────────────────────────────
//
// Designed for the busy store owner who opens admin to answer two questions:
//   1. "Is the business alive and running smoothly?" (Vitals + Live Preview)
//   2. "What needs my attention right now?" (Alerts + Orders + Setup Roadmap)
//
// Information Hierarchy & Layout:
//   • Top: Contextual greeting with date and high-frequency quick actions.
//   • Store Vitals: 4-stat performance grid (Revenue, Orders, Products, Health).
//   • Attention Strip: Exception-driven alerts (pending orders, low inventory).
//   • 2-Column Responsive Cockpit:
//       - Left (7 cols): Setup Roadmap (if incomplete) + Order Workstream / Launchpad.
//       - Right (5 cols): Quick Operations Shortcuts + Live Storefront Studio.
//
// Rules enforced:
//   - AGENTS.md §2.5: No floating point in money.
//   - Standing rule: Every number rendered as fact traces to live API response.

export const Home = () => {
  // ── Independent data sources (Option B) ───────────────────────────────
  const { user } = useMe()
  const { summary, isLoading: summaryLoading, isError: summaryError, error: summaryErrorObj } = useHomeSummary()
  const { data: onboarding, isLoading: onboardingLoading } = useOnboarding()
  const { today, isLoading: todayLoading, isError: todayError } = useTodayPulse()
  const { orders: recentOrders, totalCount: totalOrderCount, isLoading: ordersLoading } = useRecentOrders()
  const { count: productsCount, isLoading: productsLoading } = useProducts({ limit: 0 })
  const { store } = useStore()

  // ── Derived attention state ───────────────────────────────────────────
  const pendingCount = summary?.pending_orders?.count ?? 0
  const lowStockCount = summary?.low_stock?.count ?? 0
  const lowStockThreshold = summary?.low_stock?.threshold ?? 5

  const hasAttentionItems = pendingCount > 0 || lowStockCount > 0

  // ── Fatal error — whole summary request failed ────────────────────────
  if (!summaryLoading && summaryError) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] gap-3 px-4">
        <ExclamationCircle className="text-ui-fg-subtle w-8 h-8" />
        <Heading level="h2" className="text-ui-fg-base text-lg font-semibold">
          Could not load dashboard
        </Heading>
        <Text size="small" className="text-ui-fg-subtle max-w-sm text-center">
          {summaryErrorObj?.message ??
            "An unexpected error occurred. Reload the page to try again."}
        </Text>
        <Button
          variant="secondary"
          size="small"
          onClick={() => window.location.reload()}
        >
          Reload
        </Button>
      </div>
    )
  }

  // ── First-load state ──────────────────────────────────────────────────
  const isFirstLoad = summaryLoading && !summary

  const currencyCode =
    store?.supported_currencies?.find((c) => c.is_default)?.currency_code ||
    store?.supported_currencies?.[0]?.currency_code ||
    "usd"

  return (
    <div className="flex flex-col gap-y-6 py-6 px-4 sm:px-6 max-w-[1400px] mx-auto w-full">
      {/* ── 1. Greeting & Quick Actions Header ──────────────────────────── */}
      <GreetingHeader
        firstName={user?.first_name}
        hasAttentionItems={hasAttentionItems}
      />

      {/* ── 2. Store Vitals 4-Grid ───────────────────────────────────────── */}
      <TodayPulseCard
        today={today}
        totalOrders={totalOrderCount}
        productsCount={productsCount ?? 0}
        pendingOrdersCount={pendingCount}
        lowStockCount={lowStockCount}
        currencyCode={currencyCode}
        isLoading={isFirstLoad || todayLoading || productsLoading}
        todayUnavailable={todayError}
      />

      {/* ── 3. Operational Attention Alerts (only if bottlenecks exist) ─── */}
      <AttentionStrip
        pendingOrderCount={pendingCount}
        lowStockCount={lowStockCount}
        lowStockThreshold={lowStockThreshold}
      />

      {/* ── 4. Main 2-Column Command Cockpit ────────────────────────────── */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
        {/* Left Column (7 cols): Setup Roadmap & Orders Workstream */}
        <div className="lg:col-span-7 xl:col-span-8 flex flex-col gap-y-6">
          {/* Setup Roadmap (active until completed or dismissed) */}
          <OnboardingSummaryCard
            onboarding={onboarding}
            isLoading={onboardingLoading}
          />

          {/* Orders Workstream / First Sale Launchpad */}
          <RecentOrdersCard
            orders={recentOrders}
            isLoading={ordersLoading}
          />
        </div>

        {/* Right Column (5 cols): Store Management Shortcuts & Live Studio */}
        <div className="lg:col-span-5 xl:col-span-4 flex flex-col gap-y-6">
          {/* Quick Operations Panel */}
          <QuickOperationsCard />

          {/* Storefront Live Studio Preview */}
          <div className="flex flex-col gap-2">
            <StorefrontPreviewCard />
          </div>
        </div>
      </div>
    </div>
  )
}
