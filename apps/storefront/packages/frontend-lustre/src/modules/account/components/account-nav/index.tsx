"use client"

import React from "react"
import { useParams, usePathname } from "next/navigation"
import { HttpTypes } from "@medusajs/types"
import { signout } from "@stheno/storefront-core"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { CaretDown, User, MapPin, Package } from "@phosphor-icons/react/dist/ssr"
import { Disclosure, DisclosureButton, DisclosurePanel } from "@headlessui/react"

export type AccountNavProps = {
  customer: HttpTypes.StoreCustomer | null
}

const AccountNav: React.FC<AccountNavProps> = ({ customer }) => {
  const route = usePathname()
  const { countryCode } = useParams() as { countryCode: string }

  const handleLogout = async () => {
    await signout(countryCode)
  }

  const navItems = [
    {
      href: "/account",
      label: "Overview",
      icon: <User size={16} />,
      exact: true,
      testId: "account-link",
    },
    {
      href: "/account/profile",
      label: "Profile",
      icon: <User size={16} />,
      testId: "profile-link",
    },
    {
      href: "/account/addresses",
      label: "Addresses",
      icon: <MapPin size={16} />,
      testId: "addresses-link",
    },
    {
      href: "/account/orders",
      label: "Orders",
      icon: <Package size={16} />,
      testId: "orders-link",
    },
  ]

  const isActive = (itemHref: string, exact?: boolean) => {
    const fullPath = `/${countryCode}${itemHref}`
    if (exact) {
      return route === fullPath
    }
    return route.startsWith(fullPath)
  }

  const activeItem = navItems.find((item) => isActive(item.href, item.exact)) || navItems[0]

  return (
    <nav aria-label="Account Navigation" className="w-full">
      {/* Mobile view with Headless UI Disclosure */}
      <div className="sm:hidden mb-6" data-testid="mobile-account-nav">
        <div className="mb-3">
          <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
            Account Portal
          </span>
          <h2 className="text-lg font-medium text-[var(--text-primary)]">
            Welcome, {customer?.first_name || "Client"}
          </h2>
        </div>

        <Disclosure as="div" className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)]">
          {({ open }) => (
            <>
              <DisclosureButton className="flex w-full items-center justify-between p-3.5 text-xs font-medium text-[var(--text-primary)] lustre-btn-press">
                <div className="flex items-center gap-2.5">
                  <span className="text-[var(--accent-sapphire)]">{activeItem.icon}</span>
                  <span>{activeItem.label}</span>
                </div>
                <CaretDown
                  size={14}
                  className={`text-[var(--text-secondary)] transition-transform duration-200 ${
                    open ? "rotate-180" : ""
                  }`}
                />
              </DisclosureButton>
              <DisclosurePanel className="border-t border-[var(--hairline)] divide-y divide-[var(--hairline)]">
                {navItems.map((item) => {
                  const active = isActive(item.href, item.exact)
                  return (
                    <LocalizedClientLink
                      key={item.href}
                      href={item.href}
                      className={`flex items-center justify-between p-3.5 text-xs font-medium transition ${
                        active
                          ? "bg-[var(--accent-sapphire-subtle)] text-[var(--accent-sapphire)] font-semibold"
                          : "text-[var(--text-primary)] hover:bg-[var(--bg-raised)]"
                      }`}
                      data-testid={item.testId}
                    >
                      <div className="flex items-center gap-2.5">
                        <span className={active ? "text-[var(--accent-sapphire)]" : "text-[var(--text-secondary)]"}>
                          {item.icon}
                        </span>
                        <span>{item.label}</span>
                      </div>
                    </LocalizedClientLink>
                  )
                })}
                <button
                  type="button"
                  onClick={handleLogout}
                  className="flex w-full items-center justify-between p-3.5 text-left text-xs font-medium text-rose-500 hover:bg-rose-500/5 transition lustre-btn-press"
                  data-testid="logout-button"
                >
                  <span>End Session</span>
                  <span className="text-xs">→</span>
                </button>
              </DisclosurePanel>
            </>
          )}
        </Disclosure>
      </div>

      {/* Desktop view */}
      <div className="hidden sm:flex flex-col space-y-1">
        <div className="mb-4 pb-3 border-b border-[var(--hairline)]">
          <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
            Account Access
          </span>
          <p className="text-sm font-medium text-[var(--text-primary)] truncate mt-0.5">
            {customer?.first_name ? `${customer.first_name} ${customer.last_name || ""}` : "Client Portal"}
          </p>
        </div>

        {navItems.map((item) => {
          const active = isActive(item.href, item.exact)
          return (
            <LocalizedClientLink
              key={item.href}
              href={item.href}
              className={`flex items-center gap-3 rounded-full px-4 py-2.5 text-xs font-medium transition lustre-btn-press ${
                active
                  ? "bg-[var(--accent-sapphire)] text-white shadow-sm font-semibold"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-surface)] hover:text-[var(--text-primary)]"
              }`}
              data-testid={item.testId}
            >
              <span className={active ? "text-white" : "text-[var(--text-secondary)]"}>
                {item.icon}
              </span>
              <span>{item.label}</span>
            </LocalizedClientLink>
          )
        })}

        <div className="pt-4 mt-4 border-t border-[var(--hairline)]">
          <button
            type="button"
            onClick={handleLogout}
            className="flex w-full items-center gap-3 rounded-full px-4 py-2.5 text-left text-xs font-medium text-[var(--text-secondary)] hover:text-rose-500 hover:bg-rose-500/10 transition lustre-btn-press"
            data-testid="logout-button"
          >
            <span>Log Out</span>
          </button>
        </div>
      </div>
    </nav>
  )
}

export default AccountNav
