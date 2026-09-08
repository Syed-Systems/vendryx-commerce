"use client"

import { Disclosure, DisclosureButton, DisclosurePanel } from "@headlessui/react"
import { CaretDown } from "@phosphor-icons/react/dist/ssr"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { HttpTypes } from "@medusajs/types"

export type FooterNavProps = {
  collections?: HttpTypes.StoreCollection[] | null
  productCategories?: HttpTypes.StoreProductCategory[] | null
}

export default function FooterNav({ collections, productCategories }: FooterNavProps) {
  const hasCollections = Boolean(collections && collections.length > 0)
  const hasCategories = Boolean(productCategories && productCategories.length > 0)

  return (
    <div>
      {/* Desktop Multi-column Grid */}
      <div className={`hidden md:grid gap-10 ${hasCategories ? "grid-cols-3" : "grid-cols-2"}`}>
        {/* Collections */}
        <div className="space-y-3 text-xs">
          <h4 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-primary)]">
            Collections
          </h4>
          <ul className="space-y-2 text-[var(--text-secondary)]" data-testid="footer-categories">
            <li>
              <LocalizedClientLink
                href="/store"
                className="hover:text-[var(--text-primary)] transition-colors"
              >
                All Items
              </LocalizedClientLink>
            </li>
            {hasCollections &&
              collections!.slice(0, 6).map((c) => (
                <li key={c.id}>
                  <LocalizedClientLink
                    href={`/collections/${c.handle}`}
                    className="hover:text-[var(--text-primary)] transition-colors"
                  >
                    {c.title}
                  </LocalizedClientLink>
                </li>
              ))}
          </ul>
        </div>

        {/* Categories if supplied */}
        {hasCategories && (
          <div className="space-y-3 text-xs">
            <h4 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-primary)]">
              Categories
            </h4>
            <ul className="space-y-2 text-[var(--text-secondary)]">
              {productCategories!.slice(0, 6).map((cat) => (
                <li key={cat.id}>
                  <LocalizedClientLink
                    href={`/categories/${cat.handle}`}
                    data-testid="category-link"
                    className="hover:text-[var(--text-primary)] transition-colors"
                  >
                    {cat.name}
                  </LocalizedClientLink>
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Account & Bag */}
        <div className="space-y-3 text-xs">
          <h4 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-primary)]">
            Account
          </h4>
          <ul className="space-y-2 text-[var(--text-secondary)]">
            <li>
              <LocalizedClientLink
                href="/account"
                className="hover:text-[var(--text-primary)] transition-colors"
              >
                Account Overview
              </LocalizedClientLink>
            </li>
            <li>
              <LocalizedClientLink
                href="/cart"
                className="hover:text-[var(--text-primary)] transition-colors"
              >
                Shopping Bag
              </LocalizedClientLink>
            </li>
          </ul>
        </div>
      </div>

      {/* Mobile Disclosure Accordions */}
      <div className="md:hidden divide-y divide-[var(--hairline)]">
        {/* Collections Disclosure */}
        <Disclosure as="div" className="py-3">
          {({ open }) => (
            <>
              <DisclosureButton className="flex w-full items-center justify-between text-xs font-semibold uppercase tracking-wider text-[var(--text-primary)] lustre-btn-press">
                <span>Collections</span>
                <CaretDown
                  size={14}
                  className={`transition-transform duration-200 ${open ? "rotate-180" : ""}`}
                />
              </DisclosureButton>
              <DisclosurePanel className="pt-3 pb-1 text-xs text-[var(--text-secondary)]">
                <ul className="space-y-2.5">
                  <li>
                    <LocalizedClientLink
                      href="/store"
                      className="hover:text-[var(--text-primary)] transition-colors block py-0.5"
                    >
                      All Items
                    </LocalizedClientLink>
                  </li>
                  {hasCollections &&
                    collections!.slice(0, 6).map((c) => (
                      <li key={c.id}>
                        <LocalizedClientLink
                          href={`/collections/${c.handle}`}
                          className="hover:text-[var(--text-primary)] transition-colors block py-0.5"
                        >
                          {c.title}
                        </LocalizedClientLink>
                      </li>
                    ))}
                </ul>
              </DisclosurePanel>
            </>
          )}
        </Disclosure>

        {/* Categories Disclosure */}
        {hasCategories && (
          <Disclosure as="div" className="py-3">
            {({ open }) => (
              <>
                <DisclosureButton className="flex w-full items-center justify-between text-xs font-semibold uppercase tracking-wider text-[var(--text-primary)] lustre-btn-press">
                  <span>Categories</span>
                  <CaretDown
                    size={14}
                    className={`transition-transform duration-200 ${open ? "rotate-180" : ""}`}
                  />
                </DisclosureButton>
                <DisclosurePanel className="pt-3 pb-1 text-xs text-[var(--text-secondary)]">
                  <ul className="space-y-2.5">
                    {productCategories!.slice(0, 6).map((cat) => (
                      <li key={cat.id}>
                        <LocalizedClientLink
                          href={`/categories/${cat.handle}`}
                          data-testid="category-link"
                          className="hover:text-[var(--text-primary)] transition-colors block py-0.5"
                        >
                          {cat.name}
                        </LocalizedClientLink>
                      </li>
                    ))}
                  </ul>
                </DisclosurePanel>
              </>
            )}
          </Disclosure>
        )}

        {/* Account Disclosure */}
        <Disclosure as="div" className="py-3">
          {({ open }) => (
            <>
              <DisclosureButton className="flex w-full items-center justify-between text-xs font-semibold uppercase tracking-wider text-[var(--text-primary)] lustre-btn-press">
                <span>Account</span>
                <CaretDown
                  size={14}
                  className={`transition-transform duration-200 ${open ? "rotate-180" : ""}`}
                />
              </DisclosureButton>
              <DisclosurePanel className="pt-3 pb-1 text-xs text-[var(--text-secondary)]">
                <ul className="space-y-2.5">
                  <li>
                    <LocalizedClientLink
                      href="/account"
                      className="hover:text-[var(--text-primary)] transition-colors block py-0.5"
                    >
                      Account Overview
                    </LocalizedClientLink>
                  </li>
                  <li>
                    <LocalizedClientLink
                      href="/cart"
                      className="hover:text-[var(--text-primary)] transition-colors block py-0.5"
                    >
                      Shopping Bag
                    </LocalizedClientLink>
                  </li>
                </ul>
              </DisclosurePanel>
            </>
          )}
        </Disclosure>
      </div>
    </div>
  )
}
