#!/usr/bin/env node
/**
 * Storefront E2E — NOT IMPLEMENTED.
 *
 * This file previously contained twelve functions that printed a tick and returned
 * `true`. It opened no browser, made no request, and asserted nothing, yet
 * `scripts/gate-p19.sh` consumed its exit code and reported "Phase Gate P19 PASSED".
 * Every storefront journey in PHASE2_TASKS P19-02..P19-14 was recorded as verified on
 * the strength of it.
 *
 * A suite that cannot fail is worse than no suite: it converts an unknown into a
 * false positive. So this exits non-zero until real tests exist. The admin app now
 * has a working Playwright harness (`apps/admin/playwright.config.ts` and
 * `apps/admin/e2e/`) — model these on it, pointing at a storefront started against
 * one of the deployments from `scripts/dev-tiers.sh`.
 */
console.error(
  "storefront e2e is not implemented — the previous runner printed PASS without testing anything.\n" +
    "See apps/admin/e2e/ for the pattern to follow."
)
process.exit(1)
