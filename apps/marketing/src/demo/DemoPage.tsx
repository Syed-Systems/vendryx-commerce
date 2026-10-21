// @ts-nocheck
import {
  ArrowRight,
  ArrowDown,
  ArrowUpRight,
  Sun,
  Moon,
  MessageSquareQuote,
  TrendingUp,
  ClipboardList,
  Building2,
  Check,
  CheckCircle2,
  Play,
  LayoutDashboard,
  LogIn,
  Code,
  Copy,
  CreditCard,
  Box,
  Download,
  Eye,
  EyeOff,
  GitFork,
  Gauge,
  Key,
  Zap,
  Menu,
  Search,
  Package,
  Paintbrush,
  Plus,
  Printer,
  ShoppingBag,
  ShoppingCart,
  ShieldCheck,
  Star,
  Store,
  Sparkles,
  Terminal,
  Upload,
  QrCode,
  X,
} from "lucide-react"
import { useReducedMotion, motion, AnimatePresence } from "framer-motion"

import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"
import { Badge } from "@/components/ui/badge"
import { VendryxLogo } from "@/components/brand/VendryxLogo"
import { ADMIN_URL, ADMIN_LOGIN_URL } from "../lib/config"
import { formatMoney, type Currency } from "../lib/format-money"
import { useMarketing } from "../App"
import { Mark } from "../App"
import { useTheme } from "../App"
import { readDemoSession, updateDemoSession, addDemoOrder, convertSocialToOrder, clearDemoSession, createDemoSession } from "../demo-session"
import type { DemoSession, DemoOrder, DemoProduct } from "../demo-session"

function DedicatedDemoPage({ sessionId }: { sessionId: string }) {
  const { navigate } = useMarketing()
  const [session, setSession] = useState<DemoSession | null>(() => readDemoSession(sessionId))
  const [activeTab, setActiveTab] = useState<AdminTab>("dashboard")
  const [currency, setCurrency] = useState<Currency>("BDT")
  const [analyticsTimeframe, setAnalyticsTimeframe] = useState<"7d" | "30d" | "90d">("30d")
  const [saved, setSaved] = useState(false)
  const [filterCategory, setFilterCategory] = useState("All")
  const [newProductModal, setNewProductModal] = useState(false)
  const [cmdPaletteOpen, setCmdPaletteOpen] = useState(false)
  const [selectedInvoiceOrder, setSelectedInvoiceOrder] = useState<DemoOrder | null>(null)
  const [toastMessage, setToastMessage] = useState<string | null>(null)
  const [newProduct, setNewProduct] = useState({ title: "", price: 1000, stock: 20, category: "Accessories", description: "" })

  // Customer Storefront State
  const [storeCategory, setStoreCategory] = useState("All")
  const [searchQuery, setSearchQuery] = useState("")
  const [sortOrder, setSortOrder] = useState<"featured" | "price-asc" | "price-desc" | "name">("featured")
  const [cart, setCart] = useState<Array<{ product: DemoProduct; quantity: number }>>([])
  const [cartOpen, setCartOpen] = useState(false)
  const [selectedProduct, setSelectedProduct] = useState<DemoProduct | null>(null)
  const [reviewsMap, setReviewsMap] = useState<Record<string, ProductReview[]>>(defaultReviews)
  const [newReviewForm, setNewReviewForm] = useState({ author: "", rating: 5, comment: "" })
  const [promoCode, setPromoCode] = useState("")
  const [promoApplied, setPromoApplied] = useState(false)
  const [checkoutStep, setCheckoutStep] = useState<"cart" | "shipping" | "confirmation">("cart")
  const [customerInfo, setCustomerInfo] = useState({ name: "", email: "", phone: "", address: "" })
  const [lastOrderId, setLastOrderId] = useState("")

  const showToast = (msg: string) => {
    setToastMessage(msg)
    setTimeout(() => setToastMessage(null), 2500)
  }

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault()
        setCmdPaletteOpen((o) => !o)
      }
      if (e.key === "Escape") {
        setNewProductModal(false)
        setCmdPaletteOpen(false)
        setSelectedInvoiceOrder(null)
        setCartOpen(false)
        setSelectedProduct(null)
      }
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [])

  if (!session) {
    return (
      <section className="demo-page demo-expired">
        <div>
          <p className="eyebrow">Demo Session Concluded</p>
          <h1>This sandbox workspace is no longer active.</h1>
          <p>Demo sessions remain in your browser tab for testing. Start a fresh demo at any time.</p>
          <StartDemoButton className="button button-primary" />
        </div>
      </section>
    )
  }

  const totalRevenue = session.orders.reduce((sum, o) => sum + o.total, 0)
  const totalOrdersCount = session.orders.length
  const aov = totalOrdersCount > 0 ? totalRevenue / totalOrdersCount : 0
  const pendingConvsCount = session.conversations.filter((c) => !c.converted).length

  const timeframeMultiplier = analyticsTimeframe === "7d" ? 0.35 : analyticsTimeframe === "90d" ? 2.8 : 1.0
  const displayedRev = Math.round(totalRevenue * timeframeMultiplier)
  const displayedOrders = Math.round(totalOrdersCount * timeframeMultiplier)

  const update = (updates: Partial<Parameters<typeof updateDemoSession>[1]>) => {
    const next = updateDemoSession(sessionId, updates)
    if (next) setSession(next)
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  const handleProductUpdate = (productId: string, updates: Partial<DemoProduct>) => {
    const updated = session.products.map((p) => (p.id === productId ? { ...p, ...updates } : p))
    update({ products: updated })
    showToast("Product updated inline & synced to Storefront")
  }

  const handleAddProduct = (e: FormEvent) => {
    e.preventDefault()
    if (!newProduct.title.trim()) return
    const id = `PRD-${Math.floor(78200 + session.products.length + 1)}`
    const created: DemoProduct = {
      id,
      title: newProduct.title.trim(),
      price: newProduct.price,
      stock: newProduct.stock,
      category: newProduct.category,
      status: "Published",
      description: newProduct.description || "A thoughtfully handcrafted item for daily life.",
    }
    update({ products: [...session.products, created] })
    setNewProductModal(false)
    setNewProduct({ title: "", price: 1000, stock: 20, category: "Accessories", description: "" })
    showToast(`Created & published "${created.title}" to Storefront`)
  }

  const handleLoadPreset = (presetKey: string) => {
    if (!catalogPresets[presetKey]) return
    update({ products: catalogPresets[presetKey] })
    showToast(`Loaded ${presetKey.toUpperCase()} preset catalog`)
  }

  const handleExportCSV = () => {
    const csvContent =
      "data:text/csv;charset=utf-8," +
      "ID,Title,Category,Price,Stock,Status\n" +
      session.products.map((p) => `"${p.id}","${p.title}","${p.category}",${p.price},${p.stock},"${p.status}"`).join("\n")
    const encodedUri = encodeURI(csvContent)
    const link = document.createElement("a")
    link.setAttribute("href", encodedUri)
    link.setAttribute("download", `vendryx_catalog_${session.storeName.toLowerCase().replace(/\s+/g, "_")}.csv`)
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    showToast("Catalog CSV exported successfully")
  }

  const handleOrderStatus = (orderId: string, status: DemoOrder["status"]) => {
    const updated = session.orders.map((o) => (o.id === orderId ? { ...o, status } : o))
    update({ orders: updated })
    showToast(`Order ${orderId} marked as ${status}`)
  }

  const handleConvertSocial = (msgId: string) => {
    const res = convertSocialToOrder(sessionId, msgId)
    if (res) {
      const refreshed = readDemoSession(sessionId)
      if (refreshed) setSession(refreshed)
      setSaved(true)
      showToast(`1-Click Order ${res.id} placed into fulfillment queue!`)
      setTimeout(() => setSaved(false), 2000)
    }
  }

  // Storefront methods
  const filteredStoreProducts = session.products
    .filter((p) => p.status === "Published")
    .filter((p) => storeCategory === "All" || p.category === storeCategory)
    .filter(
      (p) =>
        !searchQuery.trim() ||
        p.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.category.toLowerCase().includes(searchQuery.toLowerCase()),
    )
    .sort((a, b) => {
      if (sortOrder === "price-asc") return a.price - b.price
      if (sortOrder === "price-desc") return b.price - a.price
      if (sortOrder === "name") return a.title.localeCompare(b.title)
      return 0
    })

  const storeCategories = ["All", ...Array.from(new Set(session.products.map((p) => p.category)))]

  const addToCart = (product: DemoProduct) => {
    setCart((prev) => {
      const existing = prev.find((item) => item.product.id === product.id)
      if (existing) {
        return prev.map((item) =>
          item.product.id === product.id ? { ...item, quantity: item.quantity + 1 } : item,
        )
      }
      return [...prev, { product, quantity: 1 }]
    })
    setCartOpen(true)
  }

  const updateCartQty = (productId: string, delta: number) => {
    setCart((prev) =>
      prev
        .map((item) =>
          item.product.id === productId ? { ...item, quantity: Math.max(0, item.quantity + delta) } : item,
        )
        .filter((item) => item.quantity > 0),
    )
  }

  const rawSubtotal = cart.reduce((sum, item) => sum + item.product.price * item.quantity, 0)
  const discount = promoApplied ? Math.round(rawSubtotal * 0.1) : 0
  const total = Math.max(0, rawSubtotal - discount)
  const cartItemCount = cart.reduce((sum, item) => sum + item.quantity, 0)

  const handleApplyPromo = (e: FormEvent) => {
    e.preventDefault()
    const code = promoCode.trim().toUpperCase()
    if (code === "VENDRYX10" || code === "STHENO10") {
      setPromoApplied(true)
    }
  }

  const handleAddReview = (e: FormEvent) => {
    e.preventDefault()
    if (!selectedProduct || !newReviewForm.author.trim() || !newReviewForm.comment.trim()) return

    const newRev: ProductReview = {
      id: `REV-${Date.now()}`,
      author: newReviewForm.author.trim(),
      rating: newReviewForm.rating,
      date: "Just now",
      comment: newReviewForm.comment.trim(),
      verified: true,
    }

    setReviewsMap((prev) => ({
      ...prev,
      [selectedProduct.id]: [newRev, ...(prev[selectedProduct.id] || [])],
    }))

    setNewReviewForm({ author: "", rating: 5, comment: "" })
    showToast("Review submitted successfully!")
  }

  const handlePlaceOrder = (e: FormEvent) => {
    e.preventDefault()
    if (!customerInfo.name || !customerInfo.phone) return

    const orderItems = cart.map((item) => ({
      title: item.product.title,
      quantity: item.quantity,
      price: item.product.price,
    }))

    const newOrder = addDemoOrder(sessionId, {
      customer: customerInfo.name,
      email: customerInfo.email || `${customerInfo.name.toLowerCase().replace(/\s+/g, ".")}@example.com`,
      phone: customerInfo.phone,
      items: orderItems,
      total,
      status: "Received",
    })

    if (newOrder) {
      setLastOrderId(newOrder.id)
      setCheckoutStep("confirmation")
      setCart([])
      const updated = readDemoSession(sessionId)
      if (updated) setSession(updated)
      showToast(`Order ${newOrder.id} placed & recorded in Admin queue above!`)
    }
  }

  const filteredAdminProducts = session.products.filter(
    (p) => filterCategory === "All" || p.category.toLowerCase() === filterCategory.toLowerCase(),
  )

  const adminCategories = ["All", ...Array.from(new Set(session.products.map((p) => p.category)))]

  return (
    <main className="dedicated-demo-page" style={{ "--demo-accent": session.accent } as CSSProperties}>
      {/* Toast Notification */}
      <AnimatePresence>
        {toastMessage && (
          <motion.div
            className="floating-toast"
            role="status"
            aria-live="polite"
            initial={{ opacity: 0, y: 15, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 15, scale: 0.95 }}
          >
            <CheckCircle2 size={16} className="text-accent" />
            <span>{toastMessage}</span>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Top Universal Demo Navigation Bar */}
      <header className="demo-topbar">
        <div className="demo-topbar-left-group">
          <Link className="brand" to="/">
            <Mark />
            <span className="sr-only">Vendryx home</span>
          </Link>
          <span className="demo-topbar-divider">/</span>
          <span className="demo-store-name-tag">{session.storeName} Sandbox</span>
        </div>
        <div className="demo-topbar-center">
          <button type="button" className="cmd-palette-trigger" aria-haspopup="dialog" aria-expanded={cmdPaletteOpen} onClick={() => setCmdPaletteOpen(true)}>
            <Search size={16} />
            <span>Quick search...</span>
            <kbd>⌘K</kbd>
          </button>
        </div>
        <div className="demo-topbar-actions">
          <div className="currency-selector-mini">
            {(["BDT", "USD", "EUR"] as Currency[]).map((c) => (
              <button
                key={c}
                type="button"
                aria-label={`Switch currency to ${c}`}
                aria-pressed={currency === c}
                className={`curr-btn ${currency === c ? "is-active" : ""}`}
                onClick={() => setCurrency(c)}
              >
                {CURRENCY_SYMBOLS[c]}
              </button>
            ))}
          </div>

          <a href="#live-storefront" className="button button-outline button-xs">
            <Store size={14} />
            <span>Jump to Storefront Preview <ArrowDown size={14} /></span>
          </a>

          <button
            type="button"
            className="demo-exit-btn"
            onClick={() => {
              clearDemoSession(sessionId)
              navigate("/")
            }}
          >
            Exit demo
          </button>
        </div>
      </header>

      {/* SECTION 1: MERCHANT ADMIN PANEL */}
      <section className="demo-admin-section">
        <div className="demo-section-badge-bar">
          <div className="badge-bar-left">
            <span className="section-badge admin">Merchant Admin Workspace</span>
            <p>Manage your inventory, social conversations, orders, and brand kit in real-time.</p>
          </div>
          {saved && (
            <span className="demo-sync-badge" role="status">
              <Check size={13} /> Synced to Storefront
            </span>
          )}
        </div>

        <div className="demo-app-layout">
          <aside className="demo-admin-sidebar" aria-label="Merchant admin navigation">
            <span className="sidebar-group-title">Operations</span>
            <button
              className={activeTab === "dashboard" ? "is-active" : ""}
              type="button"
              aria-selected={activeTab === "dashboard"}
              onClick={() => setActiveTab("dashboard")}
            >
              <TrendingUp size={17} />
              <span>Executive Dashboard</span>
            </button>
            <button
              className={activeTab === "social" ? "is-active" : ""}
              type="button"
              aria-selected={activeTab === "social"}
              onClick={() => setActiveTab("social")}
            >
              <MessageSquareQuote size={17} />
              <span>Social Inbox</span>
              {pendingConvsCount > 0 && <span className="nav-count-pill">{pendingConvsCount}</span>}
            </button>
            <button
              className={activeTab === "products" ? "is-active" : ""}
              type="button"
              aria-selected={activeTab === "products"}
              onClick={() => setActiveTab("products")}
            >
              <Package size={17} />
              <span>Products & Catalog</span>
            </button>
            <button
              className={activeTab === "orders" ? "is-active" : ""}
              type="button"
              aria-selected={activeTab === "orders"}
              onClick={() => setActiveTab("orders")}
            >
              <ClipboardList size={17} />
              <span>Orders Queue ({session.orders.length})</span>
            </button>
            <button
              className={activeTab === "brand" ? "is-active" : ""}
              type="button"
              aria-selected={activeTab === "brand"}
              onClick={() => setActiveTab("brand")}
            >
              <Paintbrush size={17} />
              <span>Brand & Theme</span>
            </button>
          </aside>

          <div className="demo-admin-content">
            <AnimatePresence mode="wait">
              {/* Tab 1: Executive Dashboard */}
              {activeTab === "dashboard" && (
                <motion.div
                  key="dashboard"
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -6 }}
                  transition={{ duration: 0.18 }}
                >
                  <div className="demo-admin-heading">
                    <div>
                      <p className="eyebrow">Executive Dashboard</p>
                      <h1>Operational clarity across all channels.</h1>
                      <p>Real-time revenue, order queues, and sales velocity.</p>
                    </div>
                    <div className="timeframe-switch-group">
                      {(["7d", "30d", "90d"] as const).map((tf) => (
                        <button
                          key={tf}
                          type="button"
                          className={`filter-pill ${analyticsTimeframe === tf ? "is-active" : ""}`}
                          onClick={() => setAnalyticsTimeframe(tf)}
                        >
                          {tf === "7d" ? "Last 7 Days" : tf === "30d" ? "Last 30 Days" : "Last 90 Days"}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="demo-metrics-grid">
                    <article className="demo-metric-card">
                      <div className="metric-header">
                        <span className="metric-label">Total Revenue</span>
                        <span className="metric-badge live">Live</span>
                      </div>
                      <strong className="metric-value">{formatMoney(displayedRev, currency)}</strong>
                      <span className="metric-subtext">{analyticsTimeframe.toUpperCase()} volume · +14.2%</span>
                    </article>

                    <article className="demo-metric-card">
                      <div className="metric-header">
                        <span className="metric-label">Total Orders</span>
                        <span className="metric-badge tracked">Tracked</span>
                      </div>
                      <strong className="metric-value">{displayedOrders}</strong>
                      <span className="metric-subtext">Placed orders across channels</span>
                    </article>

                    <article className="demo-metric-card">
                      <div className="metric-header">
                        <span className="metric-label">Average Order Value</span>
                        <span className="metric-badge neutral">AOV</span>
                      </div>
                      <strong className="metric-value">{formatMoney(Math.round(aov), currency)}</strong>
                      <span className="metric-subtext">Average basket size</span>
                    </article>
                  </div>

                  {/* Sales Velocity & Channel Split Bar Chart */}
                  <div className="analytics-chart-card">
                    <div className="chart-header">
                      <div>
                        <h3>Sales Velocity & Channel Attribution</h3>
                        <p>Direct Web Storefront vs WhatsApp / Instagram Conversation Commerce.</p>
                      </div>
                      <div className="chart-legend">
                        <span className="legend-item web">
                          <span className="legend-dot" /> Web Storefront (58%)
                        </span>
                        <span className="legend-item social">
                          <span className="legend-dot" /> Social Inquiries (42%)
                        </span>
                      </div>
                    </div>

                    <div className="bar-chart-visual">
                      {[
                        { day: "Mon", val: 65, soc: 40 },
                        { day: "Tue", val: 80, soc: 55 },
                        { day: "Wed", val: 45, soc: 30 },
                        { day: "Thu", val: 90, soc: 70 },
                        { day: "Fri", val: 110, soc: 85 },
                        { day: "Sat", val: 130, soc: 95 },
                        { day: "Sun", val: 95, soc: 60 },
                      ].map((d) => (
                        <div className="chart-bar-col" key={d.day}>
                          <div className="bar-stacked-track">
                            <div className="bar-fill web" style={{ height: `${(d.val / 140) * 100}%` }} />
                            <div className="bar-fill social" style={{ height: `${(d.soc / 140) * 100}%` }} />
                          </div>
                          <span className="bar-day-label">{d.day}</span>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="demo-activity-section">
                    <div className="section-title-row">
                      <h2>Recent Transactions</h2>
                      <button type="button" className="text-link" onClick={() => setActiveTab("orders")}>
                        <span>View all orders</span>
                        <ArrowRight size={14} />
                      </button>
                    </div>
                    <div className="demo-orders-list">
                      {session.orders.slice(0, 3).map((order) => (
                        <div className="demo-order-item" key={order.id}>
                          <div className="order-main-info">
                            <span className="order-id">{order.id}</span>
                            <strong>{order.customer}</strong>
                            <span className="order-item-desc">
                              {order.items.map((i) => `${i.quantity}x ${i.title}`).join(", ")}
                            </span>
                          </div>
                          <div className="order-amount">{formatMoney(order.total, currency)}</div>
                          <div className="order-status-group">
                            <span className={`status-pill ${order.status.toLowerCase().replace(/\s+/g, "-")}`}>
                              {order.status}
                            </span>
                            <button
                              type="button"
                              className="button button-outline button-xs"
                              onClick={() => setSelectedInvoiceOrder(order)}
                            >
                              <Printer size={13} />
                              <span>Invoice</span>
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </motion.div>
              )}

              {/* Tab 2: Social Inbox */}
              {activeTab === "social" && (
                <motion.div
                  key="social"
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -6 }}
                  transition={{ duration: 0.18 }}
                >
                  <div className="demo-admin-heading">
                    <div>
                      <p className="eyebrow">Social Channel Integration</p>
                      <h1>Convert conversational sales into fulfillment orders.</h1>
                      <p>Direct WhatsApp and Instagram customer messages parsed into executable store orders.</p>
                    </div>
                  </div>

                  <div className="social-convs-list">
                    {session.conversations.map((conv) => (
                      <div className={`social-conv-card ${conv.converted ? "is-converted" : ""}`} key={conv.id}>
                        <div className="conv-header">
                          <div className="conv-user-info">
                            {conv.channel === "WhatsApp" ? (
                              <WhatsAppIcon size={20} className="wa-icon text-emerald-600" />
                            ) : (
                              <InstagramIcon size={20} className="ig-icon text-pink-600" />
                            )}
                            <div>
                              <strong>{conv.customer}</strong>
                              <span className="conv-phone">{conv.phone}</span>
                            </div>
                            <span className="conv-channel-pill">{conv.channel}</span>
                          </div>
                          <strong className="conv-amount-tag">{formatMoney(conv.price, currency)}</strong>
                        </div>
                        <p className="conv-message">"{conv.message}"</p>
                        <div className="conv-footer">
                          <span className="conv-item-target">Requested Item: {conv.proposedItem}</span>
                          {conv.converted ? (
                            <span className="converted-badge">
                              <CheckCircle2 size={15} className="text-accent" /> Order Created in Queue
                            </span>
                          ) : (
                            <button
                              type="button"
                              className="button button-primary button-sm"
                              onClick={() => handleConvertSocial(conv.id)}
                            >
                              <Plus size={14} />
                              <span>1-Click Convert to Order</span>
                            </button>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </motion.div>
              )}

              {/* Tab 3: Products & Catalog */}
              {activeTab === "products" && (
                <motion.div
                  key="products"
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -6 }}
                  transition={{ duration: 0.18 }}
                >
                  <div className="demo-admin-heading">
                    <div>
                      <p className="eyebrow">Catalog & Stock Control</p>
                      <h1>Manage inventory, pricing, and visibility.</h1>
                      <p>Inline editing syncs immediately to the customer-facing storefront below.</p>
                    </div>
                    <div className="catalog-top-actions">
                      <button type="button" className="button button-outline button-sm" onClick={handleExportCSV}>
                        <Download size={15} />
                        <span>Export CSV</span>
                      </button>
                      <button
                        className="button button-primary button-sm"
                        type="button"
                        onClick={() => setNewProductModal(true)}
                      >
                        <Plus size={15} />
                        <span>Add New Product</span>
                      </button>
                    </div>
                  </div>

                  <div className="catalog-filter-bar">
                    <div className="filter-pill-group">
                      {adminCategories.map((cat) => (
                        <button
                          key={cat}
                          type="button"
                          className={`filter-pill ${filterCategory === cat ? "is-active" : ""}`}
                          onClick={() => setFilterCategory(cat)}
                        >
                          {cat}
                        </button>
                      ))}
                    </div>

                    <div className="preset-import-wrap">
                      <Upload size={14} />
                      <span>Load Preset:</span>
                      <select onChange={(e) => handleLoadPreset(e.target.value)} defaultValue="">
                        <option value="" disabled>
                          Choose Sample Catalog...
                        </option>
                        <option value="leather">Artisan Leathercraft (3 Items)</option>
                        <option value="ceramics">Studio Ceramics (3 Items)</option>
                        <option value="textiles">Heritage Jamdani Textiles (3 Items)</option>
                      </select>
                    </div>
                  </div>

                  <div className="demo-product-table">
                    <div className="product-table-header">
                      <span>Product Details</span>
                      <span>Category</span>
                      <span>Price</span>
                      <span>Stock</span>
                      <span>Status</span>
                    </div>
                    {filteredAdminProducts.map((product) => (
                      <div className="product-table-row" key={product.id}>
                        <div className="product-info-cell">
                          <div className="product-thumbnail" aria-hidden="true">
                            <ShoppingBag size={18} />
                          </div>
                          <div className="product-title-group">
                            <input
                              type="text"
                              className="product-title-input"
                              value={product.title}
                              onChange={(e) => handleProductUpdate(product.id, { title: e.target.value })}
                            />
                            <small>ID: {product.id} · {product.category}</small>
                          </div>
                        </div>
                        <div className="product-category-cell">
                          <span className="tag-pill">{product.category}</span>
                        </div>
                        <div className="product-price-cell">
                          <div className="price-stepper">
                            <button
                              type="button"
                              onClick={() =>
                                handleProductUpdate(product.id, { price: Math.max(50, product.price - 50) })
                              }
                            >
                              −
                            </button>
                            <span className="stepper-display-price">{formatMoney(product.price, currency)}</span>
                            <button
                              type="button"
                              onClick={() => handleProductUpdate(product.id, { price: product.price + 50 })}
                            >
                              +
                            </button>
                          </div>
                        </div>
                        <div className="product-stock-cell">
                          <div className="stock-stepper">
                            <button
                              type="button"
                              onClick={() =>
                                handleProductUpdate(product.id, { stock: Math.max(0, product.stock - 1) })
                              }
                            >
                              −
                            </button>
                            <span className="stock-count">{product.stock} units</span>
                            <button
                              type="button"
                              onClick={() => handleProductUpdate(product.id, { stock: product.stock + 1 })}
                            >
                              +
                            </button>
                          </div>
                        </div>
                        <div className="product-status-cell">
                          <button
                            type="button"
                            className={`status-badge-btn ${product.status.toLowerCase()}`}
                            onClick={() =>
                              handleProductUpdate(product.id, {
                                status: product.status === "Published" ? "Draft" : "Published",
                              })
                            }
                          >
                            {product.status}
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                </motion.div>
              )}

              {/* Tab 4: Orders Queue */}
              {activeTab === "orders" && (
                <motion.div
                  key="orders"
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -6 }}
                  transition={{ duration: 0.18 }}
                >
                  <div className="demo-admin-heading">
                    <div>
                      <p className="eyebrow">Fulfillment Queue</p>
                      <h1>Track and progress merchant orders.</h1>
                      <p>Orders submitted from the Storefront below appear here instantly.</p>
                    </div>
                  </div>

                  <div className="demo-orders-list">
                    {session.orders.map((order) => (
                      <div className="demo-order-item" key={order.id}>
                        <div className="order-main-info">
                          <span className="order-id">{order.id}</span>
                          <div>
                            <strong>{order.customer}</strong>
                            <span className="order-contact-meta">
                              {order.phone} · {order.email}
                            </span>
                          </div>
                          <span className="order-item-desc">
                            {order.items.map((i) => `${i.quantity}x ${i.title}`).join(", ")}
                          </span>
                        </div>
                        <div className="order-amount">{formatMoney(order.total, currency)}</div>
                        <div className="order-status-group">
                          <select
                            className={`status-select ${order.status.toLowerCase().replace(/\s+/g, "-")}`}
                            value={order.status}
                            onChange={(e) =>
                              handleOrderStatus(order.id, e.target.value as DemoOrder["status"])
                            }
                          >
                            <option value="Received">Received</option>
                            <option value="In Packing">In Packing</option>
                            <option value="Shipped">Shipped</option>
                            <option value="Completed">Completed</option>
                          </select>
                          <button
                            type="button"
                            className="button button-outline button-xs"
                            onClick={() => setSelectedInvoiceOrder(order)}
                          >
                            <Printer size={13} />
                            <span>Slip</span>
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                </motion.div>
              )}

              {/* Tab 5: Brand & Theme */}
              {activeTab === "brand" && (
                <motion.div
                  key="brand"
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -6 }}
                  transition={{ duration: 0.18 }}
                >
                  <div className="demo-admin-heading">
                    <div>
                      <p className="eyebrow">Visual Brand Kit</p>
                      <h1>Shape your storefront's visual presence.</h1>
                      <p>Styling tokens update the customer storefront below instantly.</p>
                    </div>
                  </div>

                  <div className="demo-admin-grid">
                    <form className="demo-editor" onSubmit={(e) => e.preventDefault()}>
                      <div className="form-group">
                        <label>
                          Store name
                          <input
                            type="text"
                            value={session.storeName}
                            onChange={(e) => update({ storeName: e.target.value })}
                          />
                        </label>
                      </div>

                      <div className="form-group">
                        <label>
                          Announcement bar
                          <input
                            type="text"
                            value={session.announcement}
                            onChange={(e) => update({ announcement: e.target.value })}
                          />
                        </label>
                      </div>

                      <div className="form-group">
                        <label>
                          Hero headline
                          <textarea
                            rows={2}
                            value={session.heroTitle}
                            onChange={(e) => update({ heroTitle: e.target.value })}
                          />
                        </label>
                      </div>

                      <div className="demo-field-pair">
                        <label>
                          Accent tone
                          <input
                            type="color"
                            className="color-picker-input"
                            value={session.accent}
                            onChange={(e) => update({ accent: e.target.value })}
                          />
                        </label>
                        <label>
                          Primary CTA Label
                          <input
                            type="text"
                            value={session.ctaLabel}
                            onChange={(e) => update({ ctaLabel: e.target.value })}
                          />
                        </label>
                      </div>
                    </form>

                    <aside className="demo-operator-note">
                      <span className="note-eyebrow">Instant Binding</span>
                      <h2>Real-time storefront propagation.</h2>
                      <ul>
                        <li>
                          <Check size={16} />
                          <span>Accent color updates buttons, banners, and carts below immediately.</span>
                        </li>
                        <li>
                          <Check size={16} />
                          <span>Headline changes render live in the customer hero section.</span>
                        </li>
                      </ul>
                    </aside>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </div>
      </section>

      {/* SECTION 2 (BELOW ADMIN): LIVE CUSTOMER E-COMMERCE STOREFRONT */}
      <section className="live-storefront-wrapper" id="live-storefront">
        <div className="storefront-demarcation-header">
          <div className="demarcation-badge">
            <Store size={16} />
            <span>Live Customer Storefront (Preview & Simulation)</span>
          </div>
          <p>
            This replicates the actual customer shopping experience. Items, prices, and orders sync bi-directionally
            with the Admin workspace above.
          </p>
        </div>

        <div className="demo-store-shell">
          <header className="demo-store-header">
            <div className="store-brand-group">
              <strong>{session.storeName}</strong>
              <span className="store-domain-badge">.vendryx.com</span>
            </div>
            <nav aria-label="Demo storefront">
              <button
                type="button"
                className="storefront-cart-btn"
                onClick={() => {
                  setCheckoutStep("cart")
                  setCartOpen(true)
                }}
              >
                <ShoppingCart size={18} />
                <span>Cart ({cartItemCount})</span>
              </button>
            </nav>
          </header>

          {session.announcement && (
            <div className="demo-announcement-bar">
              <p>{session.announcement}</p>
            </div>
          )}

          <section className="demo-store-hero">
            <h1>{session.heroTitle}</h1>
            <div className="storefront-hero-action">
              <a className="button button-primary" href="#store-catalog">
                <span>{session.ctaLabel}</span>
                <ArrowDown size={16} />
              </a>
            </div>
          </section>

          <section className="demo-store-catalog" id="store-catalog">
            <div className="store-catalog-header">
              <div>
                <p className="eyebrow">Curated Collection</p>
                <h2>Featured Objects & Essentials</h2>
              </div>
              <div className="store-category-filters">
                {storeCategories.map((cat) => (
                  <button
                    key={cat}
                    type="button"
                    className={`store-cat-btn ${storeCategory === cat ? "is-active" : ""}`}
                    onClick={() => setStoreCategory(cat)}
                  >
                    {cat}
                  </button>
                ))}
              </div>
            </div>

            {/* Live Search & Sort Toolbar */}
            <div className="store-search-sort-bar">
              <div className="store-search-input-wrap">
                <Search size={16} />
                <input
                  type="text"
                  placeholder="Search catalog items..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  aria-label="Search catalog items"
                />
                {searchQuery && (
                  <button type="button" className="clear-search-btn" onClick={() => setSearchQuery("")}>
                    <X size={14} />
                  </button>
                )}
              </div>
              <div className="store-sort-wrap">
                <span>Sort by:</span>
                <select
                  value={sortOrder}
                  onChange={(e) => setSortOrder(e.target.value as typeof sortOrder)}
                  aria-label="Sort catalog items"
                >
                  <option value="featured">Featured Order</option>
                  <option value="price-asc">Price: Low to High</option>
                  <option value="price-desc">Price: High to Low</option>
                  <option value="name">Product Name (A-Z)</option>
                </select>
              </div>
            </div>

            {filteredStoreProducts.length === 0 ? (
              <div className="empty-catalog-search">
                <ShoppingBag size={42} />
                <p>No products match "{searchQuery}" in {storeCategory}.</p>
                <button
                  type="button"
                  className="button button-outline button-sm"
                  onClick={() => {
                    setSearchQuery("")
                    setStoreCategory("All")
                  }}
                >
                  Reset Filters
                </button>
              </div>
            ) : (
              <div className="store-product-grid">
                {filteredStoreProducts.map((product) => {
                  const reviews = reviewsMap[product.id] || []
                  const avgRating =
                    reviews.length > 0 ? (reviews.reduce((s, r) => s + r.rating, 0) / reviews.length).toFixed(1) : "5.0"
                  return (
                    <article className="store-product-card" key={product.id}>
                      <div
                        className="store-card-art"
                        onClick={() => setSelectedProduct(product)}
                        onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); setSelectedProduct(product) } }}
                        role="button"
                        tabIndex={0}
                        aria-label={`View ${product.title} details`}
                      >
                        <ShoppingBag size={36} />
                        <span className="stock-tag">{product.stock} in stock</span>
                        <span className="card-star-pill">
                          <Star size={11} className="star-icon fill-amber-400 text-amber-400" />
                          <span>{avgRating} ({reviews.length > 0 ? reviews.length : 0} reviews)</span>
                        </span>
                      </div>
                      <div className="store-card-info">
                        <span className="card-category">{product.category}</span>
                        <h3 onClick={() => setSelectedProduct(product)}>{product.title}</h3>
                        <strong className="card-price">{formatMoney(product.price, currency)}</strong>
                      </div>
                      <button
                        type="button"
                        className="button button-outline button-sm store-add-btn"
                        onClick={() => addToCart(product)}
                      >
                        <Plus size={14} />
                        <span>Add to Cart</span>
                      </button>
                    </article>
                  )
                })}
              </div>
            )}
          </section>

          <footer className="demo-store-footer">
            <strong>{session.storeName}</strong>
            <span>Powered by Vendryx Merchant Workspace</span>
          </footer>
        </div>
      </section>

      {/* Cart Drawer */}
      <AnimatePresence>
        {cartOpen && (
          <div className="drawer-backdrop" onClick={() => setCartOpen(false)}>
            <motion.div
              className="drawer-shell"
              onClick={(e) => e.stopPropagation()}
              initial={{ x: "100%" }}
              animate={{ x: 0 }}
              exit={{ x: "100%" }}
              transition={{ type: "spring", damping: 25, stiffness: 200 }}
            >
              <div className="drawer-header">
                <h2>
                  {checkoutStep === "cart"
                    ? `Shopping Cart (${cartItemCount})`
                    : checkoutStep === "shipping"
                      ? "Shipping & Checkout"
                      : "Order Confirmed"}
                </h2>
                <button type="button" className="drawer-close-btn" onClick={() => setCartOpen(false)}>
                  <X size={18} />
                </button>
              </div>

              <div className="drawer-body">
                {checkoutStep === "cart" && (
                  <>
                    {cart.length === 0 ? (
                      <div className="empty-cart-state">
                        <ShoppingBag size={48} />
                        <p>Your shopping cart is empty.</p>
                        <button
                          type="button"
                          className="button button-outline button-sm"
                          onClick={() => setCartOpen(false)}
                        >
                          Explore Catalog
                        </button>
                      </div>
                    ) : (
                      <>
                        <div className="cart-items-list">
                          {cart.map(({ product, quantity }) => (
                            <div className="cart-item-row" key={product.id}>
                              <div className="cart-item-details">
                                <strong>{product.title}</strong>
                                <span className="cart-item-price">{formatMoney(product.price, currency)}</span>
                              </div>
                              <div className="cart-stepper">
                                <button type="button" onClick={() => updateCartQty(product.id, -1)}>
                                  −
                                </button>
                                <span>{quantity}</span>
                                <button type="button" onClick={() => updateCartQty(product.id, 1)}>
                                  +
                                </button>
                              </div>
                            </div>
                          ))}
                        </div>

                        <form className="promo-code-box" onSubmit={handleApplyPromo}>
                          <input
                            placeholder="Promo code (try VENDRYX10)"
                            value={promoCode}
                            onChange={(e) => setPromoCode(e.target.value)}
                          />
                          <button type="submit" className="button button-outline button-xs">
                            Apply
                          </button>
                        </form>
                        {promoApplied && <span className="promo-success-tag"><Check size={12} /> VENDRYX10 applied: 10% discount</span>}

                        <div className="drawer-summary">
                          <div className="summary-line">
                            <span>Subtotal</span>
                            <span>{formatMoney(rawSubtotal, currency)}</span>
                          </div>
                          {promoApplied && (
                            <div className="summary-line discount">
                              <span>Promo Discount (10%)</span>
                              <span>−{formatMoney(discount, currency)}</span>
                            </div>
                          )}
                          <div className="summary-line total">
                            <strong>Total</strong>
                            <strong>{formatMoney(total, currency)}</strong>
                          </div>
                          <button
                            type="button"
                            className="button button-primary button-full"
                            onClick={() => setCheckoutStep("shipping")}
                          >
                            <span>Proceed to Checkout</span>
                            <ArrowRight size={15} />
                          </button>
                        </div>
                      </>
                    )}
                  </>
                )}

                {checkoutStep === "shipping" && (
                  <form onSubmit={handlePlaceOrder} className="checkout-form">
                    <label>
                      Full Name
                      <input
                        required
                        autoComplete="name"
                        placeholder="Nusrat Jahan"
                        value={customerInfo.name}
                        onChange={(e) => setCustomerInfo({ ...customerInfo, name: e.target.value })}
                      />
                    </label>
                    <label>
                      Phone Number (WhatsApp)
                      <input
                        required
                        type="tel"
                        autoComplete="tel"
                        placeholder="+880 1700-000000"
                        value={customerInfo.phone}
                        onChange={(e) => setCustomerInfo({ ...customerInfo, phone: e.target.value })}
                      />
                    </label>
                    <label>
                      Email Address
                      <input
                        type="email"
                        autoComplete="email"
                        placeholder="nusrat@example.com"
                        value={customerInfo.email}
                        onChange={(e) => setCustomerInfo({ ...customerInfo, email: e.target.value })}
                      />
                    </label>
                    <label>
                      Delivery Address
                      <textarea
                        required
                        rows={2}
                        autoComplete="street-address"
                        placeholder="House, Road, Area, Dhaka"
                        value={customerInfo.address}
                        onChange={(e) => setCustomerInfo({ ...customerInfo, address: e.target.value })}
                      />
                    </label>
                    <div className="checkout-payment-choice">
                      <span className="payment-label">Payment Method:</span>
                      <div className="payment-badge">
                        <CreditCard size={16} />
                        <span>Cash on Delivery / Sandbox Demo</span>
                      </div>
                    </div>
                    <div className="drawer-summary">
                      <div className="summary-line total">
                        <strong>Order Total</strong>
                        <strong>{formatMoney(total, currency)}</strong>
                      </div>
                      <button type="submit" className="button button-primary button-full">
                        <span>Place Sandbox Order</span>
                        <ArrowRight size={15} />
                      </button>
                      <button
                        type="button"
                        className="button button-outline button-full button-sm"
                        onClick={() => setCheckoutStep("cart")}
                      >
                        Back to Cart
                      </button>
                    </div>
                  </form>
                )}

                {checkoutStep === "confirmation" && (
                  <div className="order-confirmation-view">
                    <CheckCircle2 size={48} className="confirmation-icon text-accent" />
                    <h3>Order Received!</h3>
                    <p className="order-num-tag">{lastOrderId}</p>
                    <p className="confirmation-text">
                      Your order has been recorded in the Vendryx sandbox database and synchronized directly into the
                      Admin Orders queue above!
                    </p>
                    <a href="#demo" className="button button-primary button-full" onClick={() => setCartOpen(false)}>
                      <TrendingUp size={16} />
                      <span>View in Admin Orders Above</span>
                      <ArrowRight size={15} />
                    </a>
                  </div>
                )}
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>

      {/* Product Detail Modal */}
      <AnimatePresence>
        {selectedProduct && (
          <div className="modal-backdrop" onClick={() => setSelectedProduct(null)}>
            <motion.div
              className="modal-card product-modal"
              onClick={(e) => e.stopPropagation()}
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.95 }}
            >
              <button type="button" className="modal-close-btn" onClick={() => setSelectedProduct(null)}>
                <X size={18} />
              </button>
              <div className="product-modal-art">
                <ShoppingBag size={56} />
              </div>
              <div className="product-modal-details">
                <span className="card-category">{selectedProduct.category}</span>
                <h2>{selectedProduct.title}</h2>
                <strong className="modal-price">{formatMoney(selectedProduct.price, currency)}</strong>
                <p>{selectedProduct.description}</p>
                <div className="modal-actions-row">
                  <button
                    type="button"
                    className="button button-primary button-full"
                    onClick={() => {
                      addToCart(selectedProduct)
                      setSelectedProduct(null)
                    }}
                  >
                    <ShoppingCart size={16} />
                    <span>Add to Shopping Cart</span>
                  </button>
                </div>

                {/* Verified Reviews Section */}
                <div className="product-reviews-block">
                  <div className="reviews-header-row">
                    <strong>Customer Reviews ({(reviewsMap[selectedProduct.id] || []).length})</strong>
                    {reviewsMap[selectedProduct.id]?.length ? (
                      <span className="verified-badge">
                        <ShieldCheck size={14} /> Verified Buyers
                      </span>
                    ) : null}
                  </div>

                  <div className="reviews-list-compact">
                    {(reviewsMap[selectedProduct.id] || []).map((rev) => (
                      <div className="review-item" key={rev.id}>
                        <div className="review-top-line">
                          <strong>{rev.author}</strong>
                          <div className="star-rating-row">
                            {Array.from({ length: rev.rating }).map((_, i) => (
                              <Star key={i} size={11} className="star-fill fill-amber-400 text-amber-400" />
                            ))}
                          </div>
                        </div>
                        <p>"{rev.comment}"</p>
                        <span className="review-date">{rev.date}</span>
                      </div>
                    ))}
                  </div>

                  {/* Write a Review Form */}
                  <form onSubmit={handleAddReview} className="write-review-form">
                    <div className="review-input-row">
                      <input
                        required
                        placeholder="Your name"
                        value={newReviewForm.author}
                        onChange={(e) => setNewReviewForm({ ...newReviewForm, author: e.target.value })}
                      />
                      <select
                        value={newReviewForm.rating}
                        onChange={(e) => setNewReviewForm({ ...newReviewForm, rating: Number(e.target.value) })}
                      >
                        <option value={5}>5 Stars ★★★★★</option>
                        <option value={4}>4 Stars ★★★★☆</option>
                        <option value={3}>3 Stars ★★★☆☆</option>
                      </select>
                    </div>
                    <textarea
                      required
                      rows={2}
                      placeholder="Share your experience with this object..."
                      value={newReviewForm.comment}
                      onChange={(e) => setNewReviewForm({ ...newReviewForm, comment: e.target.value })}
                    />
                    <button type="submit" className="button button-outline button-xs">
                      Post Review
                    </button>
                  </form>
                </div>
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>

      {/* Invoice Drawer */}
      <AnimatePresence>
        {selectedInvoiceOrder && (
          <div className="modal-backdrop" onClick={() => setSelectedInvoiceOrder(null)}>
            <motion.div
              className="modal-card invoice-card"
              onClick={(e) => e.stopPropagation()}
              initial={{ opacity: 0, scale: 0.95, y: 10 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.95, y: 10 }}
            >
              <div className="invoice-header">
                <div className="invoice-brand">
                  <strong>{session.storeName}</strong>
                  <span>Consignment & Packing Slip</span>
                </div>
                <div className="invoice-meta">
                  <span className="invoice-id">{selectedInvoiceOrder.id}</span>
                  <span className="invoice-date">2026-09-01</span>
                </div>
              </div>

              <div className="invoice-customer-box">
                <div className="customer-info-col">
                  <span className="box-title">Deliver To:</span>
                  <strong>{selectedInvoiceOrder.customer}</strong>
                  <span>{selectedInvoiceOrder.phone}</span>
                  <span>{selectedInvoiceOrder.email}</span>
                </div>
                <div className="shipping-info-col">
                  <span className="box-title">Carrier Method:</span>
                  <strong>Standard CoD / Dispatch</strong>
                  <span className="status-tag">{selectedInvoiceOrder.status}</span>
                </div>
              </div>

              <div className="invoice-items-table">
                <div className="table-head">
                  <span>Item Description</span>
                  <span>Qty</span>
                  <span>Price</span>
                  <span>Total</span>
                </div>
                {selectedInvoiceOrder.items.map((it, idx) => (
                  <div className="table-row" key={idx}>
                    <span>{it.title}</span>
                    <span>{it.quantity}</span>
                    <span>{formatMoney(it.price, currency)}</span>
                    <span>{formatMoney(it.price * it.quantity, currency)}</span>
                  </div>
                ))}
              </div>

              <div className="invoice-totals-row">
                <div className="barcode-simulation">
                  <QrCode size={32} />
                  <span>{selectedInvoiceOrder.id}</span>
                </div>
                <div className="totals-numbers">
                  <div className="line">
                    <span>Subtotal:</span>
                    <span>{formatMoney(selectedInvoiceOrder.total, currency)}</span>
                  </div>
                  <div className="line">
                    <span>Tax (0%):</span>
                    <span>{formatMoney(0, currency)}</span>
                  </div>
                  <div className="line grand-total">
                    <strong>Amount Due:</strong>
                    <strong>{formatMoney(selectedInvoiceOrder.total, currency)}</strong>
                  </div>
                </div>
              </div>

              <div className="invoice-actions">
                <button
                  type="button"
                  className="button button-outline button-sm"
                  onClick={() => setSelectedInvoiceOrder(null)}
                >
                  Close Slip
                </button>
                <button
                  type="button"
                  className="button button-primary button-sm"
                  onClick={() => {
                    showToast(`Print job sent for ${selectedInvoiceOrder.id}`)
                    setSelectedInvoiceOrder(null)
                  }}
                >
                  <Printer size={15} />
                  <span>Print Consignment Slip</span>
                </button>
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>

      {/* Cmd+K Palette */}
      <AnimatePresence>
        {cmdPaletteOpen && (
          <div className="modal-backdrop" onClick={() => setCmdPaletteOpen(false)}>
            <motion.div
              className="cmd-palette-card"
              onClick={(e) => e.stopPropagation()}
              initial={{ opacity: 0, scale: 0.96, y: -10 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.96, y: -10 }}
            >
              <div className="cmd-palette-input-wrap">
                <Search size={18} />
                <input
                  autoFocus
                  placeholder="Type a command or jump to screen..."
                  onKeyDown={(e) => {
                    if (e.key === "Escape") setCmdPaletteOpen(false)
                  }}
                />
                <kbd>ESC</kbd>
              </div>
              <div className="cmd-palette-list">
                <span className="cmd-group-title">Navigation</span>
                <button
                  type="button"
                  onClick={() => {
                    setActiveTab("dashboard")
                    setCmdPaletteOpen(false)
                  }}
                >
                  <TrendingUp size={16} />
                  <span>Go to Executive Dashboard</span>
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setActiveTab("social")
                    setCmdPaletteOpen(false)
                  }}
                >
                  <MessageSquareQuote size={16} />
                  <span>Go to Social Inbox</span>
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setActiveTab("products")
                    setCmdPaletteOpen(false)
                  }}
                >
                  <Package size={16} />
                  <span>Go to Products & Inventory</span>
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setActiveTab("orders")
                    setCmdPaletteOpen(false)
                  }}
                >
                  <ClipboardList size={16} />
                  <span>Go to Orders Queue</span>
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setActiveTab("brand")
                    setCmdPaletteOpen(false)
                  }}
                >
                  <Paintbrush size={16} />
                  <span>Go to Brand & Theme</span>
                </button>
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>

      {/* Add Product Modal */}
      <AnimatePresence>
        {newProductModal && (
          <div className="modal-backdrop" onClick={() => setNewProductModal(false)}>
            <motion.div
              className="modal-card"
              onClick={(e) => e.stopPropagation()}
              initial={{ opacity: 0, scale: 0.95, y: 10 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.95, y: 10 }}
            >
              <div className="modal-header">
                <h2>Add New Product</h2>
                <button type="button" className="modal-close-btn" onClick={() => setNewProductModal(false)}>
                  <X size={18} />
                </button>
              </div>
              <form onSubmit={handleAddProduct} className="modal-form">
                <label>
                  Product Title
                  <input
                    required
                    placeholder="e.g. Stoneware Espresso Cup"
                    value={newProduct.title}
                    onChange={(e) => setNewProduct({ ...newProduct, title: e.target.value })}
                  />
                </label>
                <div className="form-pair">
                  <label>
                    Price (৳)
                    <input
                      required
                      type="number"
                      min="50"
                      value={newProduct.price}
                      onChange={(e) =>
                        setNewProduct({ ...newProduct, price: Math.max(50, Number(e.target.value)) })
                      }
                    />
                  </label>
                  <label>
                    Stock Count
                    <input
                      required
                      type="number"
                      min="1"
                      value={newProduct.stock}
                      onChange={(e) =>
                        setNewProduct({ ...newProduct, stock: Math.max(0, Number(e.target.value)) })
                      }
                    />
                  </label>
                </div>
                <label>
                  Category
                  <select
                    value={newProduct.category}
                    onChange={(e) => setNewProduct({ ...newProduct, category: e.target.value })}
                  >
                    <option value="Accessories">Accessories</option>
                    <option value="Living">Living</option>
                    <option value="Apparel">Apparel</option>
                    <option value="Stationery">Stationery</option>
                  </select>
                </label>
                <label>
                  Description
                  <textarea
                    rows={3}
                    placeholder="Describe the product materials, craft, and care instructions..."
                    value={newProduct.description}
                    onChange={(e) => setNewProduct({ ...newProduct, description: e.target.value })}
                  />
                </label>
                <div className="modal-actions">
                  <button
                    type="button"
                    className="button button-outline button-sm"
                    onClick={() => setNewProductModal(false)}
                  >
                    Cancel
                  </button>
                  <button type="submit" className="button button-primary button-sm">
                    <span>Create Product</span>
                    <ArrowRight size={14} />
                  </button>
                </div>
              </form>
            </motion.div>
          </div>
        )}
      </AnimatePresence>
    </main>
  )
}

export default DedicatedDemoPage
