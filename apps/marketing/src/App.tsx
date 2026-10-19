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
import { AnimatePresence, motion, useReducedMotion } from "motion/react"
import {
  Fragment,
  useEffect,
  useRef,
  useState,
  useCallback,
  type AnchorHTMLAttributes,
  type CSSProperties,
  type FormEvent,
  type ReactNode,
} from "react"

function GoogleIcon({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor">
      <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/>
      <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
      <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z" fill="#FBBC05"/>
      <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z" fill="#EA4335"/>
    </svg>
  )
}

function WhatsAppIcon({ size = 18, className = "" }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className={className}>
      <path d="M17.472 14.382c-.301-.15-1.78-.878-2.056-.979-.276-.1-.477-.15-.678.15-.201.3-.778.979-.954 1.18-.175.201-.351.226-.652.076-.301-.15-1.272-.469-2.424-1.496-.897-.799-1.503-1.787-1.68-2.088-.175-.301-.019-.464.132-.614.136-.135.301-.351.452-.527.15-.175.201-.3.301-.501.1-.201.05-.376-.025-.526-.075-.15-.678-1.634-.929-2.238-.244-.588-.493-.508-.678-.517-.175-.009-.376-.009-.577-.009s-.527.075-.802.376c-.276.301-1.053 1.028-1.053 2.508 0 1.48 1.078 2.909 1.229 3.11.15.201 2.122 3.24 5.141 4.544.718.31 1.279.495 1.716.634.721.23 1.378.197 1.897.12.578-.087 1.78-.727 2.031-1.429.251-.702.251-1.303.176-1.429-.076-.126-.277-.201-.578-.351z"/>
      <path d="M12.004 2C6.48 2 2 6.48 2 12.004c0 1.954.561 3.78 1.531 5.327L2 22l4.823-1.507A9.957 9.957 0 0012.004 22c5.524 0 10.004-4.48 10.004-9.996C22.008 6.48 17.528 2 12.004 2zm0 18.258c-1.688 0-3.256-.475-4.593-1.3l-.329-.204-2.864.896.912-2.791-.219-.344a8.212 8.212 0 01-1.25-4.511c0-4.564 3.715-8.278 8.343-8.278 4.627 0 8.341 3.714 8.341 8.278 0 4.564-3.714 8.254-8.341 8.254z"/>
    </svg>
  )
}

function InstagramIcon({ size = 18, className = "" }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className={className}>
      <path d="M12 2.163c3.204 0 3.584.012 4.85.07 3.252.148 4.771 1.691 4.919 4.919.058 1.265.069 1.645.069 4.849 0 3.205-.012 3.584-.069 4.849-.149 3.225-1.664 4.771-4.919 4.919-1.266.058-1.644.07-4.85.07-3.204 0-3.584-.012-4.849-.07-3.26-.149-4.771-1.699-4.919-4.92-.058-1.265-.07-1.644-.07-4.849 0-3.204.013-3.583.07-4.849.149-3.227 1.664-4.771 4.919-4.919 1.266-.057 1.645-.069 4.849-.069zm0-2.163c-3.259 0-3.667.014-4.947.072-4.358.2-6.78 2.618-6.98 6.98-.059 1.281-.073 1.689-.073 4.948 0 3.259.014 3.668.072 4.948.2 4.358 2.618 6.78 6.98 6.98 1.281.058 1.689.072 4.948.072 3.259 0 3.668-.014 4.948-.072 4.354-.2 6.782-2.618 6.979-6.98.059-1.28.073-1.689.073-4.948 0-3.259-.014-3.667-.072-4.947-.196-4.354-2.617-6.78-6.979-6.98-1.281-.059-1.69-.073-4.949-.073zm0 5.838c-3.403 0-6.162 2.759-6.162 6.162s2.759 6.163 6.162 6.163 6.162-2.759 6.162-6.163c0-3.403-2.759-6.162-6.162-6.162zm0 10.162c-2.209 0-4-1.79-4-4 0-2.209 1.791-4 4-4s4 1.791 4 4c0 2.21-1.791 4-4 4zm6.406-11.845c-.796 0-1.441.645-1.441 1.44s.645 1.44 1.441 1.44c.795 0 1.439-.645 1.439-1.44s-.644-1.44-1.439-1.44z"/>
    </svg>
  )
}
import {
  addDemoOrder,
  clearDemoSession,
  convertSocialToOrder,
  createDemoSession,
  readDemoSession,
  updateDemoSession,
  type DemoOrder,
  type DemoProduct,
  type DemoSession,
} from "./demo-session"
import { PrivacyPage, TermsPage } from "./content-pages"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { VendryxLogo } from "@/components/brand/VendryxLogo"
import { ADMIN_URL, ADMIN_LOGIN_URL } from "./lib/config"

type StaticPage = "/" | "/demo" | "/privacy" | "/terms" | "/login" | "/signup" | "/forgot-password" | "/welcome" | "/404"
type DemoPage = { kind: "demo"; sessionId: string }
type Page = StaticPage | DemoPage

type Currency = "BDT" | "USD" | "EUR"

const pageMetadata: Record<StaticPage, { title: string; description: string; indexable?: boolean }> = {
  "/": {
    title: "Vendryx | Commerce Operations Infrastructure for Independent Merchants",
    description: "Vendryx is a modern commerce operations system for independent merchants. Manage catalogue, inventory, guest checkout, social-order handoff, and staff operations in one connected core.",
    indexable: true,
  },
  "/demo": {
    title: "Interactive Sandbox Demo | Vendryx Commerce Operations",
    description: "Test drive Vendryx's merchant workspace directly in your browser: 1-click social order converter, real-time catalog sync, order queue, and live customer checkout simulation.",
    indexable: true,
  },
  "/signup": {
    title: "Create Merchant Account | 14-Day Free Trial | Vendryx",
    description: "Start a 14-day Vendryx trial, then manage your catalogue, guest checkout, orders, and staff operations from one workspace.",
    indexable: true,
  },
  "/login": {
    title: "Merchant Workspace Login | Vendryx Commerce Platform",
    description: "Log in to your Vendryx merchant dashboard to manage products, view order queues, and handle customer fulfillment.",
    indexable: true,
  },
  "/forgot-password": {
    title: "Reset Merchant Password | Vendryx Commerce",
    description: "Recover and reset your Vendryx merchant account password securely.",
    indexable: false,
  },
  "/welcome": {
    title: "Welcome & Workspace Provisioning | Vendryx",
    description: "Your Vendryx store environment is being configured. Track store status and next operational steps.",
    indexable: false,
  },
  "/privacy": {
    title: "Privacy Notice & Data Protection Policy | Vendryx",
    description: "Comprehensive disclosure on how Vendryx protects merchant sovereignty, guest checkout privacy, and operational data security.",
    indexable: true,
  },
  "/terms": {
    title: "Terms of Service & Uptime Commitments | Vendryx",
    description: "Terms of service, plan commitments, merchant data ownership rights, and uptime guarantees for Vendryx by Syed Systems.",
    indexable: true,
  },
  "/404": {
    title: "404 - Page Not Found | Vendryx Commerce",
    description: "The requested commerce endpoint, document, or resource could not be found on Vendryx.",
    indexable: false,
  },
}

const CURRENCY_SYMBOLS: Record<Currency, string> = { BDT: "৳", USD: "$", EUR: "€" }

function formatMoney(bdtAmount: number, currency: Currency): string {
  if (currency === "USD") {
    const usd = bdtAmount / 120
    return `$${usd.toFixed(2)}`
  }
  if (currency === "EUR") {
    const eur = bdtAmount / 130
    return `€${eur.toFixed(2)}`
  }
  return `৳${bdtAmount.toLocaleString("en-BD")}`
}

function getPage(): Page {
  const path = window.location.pathname.replace(/\/$/, "") || "/"
  const demoMatch = path.match(/^\/demo\/([a-z0-9-]+)$/i)
  if (demoMatch) return { kind: "demo", sessionId: demoMatch[1] }
  if (path === "/demo") return "/demo"
  const validStatic: StaticPage[] = ["/", "/privacy", "/terms", "/login", "/signup", "/forgot-password", "/welcome"]
  if (validStatic.includes(path as StaticPage)) return path as StaticPage
  return "/404"
}

function pagePath(page: Page) {
  if (typeof page === "string") return page
  return `/demo/${page.sessionId}`
}

function usePage() {
  const [page, setPage] = useState<Page>(getPage)
  useEffect(() => {
    const read = () => setPage(getPage())
    window.addEventListener("popstate", read)
    return () => window.removeEventListener("popstate", read)
  }, [])
  const navigate = (to: Page | string) => {
    window.history.pushState({}, "", typeof to === "string" ? to : pagePath(to))
    setPage(typeof to === "string" ? getPage() : to)
    window.scrollTo({ top: 0, behavior: "auto" })
  }
  return { page, navigate }
}

let marketingNavigate: (to: Page | string) => void = () => undefined
function useMarketing() {
  return { navigate: marketingNavigate }
}

function useTheme() {
  const [theme, setTheme] = useState<"light" | "dark">(() => {
    if (typeof window === "undefined") return "light"
    const stored = localStorage.getItem("vendryx-theme")
    if (stored === "light" || stored === "dark") return stored
    return "light"
  })

  useEffect(() => {
    const root = document.documentElement
    root.classList.toggle("dark", theme === "dark")
    localStorage.setItem("vendryx-theme", theme)
    const meta = document.querySelector('meta[name="theme-color"]')
    if (meta) meta.setAttribute("content", theme === "dark" ? "#0a0a0a" : "#FCFBF9")
  }, [theme])

  const toggle = () => setTheme((t) => (t === "dark" ? "light" : "dark"))
  return { theme, toggle }
}

function Link({
  to,
  children,
  className,
  onClick,
  ...props
}: { to: string; children: ReactNode; className?: string; onClick?: () => void } & AnchorHTMLAttributes<HTMLAnchorElement>) {
  const { navigate } = useMarketing()
  return (
    <a
      {...props}
      className={className}
      href={to}
      onClick={(event) => {
        if (to.startsWith("http://") || to.startsWith("https://")) {
          onClick?.()
          return
        }
        if (to.startsWith("#")) {
          const el = document.querySelector(to)
          if (el) {
            event.preventDefault()
            el.scrollIntoView({ behavior: "smooth" })
            onClick?.()
            return
          }
        }
        event.preventDefault()
        onClick?.()
        navigate(to)
      }}
    >
      {children}
    </a>
  )
}

function Breadcrumbs({ items }: { items: Array<{ label: string; to?: string }> }) {
  return (
    <nav className="breadcrumbs-nav" aria-label="Breadcrumb navigation">
      <ol className="breadcrumbs-list" itemScope itemType="https://schema.org/BreadcrumbList">
        <li itemProp="itemListElement" itemScope itemType="https://schema.org/ListItem">
          <Link to="/" itemProp="item">
            <span itemProp="name">Home</span>
          </Link>
          <meta itemProp="position" content="1" />
        </li>
        {items.map((item, index) => (
          <li key={item.label} itemProp="itemListElement" itemScope itemType="https://schema.org/ListItem">
            <span className="breadcrumb-separator" aria-hidden="true">/</span>
            {item.to ? (
              <Link to={item.to} itemProp="item">
                <span itemProp="name">{item.label}</span>
              </Link>
            ) : (
              <span className="breadcrumb-current" aria-current="page" itemProp="name">{item.label}</span>
            )}
            <meta itemProp="position" content={String(index + 2)} />
          </li>
        ))}
      </ol>
    </nav>
  )
}

function NotFoundPage() {
  const { navigate } = useMarketing()
  return (
    <main className="not-found-page section-shell" role="main">
      <Breadcrumbs items={[{ label: "404 Not Found" }]} />
      <div className="not-found-card">
        <div className="not-found-badge">
          <span>Error 404</span>
        </div>
        <h1>Page Not Found</h1>
        <p className="not-found-lead">
          The commerce endpoint, document, or resource you requested cannot be found or may have moved.
        </p>
        <div className="not-found-actions">
          <Link className="button button-primary" to="/">
            <ArrowRight size={15} />
            <span>Return to Homepage</span>
          </Link>
          <button
            type="button"
            className="button button-outline"
            onClick={() => {
              const s = createDemoSession()
              navigate({ kind: "demo", sessionId: s.id })
            }}
          >
            <Sparkles size={15} />
            <span>Explore Demo Sandbox</span>
          </button>
        </div>
      </div>
    </main>
  )
}

function Mark() {
  const { theme } = useTheme()
  return (
    <span className="brand-logo-frame" aria-hidden="true">
      <VendryxLogo height={64} variant={theme === "dark" ? "dark" : "light"} />
    </span>
  )
}

function Header() {
  const [menuOpen, setMenuOpen] = useState(false)
  const [scrolled, setScrolled] = useState(false)
  const sentinelRef = useRef<HTMLDivElement | null>(null)
  const headerRef = useRef<HTMLElement | null>(null)
  const glass = useRef<{ supported: boolean; refresh: () => void; destroy: () => void } | null>(null)
  const { navigate } = useMarketing()
  const { theme, toggle: toggleTheme } = useTheme()

  useEffect(() => {
    const node = sentinelRef.current
    if (!node) return
    const observer = new IntersectionObserver(
      ([entry]) => setScrolled(!entry.isIntersecting),
      { threshold: 0 }
    )
    observer.observe(node)
    setScrolled(window.scrollY > 120)
    return () => observer.disconnect()
  }, [])

  // Attach liquid-glass refraction once the header becomes the floating pill.
  useEffect(() => {
    if (!scrolled || glass.current) return
    const el = headerRef.current
    if (!el) return

    let disposed = false
    void import("./liquid-glass").then(() => {
      if (disposed || !window.liquidGlass) return
      glass.current = window.liquidGlass(el, {
        scale: -96,
        chroma: 6,
        border: 0.055,
        mapBlur: 14,
        blur: 2,
        saturate: 1.6,
        fallbackBlur: 18,
      })
    })

    return () => {
      disposed = true
    }
  }, [scrolled])

  useEffect(
    () => () => {
      glass.current?.destroy()
      glass.current = null
    },
    []
  )

  const onTransitionEnd = useCallback(() => {
    glass.current?.refresh()
  }, [])

  const startDemo = () => {
    const session = createDemoSession()
    navigate({ kind: "demo", sessionId: session.id })
  }

  return (
    <>
      <div
        ref={sentinelRef}
        aria-hidden="true"
        className="pointer-events-none h-8 w-px"
        style={{ marginTop: 32 }}
      />
      <div className="glass-nav-shell group">
        <header
          ref={headerRef}
          className="site-header glass-nav-header"
          data-scrolled={scrolled}
          onTransitionEnd={onTransitionEnd}
        >
      <Link className="brand" to="/" onClick={() => setMenuOpen(false)} aria-label="Vendryx home">
        <Mark />
        <span className="sr-only">Vendryx</span>
      </Link>
      <nav className="desktop-nav" aria-label="Primary navigation">
        <a href="#how-it-works">How It Works</a>
        <Link to="#features">Features</Link>
        <Link to="#pricing">Pricing</Link>
        <a href="#faq">FAQ</a>
      </nav>
      <div className="nav-actions">
        <button
          type="button"
          className="theme-toggle"
          onClick={toggleTheme}
          aria-label={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
          title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
          aria-pressed={theme === "dark"}
        >
          <Sun size={16} className="theme-icon theme-icon-sun" />
          <Moon size={16} className="theme-icon theme-icon-moon" />
        </button>
        <button type="button" className="button button-outline button-sm" onClick={startDemo}>
          <Play size={13} />
          <span>Explore demo</span>
        </button>
        <a className="button button-outline button-sm" href={ADMIN_URL} title="Open Merchant Admin Dashboard">
          <LayoutDashboard size={13} />
          <span>Admin</span>
        </a>
        <a className="nav-login" href={ADMIN_LOGIN_URL}>
          <LogIn size={13} />
          <span>Log in</span>
        </a>
        <Link className="button button-primary nav-cta" to="/signup">
          <span>Sign up</span>
          <ArrowUpRight size={14} />
        </Link>
      </div>
      <button
        className="menu-toggle"
        type="button"
        onClick={() => setMenuOpen((open) => !open)}
        aria-expanded={menuOpen}
        aria-controls="mobile-menu"
      >
        {menuOpen ? <X size={20} /> : <Menu size={20} />}
        <span className="sr-only">Toggle navigation</span>
      </button>
      <AnimatePresence>
        {menuOpen && (
          <motion.nav
            id="mobile-menu"
            className="mobile-nav"
            aria-label="Mobile navigation"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
          >
            <a href="#how-it-works" onClick={() => setMenuOpen(false)}>
              How It Works
            </a>
            <Link to="#features" onClick={() => setMenuOpen(false)}>
              Features
            </Link>
            <Link to="#pricing" onClick={() => setMenuOpen(false)}>Pricing</Link>
            <a href="#faq" onClick={() => setMenuOpen(false)}>
              FAQ
            </a>
            <button
              type="button"
              className="theme-toggle theme-toggle-mobile"
              onClick={toggleTheme}
              aria-label={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
              title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
              aria-pressed={theme === "dark"}
            >
              <Sun size={16} className="theme-icon theme-icon-sun" />
              <Moon size={16} className="theme-icon theme-icon-moon" />
              <span className="theme-toggle-label">{theme === "dark" ? "Light mode" : "Dark mode"}</span>
            </button>
            <button
              type="button"
              className="button button-outline button-sm"
              onClick={() => {
                setMenuOpen(false)
                startDemo()
              }}
            >
              <Sparkles size={14} />
              <span>Explore demo</span>
            </button>
            <a className="button button-outline button-sm" href={ADMIN_URL}>
              <span>Merchant Admin Dashboard</span>
              <ArrowUpRight size={14} />
            </a>
            <a href={ADMIN_LOGIN_URL} onClick={() => setMenuOpen(false)}>
              Log in to Admin
            </a>
            <Link className="mobile-signup-link" to="/signup" onClick={() => setMenuOpen(false)}>
              Create Account
            </Link>
          </motion.nav>
        )}
      </AnimatePresence>
        </header>
      </div>
    </>
  )
}

function StartDemoButton({ className = "button button-outline" }: { className?: string }) {
  const { navigate } = useMarketing()
  return (
    <button
      className={className}
      type="button"
      onClick={() => {
        const session = createDemoSession()
        navigate({ kind: "demo", sessionId: session.id })
      }}
    >
      <Sparkles size={16} />
      <span>Launch Interactive Demo</span>
      <ArrowUpRight size={16} />
    </button>
  )
}

function TaglineReveal() {
  const reduceMotion = useReducedMotion()
  const words = "One connected system for every order your store takes.".split(" ")
  return (
    <motion.section
      className="tagline-reveal-section section-shell"
      initial={reduceMotion ? false : { opacity: 0 }}
      whileInView={reduceMotion ? undefined : { opacity: 1 }}
      viewport={{ once: true, margin: "-100px" }}
      transition={{ duration: 0.6, ease: [0.25, 0.46, 0.45, 0.94] }}
    >
      <h2 className="tagline-reveal-text">
        {words.map((word, i) => (
          <motion.span
            key={i}
            className="tagline-word"
            initial={reduceMotion ? false : { opacity: 0.3, filter: "blur(4px)" }}
            whileInView={reduceMotion ? undefined : { opacity: 1, filter: "blur(0px)" }}
            viewport={{ once: true, margin: "-100px" }}
            transition={{ duration: 0.5, delay: reduceMotion ? 0 : i * 0.08, ease: [0.25, 0.46, 0.45, 0.94] }}
          >
            {word}{" "}
          </motion.span>
        ))}
      </h2>
    </motion.section>
  )
}

function SocialProof() {
  const reduceMotion = useReducedMotion()
  const testimonials = [
    { quote: "Switched from spreadsheets to Vendryx in one afternoon. Orders stopped falling through the cracks.", author: "Tahsin A.", role: "Store owner, Dhaka" },
    { quote: "The social order handoff saved us hours every week. Staff review drafts before anything goes live.", author: "Sabrina H.", role: "Operations lead" },
  ]
  return (
    <motion.section
      className="social-proof-section section-shell"
      initial={reduceMotion ? false : { opacity: 0, y: 24 }}
      whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-60px" }}
      transition={{ duration: 0.5, ease: [0.25, 0.46, 0.45, 0.94] }}
    >
      <div className="section-header-block">
        <h2>Merchants who made the switch.</h2>
      </div>
      <div className="social-proof-grid">
        {testimonials.map((t, i) => (
          <motion.div
            key={t.author}
            className="social-proof-card"
            initial={reduceMotion ? false : { opacity: 0, y: 20 }}
            whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-60px" }}
            transition={{ duration: 0.5, delay: reduceMotion ? 0 : i * 0.1, ease: [0.25, 0.46, 0.45, 0.94] }}
          >
            <p className="social-proof-quote">"{t.quote}"</p>
            <div className="social-proof-author">
              <strong>{t.author}</strong>
              <span>{t.role}</span>
            </div>
          </motion.div>
        ))}
      </div>
    </motion.section>
  )
}

function Hero() {
  const ease = [0.25, 0.46, 0.45, 0.94] as const
  const reduceMotion = useReducedMotion()
  return (
    <section className="hero collective-hero section-shell">
      <div className="hero-split-grid">
        <div className="hero-left-column">
          <motion.p className="eyebrow" initial={reduceMotion ? false : { opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5, delay: reduceMotion ? 0 : 0, ease }}>Commerce operations by Syed Systems</motion.p>
          <motion.h1 initial={reduceMotion ? false : { opacity: 0, y: 16 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.6, delay: reduceMotion ? 0 : 0.1, ease }}>
            {"One commerce workspace for the store you run.".split(" ").map((word, i) => (
              <motion.span key={i} className="inline-block" initial={reduceMotion ? false : { opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4, delay: reduceMotion ? 0 : 0.1 + i * 0.06, ease }}>
                {word}{" "}
              </motion.span>
            ))}
          </motion.h1>
          <motion.p className="hero-lede" initial={reduceMotion ? false : { opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5, delay: reduceMotion ? 0 : 0.2, ease }}>
            Vendryx brings catalogue, inventory, guest checkout, social-order handoff, and day-to-day order work into one connected system.
          </motion.p>
          <motion.div className="hero-actions" initial={reduceMotion ? false : { opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5, delay: reduceMotion ? 0 : 0.3, ease }}>
            <Link className="button button-primary" to="/signup">
              <span>Start your 14-day trial</span>
              <ArrowRight size={16} />
            </Link>
            <StartDemoButton />
            <a className="button button-outline" href={ADMIN_URL} title="Launch Vendryx Admin">
              <span>Open Admin</span>
              <ArrowUpRight size={16} />
            </a>
          </motion.div>
          <motion.ul className="hero-foundation-list" aria-label="Core Vendryx capabilities" initial={reduceMotion ? false : { opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5, delay: reduceMotion ? 0 : 0.4, ease }}>
            <li>Catalogue and inventory management</li>
            <li>Guest checkout and cash on delivery</li>
            <li>Staff-reviewed social-order handoff</li>
          </motion.ul>
        </div>
        <motion.figure className="hero-image-panel" initial={reduceMotion ? false : { opacity: 0, scale: 0.97 }} animate={{ opacity: 1, scale: 1 }} transition={{ duration: 0.7, delay: reduceMotion ? 0 : 0.2, ease }}>
          <img src="/art/merchant-workflow-v2.png" alt="A merchant preparing products alongside a commerce workflow." loading="eager" decoding="async" />
          <figcaption>A practical operating system for the work behind an online store.</figcaption>
        </motion.figure>
      </div>
    </section>
  )
}

function MerchantValuePillars() {
  const reduceMotion = useReducedMotion()
  const capabilities = [
    {
      icon: MessageSquareQuote,
      title: "Social order handoff",
      body: "WhatsApp opens with a pre-filled order message. Messenger and Instagram use a copy-then-open handoff, and staff review each resulting draft order.",
    },
    {
      icon: ShoppingBag,
      title: "Guest checkout and COD",
      body: "Buyers can browse, add to cart, and place orders without creating an account. Cash on delivery is available in every plan.",
    },
    {
      icon: Code,
      title: "A commerce core that can grow",
      body: "Trial and Free use a supported brand kit and subdomain. Pro can map a merchant-owned domain and add a developer-built storefront after payment.",
    },
  ]

  return (
    <motion.section className="value-pillars-section section-shell" id="features" aria-labelledby="features-heading" initial={reduceMotion ? false : { opacity: 0, y: 24 }} whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }} viewport={{ once: true, margin: "-60px" }} transition={{ duration: 0.5, ease: [0.25, 0.46, 0.45, 0.94] }}>
      <div className="section-header-block feature-brief-header">
        <h2 id="features-heading">Commerce operations that stay connected as your store changes.</h2>
        <p className="section-lede">Use one operational foundation for your storefront, catalogue, orders, and the work your staff performs each day.</p>
      </div>
      <div className="feature-brief-grid">
        {capabilities.map((capability, i) => {
          const Icon = capability.icon
          return (
            <motion.div key={capability.title} initial={reduceMotion ? false : { opacity: 0, y: 20 }} whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }} viewport={{ once: true, margin: "-60px" }} transition={{ duration: 0.5, delay: reduceMotion ? 0 : i * 0.1, ease: [0.25, 0.46, 0.45, 0.94] }}>
              <Card className="feature-brief-card">
                <CardHeader>
                  <span className="feature-brief-icon"><Icon size={20} /></span>
                  <CardTitle>{capability.title}</CardTitle>
                </CardHeader>
                <CardContent><p>{capability.body}</p></CardContent>
              </Card>
            </motion.div>
          )
        })}
      </div>
    </motion.section>
  )
}

function DemoTeaserCallout() {
  const reduceMotion = useReducedMotion()
  const { navigate } = useMarketing()
  return (
    <motion.section className="demo-teaser-section section-shell" initial={reduceMotion ? false : { opacity: 0, y: 24 }} whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }} viewport={{ once: true, margin: "-60px" }} transition={{ duration: 0.5, ease: [0.25, 0.46, 0.45, 0.94] }}>
      <div className="demo-teaser-card">
        <div className="teaser-left">
          <span className="teaser-badge">
            <Sparkles size={14} /> No sign up required
          </span>
          <h2>See the merchant workflow before you request a store.</h2>
          <p>
            Explore a sample workspace in your browser: adjust catalogue data, review a sample social-order handoff, and
            see how merchant and storefront activity connect. The demo uses sample data, not a live merchant store.
          </p>
          <div className="teaser-actions">
            <button
              type="button"
              className="button button-primary"
              onClick={() => {
                const session = createDemoSession()
                navigate({ kind: "demo", sessionId: session.id })
              }}
            >
              <span>Explore demo</span>
              <ArrowUpRight size={16} />
            </button>
          </div>
        </div>
        <figure className="teaser-image-panel">
          <img src="/art/stheno-merchant-launch-v3.png" alt="A merchant preparing products and orders at a workbench." loading="lazy" decoding="async" />
        </figure>
      </div>
    </motion.section>
  )
}

/* ==========================================================================
   HOW IT WORKS (THE 3-STEP MERCHANT PATH)
   ========================================================================== */

function MerchantJourney() {
  const reduceMotion = useReducedMotion()
  const steps = [
    {
      icon: Key,
      step: "01",
      tag: "Get started",
      title: "Submit a store request",
      body: "Verify your email, add the basic store details, and submit the request when you are ready to launch.",
      artifact: "A 14-day trial begins after verification",
    },
    {
      icon: Paintbrush,
      step: "02",
      tag: "Set up your store",
      title: "Configure catalog & brand identity",
      body: "Choose a store name and supported brand details, then add products or map them from a CSV import.",
      artifact: "Supported brand kit and subdomain",
    },
    {
      icon: Store,
      step: "03",
      tag: "Run day-to-day work",
      title: "Take and manage orders",
      body: "Use guest checkout, cash on delivery, and social-order handoff. Staff review draft orders before they become live work.",
      artifact: "Provider-dependent paid services are confirmed in the service order",
    },
  ]

  return (
    <motion.section className="merchant-journey section-shell" id="how-it-works" initial={reduceMotion ? false : { opacity: 0, y: 24 }} whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }} viewport={{ once: true, margin: "-60px" }} transition={{ duration: 0.5, ease: [0.25, 0.46, 0.45, 0.94] }}>
      <div className="journey-heading">
        <p className="eyebrow">How Vendryx works</p>
        <h2>From initial setup to active order fulfillment in 3 steps.</h2>
        <p className="section-lede">
          Start with the core workflow, then choose the plan and integrations that fit the operation you run.
        </p>
      </div>
      <div className="journey-grid-cards" style={{ position: "relative" }}>
        <svg className="journey-connector" viewBox="0 0 100 8" preserveAspectRatio="none" aria-hidden="true">
          <motion.line x1="16" y1="4" x2="84" y2="4" stroke="var(--border-medium)" strokeWidth="0.5" strokeDasharray="2 2" initial={reduceMotion ? false : { pathLength: 0 }} whileInView={reduceMotion ? undefined : { pathLength: 1 }} viewport={{ once: true }} transition={{ duration: 1.2, delay: reduceMotion ? 0 : 0.3, ease: "easeInOut" }} />
        </svg>
        {steps.map((item, i) => {
          const Icon = item.icon
          return (
            <motion.article
              className="journey-step-card"
              key={item.title}
              initial={reduceMotion ? false : { opacity: 0, y: 20 }}
              whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }}
              viewport={{ once: true, margin: "-60px" }}
              transition={{ duration: 0.5, delay: reduceMotion ? 0 : i * 0.12, ease: [0.25, 0.46, 0.45, 0.94] }}
            >
              <div className="journey-card-topbar">
                <div className="journey-num-badge">{item.step}</div>
                <span className="journey-chip">{item.tag}</span>
              </div>
              <div className="journey-icon-circle">
                <Icon size={22} />
              </div>
              <h3>{item.title}</h3>
              <p>{item.body}</p>
              <div className="journey-artifact-pill">
                <CheckCircle2 size={13} className="text-accent" />
                <span>{item.artifact}</span>
              </div>
            </motion.article>
          )
        })}
      </div>
    </motion.section>
  )
}

/* ==========================================================================
   MERCHANT STORIES & VERIFIED REVIEWS
   ========================================================================== */

function MarketingFaq() {
  const reduceMotion = useReducedMotion()
  const faqs = [
    ["What is Vendryx?", "A commerce operations system for independent merchants. It connects your storefront with catalogue, inventory, guest checkout, orders, and staff work."],
    ["Do buyers need to create an account?", "No. Vendryx is built around guest checkout. You can still recognize repeat buyers from the details collected with each order."],
    ["How do social orders work?", "WhatsApp opens with a pre-filled order message. Messenger and Instagram use a copy-then-open handoff. Each creates a draft order for staff to review before it goes live."],
    ["What happens after the 14-day trial?", "Your store moves to the Free plan. Products, staff records, and order data stay intact. Free limits apply to new creation until you upgrade or reduce usage."],
    ["What does Pro add?", "Full analytics history, a bespoke storefront service, payment gateway, courier dispatch, and risk tooling where supported. Plus mapping for your own domain. $20/month, 1-month minimum."],
    ["Can I cancel anytime?", "Yes. No long-term contracts. Cancel from your workspace settings and your data stays available for 30 days after cancellation."],
    ["Do you offer support?", "Yes. Email support@syedsystems.com for merchant questions. Pro merchants get priority response within 24 hours."],
  ] as const

  return (
    <motion.section className="faq-section section-shell" id="faq" aria-labelledby="faq-heading" initial={reduceMotion ? false : { opacity: 0, y: 24 }} whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }} viewport={{ once: true, margin: "-60px" }} transition={{ duration: 0.5, ease: [0.25, 0.46, 0.45, 0.94] }}>
      <p className="eyebrow">Questions before you start</p>
      <h2 id="faq-heading">Clear answers for merchants, builders, and developers.</h2>
      <Accordion className="faq-list vendryx-faq" type="single" collapsible defaultValue="faq-0">
        {faqs.map(([question, answer], index) => (
          <AccordionItem className="faq-item" key={question} value={`faq-${index}`}>
            <AccordionTrigger className="faq-question-btn">{question}</AccordionTrigger>
            <AccordionContent className="faq-answer"><p>{answer}</p></AccordionContent>
          </AccordionItem>
        ))}
      </Accordion>
    </motion.section>
  )
}


/* ==========================================================================
   WHO IT'S FOR: AUDIENCE & STAKEHOLDERS (STORE OWNERS, BRANDS, INVESTORS)
   ========================================================================== */



function PricingSection() {
  const reduceMotion = useReducedMotion()
  const plans = [
    {
      name: "Trial",
      term: "14 days",
      bestFor: "Evaluate the complete core workflow before choosing a long-term plan.",
      highlights: ["Unlimited products during the trial", "Up to 10 staff seats", "Supported brand kit and subdomain"],
      action: "Start a trial",
    },
    {
      name: "Free",
      term: "Indefinite after trial",
      bestFor: "Run essential commerce operations with clear operating limits.",
      highlights: ["100 new products", "Owner plus two staff seats", "COD, CSV import, web push, and 30-day analytics"],
      action: "Choose Free",
    },
    {
      name: "Pro",
      term: "6 or 12 months",
      bestFor: "Operate a custom store with paid integrations and full analytics.",
      highlights: ["Unlimited products and up to 10 staff seats", "Custom domain and bespoke storefront service", "Gateway, courier, and risk tooling where supported"],
      action: "Discuss Pro",
      featured: true,
    },
  ]

  const sections = [
    {
      title: "Store operations",
      rows: [
        ["Catalogue and inventory", "Create products, organise the catalogue, and manage inventory in one workspace.", "Included for evaluation", "Included", "Included"],
        ["Product capacity", "The number of new products your store can add.", "Unlimited for 14 days", "100 new products", "Unlimited"],
        ["Team access", "People who can work on products, orders, and fulfilment.", "Up to 10 seats", "Owner plus 2 staff seats", "Up to 10 seats"],
        ["Orders and guest checkout", "Let buyers browse, add to cart, and place orders without creating an account.", "Included for evaluation", "Included", "Included"],
        ["Cash on delivery", "Manage cash-on-delivery orders in the standard order workflow.", "Included for evaluation", "Included", "Included"],
        ["CSV catalogue import", "Bring a product catalogue into Vendryx with a CSV import.", "Included for evaluation", "Included", "Included"],
        ["Social order handoff", "Create staff-reviewable draft orders from supported social conversations.", "Included for evaluation", "Included", "Included"],
      ],
    },
    {
      title: "Storefront and reporting",
      rows: [
        ["Brand kit and Vendryx subdomain", "Launch with supported brand details on a Vendryx-hosted subdomain.", "Included", "Included", "Included"],
        ["Analytics", "Review store activity and reporting available to your plan.", "Basic", "Basic, 30-day window", "Full history"],
        ["Web push", "Send supported browser notifications to opted-in buyers.", "Included", "Included", "Included"],
        ["Custom domain", "Use your own domain name for your customer-facing storefront.", "Not included", "Not included", "Included"],
        ["Bespoke storefront service", "A developer-built storefront supplied after the agreed Pro payment.", "Not included", "Not included", "Included after payment"],
      ],
    },
    {
      title: "Paid operational services",
      rows: [
        ["Payment gateway connection", "Connect payment services when they are supported for your merchant and region.", "Not included", "Not included", "Available where supported"],
        ["Courier connection", "Connect supported delivery providers to the store workflow.", "Not included", "Not included", "Available where supported"],
        ["Buyer fraud and risk score", "Use supported risk tools where they are available to the merchant.", "Not included", "Not included", "Available where supported"],
        ["Plan term and renewal", "The period before the merchant renews or changes plan.", "14 days", "Indefinite", "6 or 12 months, as agreed"],
      ],
    },
  ] as const

  return (
    <motion.section className="pricing-section section-shell pricing-section-v2" id="pricing" aria-labelledby="pricing-heading" initial={reduceMotion ? false : { opacity: 0, y: 24 }} whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }} viewport={{ once: true, margin: "-60px" }} transition={{ duration: 0.5, ease: [0.25, 0.46, 0.45, 0.94] }}>
      <div className="section-header-block pricing-header">
        <h2 id="pricing-heading">Start with a clear path. Move to Pro when your operation needs more.</h2>
        <p className="section-lede">Every merchant starts with the same commerce foundation. The difference is the capacity, customisation, and integration support the store needs.</p>
      </div>
      <div className="pricing-tier-grid">
        {plans.map((plan, i) => (
          <motion.div key={plan.name} initial={reduceMotion ? false : { opacity: 0, y: 20 }} whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }} viewport={{ once: true, margin: "-60px" }} transition={{ duration: 0.5, delay: reduceMotion ? 0 : i * 0.1, ease: [0.25, 0.46, 0.45, 0.94] }}>
            <Card className={`pricing-tier-card ${plan.featured ? "is-featured" : ""}`}>

            <CardHeader>
              <div className="pricing-tier-topline"><CardTitle>{plan.name}</CardTitle>{plan.featured && <Badge>For growing operations</Badge>}</div>
              <p className="pricing-tier-term">{plan.term}</p>
            </CardHeader>
            <CardContent>
              <p className="pricing-tier-fit"><strong>Choose {plan.name}</strong> if you need to {plan.bestFor.charAt(0).toLowerCase() + plan.bestFor.slice(1)}</p>
              <ul className="pricing-tier-highlights">{plan.highlights.map((highlight) => <li key={highlight}><Check size={16} />{highlight}</li>)}</ul>
            </CardContent>
            <CardFooter><Link className={`button button-full ${plan.featured ? "button-primary" : "button-outline"}`} to="/signup">{plan.action}<ArrowRight size={15} /></Link></CardFooter>
          </Card>
          </motion.div>
        ))}
      </div>
      <section className="service-comparison" aria-labelledby="service-comparison-heading">
        <div className="service-comparison-heading">
          <h3 id="service-comparison-heading">Compare every service before you choose a plan.</h3>
          <p>Each row explains what the service does and how it is available in Trial, Free, and Pro.</p>
        </div>
        <div className="service-comparison-table-wrap" role="region" aria-label="Vendryx plan service comparison" tabIndex={0}>
          <Table className="service-comparison-table">
            <TableHeader><TableRow><TableHead>Service</TableHead><TableHead>Trial</TableHead><TableHead>Free</TableHead><TableHead>Pro</TableHead></TableRow></TableHeader>
            <TableBody>
              {sections.map((section) => <Fragment key={section.title}>
                <TableRow className="service-comparison-group"><TableCell colSpan={4}>{section.title}</TableCell></TableRow>
                {section.rows.map(([name, description, trial, free, pro]) => <TableRow key={name}>
                  <TableCell><strong>{name}</strong><span>{description}</span></TableCell><TableCell>{trial}</TableCell><TableCell>{free}</TableCell><TableCell>{pro}</TableCell>
                </TableRow>)}
              </Fragment>)}
            </TableBody>
          </Table>
        </div>
      </section>
      <p className="pricing-disclosure">Paid-plan price, tax, regional availability, and provider-dependent features are confirmed in the service order before a merchant accepts Pro.</p>
    </motion.section>
  )
}



function Closing() {
  const reduceMotion = useReducedMotion()
  return (
    <motion.section className="closing section-shell" initial={reduceMotion ? false : { opacity: 0, y: 24 }} whileInView={reduceMotion ? undefined : { opacity: 1, y: 0 }} viewport={{ once: true, margin: "-60px" }} transition={{ duration: 0.5, ease: [0.25, 0.46, 0.45, 0.94] }}>
      <Building2 size={28} className="text-accent" />
      <h2>Start a trial and see how orders flow from cart to fulfillment.</h2>
      <p className="section-lede">Built for store owners, ecommerce builders, and developers. 14-day trial, no credit card required, cancel anytime.</p>
      <div className="closing-actions">
        <Link className="button button-primary" to="/signup">
          <span>Start your 14-day trial</span>
          <ArrowRight size={16} />
        </Link>
        <Link className="button button-outline" to="#pricing">
          <span>See plan details</span>
          <ArrowRight size={14} />
        </Link>
        <StartDemoButton />
      </div>
    </motion.section>
  )
}

function Footer({ onOpenEngineersTalk }: { onOpenEngineersTalk: () => void }) {
  const { navigate } = useMarketing()

  const startDemo = () => {
    const session = createDemoSession()
    navigate({ kind: "demo", sessionId: session.id })
  }

  return (
    <footer className="site-footer" role="contentinfo">
      <div className="footer-inner section-shell">
        {/* Top Callout Strip / Quick Action */}
        <div className="footer-cta-strip">
          <div className="footer-cta-info">
            <span className="footer-cta-badge">
              Vendryx by Syed Systems
            </span>
            <h3>Ready to take full ownership of your store operations?</h3>
            <p>Start with a 14-day trial, then choose the operating capacity and paid services your store needs.</p>
          </div>
          <div className="footer-cta-actions">
            <Link className="button button-primary" to="/signup">
              <span>Start Free Trial</span>
              <ArrowRight size={15} />
            </Link>
            <button type="button" className="button button-outline" onClick={startDemo}>
              <Sparkles size={15} />
              <span>Try Sandbox Demo</span>
            </button>
          </div>
        </div>

        {/* Main Footer Multi-Column Grid */}
        <div className="footer-grid">
          {/* Brand & Mission Column */}
          <div className="footer-brand-col">
            <Link className="brand" to="/" aria-label="Vendryx home">
              <Mark />
              <span className="sr-only">Vendryx</span>
            </Link>
            <p className="footer-brand-tagline">
              Commerce operations for independent merchants, ecommerce builders, and developers.
            </p>
            <div className="footer-syed-stamp">
              <span>A product of <strong>Syed Systems</strong>, the software and IT division of SMCTi.</span>
            </div>
          </div>

          {/* Nav Columns */}
          <div className="footer-nav-col">
            <h4>Platform</h4>
            <ul>
              <li><Link to="#features">Guest Checkout</Link></li>
              <li><Link to="#features">Social Order Handoff</Link></li>
              <li><Link to="#features">Inventory & Catalog</Link></li>
              <li><Link to="#features">Order operations</Link></li>
              <li><button type="button" className="footer-text-btn" onClick={startDemo}>Interactive Demo Sandbox</button></li>
              <li><Link to="#pricing">Plans & Pricing</Link></li>
            </ul>
          </div>

          <div className="footer-nav-col">
            <h4>Merchants</h4>
            <ul>
              <li><Link to="/signup">Start Free Trial</Link></li>
              <li><a href={ADMIN_URL}>Launch Admin Dashboard</a></li>
              <li><a href={ADMIN_LOGIN_URL}>Merchant Workspace Login</a></li>
              <li><Link to="#pricing">Plan comparison</Link></li>
              <li><Link to="#how-it-works">How Vendryx Works</Link></li>
              <li><Link to="#faq">Merchant FAQ</Link></li>
              <li><Link to="/privacy">Privacy Notice</Link></li>
              <li><Link to="/terms">Terms of Service</Link></li>
              <li><a href="mailto:support@syedsystems.com">Merchant Support Desk</a></li>
            </ul>
          </div>
        </div>

        {/* Bottom Sub-Footer Bar */}
        <div className="footer-bottom-bar">
          <div className="footer-bottom-left">
            <p>© 2026 Vendryx by Syed Systems. All rights reserved.</p>
            <span className="footer-tax-badge">Plan, tax, and provider terms are confirmed before a paid term begins.</span>
          </div>
          <div className="footer-bottom-right">
            <Link to="/privacy">Privacy</Link>
            <span className="dot-sep">·</span>
            <Link to="/terms">Terms</Link>
            <span className="dot-sep">·</span>
            <button type="button" className="footer-sub-btn" onClick={onOpenEngineersTalk}>
              <Terminal size={13} />
              <span>Developer API</span>
            </button>
          </div>
        </div>
      </div>
    </footer>
  )
}

/* ==========================================================================
   ENGINEER'S TALK MODAL (TECHNICAL DEEP-DIVE FOR ARCHITECTS & DEVELOPERS)
   ========================================================================== */

function EngineersTalkModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const [selectedLayer, setSelectedLayer] = useState<number>(0)
  const [apiSnippetLang, setApiSnippetLang] = useState<"ts" | "curl" | "rust">("ts")
  const [copied, setCopied] = useState(false)

  const layers = [
    {
      title: "Rust Starved Primitives",
      tag: "stheno-types",
      icon: Box,
      desc: "Zero-dependency starved types (Money, Id<T>, Decimal) that guarantee exact mathematical precision with zero floating-point arithmetic errors.",
      specs: ["100% Rust-native decimal math", "Deterministic transaction boundaries", "Zero cross-boundary leakages"],
    },
    {
      title: "Isolated Postgres Schemas",
      tag: "Schema Isolation",
      icon: Package,
      desc: "Each domain module (Product, Cart, Order, Pricing) owns exactly one Postgres schema with zero lateral foreign keys. Raw SQL never crosses schemas.",
      specs: ["Module isolation enforced by CI", "Event-driven subscribers", "Clean audit trails"],
    },
    {
      title: "Idempotent Sagas & Workflows",
      tag: "Durable Sagas",
      icon: GitFork,
      desc: "Multi-step sagas coordinate state mutations with automatic compensation on crash and guaranteed idempotency across payment providers.",
      specs: ["Crash-and-resume resilience", "Automated step compensations", "PSP idempotency key locks"],
    },
    {
      title: "Headless Presentation Contracts",
      tag: "TypeScript Contract",
      icon: Store,
      desc: "Strictly pure storefront presentation modules that plug into Vendryx APIs with zero backend logic modification.",
      specs: ["Sub-50ms TTFB globally", "Pure presentation architecture", "Full theme customizability"],
    },
  ]

  const apiSnippets = {
    curl: `curl -X POST https://api.vendryx.com/store/carts/cart_01HX/complete \\
  -H "Authorization: Bearer sk_live_demo_984" \\
  -H "Content-Type: application/json" \\
  -d '{
    "payment_collection_id": "pay_col_01HX",
    "provider_id": "bkash",
    "idempotency_key": "idem_849204820"
  }'`,
    ts: `import { SthenoClient } from "@stheno/sdk";

const client = new SthenoClient({ apiKey: "sk_live_demo_984" });

const order = await client.carts.complete("cart_01HX", {
  paymentCollectionId: "pay_col_01HX",
  providerId: "bkash",
  idempotencyKey: "idem_849204820",
});

console.log("Order confirmed:", order.id);`,
    rust: `use stheno_client::{Client, CompleteCartInput};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("sk_live_demo_984")?;
    let order = client.carts().complete("cart_01HX", CompleteCartInput {
        payment_collection_id: "pay_col_01HX",
        provider_id: "bkash",
        idempotency_key: "idem_849204820",
    }).await?;

    println!("Order confirmed: {:?}", order.id);
    Ok(())
}`,
  }

  const handleCopySnippet = () => {
    navigator.clipboard.writeText(apiSnippets[apiSnippetLang])
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  if (!isOpen) return null

  return (
    <AnimatePresence>
      <div className="modal-backdrop" onClick={onClose}>
        <motion.div
          className="modal-card engineers-talk-modal"
          onClick={(e) => e.stopPropagation()}
          initial={{ opacity: 0, scale: 0.96, y: 15 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.96, y: 15 }}
        >
          <div className="engineers-talk-header">
            <div className="header-title-wrap">
              <Terminal size={24} className="terminal-icon" />
              <div>
                <h2>Engineer's Talk: Under the Hood</h2>
                <p>System architecture, database schema isolation, and latency benchmarks for technical teams.</p>
              </div>
            </div>
            <button type="button" className="modal-close-btn" onClick={onClose}>
              <X size={20} />
            </button>
          </div>

          <div className="engineers-talk-body">
            {/* Architecture Layers */}
            <div className="tech-layers-block">
              <span className="block-label">Architecture Primitives</span>
              <div className="explorer-tabs" role="tablist">
                {layers.map((layer, index) => {
                  const Icon = layer.icon
                  return (
                    <button
                      key={layer.title}
                      role="tab"
                      aria-selected={selectedLayer === index}
                      className={`explorer-tab-btn ${selectedLayer === index ? "is-active" : ""}`}
                      onClick={() => setSelectedLayer(index)}
                    >
                      <Icon size={16} />
                      <span>{layer.title}</span>
                    </button>
                  )
                })}
              </div>

              <div className="explorer-content-card">
                <div className="card-tag">{layers[selectedLayer].tag}</div>
                <h3>{layers[selectedLayer].title}</h3>
                <p className="layer-desc">{layers[selectedLayer].desc}</p>
                <ul className="layer-spec-list">
                  {layers[selectedLayer].specs.map((spec) => (
                    <li key={spec}>
                      <Check size={14} className="text-accent" />
                      <span>{spec}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>

            {/* Benchmarks Grid */}
            <div className="tech-benchmarks-block">
              <span className="block-label">Measured Execution Performance</span>
              <div className="benchmark-grid">
                <div className="benchmark-card highlighted">
                  <div className="benchmark-header">
                    <Zap size={20} className="bm-icon text-accent" />
                    <span>Time-to-First-Byte</span>
                  </div>
                  <strong className="bm-value">32ms</strong>
                  <span className="bm-subtext">Vendryx Global Edge</span>
                  <small className="bm-comparison">vs 380ms legacy monoliths</small>
                </div>

                <div className="benchmark-card">
                  <div className="benchmark-header">
                    <Gauge size={20} className="bm-icon" />
                    <span>Saga Commit Latency</span>
                  </div>
                  <strong className="bm-value">&lt; 8ms</strong>
                  <span className="bm-subtext">Atomic State Transitions</span>
                  <small className="bm-comparison">Guaranteed zero double-charges</small>
                </div>

                <div className="benchmark-card">
                  <div className="benchmark-header">
                    <Box size={20} className="bm-icon" />
                    <span>Heap Footprint</span>
                  </div>
                  <strong className="bm-value">18MB</strong>
                  <span className="bm-subtext">Native Single Process Binary</span>
                  <small className="bm-comparison">vs 1.2GB node_modules heap</small>
                </div>
              </div>
            </div>

            {/* REST API Explorer */}
            <div className="tech-api-block">
              <div className="api-block-header">
                <span className="block-label">Developer Surface & REST API</span>
                <div className="api-lang-tabs">
                  {(["ts", "curl", "rust"] as const).map((lang) => (
                    <button
                      key={lang}
                      type="button"
                      className={`api-lang-btn ${apiSnippetLang === lang ? "is-active" : ""}`}
                      onClick={() => setApiSnippetLang(lang)}
                    >
                      {lang === "ts" ? "TypeScript SDK" : lang === "curl" ? "cURL" : "Rust Client"}
                    </button>
                  ))}
                </div>
              </div>

              <div className="code-terminal-card">
                <div className="terminal-header">
                  <div className="terminal-dots">
                    <span className="dot red" />
                    <span className="dot yellow" />
                    <span className="dot green" />
                  </div>
                  <span className="terminal-title">POST /store/carts/&#123;id&#125;/complete</span>
                  <button type="button" className="copy-code-btn" onClick={handleCopySnippet}>
                    {copied ? <Check size={14} /> : <Copy size={14} />}
                    <span>{copied ? "Copied!" : "Copy Code"}</span>
                  </button>
                </div>
                <pre className="terminal-code-body">
                  <code>{apiSnippets[apiSnippetLang]}</code>
                </pre>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  )
}

/* ==========================================================================
   DEDICATED FULL DEMO PAGE (ADMIN ON TOP, REAL STOREFRONT BELOW)
   ========================================================================== */

interface ProductReview {
  id: string
  author: string
  rating: number
  date: string
  comment: string
  verified: boolean
}

const defaultReviews: Record<string, ProductReview[]> = {
  "PRD-78201": [
    { id: "REV-1", author: "Tahsin Alam", rating: 5, date: "2 days ago", comment: "The canvas thickness and leather handle finish feel exceptionally premium. Perfect everyday work tote.", verified: true },
    { id: "REV-2", author: "Sabrina Hossain", rating: 5, date: "1 week ago", comment: "Stitching is flawless. Great depth and holds my 15-inch laptop comfortably.", verified: true },
  ],
  "PRD-78204": [
    { id: "REV-3", author: "Raihan Kabir", rating: 5, date: "3 days ago", comment: "Even flow rate and beautiful rustic glaze. My morning pour-over ritual is transformed.", verified: true },
  ],
}

type AdminTab = "dashboard" | "social" | "products" | "orders" | "brand"

const catalogPresets: Record<string, DemoProduct[]> = {
  leather: [
    { id: "PRD-78201", title: "Canvas Market Tote", price: 850, stock: 24, category: "Accessories", status: "Published", description: "Heavyweight organic cotton canvas with vegetable-tanned leather handles." },
    { id: "PRD-78202", title: "Minimalist Card Sleeve", price: 650, stock: 18, category: "Accessories", status: "Published", description: "Hand-stitched full grain pull-up leather with 3 card slots." },
    { id: "PRD-78203", title: "Studio Tool Roll", price: 1400, stock: 12, category: "Accessories", status: "Published", description: "Waxed canvas roll with brass buckle for atelier tools." },
  ],
  ceramics: [
    { id: "PRD-78204", title: "Ceramic Pour-Over Dripper", price: 1200, stock: 15, category: "Living", status: "Published", description: "Wheel-thrown terracotta dripper with matte stone glaze." },
    { id: "PRD-78205", title: "Studio Stoneware Mug", price: 750, stock: 30, category: "Living", status: "Published", description: "Durable high-fire stoneware mug with comfortable ergonomic handle." },
    { id: "PRD-78206", title: "Fluted Ceramic Serving Bowl", price: 1850, stock: 8, category: "Living", status: "Published", description: "Handcrafted centerpiece fruit and salad bowl." },
  ],
  textiles: [
    { id: "PRD-78207", title: "Handwoven Jamdani Scarf", price: 2400, stock: 10, category: "Apparel", status: "Published", description: "Extra-fine handspun cotton with traditional geometric motifs." },
    { id: "PRD-78208", title: "Linen Studio Apron", price: 1650, stock: 16, category: "Apparel", status: "Published", description: "Breathable natural flax linen with deep front split pockets." },
    { id: "PRD-78209", title: "Indigo Dyed Table Runner", price: 1150, stock: 14, category: "Living", status: "Published", description: "Pure organic cotton dyed with natural plant indigo." },
  ],
}

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

function AuthShell({
  children,
  eyebrow,
  title,
  intro,
}: {
  children: ReactNode
  eyebrow: string
  title: string
  intro: string
}) {
  return (
    <section className="auth-page section-shell">
      <div className="auth-aside">
        <Breadcrumbs items={[{ label: eyebrow }]} />
        <p className="eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p>{intro}</p>
      </div>
      <div className="auth-card">{children}</div>
    </section>
  )
}

function Signup() {
  const { navigate } = useMarketing()
  const [details, setDetails] = useState<{ email?: string; businessName?: string; whatsapp?: string; category?: string; plan?: string }>({})

  function submit(event: FormEvent) {
    event.preventDefault()
    localStorage.setItem("stheno-account-created", "true")
    navigate("/welcome")
  }

  return (
    <AuthShell
      eyebrow="Create Merchant Account"
      title="Make space for the store you want to run."
      intro="Start with email or Google. Optional business details can be filled now or saved for later."
    >
      <form className="auth-form" onSubmit={submit}>
        <button type="button" className="google-button">
          <GoogleIcon size={18} />
          <span>Continue with Google</span>
        </button>
        <div className="form-separator">
          <span>or continue with email</span>
        </div>
        <label>
          Email address
          <input
            required
            type="email"
            autoComplete="email"
            placeholder="merchant@yourbusiness.com"
            value={details.email ?? ""}
            onChange={(e) => setDetails({ ...details, email: e.target.value })}
          />
        </label>
        <fieldset>
          <legend>Optional business details</legend>
          <div className="form-pair">
            <label>
              Shop name
              <input
                placeholder="Nodi Studio"
                value={details.businessName ?? ""}
                onChange={(e) => setDetails({ ...details, businessName: e.target.value })}
              />
            </label>
            <label>
              WhatsApp number
              <input
                type="tel"
                placeholder="+880 1700..."
                value={details.whatsapp ?? ""}
                onChange={(e) => setDetails({ ...details, whatsapp: e.target.value })}
              />
            </label>
          </div>
          <label>
            Category
            <input
              placeholder="Fashion, ceramics, home goods..."
              value={details.category ?? ""}
              onChange={(e) => setDetails({ ...details, category: e.target.value })}
            />
          </label>
          <label>
            Intended plan
            <select
              value={details.plan ?? "free"}
              onChange={(e) => setDetails({ ...details, plan: e.target.value })}
            >
              <option value="free">Free Tier (14-day trial)</option>
              <option value="pro">Pro Tier (price confirmed in service order)</option>
            </select>
          </label>
        </fieldset>
        <button className="button button-primary button-full" type="submit">
          <span>Create account</span>
          <ArrowRight size={15} />
        </button>
        <p className="form-meta">
          Already have an account? <Link to="/login">Log in</Link>
        </p>
      </form>
    </AuthShell>
  )
}

function Login() {
  const [showPassword, setShowPassword] = useState(false)

  return (
    <AuthShell
      eyebrow="Merchant Sign In"
      title="Welcome back to your workspace."
      intro="Access your active merchant dashboard and ongoing operations."
    >
      <div style={{ marginBottom: "16px" }}>
        <a
          href={ADMIN_LOGIN_URL}
          className="button button-primary button-full flex items-center justify-center gap-2"
          style={{ textDecoration: "none", marginBottom: "12px" }}
        >
          <span>Sign In with Clerk to Admin Dashboard</span>
          <ArrowUpRight size={16} />
        </a>
      </div>
      <form
        className="auth-form"
        onSubmit={(e) => {
          e.preventDefault()
          window.location.href = ADMIN_LOGIN_URL
        }}
      >
        <button type="button" className="google-button">
          <GoogleIcon size={18} />
          <span>Continue with Google</span>
        </button>
        <div className="form-separator">
          <span>or sign in with email</span>
        </div>
        <label>
          Email address
          <input required type="email" autoComplete="email" placeholder="merchant@yourbusiness.com" />
        </label>
        <label>
          Password
          <div className="password-input-wrap">
            <input
              required
              type={showPassword ? "text" : "password"}
              autoComplete="current-password"
              placeholder="Your password"
            />
            <button
              type="button"
              className="password-toggle-btn"
              onClick={() => setShowPassword((p) => !p)}
              aria-label={showPassword ? "Hide password" : "Show password"}
            >
              {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
          </div>
        </label>
        <div className="form-split">
          <label className="check-label">
            <input type="checkbox" /> Remember this device
          </label>
          <Link to="/forgot-password">Forgot password?</Link>
        </div>
        <button className="button button-primary button-full" type="submit">
          <span>Log in</span>
          <ArrowRight size={15} />
        </button>
        <p className="form-meta">
          New to Vendryx? <Link to="/signup">Create an account</Link>
        </p>
      </form>
    </AuthShell>
  )
}

function ForgotPassword() {
  const [sent, setSent] = useState(false)
  return (
    <AuthShell
      eyebrow="Password Reset"
      title="Restore access to your workspace."
      intro="Enter your merchant account email and we will dispatch a secure reset link."
    >
      <form
        className="auth-form"
        onSubmit={(e) => {
          e.preventDefault()
          setSent(true)
        }}
      >
        <label>
          Email address
          <input required type="email" autoComplete="email" placeholder="merchant@yourbusiness.com" />
        </label>
        {sent && (
          <p className="form-notice" role="status">
            If an account matches this email, a reset message is on its way.
          </p>
        )}
        <button className="button button-primary button-full" type="submit">
          <span>Send reset link</span>
          <ArrowRight size={15} />
        </button>
        <p className="form-meta">
          <Link to="/login">Back to log in</Link>
        </p>
      </form>
    </AuthShell>
  )
}

function Welcome() {
  return (
    <section className="welcome-page section-shell">
      <div className="status-panel">
        <div className="provision-live-indicator">
          <span className="status-indicator live" />
          <span>Tenant Environment Online</span>
        </div>
        <h1>Store provisioning complete.</h1>
        <p className="welcome-lede">
          Your workspace is live. Operations can now be managed from the merchant dashboard.
        </p>
      </div>

      <div className="next-steps-grid">
        <div className="next-step-card">
          <div className="step-icon">
            <ClipboardList size={20} />
          </div>
          <h3>Populate the catalogue</h3>
          <p>Add products, variants, and stock counts to prepare the storefront.</p>
        </div>
        <div className="next-step-card">
          <div className="step-icon">
            <CreditCard size={20} />
          </div>
          <h3>Confirm settlement details</h3>
          <p>Add bank or bKash details so COD collections and card sales can be routed to your account.</p>
        </div>
        <div className="next-step-card">
          <div className="step-icon">
            <Building2 size={20} />
          </div>
          <h3>Review store settings</h3>
          <p>Ensure the support number, currency, and dispatch addresses are correct.</p>
        </div>
      </div>

      <div className="welcome-actions">
        <a className="button button-primary" href={ADMIN_URL}>
          <span>Open Merchant Dashboard</span>
          <ArrowUpRight size={15} />
        </a>
      </div>
    </section>
  )
}

function OnePageMarketingSite({ onOpenEngineersTalk }: { onOpenEngineersTalk: () => void }) {
  return (
    <>
      <Hero />
      <TaglineReveal />
      <MerchantJourney />
      <MerchantValuePillars />
      <DemoTeaserCallout />
      <SocialProof />
      <PricingSection />

      <MarketingFaq />
      <Closing />
      <Footer onOpenEngineersTalk={onOpenEngineersTalk} />
    </>
  )
}


export default function App() {
  const { page, navigate } = usePage()
  const [engineersTalkOpen, setEngineersTalkOpen] = useState(false)
  marketingNavigate = navigate
  const activePath = typeof page === "string" ? page : "/demo"

  useEffect(() => {
    const metadata = pageMetadata[activePath]
    if (!metadata) return

    document.title = metadata.title
    document.querySelector('meta[name="description"]')?.setAttribute("content", metadata.description)
    document.querySelector('meta[name="robots"]')?.setAttribute("content", metadata.indexable === false ? "noindex,nofollow" : "index,follow")

    let canonical = document.querySelector('link[rel="canonical"]') as HTMLLinkElement | null
    if (!canonical) {
      canonical = document.createElement("link")
      canonical.rel = "canonical"
      document.head.appendChild(canonical)
    }
    canonical.href = "https://vendryx.com" + (activePath === "/" ? "/" : activePath)
  }, [activePath])

  if (typeof page !== "string") {
    return <DedicatedDemoPage sessionId={page.sessionId} />
  }

  if (page === "/demo") {
    const session = createDemoSession()
    return <DedicatedDemoPage sessionId={session.id} />
  }

  const content =
    page === "/" ? (
      <OnePageMarketingSite onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
    ) : page === "/privacy" ? (
      <>
        <PrivacyPage />
        <Footer onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
      </>
    ) : page === "/terms" ? (
      <>
        <TermsPage />
        <Footer onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
      </>
    ) : page === "/login" ? (
      <>
        <Login />
        <Footer onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
      </>
    ) : page === "/signup" ? (
      <>
        <Signup />
        <Footer onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
      </>
    ) : page === "/forgot-password" ? (
      <>
        <ForgotPassword />
        <Footer onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
      </>
    ) : page === "/welcome" ? (
      <>
        <Welcome />
        <Footer onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
      </>
    ) : (
      <>
        <NotFoundPage />
        <Footer onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
      </>
    )

  return (
    <main id="main-content" className={`marketing-app page-${page.slice(1) || "home"}`}>
      <Header />
      {content}
      <EngineersTalkModal isOpen={engineersTalkOpen} onClose={() => setEngineersTalkOpen(false)} />
    </main>
  )
}
