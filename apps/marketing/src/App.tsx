import {
  ArrowRight,
  ArrowUpRight,
  Sun,
  Moon,
  MessageSquareQuote,
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
  Eye,
  EyeOff,
  GitFork,
  Gauge,
  Key,
  Zap,
  Menu,
  Package,
  Paintbrush,
  ShoppingBag,
  Store,
  Sparkles,
  Terminal,
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

import { createDemoSession } from "./demo-session"
import { PrivacyPage, TermsPage } from "./content-pages"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { VendryxLogo } from "@/components/brand/VendryxLogo"
import { ADMIN_URL, ADMIN_LOGIN_URL } from "./lib/config"

import { lazy, Suspense } from "react"
const DemoPage = lazy(() => import("./demo/DemoPage"))

type StaticPage = "/" | "/demo" | "/privacy" | "/terms" | "/login" | "/signup" | "/forgot-password" | "/welcome" | "/404"
type DemoPage = { kind: "demo"; sessionId: string }
type Page = StaticPage | DemoPage

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
export function useMarketing() {
  return { navigate: marketingNavigate }
}

export function useTheme() {
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

export function Mark() {
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
    return (
    <Suspense fallback={<div className="demo-loading">Loading demo...</div>}>
      <DemoPage sessionId={page.sessionId} />
    </Suspense>
  )
  }

  if (page === "/demo") {
    const session = createDemoSession()
    return (
    <Suspense fallback={<div className="demo-loading">Loading demo...</div>}>
      <DemoPage sessionId={session.id} />
    </Suspense>
  )
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
