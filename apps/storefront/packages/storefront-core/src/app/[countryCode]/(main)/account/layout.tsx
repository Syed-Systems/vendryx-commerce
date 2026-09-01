import { retrieveCustomer } from "@lib/data/customer"
import { buyerAccountsEnabled } from "@lib/data/store-config"
import { Toaster } from "@medusajs/ui"
import AccountLayout from "@frontend/account/templates/account-layout"
import { notFound } from "next/navigation"

export default async function AccountPageLayout({
  dashboard,
  login,
}: {
  dashboard?: React.ReactNode
  login?: React.ReactNode
}) {
  // Buyers have no accounts on this archetype, so neither does the route. 404 rather
  // than a redirect or an explanation, matching what the engine does with the customer
  // routes it does not mount: a store without accounts is indistinguishable from one
  // where the route never existed (ENTITLEMENTS.md 6.1).
  //
  // Guarded here, in core, rather than in each frontend. Both frontends re-export this
  // layout, so one guard covers every theme — including the ones C2 has not built yet.
  // ponytail: guards the route, not the chrome. Account links still render in both
  // frontends' nav, footer and cart sign-in prompt, and now lead to this 404. Hiding
  // them needs the flag passed through the frontend contract, because
  // frontend-matrix's navbar is a client component and cannot await — that is a core
  // delta beyond the two FRONTEND_MATRIX 4 budgets, so it is a contract change to
  // decide, not to slip in here. Upgrade path: add `buyerAccounts` to the nav/footer
  // contract slots and drop the per-frontend links.
  if (!(await buyerAccountsEnabled())) {
    notFound()
  }

  const customer = await retrieveCustomer().catch(() => null)

  return (
    <AccountLayout customer={customer}>
      {customer ? dashboard : login}
      <Toaster />
    </AccountLayout>
  )
}
