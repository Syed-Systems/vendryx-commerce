import {
  ArrowLeft,
  ArrowRight,
  Bag,
  CaretDown,
  Heart,
  List,
  MagnifyingGlass,
  Minus,
  Plus,
  User,
  X,
} from "@phosphor-icons/react"
import { useEffect, useMemo, useState } from "react"
import brandLogo from "../assets/vendryx-logo.svg"
import { products, styles, type FixtureProduct } from "./data"

type RouteKey = "home" | "catalog" | "product" | "cart" | "checkout" | "account"

const routeFromHash = (): RouteKey => {
  const route = window.location.hash.replace(/^#\/?/, "").split("/")[0]
  return (["catalog", "product", "cart", "checkout", "account"] as RouteKey[]).includes(
    route as RouteKey
  )
    ? (route as RouteKey)
    : "home"
}

const fixtureHero = "/media/campaign-hero.jpg"
const fixtureEditorial = "/media/kurta-green.jpg"

function BrandLogo() {
  return <img className="brand-logo" src={brandLogo} alt="Vendryx by Syed Systems" />
}

function IconButton({
  label,
  children,
  onClick,
}: {
  label: string
  children: React.ReactNode
  onClick?: () => void
}) {
  return (
    <button className="icon-button" type="button" aria-label={label} onClick={onClick}>
      {children}
    </button>
  )
}

function Header({ onCart }: { onCart: () => void }) {
  const [menuOpen, setMenuOpen] = useState(false)

  return (
    <>
      <div className="announcement" data-fixture="true">
        NEW SEASON EDIT
      </div>
      <header className="site-header">
        <div className="utility-row page-shell">
          <IconButton label="Open navigation" onClick={() => setMenuOpen((value) => !value)}>
            {menuOpen ? <X size={26} /> : <List size={28} />}
          </IconButton>
          <a className="brand-link" href="#/" aria-label="Vendryx home">
            <BrandLogo />
          </a>
          <div className="header-actions">
            <IconButton label="Account">
              <User size={21} />
            </IconButton>
            <IconButton label="Search">
              <MagnifyingGlass size={23} />
            </IconButton>
            <IconButton label="Shopping bag" onClick={onCart}>
              <Bag size={24} />
            </IconButton>
            <button className="region-button" type="button">
              BDT <CaretDown size={13} />
            </button>
          </div>
        </div>
        <nav className="primary-nav" aria-label="Primary navigation">
          <a href="#/catalog">DESIGNERS A-Z</a>
          <a href="#/catalog">NEW IN</a>
          <a href="#/catalog">WOMEN</a>
          <a href="#/catalog">MEN</a>
          <a href="#/catalog">KIDS</a>
          <button
            className={menuOpen ? "nav-trigger active" : "nav-trigger"}
            type="button"
            aria-expanded={menuOpen}
            onClick={() => setMenuOpen((value) => !value)}
          >
            JEWELLERY
          </button>
          <a href="#/catalog">ACCESSORIES</a>
          <a href="#/catalog">READY TO SHIP</a>
          <a className="sale-link" href="#/catalog">
            SALE
          </a>
        </nav>
        {menuOpen ? <MegaMenu onNavigate={() => setMenuOpen(false)} /> : null}
      </header>
    </>
  )
}

function MegaMenu({ onNavigate }: { onNavigate: () => void }) {
  const categories = [
    "Necklace Sets",
    "Earrings",
    "Rings",
    "Bangles",
    "Maang Tikkas",
    "Anklets",
    "Bracelets",
    "Nose Rings",
    "Mathapatti",
    "Kamarbandh",
  ]

  return (
    <div className="mega-menu" role="region" aria-label="Jewellery menu">
      <div className="mega-inner">
        <div className="mega-list">
          <h2>SHOP BY CATEGORY</h2>
          {categories.map((category) => (
            <a key={category} href="#/catalog" onClick={onNavigate}>
              {category}
            </a>
          ))}
        </div>
        <EditorialTile image={fixtureHero} label="THE FESTIVE EDIT" />
        <EditorialTile image={fixtureEditorial} label="BRIDAL JEWELS" />
      </div>
    </div>
  )
}

function EditorialTile({ image, label }: { image: string; label: string }) {
  return (
    <a className="editorial-tile" href="#/catalog">
      <img src={image} alt="" />
      <span>{label}</span>
    </a>
  )
}

function CartDrawer({ open, onClose }: { open: boolean; onClose: () => void }) {
  if (!open) return null

  const item = products[0]
  return (
    <div className="drawer-layer" role="presentation" onMouseDown={onClose}>
      <aside
        className="cart-drawer"
        role="dialog"
        aria-modal="true"
        aria-labelledby="bag-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="drawer-title">
          <h2 id="bag-title">YOUR BAG (1)</h2>
          <IconButton label="Close bag" onClick={onClose}>
            <X size={24} />
          </IconButton>
        </div>
        <CartLine product={item} compact />
        <div className="drawer-subtotal">
          <span>Subtotal (1 item)</span>
          <strong>{item.price}</strong>
        </div>
        <a className="solid-button" href="#/cart" onClick={onClose}>
          VIEW BAG
        </a>
        <a className="outline-button" href="#/checkout" onClick={onClose}>
          CHECKOUT
        </a>
      </aside>
    </div>
  )
}

function Footer() {
  return (
    <footer className="site-footer">
      <div className="footer-grid page-shell">
        <div className="footer-brand">
          <BrandLogo />
        </div>
        <FooterGroup title="SHOP" links={["New In", "Women", "Men", "Kids", "Accessories"]} />
        <FooterGroup title="INFORMATION" links={["About Us", "Contact Us", "FAQs"]} />
        <FooterGroup title="ACCOUNT" links={["My Account", "Shopping Bag", "Orders"]} />
      </div>
      <div className="footer-bottom page-shell">© VENDRYX BY SYED SYSTEMS</div>
    </footer>
  )
}

function FooterGroup({ title, links }: { title: string; links: string[] }) {
  return (
    <div className="footer-group">
      <h2>{title}</h2>
      {links.map((link) => (
        <a key={link} href="#/">
          {link}
        </a>
      ))}
    </div>
  )
}

function ProductCard({ product }: { product: FixtureProduct }) {
  return (
    <article className="product-card" data-fixture="true">
      <a className="product-media" href="#/product" aria-label={product.title}>
        <img src={product.image} alt="" />
        {product.badge ? <span className="product-badge">{product.badge}</span> : null}
        <Heart className="heart" size={22} aria-hidden="true" />
        <span className="quick-add" aria-hidden="true">
          <Plus size={18} />
        </span>
      </a>
      <div className="product-meta">
        <span className="product-brand">{product.brand}</span>
        <a href="#/product">{product.title}</a>
        <span className="product-price">{product.price}</span>
      </div>
    </article>
  )
}

function HomePage() {
  const styleCards = [...styles, ...styles]
  const occasionCards = [
    ["MEHENDI", styles[2][1]],
    ["SANGEET", fixtureEditorial],
    ["WEDDING", styles[0][1]],
    ["RECEPTION", styles[0][1]],
    ["ENGAGEMENT", fixtureHero],
    ["COCKTAIL", products[0].image],
  ]
  return (
    <main>
      <section className="home-hero">
        <img src={fixtureHero} alt="South Asian formal fashion edit" />
        <div className="hero-copy">
          <p>THE FESTIVE EDIT</p>
          <h1>TIMELESS TRADITIONS</h1>
          <a href="#/catalog">SHOP NOW</a>
        </div>
      </section>

      <section className="campaign-banner page-shell">
        <img src={fixtureEditorial} alt="Festive menswear edit" />
        <div><p>VENDRYX PRESENTS</p><h2>ICONS OF ELEGANCE</h2><a href="#/catalog">DISCOVER</a></div>
      </section>

      <section className="designer-spotlight page-shell section-block">
        <div className="section-heading-row"><div><p>NEW ARRIVALS</p><h2>DESIGNER SPOTLIGHT</h2></div><a href="#/catalog">EXPLORE NOW</a></div>
        <div className="designer-collage">
          {[products[4], products[1], products[0], products[2], products[4]].map((product, index) => (
            <EditorialTile key={`${product.id}-${index}`} image={product.image} label={["THE BRIDAL EDIT", "FESTIVE MENSWEAR", "EVENING KURTAS", "OCCASION WEAR", "NEW SEASON"][index]} />
          ))}
        </div>
      </section>

      <section className="quick-edits page-shell section-block">
        <div className="merch-three">
          <EditorialTile image={products[0].image} label="READY TO SHIP" />
          <EditorialTile image={products[1].image} label="FESTIVE COLLECTION" />
          <EditorialTile image={products[4].image} label="OCCASION WEAR" />
        </div>
      </section>

      <section className="spotlight page-shell section-block">
        <div className="section-heading-row">
          <h2>DESIGNER SPOTLIGHT</h2>
          <a href="#/catalog">VIEW ALL</a>
        </div>
        <a className="spotlight-lead" href="#/catalog">
          <img src={fixtureEditorial} alt="Designer collection" />
          <span>
            <strong>THE NEW COLLECTION</strong>
            <small>EXPLORE NOW</small>
          </span>
        </a>
        <div className="product-rail">
          {products.slice(0, 4).map((product) => (
            <ProductCard key={product.id} product={product} />
          ))}
        </div>
      </section>

      <section className="style-section page-shell section-block">
        <h2>SHOP BY STYLE</h2>
        <div className="style-grid">
          {styleCards.map(([label, image], index) => (
            <EditorialTile key={`${label}-${index}`} image={image} label={label} />
          ))}
        </div>
      </section>

      <section className="occasion-section">
        <div className="page-shell">
          <h2>SHOP BY OCCASION</h2>
          <div className="occasion-grid">
            {occasionCards.map(([label, image]) => (
              <EditorialTile key={label} image={image} label={label} />
            ))}
          </div>
        </div>
      </section>

      <section className="editorial-duo page-shell section-block">
        <EditorialTile image={products[4].image} label="DESIGNER OF THE WEEK" />
        <EditorialTile image={fixtureHero} label="CELEBRITY SPOTTING" />
      </section>

      <section className="wide-story page-shell section-block">
        <img src={fixtureHero} alt="Bridal collection" />
        <div>
          <p>BRIDAL EDIT</p>
          <h2>THE ART OF CELEBRATION</h2>
          <a href="#/catalog">EXPLORE THE EDIT</a>
        </div>
      </section>

      <section className="bridal-three page-shell section-block">
        <EditorialTile image={products[4].image} label="LEHENGA COUTURE" />
        <EditorialTile image={products[0].image} label="TROUSSEAU STYLES" />
        <EditorialTile image={fixtureHero} label="BRIDESMAID EDIT" />
      </section>

      <section className="campaign-banner campaign-banner-dark">
        <img src={fixtureHero} alt="Wedding collection edit" />
        <div><p>THE WEDDING EDIT</p><h2>ELEGANCE NEVER FADES</h2><a href="#/catalog">SHOP NOW</a></div>
      </section>

      <section className="accessory-edit page-shell section-block">
        <h2>ACCESSORY EDIT</h2>
        <div>
          <EditorialTile image={products[0].image} label="FLAWLESS FINISH" />
          <EditorialTile image={products[4].image} label="BRIDAL RADIANCE" />
        </div>
      </section>

      <Newsletter />
    </main>
  )
}

function Newsletter() {
  return (
    <section className="newsletter page-shell">
      <p>BE THE FIRST TO KNOW</p>
      <h2>NEWSLETTER</h2>
      <form onSubmit={(event) => event.preventDefault()}>
        <label className="sr-only" htmlFor="newsletter-email">
          Email address
        </label>
        <input id="newsletter-email" type="email" placeholder="Enter your email" />
        <button type="submit">SUBSCRIBE</button>
      </form>
    </section>
  )
}

function CatalogPage() {
  const catalog = useMemo(
    () => Array.from({ length: 32 }, (_, index) => products[index % products.length]),
    []
  )
  const filters = ["DESIGNERS", "CATEGORY", "SIZE", "COLOR"]

  return (
    <main className="catalog-page page-shell">
      <div className="breadcrumbs">Home / Men / Kurta Sets</div>
      <h1>KURTA SETS, MEN'S</h1>
      <div className="featured-row">
        {products.slice(0, 3).map((product) => (
          <ProductCard key={product.id} product={product} />
        ))}
      </div>
      <div className="catalog-toolbar">
        <button type="button">FILTER</button>
        <span>{catalog.length} PRODUCTS</span>
        <button type="button">
          SORT <CaretDown size={14} />
        </button>
      </div>
      <div className="catalog-body">
        <aside className="filter-rail" aria-label="Catalog filters">
          {filters.map((filter) => (
            <button key={filter} type="button">
              {filter} <Plus size={13} />
            </button>
          ))}
        </aside>
        <div className="catalog-grid">
          {catalog.map((product, index) => (
            <ProductCard key={`${product.id}-${index}`} product={product} />
          ))}
        </div>
      </div>
      <nav className="pagination" aria-label="Pagination">
        <button type="button" aria-label="Previous page">
          <ArrowLeft size={18} />
        </button>
        <span>1 / 2</span>
        <button type="button" aria-label="Next page">
          <ArrowRight size={18} />
        </button>
      </nav>
    </main>
  )
}

function ProductPage() {
  const product = products[0]
  const [size, setSize] = useState("M")

  return (
    <main className="product-page page-shell">
      <div className="product-layout">
        <div className="thumbnail-rail" aria-label="Product thumbnails">
          {[product.image, products[1].image, product.image, products[2].image].map((image, index) => (
            <button key={`${image}-${index}`} type="button" aria-label={`View image ${index + 1}`}>
              <img src={image} alt="" />
            </button>
          ))}
        </div>
        <div className="main-product-media">
          <img src={product.image} alt={product.title} />
        </div>
        <section className="purchase-panel" data-fixture="true">
          <p>{product.brand}</p>
          <h1>{product.title}</h1>
          <strong>{product.price}</strong>
          <div className="size-heading">
            <span>Size</span>
            <button type="button">Size Chart</button>
          </div>
          <div className="size-options">
            {["XS", "S", "M", "L", "XL"].map((option) => (
              <button
                key={option}
                className={size === option ? "selected" : ""}
                type="button"
                aria-pressed={size === option}
                onClick={() => setSize(option)}
              >
                {option}
              </button>
            ))}
          </div>
          <button className="solid-button" type="button">
            ADD TO BAG
          </button>
          <button className="outline-button" type="button">
            BUY NOW
          </button>
        </section>
      </div>

      <details className="product-details" open>
        <summary>DESCRIPTION</summary>
        <p>
          Product description, fabric, fit, and care are rendered from merchant product data. This
          fixture preserves the measured text position without publishing an invented product claim.
        </p>
      </details>

      <section className="related section-block">
        <h2>RELATED PRODUCTS</h2>
        <div className="related-grid">
          {products.slice(1, 5).map((item) => (
            <ProductCard key={item.id} product={item} />
          ))}
        </div>
      </section>
      <Newsletter />
    </main>
  )
}

function CartLine({ product, compact = false }: { product: FixtureProduct; compact?: boolean }) {
  return (
    <article className={compact ? "cart-line compact" : "cart-line"} data-fixture="true">
      <img src={product.image} alt="" />
      <div className="cart-line-copy">
        <span>{product.brand}</span>
        <h3>{product.title}</h3>
        <p>Size: M</p>
        <strong>{product.price}</strong>
        <div className="line-actions">
          <div className="stepper" aria-label="Quantity">
            <button type="button" aria-label="Decrease quantity">
              <Minus size={14} />
            </button>
            <span>1</span>
            <button type="button" aria-label="Increase quantity">
              <Plus size={14} />
            </button>
          </div>
          <button className="remove-button" type="button">
            REMOVE
          </button>
        </div>
      </div>
    </article>
  )
}

function Totals() {
  return (
    <section className="totals" data-fixture="true">
      <h2>ORDER SUMMARY</h2>
      <div><span>Subtotal (2 items)</span><span>BDT 52,300.00</span></div>
      <div><span>Discount</span><span>−</span></div>
      <div><span>Delivery</span><span>CALCULATED AT CHECKOUT</span></div>
      <div><span>Tax</span><span>−</span></div>
      <div className="total-row"><strong>Total</strong><strong>BDT 52,300.00</strong></div>
    </section>
  )
}

function CartPage() {
  return (
    <main className="cart-page page-shell">
      <h1>SHOPPING BAG (2)</h1>
      <div className="cart-layout">
        <div>
          <CartLine product={products[4]} />
          <CartLine product={products[0]} />
          <details className="discount" open>
            <summary>DISCOUNT CODE</summary>
            <form onSubmit={(event) => event.preventDefault()}>
              <input aria-label="Discount code" placeholder="Enter discount code" />
              <button type="submit">APPLY</button>
            </form>
          </details>
        </div>
        <div className="cart-summary">
          <Totals />
          <a className="solid-button" href="#/checkout">CHECKOUT</a>
          <a className="outline-button" href="#/catalog">CONTINUE SHOPPING</a>
        </div>
      </div>
    </main>
  )
}

function CheckoutPage() {
  return (
    <main className="checkout-page page-shell">
      <nav className="checkout-steps" aria-label="Checkout steps">
        {[
          ["ADDRESS", true],
          ["DELIVERY", false],
          ["PAYMENT", false],
          ["REVIEW", false],
        ].map(([label, active]) => (
          <span key={String(label)} className={active ? "active" : ""}>{label}</span>
        ))}
      </nav>
      <div className="checkout-layout">
        <form className="address-form" onSubmit={(event) => event.preventDefault()}>
          <h1>CONTACT AND DELIVERY ADDRESS</h1>
          <Field label="Email address" type="email" />
          <div className="field-row">
            <Field label="First name" />
            <Field label="Last name" />
          </div>
          <Field label="Address" />
          <Field label="Apartment, suite, etc. (optional)" />
          <div className="field-row">
            <Field label="City" />
            <Field label="Postal code" error="Please enter a valid postal code" />
          </div>
          <div className="field-row">
            <label className="field">
              <span>Country</span>
              <select defaultValue="BD"><option value="BD">Bangladesh</option></select>
            </label>
            <Field label="Phone number" type="tel" />
          </div>
          <button className="solid-button" type="submit">CONTINUE TO DELIVERY</button>
          {["DELIVERY", "PAYMENT", "REVIEW"].map((step) => (
            <button className="checkout-disclosure" key={step} type="button">
              {step} <CaretDown size={18} />
            </button>
          ))}
        </form>
        <div className="checkout-summary">
          <h2>ORDER SUMMARY</h2>
          <CartLine product={products[4]} compact />
          <CartLine product={products[0]} compact />
          <Totals />
        </div>
      </div>
    </main>
  )
}

function Field({ label, type = "text", error }: { label: string; type?: string; error?: string }) {
  const id = label.toLowerCase().replace(/[^a-z]+/g, "-")
  return (
    <label className={error ? "field invalid" : "field"} htmlFor={id}>
      <span>{label}</span>
      <input id={id} type={type} aria-invalid={Boolean(error)} aria-describedby={error ? `${id}-error` : undefined} />
      {error ? <small id={`${id}-error`}>{error}</small> : null}
    </label>
  )
}

function AccountPage() {
  return (
    <main className="account-page page-shell" data-fixture="true">
      <div className="breadcrumbs">Home / Account</div>
      <h1>MY ACCOUNT</h1>
      <p className="greeting">Hello, Rahim Khan</p>
      <nav className="account-nav" aria-label="Account navigation">
        {[
          ["OVERVIEW", true],
          ["PROFILE", false],
          ["ADDRESSES", false],
          ["ORDERS", false],
        ].map(([label, active]) => <a className={active ? "active" : ""} href="#/account" key={String(label)}>{label}</a>)}
      </nav>
      <section className="account-details">
        <h2>ACCOUNT DETAILS</h2>
        <dl>
          <div><dt>Name</dt><dd>Rahim Khan</dd></div>
          <div><dt>Email</dt><dd>rahim.khan@example.com</dd></div>
          <div><dt>Phone</dt><dd>01712345678</dd></div>
        </dl>
        <button type="button">EDIT PROFILE</button>
      </section>
      <section className="recent-orders">
        <h2>RECENT ORDERS</h2>
        {["#VDX1001", "#VDX1002"].map((order, index) => (
          <article key={order}>
            <div><strong>{order}</strong><span>{index === 0 ? "12 Dec 2024" : "05 Dec 2024"}</span></div>
            <span>{index === 0 ? "Delivered" : "Processing"}</span>
            <div><strong>{index === 0 ? "BDT 28,500.00" : "BDT 23,800.00"}</strong><a href="#/account">VIEW ORDER</a></div>
          </article>
        ))}
        <p className="end-state">NO MORE ORDERS</p>
      </section>
      <button className="outline-button sign-out" type="button">SIGN OUT</button>
    </main>
  )
}

export function App() {
  const [route, setRoute] = useState<RouteKey>(routeFromHash)
  const [cartOpen, setCartOpen] = useState(false)

  useEffect(() => {
    const onHashChange = () => {
      setRoute(routeFromHash())
      window.scrollTo({ top: 0 })
    }
    window.addEventListener("hashchange", onHashChange)
    return () => window.removeEventListener("hashchange", onHashChange)
  }, [])

  const page = {
    home: <HomePage />,
    catalog: <CatalogPage />,
    product: <ProductPage />,
    cart: <CartPage />,
    checkout: <CheckoutPage />,
    account: <AccountPage />,
  }[route]

  return (
    <div className="app-shell">
      <Header onCart={() => setCartOpen(true)} />
      {page}
      <Footer />
      <CartDrawer open={cartOpen} onClose={() => setCartOpen(false)} />
    </div>
  )
}
