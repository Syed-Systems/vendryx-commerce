/**
 * Deterministic in-memory mock backend for Vendryx / Stheno Admin Dashboard
 * when the Rust engine is not running in this container.
 */

function buildTimeSeries(windowDays) {
  const days = Math.max(7, Math.min(Number(windowDays) || 30, 365))
  const points = Math.min(days, 30)
  const revenueSeries = []
  const orderSeries = []
  const now = new Date("2026-11-25T12:00:00.000Z")

  for (let i = points - 1; i >= 0; i--) {
    const d = new Date(now.getTime() - i * 86400000)
    const date = d.toISOString().slice(0, 10)
    const wave = ((points - i) * 7) % 11
    const orders = 4 + (wave % 6)
    const revenue = (orders * 64.5 + wave * 12).toFixed(2)
    revenueSeries.push({ date, revenue })
    orderSeries.push({ date, order_count: orders })
  }
  return { revenueSeries, orderSeries }
}

export function createMockApiState() {
  const now = "2026-11-25T10:00:00.000Z"

  return {
    authenticated: true,
    user: {
      id: "usr_01JSTHENOOWNER",
      email: "owner@vendryx.store",
      first_name: "Amina",
      last_name: "Rahman",
      avatar_url: null,
      metadata: { role: "owner" },
      created_at: "2026-09-01T08:00:00.000Z",
      updated_at: now,
      deleted_at: null,
    },
    store: {
      id: "store_01JSTHENO",
      name: "Vendryx Flagship Store",
      default_sales_channel_id: "sc_01JSTHENO",
      default_region_id: "reg_01JSTHENO",
      default_location_id: "sloc_01JSTHENO",
      supported_currencies: [
        {
          currency_code: "usd",
          is_default: true,
          currency: {
            code: "usd",
            symbol: "$",
            symbol_native: "$",
            name: "US Dollar",
          },
        },
        {
          currency_code: "bdt",
          is_default: false,
          currency: {
            code: "bdt",
            symbol: "৳",
            symbol_native: "৳",
            name: "Bangladeshi Taka",
          },
        },
      ],
      created_at: "2026-09-01T08:00:00.000Z",
      updated_at: now,
    },
    branding: {
      id: "brand_01JSTHENO",
      store_name: "Vendryx Flagship Store",
      font_key: "inter",
      color_primary: "#C2410C",
      color_secondary: "#18181B",
      color_accent: "#EA580C",
      color_background: "#FCFBF9",
      color_surface: "#FFFFFF",
      color_text: "#18181B",
      logo_url: "/brand/vendryx/vendryx-mark.png",
      favicon_url: "/favicon.png",
      updated_at: now,
    },
    onboarding: {
      is_completed: false,
      dismissed: false,
      completion_percentage: 60,
      completed_steps_count: 3,
      total_steps_count: 5,
      steps: [
        {
          id: "store-profile",
          title: "Configure store identity",
          description: "Set your store name, default currency, and timezone.",
          completed: true,
          href: "/settings/branding",
        },
        {
          id: "first-product",
          title: "Add your first product",
          description: "Publish catalog items with pricing and inventory.",
          completed: true,
          href: "/categories",
        },
        {
          id: "courier-setup",
          title: "Connect courier logistics",
          description: "Enable Pathao, Steadfast, or RedX delivery zones.",
          completed: true,
          href: "/settings/couriers",
        },
        {
          id: "custom-domain",
          title: "Connect a custom domain",
          description: "Point your branded domain and provision SSL.",
          completed: false,
          href: "/settings/domains",
        },
        {
          id: "launch-campaign",
          title: "Launch a seasonal campaign",
          description: "Set a promotional budget and share discount codes.",
          completed: false,
          href: "/campaigns",
        },
      ],
    },
    orders: [
      {
        id: "order_01J1001",
        display_id: 1048,
        currency_code: "usd",
        total: "184.00",
        status: "pending",
        fulfillment_status: "not_fulfilled",
        payment_status: "captured",
        created_at: "2026-11-25T09:42:00.000Z",
        shipping_address: { first_name: "Nadia", last_name: "Karim" },
      },
      {
        id: "order_01J1002",
        display_id: 1047,
        currency_code: "usd",
        total: "92.50",
        status: "pending",
        fulfillment_status: "not_fulfilled",
        payment_status: "authorized",
        created_at: "2026-11-25T08:15:00.000Z",
        shipping_address: { first_name: "Tariq", last_name: "Hasan" },
      },
      {
        id: "order_01J1003",
        display_id: 1046,
        currency_code: "usd",
        total: "310.00",
        status: "completed",
        fulfillment_status: "shipped",
        payment_status: "captured",
        created_at: "2026-11-24T19:20:00.000Z",
        shipping_address: { first_name: "Farhana", last_name: "Islam" },
      },
      {
        id: "order_01J1004",
        display_id: 1045,
        currency_code: "usd",
        total: "68.00",
        status: "completed",
        fulfillment_status: "delivered",
        payment_status: "captured",
        created_at: "2026-11-24T14:05:00.000Z",
        shipping_address: { first_name: "Imran", last_name: "Chowdhury" },
      },
    ],
    customers: [
      {
        id: "cus_01J001",
        email: "nadia.karim@example.com",
        first_name: "Nadia",
        last_name: "Karim",
        company_name: "Atelier Dhaka",
        phone: "+8801711000101",
        has_account: true,
        groups: [{ id: "cg_vip", name: "VIP Members" }],
        addresses: [],
        created_at: "2026-10-12T10:00:00.000Z",
        updated_at: now,
      },
      {
        id: "cus_01J002",
        email: "tariq.hasan@example.com",
        first_name: "Tariq",
        last_name: "Hasan",
        company_name: null,
        phone: "+8801711000102",
        has_account: true,
        groups: [],
        addresses: [],
        created_at: "2026-10-28T11:30:00.000Z",
        updated_at: now,
      },
      {
        id: "cus_01J003",
        email: "farhana.islam@example.com",
        first_name: "Farhana",
        last_name: "Islam",
        company_name: "Loom Studio",
        phone: "+8801711000103",
        has_account: false,
        groups: [{ id: "cg_vip", name: "VIP Members" }],
        addresses: [],
        created_at: "2026-11-10T09:15:00.000Z",
        updated_at: now,
      },
    ],
    customer_groups: [
      {
        id: "cg_vip",
        name: "VIP Members",
        customers: [{ id: "cus_01J001" }, { id: "cus_01J003" }],
        metadata: {},
        created_at: "2026-09-15T10:00:00.000Z",
        updated_at: now,
      },
      {
        id: "cg_wholesale",
        name: "Wholesale Buyers",
        customers: [],
        metadata: {},
        created_at: "2026-10-02T10:00:00.000Z",
        updated_at: now,
      },
    ],
    product_categories: [
      {
        id: "pcat_01",
        name: "Apparel",
        handle: "apparel",
        description: "Everyday essentials and outerwear.",
        is_active: true,
        is_internal: false,
        rank: 0,
        parent_category_id: null,
        category_children: [],
        created_at: "2026-09-05T10:00:00.000Z",
        updated_at: now,
      },
      {
        id: "pcat_02",
        name: "Home Goods",
        handle: "home-goods",
        description: "Handcrafted ceramics and textiles.",
        is_active: true,
        is_internal: false,
        rank: 1,
        parent_category_id: null,
        category_children: [],
        created_at: "2026-09-06T10:00:00.000Z",
        updated_at: now,
      },
    ],
    collections: [
      {
        id: "pcol_01",
        title: "Autumn Edit 2026",
        handle: "autumn-edit-2026",
        products: [{ id: "prod_01" }, { id: "prod_02" }],
        created_at: "2026-09-20T10:00:00.000Z",
        updated_at: now,
      },
      {
        id: "pcol_02",
        title: "Core Staples",
        handle: "core-staples",
        products: [{ id: "prod_03" }],
        created_at: "2026-10-01T10:00:00.000Z",
        updated_at: now,
      },
    ],
    campaigns: [
      {
        id: "camp_01",
        name: "Winter Preview",
        campaign_identifier: "WINTER26",
        description: "Early-access campaign for repeat customers.",
        starts_at: "2026-11-01T00:00:00.000Z",
        ends_at: "2026-12-31T23:59:59.000Z",
        budget: {
          type: "spend",
          limit: 5000,
          used: 1420,
          currency_code: "usd",
        },
        created_at: "2026-10-25T10:00:00.000Z",
        updated_at: now,
      },
    ],
    couriers: [
      {
        id: "pathao",
        name: "Pathao Courier",
        status: "connected",
        is_enabled: true,
        webhook_url: "/webhooks/couriers/pathao",
        fields: [
          { key: "client_id", label: "Client ID", kind: "text", required: true },
          { key: "client_secret", label: "Client Secret", kind: "secret", required: true },
        ],
        credentials: { client_id: "pathao_live_demo" },
        rates: { inside_dhaka: 60, sub_dhaka: 100, outside_dhaka: 130 },
        last_probe: {
          verified: true,
          status: "connected",
          latency_ms: 84,
          message: "Credentials verified",
          probed_at: now,
        },
      },
      {
        id: "steadfast",
        name: "Steadfast Courier",
        status: "not_configured",
        is_enabled: false,
        webhook_url: "/webhooks/couriers/steadfast",
        fields: [
          { key: "api_key", label: "API Key", kind: "secret", required: true },
          { key: "secret_key", label: "Secret Key", kind: "secret", required: true },
        ],
        credentials: {},
        rates: { inside_dhaka: 65, sub_dhaka: 100, outside_dhaka: 120 },
        last_probe: null,
      },
      {
        id: "redx",
        name: "RedX Logistics",
        status: "not_configured",
        is_enabled: false,
        webhook_url: "/webhooks/couriers/redx",
        fields: [{ key: "api_token", label: "API Token", kind: "secret", required: true }],
        credentials: {},
        rates: { inside_dhaka: 60, sub_dhaka: 95, outside_dhaka: 130 },
        last_probe: null,
      },
    ],
    domains: [
      {
        id: "dom_01",
        hostname: "shop.vendryx.store",
        status: "verified",
        ssl_status: "active",
        is_primary: true,
        verification_token: "stheno-verify-89f2a1",
        created_at: "2026-09-10T10:00:00.000Z",
        updated_at: now,
      },
    ],
    api_keys: [
      {
        id: "apk_01",
        title: "Storefront Web",
        type: "publishable",
        redacted: "pk_live_...48a9",
        token: "pk_live_demo_48a9",
        created_at: "2026-09-01T08:00:00.000Z",
        updated_at: now,
        revoked_at: null,
      },
    ],
    products: [
      { id: "prod_01", title: "Linen Overshirt", handle: "linen-overshirt", status: "published" },
      { id: "prod_02", title: "Stoneware Pour-Over Set", handle: "stoneware-pour-over", status: "published" },
      { id: "prod_03", title: "Merino Crewneck Knit", handle: "merino-crewneck", status: "published" },
    ],
  }
}

/**
 * Pure request dispatcher returning `{ status, body }`.
 */
export function handleMockApiRequest(state, { method = "GET", pathname = "/", searchParams = new URLSearchParams(), body = null }) {
  const upperMethod = method.toUpperCase()
  const now = new Date().toISOString()

  // Auth routes
  if (pathname.startsWith("/auth/")) {
    if (upperMethod === "DELETE") {
      state.authenticated = false
      return { status: 200, body: { success: true } }
    }
    state.authenticated = true
    return { status: 200, body: { token: "stheno_demo_jwt_token" } }
  }

  // Current user
  if (pathname === "/admin/users/me") {
    if (!state.authenticated) {
      return { status: 401, body: { message: "Unauthorized" } }
    }
    return { status: 200, body: { user: state.user } }
  }

  // Store
  if (pathname === "/admin/stores") {
    return { status: 200, body: { stores: [state.store], count: 1, limit: 20, offset: 0 } }
  }
  if (pathname.startsWith("/admin/stores/")) {
    if (upperMethod === "POST" && body && typeof body === "object") {
      Object.assign(state.store, body, { updated_at: now })
    }
    return { status: 200, body: { store: state.store } }
  }

  // Branding
  if (pathname === "/admin/store/branding") {
    if (upperMethod === "POST" && body && typeof body === "object") {
      state.branding = { ...state.branding, ...body, updated_at: now }
    }
    return { status: 200, body: { branding: state.branding } }
  }

  // Entitlements & feature flags
  if (pathname === "/admin/entitlements") {
    return {
      status: 200,
      body: {
        tier: "Pro",
        plan: "pro",
        status: "active",
        features: ["analytics", "couriers", "custom_domains", "campaigns", "branding"],
      },
    }
  }
  if (pathname === "/admin/feature-flags") {
    return {
      status: 200,
      body: {
        feature_flags: {
          view_configurations: false,
          translation: false,
          rbac: false,
        },
      },
    }
  }

  // Billing
  if (pathname === "/admin/billing/subscription") {
    return {
      status: 200,
      body: {
        subscription: {
          plan: "Pro",
          status: "active",
          amount: 4900,
          currency_code: "usd",
          current_period_end: "2026-12-25T00:00:00.000Z",
        },
      },
    }
  }
  if (pathname === "/admin/billing/invoices") {
    return { status: 200, body: { invoices: [] } }
  }

  // Home summary
  if (pathname === "/admin/home/summary") {
    const pendingItems = state.orders.filter((o) => o.status === "pending")
    return {
      status: 200,
      body: {
        pending_orders: { count: pendingItems.length, items: pendingItems },
        low_stock: {
          count: 1,
          threshold: 5,
          items: [
            {
              inventory_item_id: "iitem_01",
              location_id: "sloc_01JSTHENO",
              stocked_quantity: 4,
              reserved_quantity: 1,
              available_quantity: 3,
            },
          ],
        },
        errors: [],
      },
    }
  }

  // Onboarding
  if (pathname === "/admin/onboarding") {
    return { status: 200, body: { onboarding: state.onboarding } }
  }
  if (pathname === "/admin/onboarding/dismiss" && upperMethod === "POST") {
    state.onboarding.dismissed = true
    return { status: 200, body: { dismissed: true } }
  }
  if (pathname === "/admin/onboarding/reset" && upperMethod === "POST") {
    state.onboarding.dismissed = false
    return { status: 200, body: { dismissed: false } }
  }

  // Reports & Analytics
  if (pathname === "/admin/reports/today") {
    return {
      status: 200,
      body: {
        today: {
          revenue: "276.50",
          currency_code: "usd",
          order_count: 2,
          items_sold: 5,
        },
      },
    }
  }
  if (pathname === "/admin/reports/revenue-over-time") {
    const windowDays = Number(searchParams.get("window_days") || 30)
    const currency = (searchParams.get("currency") || "USD").toUpperCase()
    const { revenueSeries } = buildTimeSeries(windowDays)
    const total = revenueSeries.reduce((acc, p) => acc + Number(p.revenue), 0).toFixed(2)
    return {
      status: 200,
      body: {
        report: {
          report_type: "revenue_over_time",
          window_days: windowDays,
          data: {
            currency,
            total_revenue: total,
            time_series: revenueSeries,
          },
        },
      },
    }
  }
  if (pathname === "/admin/reports/orders-over-time") {
    const windowDays = Number(searchParams.get("window_days") || 30)
    const { orderSeries } = buildTimeSeries(windowDays)
    const totalOrders = orderSeries.reduce((acc, p) => acc + p.order_count, 0)
    return {
      status: 200,
      body: {
        report: {
          report_type: "orders_over_time",
          window_days: windowDays,
          data: {
            total_orders: totalOrders,
            total_units: Math.round(totalOrders * 2.1),
            time_series: orderSeries,
          },
        },
      },
    }
  }
  if (pathname === "/admin/reports/top-products") {
    const windowDays = Number(searchParams.get("window_days") || 30)
    return {
      status: 200,
      body: {
        report: {
          report_type: "top_products",
          window_days: windowDays,
          data: {
            top_products: [
              { product_id: "prod_01", title: "Linen Overshirt", units_sold: 48, total_revenue: "4320.00" },
              { product_id: "prod_02", title: "Stoneware Pour-Over Set", units_sold: 34, total_revenue: "2890.00" },
              { product_id: "prod_03", title: "Merino Crewneck Knit", units_sold: 26, total_revenue: "2600.00" },
            ],
          },
        },
      },
    }
  }
  if (pathname === "/admin/reports/average-order-value") {
    const windowDays = Number(searchParams.get("window_days") || 30)
    const currency = (searchParams.get("currency") || "USD").toUpperCase()
    return {
      status: 200,
      body: {
        report: {
          report_type: "average_order_value",
          window_days: windowDays,
          data: {
            currency,
            average_order_value: "74.80",
            order_count: 142,
            total_revenue: "10621.60",
          },
        },
      },
    }
  }

  // Orders
  if (pathname === "/admin/orders") {
    return { status: 200, body: { orders: state.orders, count: state.orders.length, limit: 20, offset: 0 } }
  }

  // Products
  if (pathname === "/admin/products") {
    return { status: 200, body: { products: state.products, count: state.products.length, limit: 20, offset: 0 } }
  }

  // Customers
  if (pathname === "/admin/customers") {
    if (upperMethod === "POST" && body && typeof body === "object") {
      const created = {
        id: `cus_${Date.now()}`,
        email: body.email || "customer@example.com",
        first_name: body.first_name || "",
        last_name: body.last_name || "",
        company_name: body.company_name || null,
        phone: body.phone || null,
        has_account: false,
        groups: [],
        addresses: [],
        created_at: now,
        updated_at: now,
      }
      state.customers.unshift(created)
      return { status: 200, body: { customer: created } }
    }
    return { status: 200, body: { customers: state.customers, count: state.customers.length, limit: 20, offset: 0 } }
  }
  if (pathname.startsWith("/admin/customers/")) {
    const id = pathname.split("/")[3]
    const customer = state.customers.find((c) => c.id === id) || state.customers[0]
    if (upperMethod === "POST" && body && customer) {
      Object.assign(customer, body, { updated_at: now })
    }
    return { status: 200, body: { customer } }
  }

  // Customer groups
  if (pathname === "/admin/customer-groups") {
    if (upperMethod === "POST" && body && typeof body === "object") {
      const created = {
        id: `cg_${Date.now()}`,
        name: body.name || "New Group",
        customers: [],
        metadata: body.metadata || {},
        created_at: now,
        updated_at: now,
      }
      state.customer_groups.unshift(created)
      return { status: 200, body: { customer_group: created } }
    }
    return {
      status: 200,
      body: { customer_groups: state.customer_groups, count: state.customer_groups.length, limit: 20, offset: 0 },
    }
  }

  // Categories
  if (pathname === "/admin/product-categories") {
    if (upperMethod === "POST" && body && typeof body === "object") {
      const created = {
        id: `pcat_${Date.now()}`,
        name: body.name || "New Category",
        handle: body.handle || `category-${Date.now()}`,
        description: body.description || "",
        is_active: body.is_active ?? true,
        is_internal: body.is_internal ?? false,
        rank: state.product_categories.length,
        parent_category_id: body.parent_category_id || null,
        category_children: [],
        created_at: now,
        updated_at: now,
      }
      state.product_categories.push(created)
      return { status: 200, body: { product_category: created } }
    }
    return {
      status: 200,
      body: {
        product_categories: state.product_categories,
        count: state.product_categories.length,
        limit: 20,
        offset: 0,
      },
    }
  }

  // Collections
  if (pathname === "/admin/collections") {
    if (upperMethod === "POST" && body && typeof body === "object") {
      const created = {
        id: `pcol_${Date.now()}`,
        title: body.title || "New Collection",
        handle: body.handle || `collection-${Date.now()}`,
        products: [],
        created_at: now,
        updated_at: now,
      }
      state.collections.unshift(created)
      return { status: 200, body: { collection: created } }
    }
    return {
      status: 200,
      body: { collections: state.collections, count: state.collections.length, limit: 20, offset: 0 },
    }
  }

  // Campaigns
  if (pathname === "/admin/campaigns") {
    if (upperMethod === "POST" && body && typeof body === "object") {
      const created = {
        id: `camp_${Date.now()}`,
        name: body.name || "New Campaign",
        campaign_identifier: body.campaign_identifier || `CAMP${Date.now()}`,
        description: body.description || "",
        starts_at: body.starts_at || now,
        ends_at: body.ends_at || null,
        budget: body.budget || null,
        created_at: now,
        updated_at: now,
      }
      state.campaigns.unshift(created)
      return { status: 200, body: { campaign: created } }
    }
    return {
      status: 200,
      body: { campaigns: state.campaigns, count: state.campaigns.length, limit: 20, offset: 0 },
    }
  }

  // Couriers
  if (pathname === "/admin/couriers") {
    return { status: 200, body: { couriers: state.couriers } }
  }
  if (pathname.startsWith("/admin/couriers/")) {
    const parts = pathname.split("/")
    const id = parts[3]
    const action = parts[4]
    const courier = state.couriers.find((c) => c.id === id)
    if (!courier) {
      return { status: 404, body: { message: "Courier not found" } }
    }
    if (action === "probe" && upperMethod === "POST") {
      const probe = {
        verified: true,
        status: "connected",
        latency_ms: 62,
        message: "Connection verified (mock mode)",
        probed_at: now,
      }
      courier.status = "connected"
      courier.last_probe = probe
      return { status: 200, body: { ...probe, courier_id: id } }
    }
    if (action === "rates" && upperMethod === "POST" && body) {
      courier.rates = { ...courier.rates, ...body }
      return { status: 200, body: { id, rates: courier.rates } }
    }
    if (upperMethod === "POST" && body) {
      if (body.credentials) courier.credentials = { ...courier.credentials, ...body.credentials }
      if (typeof body.is_enabled === "boolean") courier.is_enabled = body.is_enabled
      courier.status = courier.is_enabled ? "connected" : "not_configured"
      return { status: 200, body: { courier } }
    }
    return { status: 200, body: { courier } }
  }

  // Domains
  if (pathname === "/admin/domains") {
    if (upperMethod === "POST" && body && typeof body === "object") {
      const created = {
        id: `dom_${Date.now()}`,
        hostname: body.hostname || "store.example.com",
        status: "pending",
        ssl_status: "pending",
        is_primary: false,
        verification_token: `stheno-verify-${Date.now().toString(36)}`,
        created_at: now,
        updated_at: now,
      }
      state.domains.push(created)
      return { status: 200, body: { domain: created } }
    }
    return { status: 200, body: { domains: state.domains } }
  }
  if (pathname.startsWith("/admin/domains/")) {
    const parts = pathname.split("/")
    const id = parts[3]
    if (upperMethod === "DELETE") {
      state.domains = state.domains.filter((d) => d.id !== id)
      return { status: 200, body: { deleted: true, id } }
    }
    const domain = state.domains.find((d) => d.id === id)
    if (domain && parts[4] === "verify") {
      domain.status = "verified"
      domain.ssl_status = "active"
    }
    return { status: 200, body: { domain: domain || null } }
  }

  // Currencies / API keys / Notifications / generic admin list endpoints
  if (pathname === "/admin/currencies") {
    return {
      status: 200,
      body: {
        currencies: [
          { code: "usd", symbol: "$", symbol_native: "$", name: "US Dollar" },
          { code: "bdt", symbol: "৳", symbol_native: "৳", name: "Bangladeshi Taka" },
          { code: "eur", symbol: "€", symbol_native: "€", name: "Euro" },
        ],
        count: 3,
        limit: 20,
        offset: 0,
      },
    }
  }
  if (pathname === "/admin/api-keys") {
    return { status: 200, body: { api_keys: state.api_keys, count: state.api_keys.length, limit: 20, offset: 0 } }
  }
  if (pathname === "/admin/notifications") {
    return { status: 200, body: { notifications: [], count: 0, limit: 20, offset: 0 } }
  }
  if (pathname === "/admin/sales-channels") {
    return {
      status: 200,
      body: {
        sales_channels: [{ id: "sc_01JSTHENO", name: "Default Sales Channel", is_disabled: false }],
        count: 1,
        limit: 20,
        offset: 0,
      },
    }
  }
  if (pathname === "/admin/regions") {
    return {
      status: 200,
      body: {
        regions: [{ id: "reg_01JSTHENO", name: "Global", currency_code: "usd", countries: [] }],
        count: 1,
        limit: 20,
        offset: 0,
      },
    }
  }
  if (pathname === "/admin/layouts") {
    return { status: 200, body: { layouts: [] } }
  }

  // Generic safe fallback for any other /admin/* route
  const segment = pathname.replace(/^\/admin\//, "").split("/")[0].replace(/-/g, "_")
  return {
    status: 200,
    body: {
      [segment || "data"]: [],
      count: 0,
      limit: 20,
      offset: 0,
    },
  }
}
