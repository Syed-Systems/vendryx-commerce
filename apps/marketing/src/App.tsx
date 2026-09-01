import {
  ArrowRight,
  ArrowDown,
  ArrowUpRight,
  Barcode,
  Calculator,
  CaretDown,
  ChatCircleDots,
  ChartLine,
  ClipboardText,
  Buildings,
  Check,
  CheckCircle,
  Code,
  Copy,
  CreditCard,
  Cube,
  DownloadSimple,
  Eye,
  EyeSlash,
  FlowArrow,
  Gauge,
  GoogleLogo,
  Key,
  Lightning,
  List,
  MagnifyingGlass,
  Package,
  PaintBrush,
  Plus,
  Printer,
  Quotes,
  ShoppingBag,
  ShoppingCartSimple,
  ShieldCheck,
  Star,
  Storefront,
  Sparkle,
  TerminalWindow,
  UploadSimple,
  WhatsappLogo,
  InstagramLogo,
  X,
} from "@phosphor-icons/react"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"
import {
  lazy,
  Suspense,
  useEffect,
  useState,
  type AnchorHTMLAttributes,
  type CSSProperties,
  type FormEvent,
  type ReactNode,
} from "react"
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
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"

const ScrollScene = lazy(async () => {
  const module = await import("./ScrollScene")
  return { default: module.ScrollScene }
})

type StaticPage = "/" | "/demo" | "/privacy" | "/terms" | "/login" | "/signup" | "/forgot-password" | "/welcome"
type DemoPage = { kind: "demo"; sessionId: string }
type Page = StaticPage | DemoPage
type Plan = "free" | "pro"
type Currency = "BDT" | "USD" | "EUR"

const accountDraftKey = "stheno-marketing-account-draft"
const launchDraftKey = "stheno-marketing-store-draft"

const pageMetadata: Partial<Record<StaticPage, { title: string; description: string; indexable?: boolean }>> = {
  "/": {
    title: "Stheno | Commerce operations for independent merchants",
    description: "Stheno helps independent merchants manage storefronts, catalogue, inventory, guest checkout, and order operations in one connected commerce system.",
  },
  "/privacy": {
    title: "Stheno privacy notice draft",
    description: "A product-specific draft describing how the Stheno service is designed to handle merchant and shopper data.",
    indexable: false,
  },
  "/terms": {
    title: "Stheno terms of service draft",
    description: "A product-specific draft describing the Stheno service model and terms still required before publication.",
    indexable: false,
  },
}

function formatMoney(bdtAmount: number, currency: Currency): string {
  if (currency === "USD") {
    const usd = bdtAmount / 120
    return `$${usd.toFixed(2)}`
  }
  if (currency === "EUR") {
    const eur = bdtAmount / 130
    return `€${eur.toFixed(2)}`
  }
  return `BDT ${bdtAmount.toLocaleString("en-BD")}`
}

function getPage(): Page {
  const path = window.location.pathname.replace(/\/$/, "") || "/"
  const demoMatch = path.match(/^\/demo\/([a-z0-9-]+)$/i)
  if (demoMatch) return { kind: "demo", sessionId: demoMatch[1] }
  if (path === "/demo") return "/demo"
  return (["/", "/privacy", "/terms", "/login", "/signup", "/forgot-password", "/welcome"] as StaticPage[]).includes(path as StaticPage)
    ? (path as StaticPage)
    : "/"
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

function Mark() {
  return (
    <span className="brand-logo-frame" aria-hidden="true">
      <img src="/brand/stheno-mark-transparent.png" alt="Stheno" className="brand-mark-img" />
      <span className="brand-text-group">
        <span className="brand-title">Stheno</span>
        <span className="brand-subtitle">by Syed Systems</span>
      </span>
    </span>
  )
}

function Header() {
  const [menuOpen, setMenuOpen] = useState(false)
  const [scrolled, setScrolled] = useState(false)
  const { navigate } = useMarketing()

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 20)
    window.addEventListener("scroll", onScroll, { passive: true })
    return () => window.removeEventListener("scroll", onScroll)
  }, [])

  const startDemo = () => {
    const session = createDemoSession()
    navigate({ kind: "demo", sessionId: session.id })
  }

  return (
    <header className={`site-header ${scrolled ? "is-scrolled" : ""}`}>
      <Link className="brand" to="/" onClick={() => setMenuOpen(false)} aria-label="Stheno home">
        <Mark />
        <span className="sr-only">Stheno</span>
      </Link>
      <nav className="desktop-nav" aria-label="Primary navigation">
        <a href="#how-it-works">How It Works</a>
        <Link to="#features">Features</Link>
        <Link to="#pricing">Plans</Link>
        <a href="#who-its-for">Who it is for</a>
        <a href="#faq">FAQ</a>
      </nav>
      <div className="nav-actions">
        <button type="button" className="button button-outline button-sm" onClick={startDemo}>
          <Sparkle size={14} weight="fill" />
          <span>Explore demo</span>
        </button>
        <Link className="nav-login" to="/login">
          Log in
        </Link>
        <Link className="button button-primary nav-cta" to="/signup">
          <span>Sign up</span>
          <ArrowUpRight size={15} weight="bold" />
        </Link>
      </div>
      <button
        className="menu-toggle"
        type="button"
        onClick={() => setMenuOpen((open) => !open)}
        aria-expanded={menuOpen}
        aria-controls="mobile-menu"
      >
        {menuOpen ? <X size={20} /> : <List size={20} />}
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
            <Link to="#pricing" onClick={() => setMenuOpen(false)}>Plans</Link>
            <a href="#who-its-for" onClick={() => setMenuOpen(false)}>
              Who it is for
            </a>
            <a href="#faq" onClick={() => setMenuOpen(false)}>
              FAQ
            </a>
            <button
              type="button"
              className="button button-outline button-sm"
              onClick={() => {
                setMenuOpen(false)
                startDemo()
              }}
            >
              <Sparkle size={14} weight="fill" />
              <span>Explore demo</span>
            </button>
            <Link to="/login" onClick={() => setMenuOpen(false)}>
              Log in
            </Link>
            <Link className="mobile-signup-link" to="/signup" onClick={() => setMenuOpen(false)}>
              Create Account
            </Link>
          </motion.nav>
        )}
      </AnimatePresence>
    </header>
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
      <Sparkle size={16} weight="fill" />
      <span>Launch Interactive Demo</span>
      <ArrowUpRight size={16} weight="bold" />
    </button>
  )
}

function Hero() {
  const reduceMotion = useReducedMotion()

  return (
    <section className="hero collective-hero section-shell">
      <motion.div
        className="hero-copy"
        initial={reduceMotion ? false : { opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, ease: [0.4, 0, 0.2, 1] }}
      >
        <p className="eyebrow">Stheno by Syed Systems, the software and IT division of SMCTi</p>
        <h1>
          One commerce workspace
          <br />
          for the store you run.
        </h1>
        <p className="hero-lede">
          Stheno helps independent merchants run a branded storefront, catalogue, inventory, guest checkout, and order
          work in one connected system. It is built for the practical work behind an online store, not just its theme.
        </p>
        <div className="hero-actions">
          <Link className="button button-primary" to="/signup">
            <span>Create your account</span>
            <ArrowRight size={16} weight="bold" />
          </Link>
          <StartDemoButton />
        </div>
      </motion.div>
    </section>
  )
}

/* ==========================================================================
   MERCHANT VALUE PILLARS (SOLVING REAL PAIN POINTS)
   ========================================================================== */

function MerchantValuePillars() {
  const pillars = [
    {
      icon: ChatCircleDots,
      title: "Social order handoff",
      subtitle: "Create a draft order from a customer conversation",
      desc: "Use a pre-filled WhatsApp message or copy an order message for Messenger and Instagram. Staff can match the conversation to a draft order and confirm it when the details are ready.",
      highlight: "Review before fulfilment",
    },
    {
      icon: ShoppingBag,
      title: "A store buyers can use without an account",
      subtitle: "Guest checkout with catalogue, cart, and order flow",
      desc: "Buyers can browse products and place orders without creating a customer account. Core checkout behaviour stays in the commerce system while storefront presentation can be tailored for the plan.",
      highlight: "Guest checkout",
    },
    {
      icon: CreditCard,
      title: "Orders, stock, and fulfilment",
      subtitle: "Keep the operational record beside the storefront",
      desc: "Manage products, variants, inventory, orders, and staff operations from the merchant workspace. Cash on delivery is core; paid operational integrations are available by plan and region.",
      highlight: "Plan-aware operations",
    },
    {
      icon: ShieldCheck,
      title: "A stable commerce core for builders",
      subtitle: "Keep presentation separate from commerce behaviour",
      desc: "Stheno is designed for developer-built storefronts, a documented commerce API, and durable commerce workflows, so a design can change without recreating the operational system.",
      highlight: "Built for extension",
    },
  ]

  return (
    <section className="value-pillars-section section-shell" id="features">
      <div className="section-header-block">
        <p className="eyebrow">Why choose Stheno</p>
        <h2>Commerce operations that stay connected as your store changes.</h2>
        <p className="section-lede">
          Choose Stheno when you need a practical operating system for sales, not another disconnected storefront layer.
        </p>
      </div>

      <div className="value-pillars-grid">
        {pillars.map((p) => {
          const Icon = p.icon
          return (
            <article className="pillar-card" key={p.title}>
              <div className="pillar-card-top">
                <div className="pillar-icon-wrap">
                  <Icon size={24} weight="regular" />
                </div>
                <span className="pillar-highlight-tag">{p.highlight}</span>
              </div>
              <h3>{p.title}</h3>
              <strong>{p.subtitle}</strong>
              <p>{p.desc}</p>
            </article>
          )
        })}
      </div>
    </section>
  )
}

/* ==========================================================================
   INTERACTIVE LIVE DEMO INVITATION CALLOUT
   ========================================================================== */

function DemoTeaserCallout() {
  const { navigate } = useMarketing()
  return (
    <section className="demo-teaser-section section-shell">
      <div className="demo-teaser-card">
        <div className="teaser-left">
          <span className="teaser-badge">
            <Sparkle size={14} weight="fill" /> No sign up required
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
              <span>Explore the interactive demo</span>
              <ArrowUpRight size={16} weight="bold" />
            </button>
          </div>
        </div>
        <div className="teaser-visual" aria-hidden="true">
          <div className="teaser-preview-window">
            <div className="window-dots">
              <span className="dot red" />
              <span className="dot yellow" />
              <span className="dot green" />
            </div>
            <div className="window-content-simulation">
              <div className="sim-row">
                <ChatCircleDots size={18} className="wa-icon" />
                <span>Customer: &quot;Can you help me place this order?&quot;</span>
                <span className="sim-btn">Create draft</span>
              </div>
              <div className="sim-order-box">
                <strong>Draft order ready for review</strong>
                <span>Confirm details before fulfilment</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}

/* ==========================================================================
   HOW IT WORKS (THE 3-STEP MERCHANT PATH)
   ========================================================================== */

function MerchantJourney() {
  const reduceMotion = useReducedMotion()
  const steps = [
    [
      Key,
      "Submit a store request",
      "Verify your email, add the basic store details, and submit the request when you are ready to launch.",
    ],
    [
      PaintBrush,
      "Set up your catalogue and storefront",
      "Choose a store name and supported brand details, then add products manually or map them from a CSV import.",
    ],
    [
      Storefront,
      "Take and manage orders",
      "Use guest checkout, cash on delivery, and social-order handoff. Staff review draft orders before they become live work.",
    ],
  ] as const

  return (
    <section className="merchant-journey section-shell" id="how-it-works">
      <div className="journey-heading">
      <p className="eyebrow">How it works</p>
      <h2>From a store request to a working commerce operation.</h2>
      </div>
      <ol className="journey-rail collective-journey" aria-label="Stheno onboarding journey">
        {steps.map(([Icon, title, body], index) => (
          <motion.li
            key={title}
            initial={reduceMotion ? false : { opacity: 0, y: 16 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, amount: 0.3 }}
            transition={{ delay: index * 0.08, duration: 0.4 }}
          >
            <div className="journey-step-number">0{index + 1}</div>
            <Icon size={24} weight="regular" className="journey-icon" />
            <strong>{title}</strong>
            <p>{body}</p>
          </motion.li>
        ))}
      </ol>
    </section>
  )
}

/* ==========================================================================
   MERCHANT STORIES & VERIFIED REVIEWS
   ========================================================================== */

function MerchantReviews() {
  return (
    <section className="merchant-stories-section section-shell" id="who-its-for">
      <div className="section-header-block">
        <p className="eyebrow">Who Stheno is for</p>
        <h2>One commerce core, useful to each person building or running the store.</h2>
      </div>
      <div className="value-pillars-grid">
        <article className="pillar-card">
          <h3>Store owners</h3>
          <p>Keep catalogue, stock, guest checkout, social-order handoff, and fulfilment work connected.</p>
        </article>
        <article className="pillar-card">
          <h3>Ecommerce builders</h3>
          <p>Use a supported brand kit or build an independent storefront module without moving commerce logic into the theme.</p>
        </article>
        <article className="pillar-card">
          <h3>Developers</h3>
          <p>Work with a Rust and Postgres commerce core designed around API compatibility, explicit data boundaries, and durable workflows.</p>
        </article>
      </div>
    </section>
  )

  const [activeStory, setActiveStory] = useState(0)

  const stories = [
    {
      brand: "Nodi Studio",
      craft: "Handcrafted Canvas & Leatherwork",
      quote:
        "Converting WhatsApp customer chats directly into fulfillment orders removed hours of manual tracking every evening. Our customer response time dropped from 40 minutes to under 2.",
      founder: "Ayesha Rahman, Founder",
      metric: "+34% Order Velocity",
      highlight: "Direct Social-to-Order Conversion",
      tag: "DTC Craft",
      rating: 5,
    },
    {
      brand: "Bengal Clayworks",
      craft: "Studio Stoneware & Living",
      quote:
        "With legacy platforms, plugin costs kept eating our margins every month. Stheno gave us a clean, fast storefront with zero add-on taxes and instant inventory synchronization.",
      founder: "Tanvir Hasan, Lead Potter",
      metric: "BDT 210,000 Saved / yr",
      highlight: "Zero App Subscriptions",
      tag: "Artisan Ceramics",
      rating: 5,
    },
    {
      brand: "Dacca Loom",
      craft: "Heritage Jamdani & Silk Textiles",
      quote:
        "Our team manages custom weaving orders and immediate store inventory from one calm dashboard. It has eliminated duplicate orders and stock discrepancies completely.",
      founder: "Farhana Islam, Creative Director",
      metric: "100% Inventory Accuracy",
      highlight: "Unified Multi-Channel Queue",
      tag: "Heritage Textiles",
      rating: 5,
    },
  ]

  return (
    <section className="merchant-stories-section section-shell" id="reviews">
      <div className="section-header-block">
        <p className="eyebrow">Merchant Stories & Verified Feedback</p>
        <h2>Trusted by independent merchants who value craft and operational calm.</h2>
      </div>

      <div className="stories-grid">
        <div className="story-nav-list" role="tablist">
          {stories.map((story, index) => (
            <button
              key={story.brand}
              type="button"
              role="tab"
              aria-selected={activeStory === index}
              className={`story-nav-item ${activeStory === index ? "is-active" : ""}`}
              onClick={() => setActiveStory(index)}
            >
              <div className="story-nav-header">
                <strong>{story.brand}</strong>
                <span className="story-tag-pill">{story.tag}</span>
              </div>
              <span className="story-craft-label">{story.craft}</span>
              <span className="story-metric-badge">{story.metric}</span>
            </button>
          ))}
        </div>

        <div className="story-feature-card">
          <div className="story-card-top-row">
            <Quotes size={36} className="quote-icon" weight="fill" />
            <div className="story-star-rating">
              {Array.from({ length: stories[activeStory].rating }).map((_, i) => (
                <Star key={i} size={15} weight="fill" className="star-fill" />
              ))}
              <span className="rating-verified-text">5.0 · Verified Merchant</span>
            </div>
          </div>
          <p className="story-quote-text">"{stories[activeStory].quote}"</p>
          <div className="story-card-footer">
            <div>
              <strong>{stories[activeStory].founder}</strong>
              <span>
                {stories[activeStory].brand} · {stories[activeStory].craft}
              </span>
            </div>
            <div className="story-highlight-pill">
              <Sparkle size={14} weight="fill" />
              <span>{stories[activeStory].highlight}</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}

/* ==========================================================================
   PRICING & ROI CALCULATOR
   ========================================================================== */

function MarketingFaq() {
  const faqs = [
    ["What is Stheno?", "Stheno is a commerce operations system for independent merchants. It connects storefront presentation with catalogue, inventory, guest checkout, orders, and staff operations."],
    ["Who is Stheno for?", "It is for store owners who need practical day-to-day operations, ecommerce builders who need a dependable commerce core behind a custom storefront, and developers who need explicit APIs and workflow boundaries."],
    ["Do buyers need to create an account?", "No. Stheno is designed around guest checkout. Merchants can still recognise repeat buyers through the buyer information collected for an order."],
    ["How do social orders work?", "WhatsApp can open with a pre-filled order message. Messenger and Instagram use a copy-then-open handoff. The handoff creates a draft order for staff to review before confirmation."],
    ["Can I move my catalogue to Stheno?", "Yes. The catalogue import flow is designed for spreadsheet data: map headers, review the result, discard unmapped columns, and continue with the imported product information."],
    ["What happens after the 14-day trial?", "The store moves to the Free plan. Existing products, staff records, and order data are not deleted or hidden. Free limits apply to new creation until you upgrade or reduce usage."],
    ["What does Pro add?", "Pro adds the paid operational capabilities specified in your service order, including full analytics history, custom domain, bespoke storefront service, payment gateway, courier dispatch, and fraud or risk tooling where available."],
    ["Is Stheno a marketplace or the seller of my goods?", "No. You remain responsible for your products, prices, fulfilment, customer communications, and the taxes on your own sales. Stheno provides the commerce system that supports your operation."],
  ] as const

  return (
    <section className="faq-section section-shell" id="faq" aria-labelledby="faq-heading">
      <p className="eyebrow">Questions before you start</p>
      <h2 id="faq-heading">Clear answers for merchants, builders, and developers.</h2>
      <Accordion className="faq-list stheno-faq" type="single" collapsible defaultValue="faq-0">
        {faqs.map(([question, answer], index) => (
          <AccordionItem className="faq-item" key={question} value={`faq-${index}`}>
            <AccordionTrigger className="faq-question-btn">{question}</AccordionTrigger>
            <AccordionContent className="faq-answer"><p>{answer}</p></AccordionContent>
          </AccordionItem>
        ))}
      </Accordion>
    </section>
  )
}

function CustomerReviews() {
  const approvedReviews: Array<{ quote: string; name: string; role: string; company: string }> = []

  return (
    <section className="customer-reviews-section section-shell" id="reviews" aria-labelledby="reviews-heading">
      <p className="eyebrow">Customer reviews</p>
      <h2 id="reviews-heading">Merchant feedback should be real, specific, and approved for publication.</h2>
      {approvedReviews.length > 0 ? (
        <div className="customer-review-grid">
          {approvedReviews.map((review) => (
            <Card className="customer-review-card" key={`${review.name}-${review.company}`}>
              <CardContent><p>{review.quote}</p></CardContent>
              <CardFooter><strong>{review.name}</strong><span>{review.role}, {review.company}</span></CardFooter>
            </Card>
          ))}
        </div>
      ) : (
        <Card className="customer-review-empty">
          <CardHeader><Badge variant="outline">Review publication standard</Badge><CardTitle>Reviews will appear here after merchant approval.</CardTitle></CardHeader>
          <CardContent><p>Stheno does not publish invented testimonials, star ratings, or savings claims. Each published review will identify the merchant, its role, the approved comment, and permission to share it.</p></CardContent>
        </Card>
      )}
    </section>
  )
}

function PricingSection() {
  const plans = [
    {
      name: "Trial",
      term: "14 days",
      bestFor: "Evaluate the core workflow before choosing a long-term plan.",
      highlights: ["Unlimited products during the trial", "Up to 10 staff seats", "Supported brand kit and subdomain"],
      action: "Start a trial",
    },
    {
      name: "Free",
      term: "Continue after trial",
      bestFor: "Run essential commerce operations with a defined operating limit.",
      highlights: ["100 new products", "Owner plus two staff seats", "Cash on delivery, CSV import, and 30-day analytics"],
      action: "Choose Free",
    },
    {
      name: "Pro",
      term: "6 or 12 months",
      bestFor: "Operate a custom store with paid integrations and full operational support.",
      highlights: ["Unlimited products and up to 10 staff seats", "Custom domain and bespoke storefront service", "Gateway, courier, risk tooling, and full analytics where available"],
      action: "Discuss Pro",
      featured: true,
    },
  ]

  return (
    <section className="pricing-section section-shell pricing-section-v2" id="pricing" aria-labelledby="pricing-heading">
      <div className="section-header-block pricing-header">
        <p className="eyebrow">Plans that match the work you need to do</p>
        <h2 id="pricing-heading">Start with a clear path. Move to Pro when your operation needs more.</h2>
        <p className="section-lede">Every merchant starts with the same commerce foundation. The difference is how much operational capacity, customisation, and integration support the store needs.</p>
      </div>
      <div className="pricing-tier-grid">
        {plans.map((plan) => (
          <Card className={`pricing-tier-card ${plan.featured ? "is-featured" : ""}`} key={plan.name}>
            <CardHeader>
              <div className="pricing-tier-topline">
                <CardTitle>{plan.name}</CardTitle>
                {plan.featured && <Badge className="pricing-tier-badge">For growing operations</Badge>}
              </div>
              <CardDescription>{plan.term}</CardDescription>
            </CardHeader>
            <CardContent>
              <p className="pricing-tier-fit"><strong>Choose {plan.name}</strong> if you need to {plan.bestFor.charAt(0).toLowerCase() + plan.bestFor.slice(1)}</p>
              <Separator className="pricing-tier-separator" />
              <ul className="pricing-tier-highlights">
                {plan.highlights.map((highlight) => <li key={highlight}><Check size={16} weight="bold" />{highlight}</li>)}
              </ul>
            </CardContent>
            <CardFooter>
              <Button asChild className="pricing-tier-action" variant={plan.featured ? "default" : "outline"} size="lg">
                <Link to="/signup">{plan.action}<ArrowRight size={16} weight="bold" /></Link>
              </Button>
            </CardFooter>
          </Card>
        ))}
      </div>
      <section className="service-comparison" aria-labelledby="service-comparison-heading">
        <div className="service-comparison-heading">
          <h3 id="service-comparison-heading">Compare every service before you choose a plan.</h3>
          <p>Each row explains what the service does and exactly how it is available in Trial, Free, and Pro.</p>
        </div>
        <div className="service-comparison-table-wrap" role="region" aria-label="Stheno plan service comparison" tabIndex={0}>
          <Table className="service-comparison-table">
            <TableHeader>
              <TableRow>
                <TableHead scope="col">Service</TableHead>
                <TableHead scope="col">Trial</TableHead>
                <TableHead scope="col">Free</TableHead>
                <TableHead scope="col">Pro</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow className="service-comparison-group"><TableCell colSpan={4}>Store operations</TableCell></TableRow>
              <TableRow>
                <TableCell><strong>Catalogue and inventory</strong><span>Create products, organise the catalogue, and manage inventory from one workspace.</span></TableCell>
                <TableCell>Included for evaluation</TableCell>
                <TableCell>Included</TableCell>
                <TableCell>Included</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Product capacity</strong><span>The number of new products your store can add.</span></TableCell>
                <TableCell>Unlimited for 14 days</TableCell>
                <TableCell>100 new products</TableCell>
                <TableCell>Unlimited</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Team access</strong><span>People who can work on products, orders, and fulfilment.</span></TableCell>
                <TableCell>Up to 10 seats</TableCell>
                <TableCell>Owner plus 2 staff seats</TableCell>
                <TableCell>Up to 10 seats</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Orders and guest checkout</strong><span>Let buyers browse, add to cart, and place orders without creating an account.</span></TableCell>
                <TableCell>Included for evaluation</TableCell>
                <TableCell>Included</TableCell>
                <TableCell>Included</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Cash on delivery</strong><span>Accept and manage cash-on-delivery orders in the standard order workflow.</span></TableCell>
                <TableCell>Included for evaluation</TableCell>
                <TableCell>Included</TableCell>
                <TableCell>Included</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>CSV catalogue import</strong><span>Bring a product catalogue into Stheno with a CSV import.</span></TableCell>
                <TableCell>Included for evaluation</TableCell>
                <TableCell>Included</TableCell>
                <TableCell>Included</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Social order handoff</strong><span>Create staff-reviewable draft orders from supported social conversations.</span></TableCell>
                <TableCell>Included for evaluation</TableCell>
                <TableCell>Included</TableCell>
                <TableCell>Included</TableCell>
              </TableRow>
              <TableRow className="service-comparison-group"><TableCell colSpan={4}>Storefront and reporting</TableCell></TableRow>
              <TableRow>
                <TableCell><strong>Brand kit and Stheno subdomain</strong><span>Launch with supported brand details on a Stheno-hosted subdomain.</span></TableCell>
                <TableCell>Included</TableCell>
                <TableCell>Included</TableCell>
                <TableCell>Included</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Analytics</strong><span>See store activity and reporting available to your plan.</span></TableCell>
                <TableCell>Available during evaluation</TableCell>
                <TableCell>30-day basic analytics</TableCell>
                <TableCell>Full analytics</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Web push</strong><span>Send supported browser notifications to opted-in buyers.</span></TableCell>
                <TableCell>Available during evaluation</TableCell>
                <TableCell>Included</TableCell>
                <TableCell>Included</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Custom domain</strong><span>Use your own domain name for your customer-facing storefront.</span></TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Included</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Bespoke storefront service</strong><span>A developer-built storefront supplied after the agreed Pro payment.</span></TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Included after payment</TableCell>
              </TableRow>
              <TableRow className="service-comparison-group"><TableCell colSpan={4}>Paid operational services</TableCell></TableRow>
              <TableRow>
                <TableCell><strong>Payment gateway connection</strong><span>Connect payment services when they are supported for your merchant and region.</span></TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Available where supported</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Courier connection</strong><span>Connect supported delivery providers to the store workflow.</span></TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Available where supported</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Risk tooling</strong><span>Use supported operational-risk tools where they are available to the merchant.</span></TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Not included</TableCell>
                <TableCell>Available where supported</TableCell>
              </TableRow>
              <TableRow>
                <TableCell><strong>Plan term and renewal</strong><span>How long the service continues before the merchant renews or changes plan.</span></TableCell>
                <TableCell>14 days</TableCell>
                <TableCell>Indefinite</TableCell>
                <TableCell>6 or 12 months, as agreed</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </section>
      <p className="pricing-disclosure">Paid-plan price, tax, regional availability, and provider-dependent features are confirmed in the service order before a merchant accepts Pro.</p>
    </section>
  )

  const [commitment, setCommitment] = useState<6 | 12>(12)
  const [currency, setCurrency] = useState<Currency>("BDT")
  const [monthlyOrders, setMonthlyOrders] = useState(600)
  const [aov, setAov] = useState(1200)
  const [openFaq, setOpenFaq] = useState<number | null>(null)

  const grossRevenue = monthlyOrders * aov
  const legacyPluginCost = 14000
  const legacyTxFee = Math.round(grossRevenue * 0.02)
  const legacyBaseFee = 4500
  const totalLegacyMonthly = legacyBaseFee + legacyPluginCost + legacyTxFee

  const sthenoMonthly = commitment === 12 ? 1350 : 1500
  const monthlySavings = Math.max(0, totalLegacyMonthly - sthenoMonthly)
  const annualSavings = monthlySavings * 12

  const matrixFeatures = [
    { name: "Brand kit & custom accent styling", free: true, pro: true },
    { name: "Products & category management", free: "Up to 50 items", pro: "Unlimited" },
    { name: "Social inbox (WhatsApp & Instagram)", free: true, pro: true },
    { name: "Staff operator accounts", free: "2 seats", pro: "Unlimited" },
    { name: "Automated bKash & Nagad gateways", free: false, pro: true },
    { name: "Pathao, RedX & Steadfast couriers", free: false, pro: true },
    { name: "Custom domain (yourshop.com)", free: false, pro: true },
    { name: "Zero transaction platform tax (0%)", free: true, pro: true },
    { name: "Bespoke storefront developer brief", free: false, pro: true },
  ]

  const faqs = [
    {
      q: "How does the 14-day Free Trial work?",
      a: "Upon submitting your store request, your sandbox and merchant environment are provisioned immediately with full access to all Free tier capabilities for 14 days without requiring credit card information.",
    },
    {
      q: "Do you charge extra transaction fees on sales?",
      a: "Never. Stheno charges 0% platform transaction fees. You only pay the direct standard gateway interchange rate to your payment provider (e.g. bKash or SSLCommerz).",
    },
    {
      q: "Can I bring my existing custom domain?",
      a: "Yes. Pro merchants can link any custom domain (e.g. yourstudio.com) directly through our DNS routing layer with automated SSL certificate renewal.",
    },
    {
      q: "How are WhatsApp and Instagram orders handled?",
      a: "Customer chat inquiries can be parsed directly in your merchant Social Inbox, allowing you to convert conversations into real fulfillment orders in 1 click.",
    },
    {
      q: "Can I migrate products from WooCommerce or Shopify?",
      a: "Yes. Our standard CSV catalog importer allows you to transfer your existing product titles, descriptions, pricing, and stock levels in seconds.",
    },
  ]

  return (
    <section className="pricing-section section-shell" id="pricing">
      <div className="section-header-block">
        <p className="eyebrow">Clear, Honest Pricing</p>
        <h2>Start free. Choose your tier when operations scale.</h2>
        <p className="section-lede">
          Free starts with a 14-day trial upon submitting your store request. Pro provides dedicated infrastructure, zero
          plugin taxes, and zero transaction fee penalties.
        </p>
      </div>

      {/* Currency Switcher & Commitment Switcher */}
      <div className="pricing-switcher-wrap">
        <div className="currency-selector-row">
          <span className="curr-label">Display Currency:</span>
          {(["BDT", "USD", "EUR"] as Currency[]).map((c) => (
            <button
              key={c}
              type="button"
              className={`curr-btn ${currency === c ? "is-active" : ""}`}
              onClick={() => setCurrency(c)}
            >
              {c}
            </button>
          ))}
        </div>

        <div className="pricing-switcher">
          <button
            type="button"
            className={commitment === 6 ? "is-active" : ""}
            onClick={() => setCommitment(6)}
          >
            6-Month Commitment
          </button>
          <button
            type="button"
            className={commitment === 12 ? "is-active" : ""}
            onClick={() => setCommitment(12)}
          >
            12-Month Commitment
            <span className="savings-pill">Save {formatMoney(1800, currency)}/yr</span>
          </button>
        </div>
      </div>

      <div className="pricing-grid">
        <article className="price-card">
          <p className="plan-name">Free Tier</p>
          <h2>
            {formatMoney(0, currency)}<span>/ month</span>
          </h2>
          <strong>14-day trial starts immediately upon store request.</strong>
          <ul>
            <li>
              <CheckCircle size={16} weight="fill" />
              <span>Brand kit & approved store icon</span>
            </li>
            <li>
              <CheckCircle size={16} weight="fill" />
              <span>Orders, social queue, catalog & inventory</span>
            </li>
            <li>
              <CheckCircle size={16} weight="fill" />
              <span>Staff accounts & core analytics</span>
            </li>
          </ul>
          <Link className="button button-outline button-full" to="/signup">
            <span>Start with Free</span>
            <ArrowRight size={15} />
          </Link>
        </article>

        <article className="price-card is-featured">
          <span className="plan-tag">
            <Sparkle size={13} weight="fill" /> Recommended for Growth
          </span>
          <p className="plan-name">Pro Tier</p>
          <h2>
            {formatMoney(commitment === 12 ? 1350 : 1500, currency)}
            <span>/ month</span>
          </h2>
          <strong>
            {commitment === 12
              ? `Billed ${formatMoney(16200, currency)} annually (${formatMoney(1350, currency)}/mo equivalent)`
              : `Billed ${formatMoney(9000, currency)} for 6 months`}
          </strong>
          <ul>
            <li>
              <CheckCircle size={16} weight="fill" />
              <span>Everything in Free</span>
            </li>
            <li>
              <CheckCircle size={16} weight="fill" />
              <span>Automated gateways, couriers & custom domains</span>
            </li>
            <li>
              <CheckCircle size={16} weight="fill" />
              <span>Bespoke storefront brief & developer queue priority</span>
            </li>
          </ul>
          <Link className="button button-primary button-full" to="/signup">
            <span>Choose Pro</span>
            <ArrowRight size={15} />
          </Link>
        </article>
      </div>

      {/* Interactive ROI & Savings Calculator */}
      <div className="roi-calculator-shell">
        <div className="roi-header">
          <Calculator size={24} className="roi-icon" />
          <div>
            <h2>Independent Merchant Cost Comparison</h2>
            <p>See what you save annually by eliminating legacy app subscription bloat and 2% platform cuts.</p>
          </div>
        </div>

        <div className="roi-sliders-grid">
          <div className="roi-slider-group">
            <div className="slider-label-row">
              <span>Monthly Order Volume</span>
              <strong>{monthlyOrders.toLocaleString("en-BD")} orders</strong>
            </div>
            <input
              type="range"
              min="100"
              max="3000"
              step="50"
              value={monthlyOrders}
              onChange={(e) => setMonthlyOrders(Number(e.target.value))}
              aria-label="Monthly order volume slider"
            />
          </div>

          <div className="roi-slider-group">
            <div className="slider-label-row">
              <span>Average Basket Size</span>
              <strong>{formatMoney(aov, currency)}</strong>
            </div>
            <input
              type="range"
              min="400"
              max="3500"
              step="50"
              value={aov}
              onChange={(e) => setAov(Number(e.target.value))}
              aria-label="Average basket size slider"
            />
          </div>
        </div>

        <div className="roi-comparison-cards">
          <div className="roi-card legacy">
            <span className="roi-card-tag">Legacy Platforms (Shopify + Apps)</span>
            <strong className="roi-price">{formatMoney(totalLegacyMonthly, currency)}/mo</strong>
            <small>Includes $39 base + 2% tx cuts ({formatMoney(legacyTxFee, currency)}) + app bloat</small>
          </div>

          <div className="roi-card stheno">
            <span className="roi-card-tag">Stheno Pro Tier</span>
            <strong className="roi-price">{formatMoney(sthenoMonthly, currency)}/mo</strong>
            <small>All features built-in · 0% transaction penalty · 0 app taxes</small>
          </div>

          <div className="roi-savings-callout">
            <span>Estimated Annual Savings</span>
            <strong>{formatMoney(annualSavings, currency)} / year</strong>
          </div>
        </div>
      </div>

      {/* Side-by-Side Plan Matrix Table */}
      <div className="plan-matrix-shell">
        <h2>Detailed Plan Feature Matrix</h2>
        <div className="matrix-table-wrap">
          <table className="matrix-table">
            <thead>
              <tr>
                <th>Feature Capability</th>
                <th>Free Tier (Trial)</th>
                <th>Pro Tier</th>
              </tr>
            </thead>
            <tbody>
              {matrixFeatures.map((row) => (
                <tr key={row.name}>
                  <td>{row.name}</td>
                  <td>
                    {typeof row.free === "boolean" ? (
                      row.free ? (
                        <Check size={16} weight="bold" className="matrix-check" />
                      ) : (
                        <span className="matrix-dash">−</span>
                      )
                    ) : (
                      <span>{row.free}</span>
                    )}
                  </td>
                  <td>
                    {typeof row.pro === "boolean" ? (
                      row.pro ? (
                        <Check size={16} weight="bold" className="matrix-check" />
                      ) : (
                        <span className="matrix-dash">−</span>
                      )
                    ) : (
                      <span className="pro-highlight">{row.pro}</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Accordion Merchant FAQs */}
      <div className="faq-section">
        <h2>Frequently Asked Questions</h2>
        <div className="faq-list">
          {faqs.map((faq, idx) => (
            <div className="faq-item" key={faq.q}>
              <button
                type="button"
                className="faq-question-btn"
                onClick={() => setOpenFaq(openFaq === idx ? null : idx)}
                aria-expanded={openFaq === idx}
              >
                <span>{faq.q}</span>
                <CaretDown
                  size={16}
                  weight="bold"
                  className={`faq-caret ${openFaq === idx ? "is-open" : ""}`}
                />
              </button>
              <AnimatePresence>
                {openFaq === idx && (
                  <motion.div
                    className="faq-answer"
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                  >
                    <p>{faq.a}</p>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

function LaunchChapter() {
  const statuses = [
    { label: "Submitted", desc: "Your store request has reached Stheno." },
    { label: "In review", desc: "The submitted store details are checked before provisioning." },
    { label: "Provisioning", desc: "Your merchant environment is being configured." },
    { label: "Ready", desc: "Your merchant workspace is ready for you and your staff." },
  ]
  return (
    <section className="workflow-chapter collective-launch section-shell" id="launch">
      <div className="workflow-copy">
        <p className="eyebrow">A clear launch path</p>
        <h2>Know what happens from store request to launch.</h2>
        <p>
          Your welcome workspace shows the current stage, the next action, and any information needed before the store
          is ready to operate.
        </p>
        <Link className="button button-primary" to="/signup">
          <span>Start your launch</span>
          <ArrowRight size={16} weight="bold" />
        </Link>
      </div>
      <ol className="launch-states" aria-label="Possible store launch statuses">
        {statuses.map((item, index) => (
          <li className={index === 0 ? "is-current" : ""} key={item.label}>
            <div className="launch-step-dot" />
            <div className="launch-step-info">
              <strong>{item.label}</strong>
              <p>{item.desc}</p>
            </div>
          </li>
        ))}
      </ol>
    </section>
  )
}

function Closing() {
  return (
    <section className="closing section-shell">
      <Buildings size={28} weight="regular" />
      <h2>Run your store with a system built around the work behind every order.</h2>
      <div className="closing-actions">
        <Link className="button button-primary" to="/signup">
          <span>Create your merchant account</span>
          <ArrowRight size={16} weight="bold" />
        </Link>
        <StartDemoButton />
      </div>
    </section>
  )
}

function Footer({ onOpenEngineersTalk }: { onOpenEngineersTalk: () => void }) {
  return (
    <footer className="site-footer section-shell">
      <div className="footer-left">
        <Link className="brand" to="/" aria-label="Stheno home">
          <Mark />
          <span className="sr-only">Stheno</span>
        </Link>
        <p>Commerce infrastructure engineered for independent merchants.</p>
      </div>
      <div className="footer-right">
        <button type="button" className="engineers-talk-link-btn" onClick={onOpenEngineersTalk}>
          <Code size={16} />
          <span>Architecture overview</span>
        </button>
        <Link className="text-link" to="#pricing">
          <span>Compare plans</span>
          <ArrowUpRight size={15} />
        </Link>
        <Link className="text-link" to="/privacy">Privacy notice</Link>
        <Link className="text-link" to="/terms">Terms of service</Link>
      </div>
    </footer>
  )
}

/* ==========================================================================
   ENGINEER'S TALK MODAL (TECHNICAL DEEP-DIVE FOR ARCHITECTS & DEVELOPERS)
   ========================================================================== */

function EngineersTalkModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  if (!isOpen) return null

  return (
    <div className="modal-backdrop" onClick={onClose} role="presentation">
      <section className="modal-card engineers-talk-modal" role="dialog" aria-modal="true" aria-labelledby="engineering-talk-title" onClick={(event) => event.stopPropagation()}>
        <div className="engineers-talk-header">
          <div className="header-title-wrap">
            <TerminalWindow size={24} className="terminal-icon" />
            <div>
              <h2 id="engineering-talk-title">Architecture overview</h2>
              <p>Stheno’s product design for teams who need to understand the commerce core.</p>
            </div>
          </div>
          <button type="button" className="modal-close-btn" onClick={onClose} aria-label="Close architecture overview">
            <X size={20} />
          </button>
        </div>
        <div className="engineers-talk-body">
          <div className="content-prose">
            <p><strong>Exact money handling:</strong> money paths use decimal values and a single provider-boundary conversion layer rather than floating-point calculations.</p>
            <p><strong>Durable workflows:</strong> multi-step operations are designed to compensate safely after a failure and to reuse idempotency keys when a provider outcome is uncertain.</p>
            <p><strong>Bounded modules:</strong> domain modules own their data and communicate across boundaries through workflows, queries, events, and links.</p>
            <p><strong>Swappable presentation:</strong> storefront presentation can be independently implemented while core routes, data fetching, and checkout responsibilities remain in the trusted commerce layer.</p>
          </div>
          <p className="form-notice">Performance figures, provider availability, and API compatibility are release-gated. This overview intentionally does not present unverified benchmarks as measured results.</p>
        </div>
      </section>
    </div>
  )

  const [selectedLayer, setSelectedLayer] = useState<number>(0)
  const [apiSnippetLang, setApiSnippetLang] = useState<"ts" | "curl" | "rust">("ts")
  const [copied, setCopied] = useState(false)

  const layers = [
    {
      title: "Rust Starved Primitives",
      tag: "stheno-types",
      icon: Cube,
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
      icon: FlowArrow,
      desc: "Multi-step sagas coordinate state mutations with automatic compensation on crash and guaranteed idempotency across payment providers.",
      specs: ["Crash-and-resume resilience", "Automated step compensations", "PSP idempotency key locks"],
    },
    {
      title: "Headless Presentation Contracts",
      tag: "TypeScript Contract",
      icon: Storefront,
      desc: "Strictly pure storefront presentation modules that plug into Stheno APIs with zero backend logic modification.",
      specs: ["Sub-50ms TTFB globally", "Pure presentation architecture", "Full theme customizability"],
    },
  ]

  const apiSnippets = {
    curl: `curl -X POST https://api.stheno.shop/store/carts/cart_01HX/complete \\
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
              <TerminalWindow size={24} className="terminal-icon" />
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
                      <Check size={14} weight="bold" />
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
                    <Lightning size={20} weight="fill" className="bm-icon" />
                    <span>Time-to-First-Byte</span>
                  </div>
                  <strong className="bm-value">32ms</strong>
                  <span className="bm-subtext">Stheno Global Edge</span>
                  <small className="bm-comparison">vs 380ms legacy monoliths</small>
                </div>

                <div className="benchmark-card">
                  <div className="benchmark-header">
                    <Gauge size={20} weight="regular" className="bm-icon" />
                    <span>Saga Commit Latency</span>
                  </div>
                  <strong className="bm-value">&lt; 8ms</strong>
                  <span className="bm-subtext">Atomic State Transitions</span>
                  <small className="bm-comparison">Guaranteed zero double-charges</small>
                </div>

                <div className="benchmark-card">
                  <div className="benchmark-header">
                    <Cube size={20} weight="regular" className="bm-icon" />
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
                  <span className="terminal-title">POST /store/carts/{`{id}`}/complete</span>
                  <button type="button" className="copy-code-btn" onClick={handleCopySnippet}>
                    {copied ? <Check size={14} weight="bold" /> : <Copy size={14} />}
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
      "ID,Title,Category,Price_BDT,Stock,Status\n" +
      session.products.map((p) => `"${p.id}","${p.title}","${p.category}",${p.price},${p.stock},"${p.status}"`).join("\n")
    const encodedUri = encodeURI(csvContent)
    const link = document.createElement("a")
    link.setAttribute("href", encodedUri)
    link.setAttribute("download", `stheno_catalog_${session.storeName.toLowerCase().replace(/\s+/g, "_")}.csv`)
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
    if (promoCode.trim().toUpperCase() === "STHENO10") {
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
            initial={{ opacity: 0, y: 15, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 15, scale: 0.95 }}
          >
            <CheckCircle size={16} weight="fill" />
            <span>{toastMessage}</span>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Top Universal Demo Navigation Bar */}
      <header className="demo-topbar">
        <div className="demo-topbar-left-group">
          <Link className="brand" to="/">
            <Mark />
            <span className="sr-only">Stheno home</span>
          </Link>
          <span className="demo-topbar-divider">/</span>
          <span className="demo-store-name-tag">{session.storeName} Sandbox</span>
        </div>
        <div className="demo-topbar-center">
          <button type="button" className="cmd-palette-trigger" onClick={() => setCmdPaletteOpen(true)}>
            <MagnifyingGlass size={16} />
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
                className={`curr-btn ${currency === c ? "is-active" : ""}`}
                onClick={() => setCurrency(c)}
              >
                {c}
              </button>
            ))}
          </div>

          <a href="#live-storefront" className="button button-outline button-xs">
            <Storefront size={14} />
            <span>Jump to Storefront Preview ↓</span>
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
              <Check size={13} weight="bold" /> Synced to Storefront
            </span>
          )}
        </div>

        <div className="demo-app-layout">
          <aside className="demo-admin-sidebar" aria-label="Merchant admin navigation">
            <span className="sidebar-group-title">Operations</span>
            <button
              className={activeTab === "dashboard" ? "is-active" : ""}
              type="button"
              onClick={() => setActiveTab("dashboard")}
            >
              <ChartLine size={17} />
              <span>Executive Dashboard</span>
            </button>
            <button
              className={activeTab === "social" ? "is-active" : ""}
              type="button"
              onClick={() => setActiveTab("social")}
            >
              <ChatCircleDots size={17} />
              <span>Social Inbox</span>
              {pendingConvsCount > 0 && <span className="nav-count-pill">{pendingConvsCount}</span>}
            </button>
            <button
              className={activeTab === "products" ? "is-active" : ""}
              type="button"
              onClick={() => setActiveTab("products")}
            >
              <Package size={17} />
              <span>Products & Catalog</span>
            </button>
            <button
              className={activeTab === "orders" ? "is-active" : ""}
              type="button"
              onClick={() => setActiveTab("orders")}
            >
              <ClipboardText size={17} />
              <span>Orders Queue ({session.orders.length})</span>
            </button>
            <button
              className={activeTab === "brand" ? "is-active" : ""}
              type="button"
              onClick={() => setActiveTab("brand")}
            >
              <PaintBrush size={17} />
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
                              <WhatsappLogo size={20} weight="fill" className="wa-icon" />
                            ) : (
                              <InstagramLogo size={20} weight="fill" className="ig-icon" />
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
                              <CheckCircle size={15} weight="fill" /> Order Created in Queue
                            </span>
                          ) : (
                            <button
                              type="button"
                              className="button button-primary button-sm"
                              onClick={() => handleConvertSocial(conv.id)}
                            >
                              <Plus size={14} weight="bold" />
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
                        <DownloadSimple size={15} />
                        <span>Export CSV</span>
                      </button>
                      <button
                        className="button button-primary button-sm"
                        type="button"
                        onClick={() => setNewProductModal(true)}
                      >
                        <Plus size={15} weight="bold" />
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
                      <UploadSimple size={14} />
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
                          <Check size={16} weight="bold" />
                          <span>Accent color updates buttons, banners, and carts below immediately.</span>
                        </li>
                        <li>
                          <Check size={16} weight="bold" />
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
            <Storefront size={16} weight="fill" />
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
              <span className="store-domain-badge">.stheno.shop</span>
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
                <ShoppingCartSimple size={18} />
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
                <MagnifyingGlass size={16} />
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
                <ShoppingBag size={42} weight="light" />
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
                        role="button"
                        tabIndex={0}
                        aria-label={`View ${product.title} details`}
                      >
                        <ShoppingBag size={36} weight="light" />
                        <span className="stock-tag">{product.stock} in stock</span>
                        <span className="card-star-pill">
                          <Star size={11} weight="fill" className="star-icon" />
                          <span>{avgRating} ({reviews.length > 0 ? reviews.length : 12})</span>
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
                        <Plus size={14} weight="bold" />
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
            <span>Powered by Stheno Merchant Workspace</span>
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
                        <ShoppingBag size={48} weight="light" />
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
                            placeholder="Promo code (try STHENO10)"
                            value={promoCode}
                            onChange={(e) => setPromoCode(e.target.value)}
                          />
                          <button type="submit" className="button button-outline button-xs">
                            Apply
                          </button>
                        </form>
                        {promoApplied && <span className="promo-success-tag">✓ STHENO10 applied: 10% discount</span>}

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
                        placeholder="+880 1700-000000"
                        value={customerInfo.phone}
                        onChange={(e) => setCustomerInfo({ ...customerInfo, phone: e.target.value })}
                      />
                    </label>
                    <label>
                      Email Address
                      <input
                        type="email"
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
                    <CheckCircle size={48} weight="fill" className="confirmation-icon" />
                    <h3>Order Received!</h3>
                    <p className="order-num-tag">{lastOrderId}</p>
                    <p className="confirmation-text">
                      Your order has been recorded in the Stheno sandbox database and synchronized directly into the
                      Admin Orders queue above!
                    </p>
                    <a href="#demo" className="button button-primary button-full" onClick={() => setCartOpen(false)}>
                      <ChartLine size={16} />
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
                <ShoppingBag size={56} weight="light" />
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
                    <ShoppingCartSimple size={16} />
                    <span>Add to Shopping Cart</span>
                  </button>
                </div>

                {/* Verified Reviews Section */}
                <div className="product-reviews-block">
                  <div className="reviews-header-row">
                    <strong>Customer Reviews ({(reviewsMap[selectedProduct.id] || []).length})</strong>
                    <span className="verified-badge">
                      <ShieldCheck size={14} weight="fill" /> Verified Buyers
                    </span>
                  </div>

                  <div className="reviews-list-compact">
                    {(reviewsMap[selectedProduct.id] || []).map((rev) => (
                      <div className="review-item" key={rev.id}>
                        <div className="review-top-line">
                          <strong>{rev.author}</strong>
                          <div className="star-rating-row">
                            {Array.from({ length: rev.rating }).map((_, i) => (
                              <Star key={i} size={11} weight="fill" className="star-fill" />
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
                  <Barcode size={32} />
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
                <MagnifyingGlass size={18} />
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
                  <ChartLine size={16} />
                  <span>Go to Executive Dashboard</span>
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setActiveTab("social")
                    setCmdPaletteOpen(false)
                  }}
                >
                  <ChatCircleDots size={16} />
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
                  <ClipboardText size={16} />
                  <span>Go to Orders Queue</span>
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setActiveTab("brand")
                    setCmdPaletteOpen(false)
                  }}
                >
                  <PaintBrush size={16} />
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
                    Price (BDT)
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
        <p className="eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p>{intro}</p>
      </div>
      <div className="auth-card">{children}</div>
    </section>
  )
}

function readDraft() {
  try {
    return JSON.parse(localStorage.getItem(accountDraftKey) ?? "{}") as Record<string, string>
  } catch {
    return {}
  }
}

function Signup() {
  const { navigate } = useMarketing()
  const existing = readDraft()
  const [details, setDetails] = useState(existing)
  const [notice, setNotice] = useState("")
  const [showPassword, setShowPassword] = useState(false)

  const passwordLength = details.password?.length ?? 0
  const passwordStrength =
    passwordLength === 0 ? "none" : passwordLength < 8 ? "weak" : passwordLength < 12 ? "good" : "strong"

  function persist() {
    localStorage.setItem(accountDraftKey, JSON.stringify(details))
    setNotice("Draft saved. You can complete these details after account creation.")
  }

  function submit(event: FormEvent) {
    event.preventDefault()
    persist()
    localStorage.setItem("stheno-account-created", "true")
    navigate("/welcome")
  }

  return (
    <AuthShell
      eyebrow="Create Merchant Account"
      title="Make space for the store you want to run."
      intro="Start with email or Google. Optional business details can be filled now or saved as a draft."
    >
      <form className="auth-form" onSubmit={submit}>
        <button type="button" className="google-button">
          <GoogleLogo size={18} weight="bold" />
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
        <label>
          Password
          <div className="password-input-wrap">
            <input
              required
              minLength={8}
              type={showPassword ? "text" : "password"}
              autoComplete="new-password"
              placeholder="At least 8 characters"
              value={details.password ?? ""}
              onChange={(e) => setDetails({ ...details, password: e.target.value })}
            />
            <button
              type="button"
              className="password-toggle-btn"
              onClick={() => setShowPassword((p) => !p)}
              aria-label={showPassword ? "Hide password" : "Show password"}
            >
              {showPassword ? <EyeSlash size={16} /> : <Eye size={16} />}
            </button>
          </div>
          {passwordLength > 0 && (
            <div className={`password-meter ${passwordStrength}`}>
              <div className="meter-track">
                <div className="meter-bar" />
              </div>
              <span className="meter-label">
                {passwordStrength === "weak"
                  ? "Too short (min 8 chars)"
                  : passwordStrength === "good"
                    ? "Good password"
                    : "Strong password"}
              </span>
            </div>
          )}
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
              <option value="pro">Pro Tier (BDT 1,500/mo)</option>
            </select>
          </label>
        </fieldset>
        {notice && (
          <p className="form-notice" role="status">
            {notice}
          </p>
        )}
        <button className="button button-primary button-full" type="submit">
          <span>Create account</span>
          <ArrowRight size={15} />
        </button>
        <button className="save-draft" type="button" onClick={persist}>
          Save draft and finish later
        </button>
        <p className="form-meta">
          Already have an account? <Link to="/login">Log in</Link>
        </p>
      </form>
    </AuthShell>
  )
}

function Login() {
  const { navigate } = useMarketing()
  const [showPassword, setShowPassword] = useState(false)

  return (
    <AuthShell
      eyebrow="Merchant Sign In"
      title="Welcome back to your workspace."
      intro="Access your active merchant dashboard and ongoing operations."
    >
      <form
        className="auth-form"
        onSubmit={(e) => {
          e.preventDefault()
          navigate("/welcome")
        }}
      >
        <button type="button" className="google-button">
          <GoogleLogo size={18} weight="bold" />
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
              {showPassword ? <EyeSlash size={16} /> : <Eye size={16} />}
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
          New to Stheno? <Link to="/signup">Create an account</Link>
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
  const { navigate } = useMarketing()
  const saved = readDraft()
  const [created, setCreated] = useState(Boolean(localStorage.getItem(launchDraftKey)))
  const [request, setRequest] = useState({
    name: saved.businessName ?? "",
    subdomain: "",
    accent: "#1c8b82",
    icon: "rise",
    plan: saved.plan ?? "free",
    buyerAccounts: false,
  })
  const [submitted, setSubmitted] = useState(false)
  const [provisionStep, setProvisionStep] = useState(1)

  useEffect(() => {
    if (submitted) {
      const t1 = setTimeout(() => setProvisionStep(2), 1500)
      const t2 = setTimeout(() => setProvisionStep(3), 3500)
      const t3 = setTimeout(() => setProvisionStep(4), 5500)
      return () => {
        clearTimeout(t1)
        clearTimeout(t2)
        clearTimeout(t3)
      }
    }
  }, [submitted])

  function submit(event: FormEvent) {
    event.preventDefault()
    localStorage.setItem(launchDraftKey, JSON.stringify(request))
    setCreated(true)
    setSubmitted(true)
  }

  if (submitted) {
    const steps = [
      { num: 1, title: "Submission Verified", desc: "Store parameters validated against schema." },
      { num: 2, title: "DNS & Route Binding", desc: `${request.subdomain || "store"}.stheno.shop DNS record mapped.` },
      { num: 3, title: "Tenant Database Schema", desc: "Postgres isolated tenant schema provisioned." },
      { num: 4, title: "Workspace Activated", desc: "Merchant Dashboard & Admin live for operations." },
    ]

    return (
      <section className="welcome-page section-shell">
        <div className="status-panel">
          <div className="provision-live-indicator">
            <span className={`status-indicator ${provisionStep === 4 ? "live" : "in-packing"}`} />
            <span>{provisionStep === 4 ? "Tenant Environment Online" : "Provisioning Tenant..."}</span>
          </div>

          <CheckCircle size={44} weight="fill" className="status-success-icon" />
          <p className="eyebrow">Real-Time Provisioning Pipeline</p>
          <h1>{request.name || "Your Store"} is taking shape.</h1>
          <p>
            Your 14-day trial has begun. Watch the deployment pipeline configure your isolated tenant environment below.
          </p>

          <div className="provision-steps-tracker">
            {steps.map((s) => (
              <div
                className={`tracker-step ${provisionStep >= s.num ? "is-done" : ""} ${
                  provisionStep === s.num ? "is-active" : ""
                }`}
                key={s.num}
              >
                <div className="tracker-circle">
                  {provisionStep > s.num ? <Check size={14} weight="bold" /> : s.num}
                </div>
                <div className="tracker-text">
                  <strong>{s.title}</strong>
                  <span>{s.desc}</span>
                </div>
              </div>
            ))}
          </div>

          <div className="provision-actions">
            <button className="button button-primary" onClick={() => navigate("/")}>
              <span>Return to Stheno Home</span>
              <ArrowRight size={15} />
            </button>
          </div>
        </div>
      </section>
    )
  }

  return (
    <section className="welcome-page section-shell">
      <div className="welcome-intro">
        <p className="eyebrow">Your Launch Workspace</p>
        <h1>{created ? "Your saved store draft is ready." : "Configure the store your customers will recognize."}</h1>
        <p>
          Submit your store parameters below. Our provisioning pipeline will configure your tenant environment and
          activate your merchant dashboard.
        </p>
      </div>
      <form className="store-form" onSubmit={submit}>
        <div className="form-heading">
          <Storefront size={24} weight="regular" />
          <div>
            <h2>Store Setup</h2>
            <p>These parameters initialize your merchant environment and customer domain.</p>
          </div>
        </div>
        <div className="form-pair">
          <label>
            Store name
            <input
              required
              value={request.name}
              onChange={(e) => setRequest({ ...request, name: e.target.value })}
              placeholder="e.g. Nodi Studio"
            />
          </label>
          <label>
            Subdomain
            <div className="subdomain-input-wrap">
              <input
                required
                pattern="[a-z0-9-]+"
                value={request.subdomain}
                onChange={(e) => setRequest({ ...request, subdomain: e.target.value.toLowerCase() })}
                placeholder="nodi-studio"
              />
              <span className="domain-suffix">.stheno.shop</span>
            </div>
          </label>
        </div>
        <div className="form-pair">
          <label>
            Brand logo
            <input type="file" accept="image/png,image/jpeg,image/webp,image/svg+xml" />
          </label>
          <label>
            Approved store mark
            <select value={request.icon} onChange={(e) => setRequest({ ...request, icon: e.target.value })}>
              <option value="rise">Rising mark</option>
              <option value="dot">Solid dot</option>
              <option value="arc">Open arc</option>
            </select>
          </label>
        </div>
        <label>
          Accent colour
          <input
            className="color-input"
            type="color"
            value={request.accent}
            onChange={(e) => setRequest({ ...request, accent: e.target.value })}
          />
        </label>
        <label>
          Buyer account policy
          <select
            value={request.buyerAccounts ? "yes" : "no"}
            onChange={(e) => setRequest({ ...request, buyerAccounts: e.target.value === "yes" })}
          >
            <option value="no">Guest checkout (Fastest checkout, repeat buyers recognized by phone)</option>
            <option value="yes">Mandatory accounts (Buyers retain saved addresses and order history)</option>
          </select>
        </label>
        <label>
          Selected plan
          <select
            value={request.plan}
            onChange={(e) => setRequest({ ...request, plan: e.target.value as Plan })}
          >
            <option value="free">Free Tier (with 14-day trial)</option>
            <option value="pro">Pro Tier (BDT 1,500/month)</option>
          </select>
        </label>
        {request.plan === "pro" && (
          <div className="pro-note">
            <CreditCard size={18} weight="regular" />
            <span>Pro gateway and custom domain features enter provisioning upon payment verification.</span>
          </div>
        )}
        <button className="button button-primary button-full" type="submit">
          <span>Submit store request</span>
          <ArrowRight size={15} />
        </button>
      </form>
    </section>
  )
}

function OnePageMarketingSite({ onOpenEngineersTalk }: { onOpenEngineersTalk: () => void }) {
  return (
    <>
      <Hero />
      <MerchantJourney />
      <MerchantValuePillars />
      <DemoTeaserCallout />
      <PricingSection />
      <MerchantReviews />
      <CustomerReviews />
      <MarketingFaq />
      <LaunchChapter />
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
    canonical.href = "https://stheno.shop" + (activePath === "/" ? "/" : activePath)
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
    ) : (
      <>
        <Welcome />
        <Footer onOpenEngineersTalk={() => setEngineersTalkOpen(true)} />
      </>
    )

  return (
    <main className={`marketing-app page-${page.slice(1) || "home"}`}>
      <Suspense fallback={null}>{page === "/" && <ScrollScene />}</Suspense>
      <Header />
      {content}
      <EngineersTalkModal isOpen={engineersTalkOpen} onClose={() => setEngineersTalkOpen(false)} />
    </main>
  )
}
