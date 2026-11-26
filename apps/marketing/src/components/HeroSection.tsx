import {
  AlertCircle,
  ArrowRight,
  ArrowUpRight,
  Box,
  Check,
  CheckCircle2,
  CreditCard,
  DollarSign,
  ExternalLink,
  LayoutGrid,
  Palette,
  PieChart,
  Plus,
  Search,
  Settings,
  ShoppingCart,
  Sparkles,
  Store,
  Tag,
  Truck,
  Users,
} from "lucide-react"
import { useMemo, useState } from "react"
import {
  INITIAL_DASHBOARD_ORDERS,
  type DashboardOrderRow,
} from "../data/marketing-content"
import { formatWholeBdt } from "../lib/marketing-logic"

interface HeroSectionProps {
  onOpenOnboarding: (plan: "trial" | "free" | "pro") => void
}

type OrderView = "all" | "social" | "pending"

const QUICK_SHORTCUTS = [
  {
    title: "Payment Providers",
    description: "Manage COD, bKash & gateways",
    icon: CreditCard,
    iconColor: "text-blue-400",
  },
  {
    title: "Inventory & Stock",
    description: "Stock levels and replenishment",
    icon: Box,
    iconColor: "text-[#FFAF24]",
  },
  {
    title: "Promotions & Discounts",
    description: "Coupon codes and sales rules",
    icon: Tag,
    iconColor: "text-emerald-400",
  },
  {
    title: "Customer Directory",
    description: "Buyers and repeat orders",
    icon: Users,
    iconColor: "text-amber-400",
  },
] as const

export function HeroSection({ onOpenOnboarding }: HeroSectionProps) {
  const [orders, setOrders] = useState<DashboardOrderRow[]>(INITIAL_DASHBOARD_ORDERS)
  const [view, setView] = useState<OrderView>("all")
  const [bookedMap, setBookedMap] = useState<Record<string, string>>({})
  const [toastText, setToastText] = useState<string | null>(null)

  const pendingOrdersCount = useMemo(
    () => orders.filter((o) => o.status === "pending").length,
    [orders]
  )
  const lowStockCount = 2
  const urgentCount = pendingOrdersCount + lowStockCount

  const todayRevenue = useMemo(() => {
    const baseCompleted = 18350
    const newlyConfirmed = orders
      .filter(
        (o) =>
          o.status === "completed" &&
          (o.id === "ord-1048" || o.id === "ord-1047")
      )
      .reduce((acc, o) => acc + o.amountNum, 0)
    return baseCompleted + newlyConfirmed
  }, [orders])

  const todayOrderCount = useMemo(() => {
    const confirmedExtras = orders.filter(
      (o) =>
        o.status === "completed" &&
        (o.id === "ord-1048" || o.id === "ord-1047")
    ).length
    return 6 + confirmedExtras
  }, [orders])

  const visibleOrders = useMemo(() => {
    if (view === "pending") return orders.filter((o) => o.status === "pending")
    if (view === "social") {
      return orders.filter((o) => o.source === "Instagram DM" || o.source === "WhatsApp")
    }
    return orders
  }, [orders, view])

  function handleConfirmOrder(order: DashboardOrderRow) {
    setOrders((prev) =>
      prev.map((row) =>
        row.id === order.id ? { ...row, status: "completed" } : row
      )
    )
    setToastText(`Order ${order.displayId} (${order.customer}) confirmed.`)
  }

  function handleBookCourier(order: DashboardOrderRow) {
    const prefix =
      order.courier === "Pathao" ? "PTH" : order.courier === "Steadfast" ? "STF" : "RDX"
    const tracking = `${prefix}-${order.displayId.slice(1)}`
    setBookedMap((prev) => ({ ...prev, [order.id]: tracking }))
    setToastText(`${order.courier} booked for ${order.displayId} · Tracking ${tracking}`)
  }

  return (
    <section
      id="cockpit"
      className="relative scroll-mt-16 overflow-hidden border-b border-[#FFFCF8]/[0.08] vendryx-hero-glow"
    >
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 vendryx-grid-mask"
      />

      <div className="relative mx-auto max-w-[1200px] px-4 pb-12 pt-8 sm:px-6 sm:pt-11 lg:pb-14">
        <div className="mx-auto max-w-[780px] text-center">
          <h1 className="text-[34px] font-semibold leading-[1.06] tracking-[-0.035em] text-[#FFFCF8] sm:text-[46px] lg:text-[52px]">
            One cart. Every channel. One simple dashboard.
          </h1>

          <p className="mx-auto mt-3 max-w-[630px] text-[15px] leading-[1.55] text-[#94A3B8] sm:text-[16px]">
            Sell from your own online store, WhatsApp, Facebook, and Instagram—with bKash,
            Cash on Delivery, and Pathao, Steadfast &amp; RedX delivery all in one calm place.
          </p>

          <div className="mt-5 flex flex-wrap items-center justify-center gap-3">
            <button
              type="button"
              onClick={() => onOpenOnboarding("trial")}
              className="inline-flex items-center gap-2 rounded-md bg-[#FF7A00] px-4 py-2.5 text-[14px] font-semibold text-[#080A0C] transition-colors hover:bg-[#D95700] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#FFAF24]"
            >
              Start 14-day free trial
              <ArrowRight className="h-4 w-4" />
            </button>
            <a
              href="#pricing"
              className="inline-flex items-center gap-2 rounded-md border border-[#FFFCF8]/12 bg-[#1B1E24]/70 px-4 py-2.5 text-[14px] font-medium text-[#FFFCF8] transition-colors hover:border-[#FFFCF8]/25 hover:bg-[#1B1E24]"
            >
              See plans &amp; pricing
            </a>
          </div>
        </div>

        {/* Exact Visual Replica of Vendryx Admin Dashboard (apps/admin MainLayout + Home) */}
        <div
          className="mt-7 overflow-hidden rounded-xl border border-[#FFFCF8]/[0.10] bg-[#0F1115] shadow-[0_24px_80px_rgba(0,0,0,0.65)]"
          aria-label="Live interactive preview of the Vendryx Merchant Dashboard"
        >
          <div className="grid grid-cols-1 lg:grid-cols-[224px_1fr]">
            {/* Exact Sidebar Replica (apps/admin/src/components/layout/main-layout/main-layout.tsx) */}
            <aside className="hidden flex-col justify-between border-r border-[#FFFCF8]/[0.08] bg-[#141416] lg:flex">
              <div>
                {/* Store Header Switcher */}
                <div className="border-b border-[#FFFCF8]/[0.07] p-2.5">
                  <div className="grid grid-cols-[28px_1fr_auto] items-center gap-x-2.5 rounded-lg p-1.5">
                    <div className="flex h-7 w-7 items-center justify-center overflow-hidden rounded-md border border-[#FFFCF8]/10 bg-[#0F1115]">
                      <img
                        src="/brand/vendryx/vendryx-mark.svg"
                        alt=""
                        width={20}
                        height={20}
                        className="h-5 w-5"
                      />
                    </div>
                    <div className="min-w-0">
                      <p className="truncate text-[13px] font-semibold text-[#FFFCF8]">
                        Dhaka Loom Co.
                      </p>
                      <div className="mt-0.5 flex items-center gap-1.5">
                        <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-emerald-500" />
                        <span className="truncate text-[11px] text-[#94A3B8]">
                          Active Store
                        </span>
                      </div>
                    </div>
                    <span className="rounded-md border border-[#FFFCF8]/10 bg-[#0F1115] px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-[#CBD5E1]">
                      PRO
                    </span>
                  </div>
                </div>

                {/* Searchbar & Navigation Sections */}
                <div className="space-y-2 p-2.5">
                  <div className="flex h-8 items-center gap-2 rounded-lg border border-[#FFFCF8]/[0.08] bg-[#FFFCF8]/[0.04] px-2.5 text-[12px] text-[#94A3B8]">
                    <Search className="h-3.5 w-3.5 shrink-0" />
                    <span className="flex-1 text-left">Search...</span>
                    <kbd className="rounded border border-[#FFFCF8]/10 bg-[#0F1115] px-1.5 py-0.5 font-mono text-[10px] text-[#94A3B8]">
                      ⌘K
                    </kbd>
                  </div>

                  {/* Overview */}
                  <div className="space-y-0.5">
                    <button
                      type="button"
                      onClick={() => setView("all")}
                      className={`flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[12.5px] font-medium transition-colors ${
                        view === "all"
                          ? "bg-[#FFFCF8]/[0.08] text-[#FFFCF8]"
                          : "text-[#94A3B8] hover:bg-[#FFFCF8]/[0.04] hover:text-[#FFFCF8]"
                      }`}
                    >
                      <LayoutGrid className="h-3.5 w-3.5 shrink-0 text-[#FF7A00]" />
                      <span>Overview</span>
                    </button>
                  </div>

                  {/* Commerce */}
                  <div className="space-y-0.5">
                    <p className="px-2.5 pt-1 text-[10px] font-semibold uppercase tracking-[0.08em] text-[#64748B]">
                      Commerce
                    </p>
                    <button
                      type="button"
                      onClick={() => setView("pending")}
                      className={`flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-[12.5px] font-medium transition-colors ${
                        view === "pending"
                          ? "bg-[#FFFCF8]/[0.08] text-[#FFFCF8]"
                          : "text-[#94A3B8] hover:bg-[#FFFCF8]/[0.04] hover:text-[#FFFCF8]"
                      }`}
                    >
                      <span className="flex items-center gap-2">
                        <ShoppingCart className="h-3.5 w-3.5 shrink-0" />
                        <span>Orders</span>
                      </span>
                      {pendingOrdersCount > 0 && (
                        <span className="font-mono text-[10px] font-semibold text-[#FFAF24]">
                          {pendingOrdersCount}
                        </span>
                      )}
                    </button>
                    <button
                      type="button"
                      onClick={() => setView("social")}
                      className={`flex w-full items-center justify-between rounded-md py-1 pl-8 pr-2.5 text-[12px] transition-colors ${
                        view === "social"
                          ? "bg-[#FFFCF8]/[0.08] text-[#FFFCF8] font-medium"
                          : "text-[#94A3B8] hover:text-[#FFFCF8]"
                      }`}
                    >
                      <span>Social Commerce</span>
                      <span className="font-mono text-[10px] text-[#94A3B8]">2</span>
                    </button>
                    <div className="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-[12.5px] text-[#94A3B8]">
                      <Users className="h-3.5 w-3.5 shrink-0" />
                      <span>Customers</span>
                    </div>
                  </div>

                  {/* Catalog */}
                  <div className="space-y-0.5">
                    <p className="px-2.5 pt-1 text-[10px] font-semibold uppercase tracking-[0.08em] text-[#64748B]">
                      Catalog
                    </p>
                    <div className="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-[12.5px] text-[#94A3B8]">
                      <Tag className="h-3.5 w-3.5 shrink-0" />
                      <span>Products</span>
                    </div>
                    <div className="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-[12.5px] text-[#94A3B8]">
                      <Box className="h-3.5 w-3.5 shrink-0" />
                      <span>Inventory</span>
                    </div>
                  </div>

                  {/* Operations & Finance */}
                  <div className="space-y-0.5">
                    <p className="px-2.5 pt-1 text-[10px] font-semibold uppercase tracking-[0.08em] text-[#64748B]">
                      Operations &amp; Finance
                    </p>
                    <div className="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-[12.5px] text-[#94A3B8]">
                      <PieChart className="h-3.5 w-3.5 shrink-0" />
                      <span>Analytics</span>
                    </div>
                    <div className="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-[12.5px] text-[#94A3B8]">
                      <CreditCard className="h-3.5 w-3.5 shrink-0" />
                      <span>COD Reconciliation</span>
                    </div>
                  </div>
                </div>
              </div>

              <div className="border-t border-[#FFFCF8]/[0.07] p-2.5">
                <div className="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-[12.5px] text-[#94A3B8]">
                  <Settings className="h-3.5 w-3.5 shrink-0" />
                  <span>Settings</span>
                </div>
              </div>
            </aside>

            {/* Exact Main Home Replica (apps/admin/src/routes/home/home.tsx) */}
            <div className="flex flex-col gap-4 bg-[#0F1115] p-4 sm:p-5">
              {/* 1. GreetingHeader (greeting-header.tsx) */}
              <div className="flex flex-col justify-between gap-3 border-b border-[#FFFCF8]/[0.08] pb-3 sm:flex-row sm:items-center">
                <div>
                  <div className="flex flex-wrap items-center gap-2.5">
                    <h2 className="text-[18px] font-bold tracking-tight text-[#FFFCF8] sm:text-[20px]">
                      Good morning, Farzana
                    </h2>
                    <span className="inline-flex items-center gap-1.5 rounded-md border border-emerald-500/25 bg-emerald-500/10 px-2 py-0.5 text-[11px] font-medium text-emerald-400">
                      <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
                      Live Store
                    </span>
                  </div>
                  <p className="mt-0.5 text-[12px] text-[#94A3B8]">
                    {urgentCount > 0
                      ? "Today · Attention items require review"
                      : "Today · Everything is caught up"}
                  </p>
                </div>

                <div className="flex flex-wrap items-center gap-2">
                  <a
                    href="#operations"
                    className="inline-flex items-center gap-1.5 rounded-md border border-[#FFFCF8]/10 bg-[#1B1E24] px-2.5 py-1.5 text-[11.5px] font-medium text-[#CBD5E1] hover:text-[#FFFCF8]"
                  >
                    <Palette className="h-3.5 w-3.5 text-[#94A3B8]" />
                    Theme
                  </a>
                  <a
                    href="#operations"
                    className="inline-flex items-center gap-1.5 rounded-md border border-[#FFFCF8]/10 bg-[#1B1E24] px-2.5 py-1.5 text-[11.5px] font-medium text-[#CBD5E1] hover:text-[#FFFCF8]"
                  >
                    <Store className="h-3.5 w-3.5 text-[#94A3B8]" />
                    Storefront
                    <ExternalLink className="h-3 w-3 text-[#64748B]" />
                  </a>
                  <button
                    type="button"
                    onClick={() => onOpenOnboarding("trial")}
                    className="inline-flex items-center gap-1.5 rounded-md bg-[#FF7A00] px-2.5 py-1.5 text-[11.5px] font-semibold text-[#080A0C] hover:bg-[#D95700]"
                  >
                    <Plus className="h-3.5 w-3.5" />
                    Add Product
                  </button>
                </div>
              </div>

              {/* 2. Store Vitals — Exact 5-Column Asymmetric Grid (today-pulse.tsx) */}
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-5">
                {/* Lead Card (2 cols): Today's Revenue */}
                <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.09] bg-[#14171D] p-4 sm:col-span-2 lg:col-span-2">
                  <div>
                    <div className="flex items-center justify-between gap-2">
                      <div className="flex items-center gap-2">
                        <div className="flex h-7 w-7 items-center justify-center rounded-lg border border-[#FFFCF8]/10 bg-[#1B1E24]">
                          <DollarSign className="h-3.5 w-3.5 text-[#CBD5E1]" />
                        </div>
                        <span className="text-[12.5px] font-medium text-[#CBD5E1]">
                          Today&apos;s Revenue
                        </span>
                      </div>
                      <span className="rounded-md border border-emerald-500/25 bg-emerald-500/10 px-2 py-0.5 text-[10.5px] font-medium text-emerald-400">
                        Today
                      </span>
                    </div>
                    <p className="mt-2.5 text-[28px] font-bold tracking-tight tabular-nums text-[#FFFCF8]">
                      {formatWholeBdt(todayRevenue)}
                    </p>
                  </div>
                  <p className="mt-1.5 text-[12px] text-[#94A3B8]">
                    {todayOrderCount} orders today
                  </p>
                </div>

                {/* Card 2 (1 col): Orders */}
                <button
                  type="button"
                  onClick={() => setView(pendingOrdersCount > 0 ? "pending" : "all")}
                  className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.09] bg-[#14171D] p-3.5 text-left transition-colors hover:border-[#FFFCF8]/20"
                >
                  <div>
                    <div className="flex items-center gap-2">
                      <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border border-[#FFFCF8]/10 bg-[#1B1E24]">
                        <ShoppingCart className="h-3.5 w-3.5 text-[#CBD5E1]" />
                      </div>
                      <span className="text-[12px] font-medium text-[#CBD5E1]">Orders</span>
                    </div>
                    <div className="mt-2.5 flex items-baseline justify-between gap-1.5">
                      <p className="text-[22px] font-bold tracking-tight tabular-nums text-[#FFFCF8]">
                        142
                      </p>
                      {pendingOrdersCount > 0 && (
                        <span className="shrink-0 whitespace-nowrap rounded-md border border-amber-500/25 bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-medium text-[#FFAF24]">
                          {pendingOrdersCount} pending
                        </span>
                      )}
                    </div>
                  </div>
                  <p className="mt-1 text-[11.5px] text-[#94A3B8]">
                    {pendingOrdersCount > 0
                      ? `${pendingOrdersCount} awaiting confirmation`
                      : "All orders confirmed"}
                  </p>
                </button>

                {/* Card 3 (1 col): Active Products */}
                <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.09] bg-[#14171D] p-3.5">
                  <div>
                    <div className="flex items-center gap-2">
                      <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border border-[#FFFCF8]/10 bg-[#1B1E24]">
                        <Tag className="h-3.5 w-3.5 text-[#CBD5E1]" />
                      </div>
                      <span className="truncate text-[12px] font-medium text-[#CBD5E1]">
                        Active Products
                      </span>
                    </div>
                    <p className="mt-2.5 text-[22px] font-bold tracking-tight tabular-nums text-[#FFFCF8]">
                      84
                    </p>
                  </div>
                  <p className="mt-1 text-[11.5px] text-[#94A3B8]">Published in catalog</p>
                </div>

                {/* Card 4 (1 col): Store Health */}
                <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.09] bg-[#14171D] p-3.5">
                  <div>
                    <div className="flex items-center justify-between gap-1.5">
                      <div className="flex items-center gap-2 min-w-0">
                        <div
                          className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border ${
                            pendingOrdersCount > 0
                              ? "border-amber-500/25 bg-amber-500/10"
                              : "border-emerald-500/25 bg-emerald-500/10"
                          }`}
                        >
                          {pendingOrdersCount > 0 ? (
                            <AlertCircle className="h-3.5 w-3.5 text-[#FFAF24]" />
                          ) : (
                            <CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />
                          )}
                        </div>
                        <span className="truncate text-[12px] font-medium text-[#CBD5E1]">
                          Store Health
                        </span>
                      </div>
                    </div>
                    <p className="mt-2.5 text-[20px] font-bold tracking-tight text-[#FFFCF8]">
                      {pendingOrdersCount > 0 ? `${urgentCount} to Review` : "Optimal"}
                    </p>
                  </div>
                  <p className="mt-1 text-[11.5px] text-[#94A3B8]">
                    {pendingOrdersCount > 0
                      ? `${pendingOrdersCount} pending · ${lowStockCount} low stock`
                      : "All orders confirmed"}
                  </p>
                </div>
              </div>

              {/* 3. Operational Attention Strip (attention-strip.tsx — subtle tinted surface, no left border) */}
              {pendingOrdersCount > 0 && (
                <div className="space-y-1.5">
                  <div className="flex items-center gap-1.5 px-0.5">
                    <AlertCircle className="h-3.5 w-3.5 text-[#FFAF24]" />
                    <span className="text-[10.5px] font-semibold uppercase tracking-[0.08em] text-[#CBD5E1]">
                      Action Required
                    </span>
                  </div>
                  <div className="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
                    <button
                      type="button"
                      onClick={() => setView("pending")}
                      className="flex items-center justify-between rounded-xl border border-[#FFFCF8]/[0.08] bg-amber-500/[0.08] px-3.5 py-2.5 text-left transition-colors hover:border-amber-500/30"
                    >
                      <div className="flex items-center gap-2.5 min-w-0">
                        <div className="rounded-lg border border-[#FFFCF8]/10 bg-[#0F1115] p-1.5 shrink-0">
                          <ShoppingCart className="h-3.5 w-3.5 text-[#FFAF24]" />
                        </div>
                        <div className="min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="rounded-md border border-[#FFFCF8]/12 bg-[#0F1115] px-1.5 py-0.5 font-mono text-[11px] font-bold tabular-nums text-[#FFFCF8]">
                              {pendingOrdersCount}
                            </span>
                            <span className="truncate text-[12.5px] font-semibold text-[#FFFCF8]">
                              Orders Awaiting Processing
                            </span>
                          </div>
                          <p className="truncate text-[11px] text-[#94A3B8]">
                            Confirm social &amp; storefront orders before shipping
                          </p>
                        </div>
                      </div>
                      <ArrowRight className="ml-2 h-3.5 w-3.5 shrink-0 text-[#94A3B8]" />
                    </button>

                    <div className="flex items-center justify-between rounded-xl border border-[#FFFCF8]/[0.08] bg-rose-500/[0.07] px-3.5 py-2.5">
                      <div className="flex items-center gap-2.5 min-w-0">
                        <div className="rounded-lg border border-[#FFFCF8]/10 bg-[#0F1115] p-1.5 shrink-0">
                          <Box className="h-3.5 w-3.5 text-rose-400" />
                        </div>
                        <div className="min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="rounded-md border border-[#FFFCF8]/12 bg-[#0F1115] px-1.5 py-0.5 font-mono text-[11px] font-bold tabular-nums text-[#FFFCF8]">
                              {lowStockCount}
                            </span>
                            <span className="truncate text-[12.5px] font-semibold text-[#FFFCF8]">
                              Low Stock Inventory Alerts
                            </span>
                          </div>
                          <p className="truncate text-[11px] text-[#94A3B8]">
                            Items with 5 or fewer units left in stock
                          </p>
                        </div>
                      </div>
                      <ArrowUpRight className="ml-2 h-3.5 w-3.5 shrink-0 text-[#94A3B8]" />
                    </div>
                  </div>
                </div>
              )}

              {/* 4. Recent Orders Card (recent-orders-card.tsx) */}
              <div className="overflow-hidden rounded-xl border border-[#FFFCF8]/[0.09] bg-[#14171D]">
                <div className="flex flex-wrap items-center justify-between gap-2 border-b border-[#FFFCF8]/[0.08] bg-[#181C23]/60 px-4 py-3">
                  <div className="flex items-center gap-2.5">
                    <div className="flex h-7 w-7 items-center justify-center rounded-lg border border-[#FFFCF8]/10 bg-[#1B1E24]">
                      <ShoppingCart className="h-3.5 w-3.5 text-[#CBD5E1]" />
                    </div>
                    <div>
                      <h3 className="text-[13px] font-semibold text-[#FFFCF8]">Recent Orders</h3>
                      <p className="text-[11px] text-[#94A3B8]">
                        {toastText ?? "Live incoming customer orders — try confirming one"}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5">
                    {(
                      [
                        { id: "all", label: "All" },
                        { id: "pending", label: `Pending (${pendingOrdersCount})` },
                        { id: "social", label: "Social" },
                      ] as const
                    ).map((tab) => (
                      <button
                        key={tab.id}
                        type="button"
                        onClick={() => setView(tab.id)}
                        className={`shrink-0 whitespace-nowrap rounded-md px-2.5 py-1 text-[11.5px] font-medium transition-colors ${
                          view === tab.id
                            ? "bg-[#FF7A00] text-[#080A0C] font-semibold"
                            : "bg-[#0F1115] text-[#94A3B8] hover:text-[#FFFCF8]"
                        }`}
                      >
                        {tab.label}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="divide-y divide-[#FFFCF8]/[0.07]">
                  {visibleOrders.length === 0 ? (
                    <div className="px-4 py-5 text-center text-[12.5px] text-[#94A3B8]">
                      All pending orders are confirmed!{" "}
                      <button
                        type="button"
                        onClick={() => setView("all")}
                        className="font-medium text-[#FFAF24] underline"
                      >
                        Show all orders
                      </button>
                    </div>
                  ) : (
                    visibleOrders.map((order) => {
                      const tracking = bookedMap[order.id]
                      return (
                        <div
                          key={order.id}
                          className="flex flex-wrap items-center justify-between gap-2 px-4 py-2.5 transition-colors hover:bg-[#1B1E24]/50"
                        >
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                              <span className="shrink-0 whitespace-nowrap text-[12.5px] font-semibold text-[#FFFCF8]">
                                {order.displayId}
                              </span>
                              <span className="truncate text-[12.5px] text-[#CBD5E1]">
                                {order.customer}
                              </span>
                              <span className="hidden truncate text-[11.5px] text-[#64748B] sm:inline">
                                · {order.itemSummary}
                              </span>
                            </div>
                            <p className="mt-0.5 text-[11px] text-[#94A3B8]">
                              {order.timeAgo} · {order.source} · {order.area}
                            </p>
                          </div>

                          <div className="flex shrink-0 items-center gap-2.5">
                            <span className="font-mono text-[12.5px] font-semibold tabular-nums text-[#FFFCF8]">
                              {order.amountFormatted}
                            </span>

                            {order.status === "pending" ? (
                              <button
                                type="button"
                                onClick={() => handleConfirmOrder(order)}
                                className="inline-flex shrink-0 items-center gap-1 rounded-md bg-[#FF7A00] px-2.5 py-1 text-[11.5px] font-semibold text-[#080A0C] transition-colors hover:bg-[#D95700]"
                              >
                                <Check className="h-3 w-3" />
                                Confirm
                              </button>
                            ) : tracking ? (
                              <span className="shrink-0 rounded-md border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 font-mono text-[11px] text-emerald-400">
                                {order.courier} · {tracking}
                              </span>
                            ) : (
                              <button
                                type="button"
                                onClick={() => handleBookCourier(order)}
                                className="inline-flex shrink-0 items-center gap-1 rounded-md border border-[#FFFCF8]/12 bg-[#1B1E24] px-2.5 py-1 text-[11.5px] font-medium text-[#FFFCF8] transition-colors hover:border-[#FF7A00]/60"
                              >
                                <Truck className="h-3 w-3 text-[#FFAF24]" />
                                Book {order.courier}
                              </button>
                            )}
                          </div>
                        </div>
                      )
                    })
                  )}
                </div>
              </div>

              {/* 5. Store Management Quick Shortcuts (quick-operations-card.tsx) */}
              <div className="hidden overflow-hidden rounded-xl border border-[#FFFCF8]/[0.08] bg-[#14171D] md:block">
                <div className="flex items-center justify-between border-b border-[#FFFCF8]/[0.07] px-4 py-2">
                  <div className="flex items-center gap-1.5">
                    <Sparkles className="h-3.5 w-3.5 text-[#94A3B8]" />
                    <span className="text-[12px] font-semibold text-[#FFFCF8]">
                      Store Management
                    </span>
                  </div>
                  <span className="text-[11px] text-[#64748B]">Quick shortcuts</span>
                </div>
                <div className="grid grid-cols-4 divide-x divide-[#FFFCF8]/[0.07]">
                  {QUICK_SHORTCUTS.map((item) => {
                    const Icon = item.icon
                    return (
                      <a
                        key={item.title}
                        href="#operations"
                        className="flex items-center gap-2.5 px-3 py-2.5 transition-colors hover:bg-[#1B1E24]/60"
                      >
                        <div className="rounded-md border border-[#FFFCF8]/[0.08] bg-[#1B1E24] p-1.5 shrink-0">
                          <Icon className={`h-3.5 w-3.5 ${item.iconColor}`} />
                        </div>
                        <div className="min-w-0">
                          <p className="truncate text-[11.5px] font-semibold text-[#FFFCF8]">
                            {item.title}
                          </p>
                          <p className="truncate text-[10.5px] text-[#94A3B8]">
                            {item.description}
                          </p>
                        </div>
                      </a>
                    )
                  })}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
