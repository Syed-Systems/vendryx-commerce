import type { ComponentPropsWithoutRef, ReactNode } from "react"
import { convertToLocale, getProductPrice } from "@stheno/storefront-core"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import campaignHero from "../../public/media/campaign-hero.jpg"
import kurtaBlack from "../../public/media/kurta-black.jpg"
import kurtaGreen from "../../public/media/kurta-green.jpg"
import lehengaPink from "../../public/media/lehenga-pink.jpg"

type RecordLike = Record<string, any>

const localAsset = (asset: string | { src: string }) =>
  typeof asset === "string" ? asset : asset.src

const staticMedia = {
  hero: localAsset(campaignHero),
  black: localAsset(kurtaBlack),
  green: localAsset(kurtaGreen),
  pink: localAsset(lehengaPink),
}

const money = (amount: number | null | undefined, currency = "BDT") =>
  convertToLocale({ amount: amount ?? 0, currency_code: currency })

const productImage = (product: RecordLike) =>
  product?.thumbnail || product?.images?.[0]?.url || staticMedia.black

const productBrand = (product: RecordLike) =>
  product?.collection?.title || product?.subtitle || ""

const productPrice = (product: RecordLike) => {
  if (!product?.id) return null
  return getProductPrice({ product: product as any })?.cheapestPrice?.calculated_price || null
}

export function BrandLogo() {
  return (
    <span className="mahal-wordmark" aria-label="Vendryx by Syed Systems">
      <span aria-hidden="true">V</span>
      <strong>VENDRYX</strong>
      <small>BY SYED SYSTEMS</small>
    </span>
  )
}

export function ProductCard({ product, region: _region }: RecordLike) {
  const price = productPrice(product)
  return (
    <article className="product-card" data-testid="product-wrapper">
      <LocalizedClientLink
        className="product-media"
        href={`/products/${product.handle}`}
        aria-label={product.title}
      >
        <img src={productImage(product)} alt={product.title || ""} loading="lazy" />
        <span className="heart" aria-hidden="true">♡</span>
        <span className="quick-add" aria-hidden="true">+</span>
      </LocalizedClientLink>
      <div className="product-meta">
        {productBrand(product) ? <span className="product-brand">{productBrand(product)}</span> : null}
        <LocalizedClientLink href={`/products/${product.handle}`} data-testid="product-title">
          {product.title}
        </LocalizedClientLink>
        {price ? <span className="product-price">{price}</span> : null}
      </div>
    </article>
  )
}

export function Nav({ regions, locales, currentLocale, cartButton }: RecordLike) {
  const currentRegion = regions?.[0]
  return (
    <>
      <div className="announcement">VENDRYX EDIT</div>
      <header className="site-header">
        <div className="utility-row page-shell">
          <details className="mobile-menu">
            <summary aria-label="Open navigation">☰</summary>
            <nav aria-label="Mobile navigation">
              <LocalizedClientLink href="/store">New in</LocalizedClientLink>
              <LocalizedClientLink href="/store">Women</LocalizedClientLink>
              <LocalizedClientLink href="/store">Men</LocalizedClientLink>
              <LocalizedClientLink href="/store">Jewellery</LocalizedClientLink>
              <LocalizedClientLink href="/account">Account</LocalizedClientLink>
            </nav>
          </details>
          <LocalizedClientLink className="brand-link" href="/" data-testid="nav-home-link">
            <BrandLogo />
          </LocalizedClientLink>
          <div className="header-actions">
            <LocalizedClientLink className="icon-button" href="/account" aria-label="Account" data-testid="nav-account-link">♙</LocalizedClientLink>
            <LocalizedClientLink className="icon-button" href="/store" aria-label="Search products">⌕</LocalizedClientLink>
            {cartButton}
            {currentRegion ? (
              <span className="region-button" title={currentLocale || locales?.[0]?.name || "Store region"}>
                {currentRegion.currency_code?.toUpperCase()}
              </span>
            ) : null}
          </div>
        </div>
        <nav className="primary-nav" aria-label="Primary navigation">
          <LocalizedClientLink href="/store">DESIGNERS A-Z</LocalizedClientLink>
          <LocalizedClientLink href="/store?sortBy=created_at">NEW IN</LocalizedClientLink>
          <LocalizedClientLink href="/store">WOMEN</LocalizedClientLink>
          <LocalizedClientLink href="/store">MEN</LocalizedClientLink>
          <LocalizedClientLink href="/store">KIDS</LocalizedClientLink>
          <details className="nav-mega">
            <summary>JEWELLERY</summary>
            <div className="mega-menu">
              <div className="mega-inner">
                <div className="mega-list">
                  <h2>SHOP BY CATEGORY</h2>
                  {["Necklace Sets", "Earrings", "Rings", "Bangles", "Maang Tikkas", "Anklets", "Bracelets"].map((label) => (
                    <LocalizedClientLink key={label} href="/store">{label}</LocalizedClientLink>
                  ))}
                </div>
                <EditorialTile image={staticMedia.pink} label="THE BRIDAL EDIT" />
                <EditorialTile image={staticMedia.hero} label="FESTIVE JEWELS" />
              </div>
            </div>
          </details>
          <LocalizedClientLink href="/store">ACCESSORIES</LocalizedClientLink>
          <LocalizedClientLink href="/store">READY TO SHIP</LocalizedClientLink>
          <LocalizedClientLink className="sale-link" href="/store">SALE</LocalizedClientLink>
        </nav>
      </header>
    </>
  )
}

export function CartDropdown({ cart }: RecordLike) {
  const count = cart?.items?.reduce((sum: number, item: RecordLike) => sum + (item.quantity || 0), 0) || 0
  return (
    <LocalizedClientLink className="icon-button bag-link" href="/cart" aria-label={`Shopping bag, ${count} items`} data-testid="nav-cart-link">
      ♧{count ? <sup>{count}</sup> : null}
    </LocalizedClientLink>
  )
}

function EditorialTile({ image, label }: { image: string; label: string }) {
  return (
    <LocalizedClientLink className="editorial-tile" href="/store">
      <img src={image} alt="" />
      <span>{label}</span>
    </LocalizedClientLink>
  )
}

export function Hero() {
  return (
    <section className="home-hero">
      <img src={staticMedia.hero} alt="South Asian formal fashion edit" />
      <div className="hero-copy">
        <p>THE FESTIVE EDIT</p>
        <h1>TIMELESS TRADITIONS</h1>
        <LocalizedClientLink href="/store">SHOP NOW</LocalizedClientLink>
      </div>
    </section>
  )
}

export function BrandFeatures() {
  return (
    <div className="mahal-home-editorial">
      <section className="campaign-banner page-shell">
        <img src={staticMedia.green} alt="Festive menswear edit" />
        <div><p>VENDRYX PRESENTS</p><h2>ICONS OF ELEGANCE</h2><LocalizedClientLink href="/store">DISCOVER</LocalizedClientLink></div>
      </section>
      <section className="designer-spotlight page-shell section-block">
        <div className="section-heading-row"><div><p>NEW ARRIVALS</p><h2>DESIGNER SPOTLIGHT</h2></div><LocalizedClientLink href="/store">EXPLORE NOW</LocalizedClientLink></div>
        <div className="designer-collage">
          <EditorialTile image={staticMedia.pink} label="THE BRIDAL EDIT" />
          <EditorialTile image={staticMedia.green} label="FESTIVE MENSWEAR" />
          <EditorialTile image={staticMedia.black} label="EVENING KURTAS" />
          <EditorialTile image={staticMedia.hero} label="OCCASION WEAR" />
          <EditorialTile image={staticMedia.pink} label="NEW SEASON" />
        </div>
      </section>
    </div>
  )
}

export function CategoryGrid() {
  const cards = [
    ["READY TO SHIP", staticMedia.black],
    ["FESTIVE COLLECTION", staticMedia.green],
    ["OCCASION WEAR", staticMedia.pink],
  ]
  return (
    <section className="quick-edits page-shell section-block" aria-label="Featured edits">
      <div className="merch-three">
        {cards.map(([label, image]) => <EditorialTile key={label} label={label} image={image} />)}
      </div>
    </section>
  )
}

export function LookbookStory() {
  const styles = [
    ["LEHENGAS", staticMedia.pink],
    ["CO-ORDS", staticMedia.hero],
    ["ANARKALIS", staticMedia.green],
    ["SAREES", staticMedia.pink],
    ["KAFTANS", staticMedia.black],
    ["KURTA SETS", staticMedia.green],
    ["GOWNS", staticMedia.hero],
    ["SHARARA SETS", staticMedia.pink],
  ]
  const occasions = [
    ["MEHENDI", staticMedia.hero],
    ["SANGEET", staticMedia.green],
    ["WEDDING", staticMedia.pink],
    ["RECEPTION", staticMedia.pink],
    ["ENGAGEMENT", staticMedia.hero],
    ["COCKTAIL", staticMedia.black],
  ]
  return (
    <div className="lookbook-sequence">
      <section className="style-section page-shell section-block"><h2>SHOP BY STYLE</h2><div className="style-grid">{styles.map(([label, image]) => <EditorialTile key={label} label={label} image={image} />)}</div></section>
      <section className="occasion-section"><div className="page-shell"><h2>SHOP BY OCCASIONS</h2><div className="occasion-grid">{occasions.map(([label, image]) => <EditorialTile key={label} label={label} image={image} />)}</div></div></section>
      <section className="editorial-duo page-shell section-block"><EditorialTile image={staticMedia.pink} label="DESIGNER OF THE WEEK" /><EditorialTile image={staticMedia.hero} label="CELEBRITY SPOTTING" /></section>
      <section className="wide-story page-shell section-block"><img src={staticMedia.pink} alt="Bridal fashion edit" /><div><p>BRIDAL EDIT</p><h2>THE ART OF CELEBRATION</h2><LocalizedClientLink href="/store">EXPLORE THE EDIT</LocalizedClientLink></div></section>
      <section className="bridal-three page-shell section-block"><EditorialTile image={staticMedia.pink} label="LEHENGA COUTURE" /><EditorialTile image={staticMedia.black} label="TROUSSEAU STYLES" /><EditorialTile image={staticMedia.hero} label="BRIDESMAID EDIT" /></section>
      <section className="campaign-banner campaign-banner-dark"><img src={staticMedia.hero} alt="Wedding collection edit" /><div><p>THE WEDDING EDIT</p><h2>ELEGANCE NEVER FADES</h2><LocalizedClientLink href="/store">SHOP NOW</LocalizedClientLink></div></section>
      <section className="accessory-edit page-shell section-block"><h2>ACCESSORY EDIT</h2><div><EditorialTile image={staticMedia.black} label="FLAWLESS FINISH" /><EditorialTile image={staticMedia.pink} label="BRIDAL RADIANCE" /></div></section>
    </div>
  )
}

export function NewsletterSection() {
  return null
}

export function ProductRail({ collection, region, products }: RecordLike) {
  if (!products?.length) return null
  return (
    <section className="spotlight page-shell section-block" aria-label={collection.title}>
      <div className="section-heading-row">
        <h2>{collection.title}</h2>
        <LocalizedClientLink href={`/collections/${collection.handle}`}>VIEW ALL</LocalizedClientLink>
      </div>
      <div className="product-rail">
        {products.map((product: RecordLike) => <ProductCard key={product.id} product={product} region={region} />)}
      </div>
    </section>
  )
}

function CatalogHeading({ eyebrow, title, description }: { eyebrow?: string; title: string; description?: string | null }) {
  return (
    <header className="catalog-heading">
      {eyebrow ? <p>{eyebrow}</p> : null}
      <h1>{title}</h1>
      {description ? <div>{description}</div> : null}
    </header>
  )
}

function CatalogShell({ heading, count, children }: { heading: ReactNode; count?: number; children: ReactNode }) {
  return (
    <main className="catalog-page page-shell">
      {heading}
      <div className="catalog-toolbar">
        <span>FILTER</span>
        <span>{typeof count === "number" ? `${count} PRODUCTS` : "PRODUCTS"}</span>
        <span>SORT⌄</span>
      </div>
      <div className="catalog-body">
        <aside className="filter-rail" aria-label="Catalog filters">
          {["DESIGNERS", "CATEGORY", "SIZE", "COLOR"].map((filter) => <button key={filter} type="button">{filter}<span>+</span></button>)}
        </aside>
        <div className="catalog-results">{children}</div>
      </div>
    </main>
  )
}

type ProductListRenderer = (args: {
  sortBy: SortOptions
  page: number
  countryCode: string
  categoryId?: string
  collectionId?: string
}) => ReactNode

type StoreTemplateInput = {
  productList: ProductListRenderer
  sortBy?: SortOptions
  page?: string
  countryCode: string
}

export function StoreTemplate({ productList, sortBy, page, countryCode }: StoreTemplateInput) {
  const sort = sortBy || "created_at"
  const pageNumber = page ? Number.parseInt(page, 10) : 1
  return (
    <CatalogShell heading={<CatalogHeading eyebrow="CATALOG" title="THE VENDRYX EDIT" />}>
      {productList({ sortBy: sort, page: pageNumber, countryCode })}
    </CatalogShell>
  )
}

export function CategoryTemplate({ productList, category, sortBy, page, countryCode }: StoreTemplateInput & { category: RecordLike }) {
  const sort = sortBy || "created_at"
  const pageNumber = page ? Number.parseInt(page, 10) : 1
  if (!category) return null
  return (
    <CatalogShell heading={<CatalogHeading eyebrow="CATEGORY" title={category.name} description={category.description} />} count={category.products?.length}>
      {productList({ sortBy: sort, page: pageNumber, categoryId: category.id, countryCode })}
    </CatalogShell>
  )
}

export function CollectionTemplate({ productList, collection, sortBy, page, countryCode }: StoreTemplateInput & { collection: RecordLike }) {
  const sort = sortBy || "created_at"
  const pageNumber = page ? Number.parseInt(page, 10) : 1
  if (!collection) return null
  return (
    <CatalogShell heading={<CatalogHeading eyebrow="COLLECTION" title={collection.title} />} count={collection.products?.length}>
      {productList({ sortBy: sort, page: pageNumber, collectionId: collection.id, countryCode })}
    </CatalogShell>
  )
}

export function PaginatedProducts({ products, region, page, count }: RecordLike) {
  const totalPages = Math.max(1, Math.ceil((count || 0) / 12))
  return (
    <>
      <div className="catalog-grid" data-testid="products-list">
        {products?.map((product: RecordLike) => <ProductCard key={product.id} product={product} region={region} />)}
      </div>
      {totalPages > 1 ? <nav className="pagination" aria-label="Pagination"><span>←</span><span>{page} / {totalPages}</span><span>→</span></nav> : null}
    </>
  )
}

export type SortOptions = "created_at" | "price_asc" | "price_desc"

export function SortProducts({ sortBy: _sortBy }: { sortBy?: SortOptions }) {
  return <span className="sort-label">SORT BY</span>
}

export function ProductTemplate({ product, images, productActions, relatedProducts }: RecordLike) {
  if (!product) return null
  const media = images?.length ? images : product.images || []
  return (
    <main className="product-page page-shell" data-testid="product-container">
      <div className="product-layout">
        <div className="thumbnail-rail" aria-label="Product thumbnails">
          {(media.length ? media : [{ url: productImage(product) }]).slice(0, 6).map((image: RecordLike, index: number) => (
            <span key={image.id || index}><img src={image.url} alt="" /></span>
          ))}
        </div>
        <div className="main-product-media"><img src={productImage(product)} alt={product.title || ""} /></div>
        <section className="purchase-panel">
          {productBrand(product) ? <p>{productBrand(product)}</p> : null}
          <h1>{product.title}</h1>
          {product.subtitle ? <p className="product-subtitle">{product.subtitle}</p> : null}
          {productActions}
        </section>
      </div>
      {product.description ? <details className="product-details" open><summary>DESCRIPTION</summary><p>{product.description}</p></details> : null}
      {relatedProducts}
    </main>
  )
}

export function RelatedProducts({ region, products }: RecordLike) {
  if (!products?.length) return null
  return (
    <section className="related section-block">
      <h2>RELATED PRODUCTS</h2>
      <div className="related-grid">{products.slice(0, 4).map((product: RecordLike) => <ProductCard key={product.id} product={product} region={region} />)}</div>
    </section>
  )
}

function CartLine({ item, currencyCode, compact = false }: { item: RecordLike; currencyCode: string; compact?: boolean }) {
  return (
    <article className={`cart-line${compact ? " compact" : ""}`}>
      <img src={item.thumbnail || item.variant?.product?.thumbnail || staticMedia.black} alt={item.product_title || ""} />
      <div className="cart-line-copy">
        {item.product_collection ? <span>{item.product_collection}</span> : null}
        <h3>{item.product_title || item.title}</h3>
        {item.variant_title ? <p>{item.variant_title}</p> : null}
        <strong>{money(item.total, currencyCode)}</strong>
        {!compact ? <div className="line-actions"><span>QTY {item.quantity}</span></div> : null}
      </div>
    </article>
  )
}

export function CartTotals({ totals }: RecordLike) {
  const currency = totals?.currency_code || "BDT"
  return (
    <section className="totals">
      <h2>ORDER SUMMARY</h2>
      <div><span>Subtotal</span><span data-testid="cart-subtotal">{money(totals?.item_subtotal ?? totals?.subtotal, currency)}</span></div>
      {totals?.discount_subtotal ? <div><span>Discount</span><span>− {money(totals.discount_subtotal, currency)}</span></div> : null}
      <div><span>Delivery</span><span data-testid="cart-shipping">{money(totals?.shipping_subtotal, currency)}</span></div>
      <div><span>Tax</span><span data-testid="cart-taxes">{money(totals?.tax_total, currency)}</span></div>
      <div className="total-row"><strong>Total</strong><strong data-testid="cart-total">{money(totals?.total, currency)}</strong></div>
    </section>
  )
}

export function ItemsPreviewTemplate({ cart }: RecordLike) {
  return <div data-testid="items-table">{cart?.items?.map((item: RecordLike) => <CartLine key={item.id} item={item} currencyCode={cart.currency_code} compact />)}</div>
}

export function CartTemplate({ cart, customer }: RecordLike) {
  if (!cart?.items?.length) {
    return <main className="cart-page page-shell empty-state"><h1>YOUR BAG IS EMPTY</h1><LocalizedClientLink className="solid-button" href="/store">CONTINUE SHOPPING</LocalizedClientLink></main>
  }
  return (
    <main className="cart-page page-shell" data-testid="cart-container">
      <h1>SHOPPING BAG ({cart.items.reduce((sum: number, item: RecordLike) => sum + item.quantity, 0)})</h1>
      {!customer ? <p className="cart-note">Sign in from your account to keep this bag available across devices.</p> : null}
      <div className="cart-layout">
        <div>{cart.items.map((item: RecordLike) => <CartLine key={item.id} item={item} currencyCode={cart.currency_code} />)}<DiscountCode cart={cart} /></div>
        <aside className="cart-summary"><CartTotals totals={cart} /><LocalizedClientLink className="solid-button" href="/checkout?step=address">CHECKOUT</LocalizedClientLink><LocalizedClientLink className="outline-button" href="/store">CONTINUE SHOPPING</LocalizedClientLink></aside>
      </div>
    </main>
  )
}

export function DiscountCode({ cart: _cart }: RecordLike) {
  return <details className="discount"><summary>DISCOUNT CODE</summary><p>Discount codes can be applied during checkout.</p></details>
}

export function CartMismatchBanner({ customer, cart }: RecordLike) {
  if (!customer?.email || !cart?.email || customer.email === cart.email) return null
  return <div className="mahal-banner" role="status">This bag belongs to a different account. Review it before checkout.</div>
}

export function FreeShippingPriceNudge(_props: RecordLike) {
  return null
}

export function Footer({ collections, productCategories }: RecordLike) {
  return (
    <footer className="site-footer">
      <div className="footer-grid page-shell">
        <div className="footer-brand"><BrandLogo /><LocalizedClientLink href="/store">BROWSE THE CATALOG</LocalizedClientLink></div>
        <FooterGroup title="SHOP" links={(productCategories || []).slice(0, 5).map((item: RecordLike) => ({ label: item.name, href: `/categories/${item.handle}` }))} />
        <FooterGroup title="COLLECTION" links={(collections || []).slice(0, 5).map((item: RecordLike) => ({ label: item.title, href: `/collections/${item.handle}` }))} />
        <FooterGroup title="ACCOUNT" links={[{ label: "My account", href: "/account" }, { label: "Shopping bag", href: "/cart" }, { label: "Orders", href: "/account/orders" }]} />
      </div>
      <div className="footer-bottom page-shell">© {new Date().getFullYear()} VENDRYX BY SYED SYSTEMS</div>
    </footer>
  )
}

function FooterGroup({ title, links }: { title: string; links: { label: string; href: string }[] }) {
  if (!links.length) return null
  return <nav className="footer-group" aria-label={title}><h2>{title}</h2>{links.map((link) => <LocalizedClientLink key={link.href + link.label} href={link.href}>{link.label}</LocalizedClientLink>)}</nav>
}

export function AccountLayout({ customer, children }: RecordLike) {
  return (
    <main className="account-page page-shell">
      <div className="breadcrumbs">Home / Account</div>
      <h1>MY ACCOUNT</h1>
      {customer?.first_name ? <p className="greeting">Hello, {customer.first_name}</p> : null}
      <nav className="account-nav" aria-label="Account navigation">
        <LocalizedClientLink href="/account">OVERVIEW</LocalizedClientLink>
        <LocalizedClientLink href="/account/profile">PROFILE</LocalizedClientLink>
        <LocalizedClientLink href="/account/addresses">ADDRESSES</LocalizedClientLink>
        <LocalizedClientLink href="/account/orders">ORDERS</LocalizedClientLink>
      </nav>
      {children}
    </main>
  )
}

export function LoginTemplate({ login, register }: RecordLike) {
  return <main className="auth-page page-shell"><section><h1>WELCOME BACK</h1>{login}</section><aside><h2>NEW TO VENDRYX?</h2>{register}</aside></main>
}

export function Overview({ customer, orders }: RecordLike) {
  return (
    <div>
      <section className="account-details"><h2>ACCOUNT DETAILS</h2><dl><div><dt>Name</dt><dd>{[customer?.first_name, customer?.last_name].filter(Boolean).join(" ")}</dd></div><div><dt>Email</dt><dd>{customer?.email}</dd></div><div><dt>Phone</dt><dd>{customer?.phone || ""}</dd></div></dl></section>
      <OrderOverview orders={orders} />
    </div>
  )
}

export function OrderOverview({ orders }: RecordLike) {
  return <section className="recent-orders"><h2>RECENT ORDERS</h2>{orders?.length ? orders.map((order: RecordLike) => <OrderRow key={order.id} order={order} />) : <p className="end-state">NO ORDERS YET</p>}</section>
}

function OrderRow({ order }: { order: RecordLike }) {
  return <article><div><strong>#{order.display_id}</strong><span>{order.created_at ? new Date(order.created_at).toLocaleDateString() : ""}</span></div><span>{order.fulfillment_status || order.status}</span><div><strong>{money(order.total, order.currency_code)}</strong><LocalizedClientLink href={`/account/orders/details/${order.id}`}>VIEW ORDER</LocalizedClientLink></div></article>
}

export function AddressBook({ customer, addresses }: RecordLike) {
  const source = addresses || customer?.addresses || []
  return <section className="account-details"><h2>ADDRESS BOOK</h2><div className="address-grid">{source.map((address: RecordLike) => <address key={address.id}>{address.first_name} {address.last_name}<br />{address.address_1}<br />{address.city} {address.postal_code}<br />{address.country_code?.toUpperCase()}</address>)}</div></section>
}

function ProfileField({ label, value, children }: { label: string; value?: ReactNode; children?: ReactNode }) {
  return <section className="profile-field"><div><span>{label}</span><strong>{value}</strong></div>{children}</section>
}

export function ProfileName({ customer, children }: RecordLike) { return <ProfileField label="NAME" value={[customer?.first_name, customer?.last_name].filter(Boolean).join(" ")}>{children}</ProfileField> }
export function ProfileEmail({ customer, children }: RecordLike) { return <ProfileField label="EMAIL" value={customer?.email}>{children}</ProfileField> }
export function ProfilePhone({ customer, children }: RecordLike) { return <ProfileField label="PHONE" value={customer?.phone}>{children}</ProfileField> }
export function ProfilePassword({ children }: RecordLike) { return <ProfileField label="PASSWORD" value="••••••••">{children}</ProfileField> }
export function ProfileBillingAddress({ customer, children }: RecordLike) { return <ProfileField label="BILLING ADDRESS" value={customer?.billing_address?.address_1}>{children}</ProfileField> }

export function TransferRequestForm({ order, children }: RecordLike) {
  return <section className="transfer-panel"><h2>TRANSFER ORDER</h2>{order?.display_id ? <p>Order #{order.display_id}</p> : null}{children}</section>
}

export function TransferActions({ children, className = "" }: RecordLike) { return <div className={`transfer-actions ${className}`}>{children}</div> }
export function TransferImage({ className = "" }: RecordLike) { return <div className={`transfer-image ${className}`} aria-hidden="true">V</div> }

export function OrderCompletedTemplate({ order, children }: RecordLike) {
  return <main className="order-page page-shell"><p className="order-kicker">ORDER CONFIRMED</p><h1>THANK YOU</h1>{order?.display_id ? <p>Order #{order.display_id}</p> : null}<OrderDetailsTemplate order={order} />{children}</main>
}

export function OrderDetailsTemplate({ order, children }: RecordLike) {
  return <section className="order-details"><h2>ORDER DETAILS</h2>{order?.items?.map((item: RecordLike) => <CartLine key={item.id} item={item} currencyCode={order.currency_code} compact />)}{order ? <CartTotals totals={order} /> : null}{children}</section>
}

export function SkeletonCardDetails() { return <div className="skeleton-card" aria-hidden="true"><span /><span /><span /></div> }
export function SkeletonCartPage() { return <main className="cart-page page-shell skeleton-page" aria-busy="true"><SkeletonCardDetails /><SkeletonCardDetails /></main> }
export function SkeletonOrderConfirmed() { return <main className="order-page page-shell skeleton-page" aria-busy="true"><SkeletonCardDetails /></main> }

export function Divider({ className = "", ...props }: RecordLike) { return <hr className={`mahal-divider ${className}`} {...props} /> }
export function ErrorMessage({ error, className = "" }: RecordLike) { return error ? <p className={`field-error ${className}`} role="alert">{error}</p> : null }
export function ChevronDown({ size, ...props }: ComponentPropsWithoutRef<"svg"> & { size?: number }) { return <svg width={size} height={size} viewBox="0 0 12 8" aria-hidden="true" {...props}><path d="m1 1 5 5 5-5" fill="none" stroke="currentColor" /></svg> }
export function Spinner({ size, ...props }: RecordLike) { return <span aria-label="Loading" style={size ? { fontSize: size } : undefined} {...props}>◌</span> }

export function InteractiveLink({ children, href, ...props }: RecordLike) {
  return <LocalizedClientLink className="text-link" href={href} {...props}>{children}<span aria-hidden="true">→</span></LocalizedClientLink>
}

export function Checkbox({ checked, ...props }: RecordLike) {
  return <span className={`mahal-check${checked ? " checked" : ""}`} aria-hidden="true" {...props}>{checked ? "✓" : ""}</span>
}

export function RadioIndicator({ checked, ...props }: RecordLike) {
  return <span className={`mahal-radio${checked ? " checked" : ""}`} aria-hidden="true" {...props}><span /></span>
}

export function CountrySelect({ placeholder = "Country", children, ...props }: RecordLike) {
  return <label className="field"><span>{placeholder}</span><select {...props}>{children}</select></label>
}

export function Input({ label, name, errors, ...props }: RecordLike) {
  const message = errors?.[name]?.message
  return <label className={`field${message ? " invalid" : ""}`}><span>{label}</span><input name={name} aria-invalid={Boolean(message)} {...props} />{message ? <small>{message}</small> : null}</label>
}

export function SubmitButton({ children, className = "", ...props }: RecordLike) {
  return <button className={`solid-button ${className}`} type="submit" {...props}>{children}</button>
}
