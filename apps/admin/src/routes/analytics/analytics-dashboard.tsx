import {
  ArrowDownTray,
  ArrowPath,
  CurrencyDollar,
  ReceiptPercent,
  ShoppingCart,
  Tag,
  Users,
} from "@medusajs/icons"
import { Badge, Button, Container, Heading, Select, Table, Text, clx } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { SingleColumnPage } from "../../components/layout/pages"

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
  quantity_sold: number
  revenue: string
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
  const { t } = useTranslation()
  const [windowDays, setWindowDays] = useState<TimeWindow>("30")
  const [currency, setCurrency] = useState("USD")

  // Fetch Revenue Over Time
  const {
    data: revenueData,
    isLoading: isRevenueLoading,
    refetch: refetchRevenue,
  } = useQuery<ReportResponse<RevenueData>>({
    queryKey: ["reports", "revenue", windowDays, currency],
    queryFn: async () => {
      const res = await fetch(
        `/admin/reports/revenue-over-time?window_days=${windowDays}&currency=${currency}`
      )
      if (!res.ok) throw new Error("Failed to fetch revenue report")
      return res.json()
    },
  })

  // Fetch Orders Over Time
  const {
    data: ordersData,
    isLoading: isOrdersLoading,
    refetch: refetchOrders,
  } = useQuery<ReportResponse<OrdersData>>({
    queryKey: ["reports", "orders", windowDays],
    queryFn: async () => {
      const res = await fetch(
        `/admin/reports/orders-over-time?window_days=${windowDays}`
      )
      if (!res.ok) throw new Error("Failed to fetch orders report")
      return res.json()
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
      const res = await fetch(
        `/admin/reports/top-products?window_days=${windowDays}&limit=10`
      )
      if (!res.ok) throw new Error("Failed to fetch top products report")
      return res.json()
    },
  })

  // Fetch Average Order Value (AOV)
  const {
    data: aovData,
    isLoading: isAovLoading,
    refetch: refetchAov,
  } = useQuery<ReportResponse<AovData>>({
    queryKey: ["reports", "aov", windowDays, currency],
    queryFn: async () => {
      const res = await fetch(
        `/admin/reports/average-order-value?window_days=${windowDays}&currency=${currency}`
      )
      if (!res.ok) throw new Error("Failed to fetch AOV report")
      return res.json()
    },
  })

  const handleRefreshAll = () => {
    refetchRevenue()
    refetchOrders()
    refetchTopProducts()
    refetchAov()
  }

  const exportCsv = (type: string) => {
    window.open(
      `/admin/reports/${type}?window_days=${windowDays}&currency=${currency}&format=csv`,
      "_blank"
    )
  }

  const totalRevenue = parseFloat(revenueData?.report?.data?.total_revenue || "0")
  const totalOrders = ordersData?.report?.data?.total_orders || 0
  const aovValue = parseFloat(aovData?.report?.data?.average_order_value || "0")
  const topProducts = topProductsData?.report?.data?.top_products || []

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
              <Select.Trigger className="w-[100px]">
                <Select.Value />
              </Select.Trigger>
              <Select.Content>
                <Select.Item value="USD">USD ($)</Select.Item>
                <Select.Item value="EUR">EUR (€)</Select.Item>
                <Select.Item value="GBP">GBP (£)</Select.Item>
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
                ${totalRevenue.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
              </Text>
              <Badge size="xsmall" color="green">Live</Badge>
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
                {totalOrders}
              </Text>
              <Badge size="xsmall" color="blue">Tracked</Badge>
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
                ${aovValue.toFixed(2)}
              </Text>
              <Badge size="xsmall" color="purple">AOV</Badge>
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

        {/* Top Products Leaderboard */}
        <Container className="p-0 overflow-hidden">
          <div className="flex items-center justify-between p-6 border-b border-ui-border-base">
            <div>
              <Heading level="h2" className="text-ui-fg-base">
                Top Performing Products
              </Heading>
              <Text size="small" className="text-ui-fg-subtle">
                Ranked by unit sales volume and revenue generation across all sales channels.
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
                  <Table.Cell colSpan={4} className="text-center py-8 text-ui-fg-muted">
                    Loading top products metrics...
                  </Table.Cell>
                </Table.Row>
              ) : topProducts.length === 0 ? (
                <Table.Row>
                  <Table.Cell colSpan={4} className="text-center py-8 text-ui-fg-muted">
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
                    <Table.Cell className="text-right font-mono text-ui-fg-subtle">
                      {p.quantity_sold || 0}
                    </Table.Cell>
                    <Table.Cell className="text-right font-mono text-ui-fg-base font-semibold">
                      ${parseFloat(p.revenue || "0").toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
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
