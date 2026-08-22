import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import styles from "./footer.module.css"

/**
 * PORT of The Matrix Hours `Footer` — its stylesheet, its four-column top
 * section, its brand block with tagline and social row, its bottom bar.
 *
 * Core resolves collections and categories (ADR-013 decision 4); the source
 * footer read them from `ShopContext` and hard-coded a single "All Watches"
 * link, so the link lists here are real catalog data where the source had a
 * placeholder.
 *
 * Dropped: the GSAP scroll-reveal, and the `settings.instagram_link` /
 * `facebook_link` / `whatsapp_number` block — those came from the source shop's
 * own settings sheet, and this theme has no contract slot to receive them.
 * Rather than invent handles, the social row renders only what it is given,
 * which today is nothing. AGENTS.md §12's standing rule covers why.
 */
export default async function Footer({
  collections,
  productCategories,
}: {
  collections: HttpTypes.StoreCollection[]
  productCategories: HttpTypes.StoreProductCategory[]
}) {
  const year = new Date().getFullYear()

  return (
    <footer className={styles.footer}>
      <div className={styles.container}>
        <div className={styles.topSection}>
          <div className={styles.colBrand}>
            <LocalizedClientLink href="/" className={styles.brandLogo}>
              THE MATRIX HOURS
            </LocalizedClientLink>
            <p className={styles.tagline}>Time Redefined.</p>
          </div>

          <div className={styles.colLinks}>
            <h4 className={styles.colHeader}>Shop</h4>
            <div className={styles.linkList}>
              <LocalizedClientLink href="/store" className={styles.link}>
                All Watches
              </LocalizedClientLink>
              <LocalizedClientLink
                href="/store?sortBy=created_at"
                className={styles.link}
              >
                New Arrivals
              </LocalizedClientLink>
              {productCategories?.slice(0, 4).map((category) => (
                <LocalizedClientLink
                  key={category.id}
                  href={`/categories/${category.handle}`}
                  className={styles.link}
                  data-testid="category-link"
                >
                  {category.name}
                </LocalizedClientLink>
              ))}
            </div>
          </div>

          <div className={styles.colLinks}>
            <h4 className={styles.colHeader}>Collections</h4>
            <div className={styles.linkList}>
              {collections?.slice(0, 6).map((collection) => (
                <LocalizedClientLink
                  key={collection.id}
                  href={`/collections/${collection.handle}`}
                  className={styles.link}
                  data-testid="collection-link"
                >
                  {collection.title}
                </LocalizedClientLink>
              ))}
            </div>
          </div>

          <div className={styles.colLinks}>
            <h4 className={styles.colHeader}>Account</h4>
            <div className={styles.linkList}>
              <LocalizedClientLink href="/account" className={styles.link}>
                Sign In
              </LocalizedClientLink>
              <LocalizedClientLink
                href="/account/orders"
                className={styles.link}
              >
                Orders
              </LocalizedClientLink>
              <LocalizedClientLink href="/cart" className={styles.link}>
                Cart
              </LocalizedClientLink>
            </div>
          </div>
        </div>

        <div className={styles.bottomSection}>
          <span className={styles.copyright}>
            © {year} The Matrix Hours. All rights reserved.
          </span>
        </div>
      </div>
    </footer>
  )
}
