"use server"

import { sdk } from "@lib/config"

/**
 * Whether this store's buyers have accounts.
 *
 * The archetype is chosen by the merchant on their store request and resolved per
 * tenant, so it cannot come from an env var: one build serves every tenant.
 *
 * Deliberately not `force-cache`. Regions can be cached because they are the same for
 * every request this build serves; the archetype is not — caching it would let one
 * tenant's answer be served to another, which is the shape of a real leak rather than a
 * stale render.
 *
 * Fails closed. If the engine cannot be reached we return false, which hides the account
 * routes rather than rendering a login form against a store that has no accounts.
 */
export const buyerAccountsEnabled = async (): Promise<boolean> => {
  return sdk.client
    .fetch<{ buyer_accounts: boolean }>(`/store/config`, {
      method: "GET",
      cache: "no-store",
    })
    .then(({ buyer_accounts }) => buyer_accounts === true)
    .catch(() => false)
}
