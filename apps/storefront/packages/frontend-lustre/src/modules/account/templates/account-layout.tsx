import React from "react"
import { HttpTypes } from "@medusajs/types"
import AccountNav from "../components/account-nav"

export type AccountLayoutProps = {
  customer: HttpTypes.StoreCustomer | null
  children: React.ReactNode
}

/**
 * Lustre AccountLayout (Slot 17).
 * Framed portal layout featuring navigation rail and customer advisory banner.
 */
const AccountLayout: React.FC<AccountLayoutProps> = ({ customer, children }) => {
  return (
    <main className="min-h-screen bg-[var(--bg-primary)] py-10 sm:py-16" data-testid="account-page">
      <div className="content-container max-w-6xl">
        <div className="grid grid-cols-1 sm:grid-cols-[260px_1fr] gap-10 lg:gap-14 items-start">
          {/* Navigation rail */}
          <aside className="sm:sticky sm:top-24">
            {customer && <AccountNav customer={customer} />}
          </aside>

          {/* Main workspace */}
          <section className="min-h-[500px] w-full">
            {children}
          </section>
        </div>

      </div>
    </main>
  )
}

export default AccountLayout
