import {
  ArrowDownTray,
  ArrowPath,
  CurrencyDollar,
  ReceiptPercent,
  ShoppingCart,
  Tag,
} from "@medusajs/icons"
import { Badge, Button, Container, Heading, Select, Table, Text } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useEffect, useMemo, useState } from "react"
import { TimeSeriesChart } from "../../components/charts/time-series-chart"
import { SingleColumnPage } from "../../components/layout/pages"
import { useCurrencies } from "../../hooks/api/currencies"
import { getCurrencySymbol } from "../../lib/data/currencies"
import { useStore } from "../../hooks/api/store"
import { sdk } from "../../lib/client"

// @medusajs/ui types Table.Cell as HTMLAttributes rather than TdHTMLAttributes, so
// colSpan — valid on any <td> and working at runtime — is missing from the prop type.
// Passed through here until the upstream type is widened.
const spanAllColumns = { colSpan: 4 } as React.TdHTMLAttributes<HTMLTableCellElement>

type TimeWindow = "7" | "30" | "90" | "365"

interface ReportResponse<T> {
  report: {
    report_type: string
    window_days?: number
    data: T
  }
}

interface RevenueData {
  currency: string
  total_revenue: string
  time_series?: Array<{
    date: string
    revenue: string
  }>
}

interface OrdersData {
  total_orders: number
  total_units: number
  time_series?: Array<{
    date: string
    order_count: number
  }>
}

interface TopProductItem {
  product_id: string
  title: string
  units_sold: number
  total_revenue: string
}

interface TopProductsData {
  top_products: TopProductItem[]
}

interface AovData {
  currency: string
  average_order_value: string
  order_count: number
  total_revenue: string
}

export const AnalyticsDashboard = () => {
  const { store } = useStore()
  const { currencies: registeredCurrencies } = useCurrencies()

  const availableCurrencies = useMemo(() => {
    const list: Array<{ code: string; label: string }> = []
    const seen = new Set<string>()

    if (store?.supported_currencies) {
      for (const sc of store.supported_currencies) {
        const code = sc.currency_code?.toUpperCase()
        if (code && !seen.has(code)) {
          seen.add(code)
          const sym = sc.currency?.symbol_native || sc.currency?.symbol || code
          list.push({ code, label: `${code} (${sym})` })
        }
      }
    }

    if (registeredCurrencies) {
      for (const c of registeredCurrencies) {
        const code = c.code?.toUpperCase()
        if (code && !seen.has(code)) {
          seen.add(code)
          const sym = c.symbol_native || c.symbol || code
          list.push({ code, label: `${code} (${sym})` })
        }
      }
    }

    const defaults = [
      { code: "BDT", label: "৳" },
      { code: "USD", label: "$" },
      { code: "EUR", label: "€" },
      { code: "GBP", label: "GBP (£)" },
    ]
    for (const d of defaults) {
      if (!seen.has(d.code)) {
        seen.add(d.code)
        list.push(d)
      }
    }

    return list
  }, [store, registeredCurrencies])

  const [windowDays, setWindowDays] = useState<TimeWindow>("30")
  const [currency, setCurrency] = useState("USD")

  const defaultCurrencyCode =
    store?.supported_currencies?.find((c) => c.is_default)?.currency_code ||
    store?.supported_currencies?.[0]?.currency_code

  useEffect(() => {
    if (defaultCurrencyCode) {
      setCurrency(defaultCurrencyCode.toUpperCase())
    }
  }, [defaultCurrencyCode])

  // Fetch Revenue Over Time
  const {
    data: revenueData,
    refetch: refetchRevenue,
  } = useQuery<ReportResponse<RevenueData>>({
    queryKey: ["reports", "revenue", windowDays, currency],
    queryFn: async () => {
      return sdk.client.fetch(`/admin/reports/revenue-over-time?window_days=${windowDays}&currency=${currency}`, { method: "GET" })
    },
  })

  // Fetch Orders Over Time
  const {
    data: ordersData,
    refetch: refetchOrders,
  } = useQuery<ReportResponse<OrdersData>>({
    queryKey: ["reports", "orders", windowDays],
    queryFn: async () => {
      return sdk.client.fetch(`/admin/reports/orders-over-time?window_days=${windowDays}`, { method: "GET" })
    },
  })

  // Fetch Top Products
  const {
    data: topProductsData,
    isLoading: isTopProductsLoading,
    refetch: refetchTopProducts,
  } = useQuery<ReportResponse<TopProductsData>>({
    queryKey: ["reports", "top-products", windowDays],
    queryFn: async () => {
      return sdk.client.fetch(`/admin/reports/top-products?window_days=${windowDays}&limit=10`, { method: "GET" })
    },
  })

  // Fetch Average Order Value (AOV)
  const {
    data: aovData,
    refetch: refetchAov,
  } = useQuery<ReportResponse<AovData>>({
    queryKey: ["reports", "aov", windowDays, currency],
    queryFn: async () => {
      return sdk.client.fetch(`/admin/reports/average-order-value?window_days=${windowDays}&currency=${currency}`, { method: "GET" })
    },
  })

  const handleRefreshAll = () => {
    refetchRevenue()
    refetchOrders()
    refetchTopProducts()
    refetchAov()
  }

  const exportCsv = async (type: string) => {
    // `window.open` cannot attach the bearer token, so it downloaded a 401 body as a
    // .csv. Fetch it through the SDK and hand the browser a blob instead.
    // The SDK hands back the raw Response for non-JSON requests.
    const response = await sdk.client.fetch<Response>(
      `/admin/reports/${type}?window_days=${windowDays}&currency=${currency}&format=csv`,
      { method: "GET", headers: { accept: "text/csv" } }
    )
    const url = URL.createObjectURL(await response.blob())
    const link = document.createElement("a")
    link.href = url
    link.download = `${type}-${windowDays}d.csv`
    link.click()
    URL.revokeObjectURL(url)
  }

  const currencySymbol = getCurrencySymbol(currency)
  const currencyFormat = new Intl.NumberFormat(undefined, {
    style: "decimal",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })
  const formatCurrency = (value: number) => `${currencySymbol}${currencyFormat.format(value)}`

  // These series have been in the API response since the reporting module landed;
  // the screen only ever rendered the totals and threw the rest away.
  const revenueSeries = (revenueData?.report?.data?.time_series ?? []).map((p) => ({
    date: p.date,
    value: parseFloat(p.revenue || "0"),
  }))
  const ordersSeries = (ordersData?.report?.data?.time_series ?? []).map((p) => ({
    date: p.date,
    value: p.order_count,
  }))

  const totalRevenue = parseFloat(revenueData?.report?.data?.total_revenue || "0")
  const totalOrders = ordersData?.report?.data?.total_orders || 0
  const aovValue = parseFloat(aovData?.report?.data?.average_order_value || "0")
  const topProducts = topProductsData?.report?.data?.top_products || []
  const topRevenue = Math.max(
    ...topProducts.map((p) => parseFloat(p.total_revenue || "0")),
    1
  )

  return (
    <SingleColumnPage
      widgets={{
        after: [],
        before: [],
      }}
    >
      <div className="flex flex-col gap-y-6">
        {/* Header and Controls */}
        <div className="flex flex-col gap-y-4 md:flex-row md:items-center md:justify-between">
          <div>
            <Heading level="h1" className="text-ui-fg-base">
              Analytics & Executive Reports
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Real-time revenue, order volume, catalog performance, and average order value metrics.
            </Text>
          </div>
          <div className="flex items-center gap-x-3">
            <Select
              value={currency}
              onValueChange={(val) => setCurrency(val)}
              size="small"
            >
              <Select.Trigger className="w-[130px]">
                <Select.Value />
              </Select.Trigger>
              <Select.Content>
                {availableCurrencies.map((c) => (
                  <Select.Item key={c.code} value={c.code}>
                    {c.label}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select>

            <Select
              value={windowDays}
              onValueChange={(val) => setWindowDays(val as TimeWindow)}
              size="small"
            >
              <Select.Trigger className="w-[140px]">
                <Select.Value />
              </Select.Trigger>
              <Select.Content>
                <Select.Item value="7">Last 7 Days</Select.Item>
                <Select.Item value="30">Last 30 Days</Select.Item>
                <Select.Item value="90">Last 90 Days</Select.Item>
                <Select.Item value="365">Last 1 Year</Select.Item>
              </Select.Content>
            </Select>

            <Button
              variant="secondary"
              size="small"
              onClick={handleRefreshAll}
            >
              <ArrowPath className="text-ui-fg-subtle" />
            </Button>
          </div>
        </div>

        {/* Top Summary KPI Cards */}
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <Container className="p-4">
            <div className="flex items-center justify-between">
              <Text size="small" weight="plus" className="text-ui-fg-subtle">
                Total Revenue
              </Text>
              <div className="bg-ui-bg-subtle text-ui-fg-subtle flex h-8 w-8 items-center justify-center rounded-md">
                <CurrencyDollar />
              </div>
            </div>
            <div className="mt-3 flex items-baseline gap-x-2">
              <Text size="xlarge" weight="plus" className="text-ui-fg-base">
                {formatCurrency(totalRevenue)}
              </Text>
              <Badge size="xsmall" color="blue">Last {windowDays}d</Badge>
            </div>
            <Text size="xsmall" className="text-ui-fg-muted mt-1">
              Window: past {windowDays} days
            </Text>
          </Container>

          <Container className="p-4">
            <div className="flex items-center justify-between">
              <Text size="small" weight="plus" className="text-ui-fg-subtle">
                Total Orders
              </Text>
              <div className="bg-ui-bg-subtle text-ui-fg-subtle flex h-8 w-8 items-center justify-center rounded-md">
                <ShoppingCart />
              </div>
            </div>
            <div className="mt-3 flex items-baseline gap-x-2">
              <Text size="xlarge" weight="plus" className="text-ui-fg-base">
                {totalOrders.toLocaleString()}
              </Text>
              <Badge size="xsmall" color="blue">Volume</Badge>
            </div>
            <Text size="xsmall" className="text-ui-fg-muted mt-1">
              Confirmed checkout orders
            </Text>
          </Container>

          <Container className="p-4">
            <div className="flex items-center justify-between">
              <Text size="small" weight="plus" className="text-ui-fg-subtle">
                Average Order Value
              </Text>
              <div className="bg-ui-bg-subtle text-ui-fg-subtle flex h-8 w-8 items-center justify-center rounded-md">
                <ReceiptPercent />
              </div>
            </div>
            <div className="mt-3 flex items-baseline gap-x-2">
              <Text size="xlarge" weight="plus" className="text-ui-fg-base">
                {formatCurrency(aovValue)}
              </Text>
              <Badge size="xsmall" color="blue">Per basket</Badge>
            </div>
            <Text size="xsmall" className="text-ui-fg-muted mt-1">
              Gross revenue per customer basket
            </Text>
          </Container>

          <Container className="p-4">
            <div className="flex items-center justify-between">
              <Text size="small" weight="plus" className="text-ui-fg-subtle">
                Export Reports
              </Text>
              <div className="bg-ui-bg-subtle text-ui-fg-subtle flex h-8 w-8 items-center justify-center rounded-md">
                <ArrowDownTray />
              </div>
            </div>
            <div className="mt-3 flex flex-wrap gap-2">
              <Button
                variant="secondary"
                size="small"
                className="text-xs"
                onClick={() => exportCsv("revenue-over-time")}
              >
                Revenue CSV
              </Button>
              <Button
                variant="secondary"
                size="small"
                className="text-xs"
                onClick={() => exportCsv("orders-over-time")}
              >
                Orders CSV
              </Button>
            </div>
          </Container>
        </div>

        {/* Revenue and orders each get their own plot: two measures on one pair of
            axes would be a dual-axis chart, which is unreadable at any scale. */}
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <Container className="p-0">
            <div className="border-ui-border-base border-b px-6 py-4">
              <Heading level="h2" className="text-ui-fg-base">
                Revenue over time
              </Heading>
              <Text size="small" className="text-ui-fg-subtle">
                Gross revenue per day in {currency === "BDT" ? "৳" : currency === "USD" ? "$" : "€"}.
              </Text>
            </div>
            <div className="px-4 pb-4 pt-6">
              <TimeSeriesChart
                data={revenueSeries}
                label={`Revenue per day in ${currency === "BDT" ? "৳" : currency === "USD" ? "$" : "€"}`}
                format={(value) => formatCurrency(value)}
              />
            </div>
          </Container>

          <Container className="p-0">
            <div className="border-ui-border-base border-b px-6 py-4">
              <Heading level="h2" className="text-ui-fg-base">
                Orders over time
              </Heading>
              <Text size="small" className="text-ui-fg-subtle">
                Confirmed orders per day.
              </Text>
            </div>
            <div className="px-4 pb-4 pt-6">
              <TimeSeriesChart
                data={ordersSeries}
                label="Orders per day"
                format={(value) => `${value.toLocaleString()} orders`}
              />
            </div>
          </Container>
        </div>

        {/* Top Products Leaderboard */}
        <Container className="p-0 overflow-hidden">
          <div className="flex items-center justify-between p-6 border-b border-ui-border-base">
            <div>
              <Heading level="h2" className="text-ui-fg-base">
                Top Performing Products
              </Heading>
              <Text size="small" className="text-ui-fg-subtle">
                Ranked by revenue contribution across all sales channels.
              </Text>
            </div>
            <Button
              variant="secondary"
              size="small"
              onClick={() => exportCsv("top-products")}
            >
              <ArrowDownTray className="text-ui-fg-subtle me-1.5" />
              Export CSV
            </Button>
          </div>

          <Table>
            <Table.Header>
              <Table.Row>
                <Table.HeaderCell>Product</Table.HeaderCell>
                <Table.HeaderCell className="text-right">Units Sold</Table.HeaderCell>
                <Table.HeaderCell className="text-right">Revenue Contribution</Table.HeaderCell>
                <Table.HeaderCell className="text-center">Status</Table.HeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {isTopProductsLoading ? (
                <Table.Row>
                  <Table.Cell {...spanAllColumns} className="text-center py-8 text-ui-fg-muted">
                    Loading top products metrics...
                  </Table.Cell>
                </Table.Row>
              ) : topProducts.length === 0 ? (
                <Table.Row>
                  <Table.Cell {...spanAllColumns} className="text-center py-8 text-ui-fg-muted">
                    No product transactions recorded in this time window.
                  </Table.Cell>
                </Table.Row>
              ) : (
                topProducts.map((p, idx) => (
                  <Table.Row key={p.product_id || idx}>
                    <Table.Cell className="font-medium text-ui-fg-base">
                      <div className="flex items-center gap-x-2">
                        <Tag className="text-ui-fg-subtle" />
                        <span>{p.title || `Product (${p.product_id})`}</span>
                      </div>
                    </Table.Cell>
                    <Table.Cell className="text-ui-fg-subtle text-right font-mono tabular-nums">
                      {p.units_sold || 0}
                    </Table.Cell>
                    <Table.Cell>
                      {/* The bar encodes revenue because revenue is what the rows are
                          ranked by; the table row is the accessible view of the same
                          numbers. */}
                      <div className="flex items-center justify-end gap-x-3">
                        <div className="bg-ui-bg-subtle h-2 w-28 overflow-hidden rounded-full">
                          <div
                            className="h-full rounded-full bg-[#3B82F6] dark:bg-[#5591F5]"
                            style={{
                              width: `${Math.max((parseFloat(p.total_revenue || "0") / topRevenue) * 100, 2)}%`,
                            }}
                          />
                        </div>
                        <span className="text-ui-fg-base w-24 text-right font-mono font-semibold tabular-nums">
                          {formatCurrency(parseFloat(p.total_revenue || "0"))}
                        </span>
                      </div>
                    </Table.Cell>
                    <Table.Cell className="text-center">
                      <Badge size="xsmall" color={idx === 0 ? "green" : "grey"}>
                        #{idx + 1} Rank
                      </Badge>
                    </Table.Cell>
                  </Table.Row>
                ))
              )}
            </Table.Body>
          </Table>
        </Container>
      </div>
    </SingleColumnPage>
  )
}


