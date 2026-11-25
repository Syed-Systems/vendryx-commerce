# System Work Log & Version History

Chronological ledger of platform increments, surface adaptations, and documentation
synchronizations. Enforced by `python scripts/doc-sync-workflow.py`.

---

### Session Log — 2026-10-25 (ADR-034 refund ambiguity safety)

**Objective**: Ensure a timed-out refund cannot be retried based on a payment-level retrieve result that does not prove refund state.

- The refund service records provider operations in the durable outbound-request ledger, but payment authorization/capture status cannot establish whether a refund was accepted.
- Unknown/in-flight `refund` operations return the explicit `provider_request_unknown` conflict and require reconciliation; they never fall through to a second provider call.
- Stripe now supplies durable-key request and provider-specific reconciliation. Other providers without a proving lookup remain fail-closed. Covered by the later Stripe refund-reconciliation session below.
- The initial Space run passed 2 tests for generic authorization reconciliation and refund fail-closed behavior; the later focused refund target passed 3 tests after adding provider reconciliation and durable replay coverage.
- Ladder: rung 2 — reused the existing payment provider request ledger and `idempotent_call` boundary.

---

### Session Log — 2026-10-25 (ADR-032 Meta webhook rate-limit route)

**Objective**: Close the missing HTTP-level rate-limit response case in the Meta webhook phase.

- Added a PostgreSQL-backed route test that submits 60 valid deliveries for one hook, asserts each accepted delivery returns 200, then verifies delivery 61 returns 429 and does not invoke the handler again.
- `pwsh -NoProfile -File scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-api-store -CargoTestTarget meta_webhooks -CargoTestServices`: PASSED (7 tests) on Space; target completed in 97.82s after the focused package build.
- The Space Cargo target cache was reused and remained under its 50 GiB cap. No local build or test was run.
- Ladder: rung 2 — exercised the existing bounded per-hook rate limiter through its mounted Axum route.

---

### Session Log — 2026-10-25 (ADR-033 billing portal component journeys)

**Objective**: Verify the billing upgrade UI fallback and keyboard accessibility, using Space for all web checks.

- Added component journeys for billing portal success and failed-portal fallback. The fallback checks modal semantics, Shift+Tab/Tab focus cycling, Escape dismissal, and restoration to the original Change plan button.
- Changed the admin Vite/Vitest configs to `.cts` and updated `tsconfig.node.json`. The Space container binds source read-only; Vite's `.mts` config loader writes a temporary file beside the config and failed with EROFS. The CJS config loader removed that write.
- The first full admin suite exposed an outdated home smoke-test expectation (`Overview`) after the route had changed; updated it to assert the current `Recent Orders` heading.
- Space's shared `build-on-space.ps1` now mounts Admin `dist` and Storefront `.next` as disposable tmpfs outputs, while keeping source read-only. Its `test-build-on-space-template.ps1` contract passed after adding those mountpoints. The web mode now uses the repository's Yarn 4 binary and hashes `apps/storefront/yarn.lock` for its cache namespace.
- `pwsh -NoProfile -File scripts/build-on-space.ps1 -Mode web-test`: all admin tests passed (28 files, 173 tests) and the admin production build completed (5,607 modules). Storefront install then failed because the modified checked-in `yarn.lock` is Yarn v1 while `package.json` pins Yarn 4.12.0; Yarn reported YN0014 for local `file:` patterns and a closed registry socket. Storefront build was not reached, so the full web-test mode remains failed.
- `pwsh -NoProfile -File scripts/build-on-space.ps1 -Mode web-admin`: PASSED; `tsc -p tsconfig.typecheck.json --noEmit` returned exit 0 on Space.
- No local build or test was run. The npm audit reported 14 dependency advisories in Space output; dependency remediation was outside this task and no audit fix was applied.
- Ladder: rung 2 — reused the existing billing portal route, modal implementation, and React Testing Library setup.

---

### Session Log — 2026-10-25 (ADR-033 quota write authorization)

**Objective**: Add server-router role coverage for a quota-bearing write, complementing the direct-handler quota-limit tests.

- Added a server integration test for `POST /admin/products`: anonymous access is rejected with 401, customer access with 403, staff access creates the product with 201, and a subsequent list confirms only one product exists.
- `pwsh -NoProfile -File scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-server -CargoTestTarget quota_write_authorization -CargoTestServices`: PASSED (1 test) on Space. The first run showed the handler returns 201 Created; corrected the test expectation and reran successfully.
- `rustfmt --edition 2021 crates/stheno-server/tests/quota_write_authorization.rs`: PASSED. Space memory preflight initially deferred container startup; no cache was cleared, and the run started after capacity recovered.
- Ladder: rung 2 — exercised the existing server authentication middleware and quota-bearing product route.

---

### Session Log — 2026-10-25 (ADR-032 authenticated courier dispatch route)

**Objective**: Close the staff draft-conversion/courier-dispatch route coverage gap with an end-to-end database-backed fixture.

- Added `social_courier_dispatch.rs` coverage for successful staff conversion and courier dispatch, stable replay with one provider call, preflight rejection that leaves an invalid-address draft unchanged, and ambiguous provider outcome blocking a second courier attempt.
- The fixture supplies the required sales channel and the mock preserves merchant reference data, matching the built-in courier adapters' response contract.
- `pwsh -NoProfile -File scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-api-admin -CargoTestTarget social_courier_dispatch -CargoTestServices`: PASSED (1 test) on Space. Two earlier runs exposed fixture issues (missing sales channel; mock response omitted the stable reference); the final run passed.
- `rustfmt --edition 2021 crates/api/stheno-api-admin/tests/social_courier_dispatch.rs`: PASSED. Scoped `git diff --check` is recorded after the documentation update.
- `python scripts/doc-sync-workflow.py check` was stopped before completion because its provider-doc step runs `cargo run` locally; doc-sync remains unverified. Its local target output was removed with `cargo clean` (641.1 MiB; 2,249 generated files); the Cargo registry was preserved. Space's Cargo cache was reused. Real courier account behavior remains an external acceptance item; the existing server auth target separately verifies missing/invalid credentials (401), customer rejection (403), and staff admission to the conversion/reconciliation route lookups.

---

### Session Log — 2026-10-24 (ADR-032 courier provider contracts)

**Objective**: Continue ADR-032 through ADR-036 implementation, completing locally verifiable phase evidence without claiming the unavailable database/Space gates.

- The focused Pathao, RedX, and Steadfast mock-backed integration suites pass all 18 tests (8, 5, and 5 respectively). The initial RedX lifecycle fixture failed because it omitted the required delivery area ID; the fixture now includes `delivery_area_id: 101`, and the rerun passes.
- Focused all-target clippy passes: `cargo clippy --locked --offline -p stheno-fulfillment-pathao -p stheno-fulfillment-redx -p stheno-fulfillment-steadfast --all-targets -- -D warnings`.
- The provider checks do not establish the staff-confirmed social-order handoff, database persistence after provider success, or ambiguous courier response recovery. Those gates remain pending an isolated Space/PostgreSQL run; the local Docker backend is still unresponsive with its temporary test listener present.
- Added the ADR-035 source metric value contract in `stheno-ai`: validated provenance, source/account/scope, metric definition and unit, collection/ingestion bounds, decimal values, coverage, and sample metadata; only complete, adjacent equal-duration observations with matching tenant/source semantics are comparable. Relative changes use exact decimal basis points and require a positive baseline. This does not implement or claim source adapters or persisted ingestion.
- `cargo test --locked --offline -p stheno-ai --lib store_intelligence_sources`: PASSED (3 tests); full `cargo test --locked --offline -p stheno-ai --lib`: PASSED (19 tests).
- `rustfmt --edition 2021 --check crates/modules/stheno-ai/src/store_intelligence_sources.rs crates/modules/stheno-ai/src/lib.rs`: PASSED. `cargo clippy --locked --offline -p stheno-ai --lib -- -D warnings`: PASSED. Scoped `git diff --check`: PASSED.
- Added the first authorized online-search source path for ADR-035: `stheno-search-console` refreshes Google OAuth credentials, queries only daily property aggregates, converts provider numbers to Decimal, records Pacific calendar windows and ingestion time, and labels missing days partial. It rejects malformed, oversized, stale-credential, permission, rate-limit, and provider-failure cases without returning provider body text.
- Search Console properties and credentials reuse tenant-bound encrypted `ai.configuration` storage. Owner/admin-only settings support manual OAuth credential configuration, fetching, and permanent disconnect; database-backed save/redaction/disconnect coverage is authored but cannot run while the local Docker/PostgreSQL backend is unresponsive. This is on-demand only: no in-product OAuth consent callback or persistent freshness/history exists yet.
- `cargo test --locked --offline -p stheno-search-console`: PASSED (3 WireMock tests). `cargo clippy --locked --offline -p stheno-search-console --all-targets -- -D warnings`: PASSED.
- `cargo test --locked --offline -p stheno-ai --lib`: PASSED (20 tests). `cargo clippy --locked --offline -p stheno-ai --all-targets -- -D warnings`: PASSED; this compiles the authored DB-backed credential/disconnect test without executing it.
- `cargo check --locked --offline -p stheno-api-admin`: PASSED. `cargo clippy --locked --offline -p stheno-api-admin --all-targets -- -D warnings`: PASSED. `npm --prefix apps/admin run typecheck`: PASSED.
- Focused `rustfmt --check` passes for the new adapter, AI configuration implementation, metric contract, and admin API files. `rustfmt --edition 2021 --check crates/modules/stheno-ai/tests/configuration_secrets.rs` still reports formatting differences in the pre-existing `unsupported_fields_and_incomplete_enabled_configs_fail_closed` assertions; the new Search Console assertions match rustfmt and the unrelated existing block was left unchanged. `git diff --check` and `python scripts/doc-sync-workflow.py check` pass.

---

### Session Log — 2026-10-23 (ADR-036 isolated manual build account)

**Objective**: Complete isolated manual builds on Space without keeping Docker resident while idle.

- Provisioned the separate `syed-build` account (UID 1001), verified its subordinate UID/GID ranges, installed `uidmap`, and enabled user lingering so its user service manager remains reachable across the builder's separate SSH sessions. The rootless Docker service is disabled for automatic startup; it runs under `syed-build` only while the build wrapper needs it.
- Store's `scripts/build-on-space.ps1` now selects `syed-build`. Space's shared builder accepts a remote user, starts rootless Docker before isolation/capacity checks, and stops it in `finally`. Rust package gates default to one CPU/job, a 3 GiB container memory cap, no Postgres/Redis, and a 3.5 GiB available-memory preflight. Package tests that require services can opt in explicitly.
- A targeted build passed the host isolation and capacity checks, then exposed a stale `Cargo.lock` edge for the newly added `chrono` dependency; the scoped lock entry was corrected. The 2 GiB cold compile was killed while compiling Diesel (rustc reached approximately 2.4 GiB RSS). A 3 GiB retry compiled Diesel and the dependency graph until it exposed three `String`/`&str` comparison errors in the modified fulfillment service; those comparisons now borrow `reference.as_str()`.
- The latest focused retry passed host isolation but stopped before container startup because an active Android/Java build on Space reduced `MemAvailable` to 1.6 GiB, below the 3.5 GiB package-test guard. No test result is claimed. Docker was confirmed inactive after the failed run; the rootless service has its own cache namespace and created a pinned Rust image and package caches.
- Focused Space script-template, PowerShell parse, and capacity fixtures passed. The package test, including the `entitlements` integration target, remains pending until concurrent host activity releases enough memory. No runner-group or CI workflow changes were made.

---

### Session Log — 2026-10-23 (ADR-033 billing state and grace UI)

**Objective**: Continue ADR-032 through ADR-036 implementation without using the marketing branch; use the configured Space host for scoped builds.

- `GET /admin/billing/subscription` now reports persisted subscription status rather than hardcoding `active`, returns the effective trial and paid-term dates, and derives the seven-day grace end from the paid-term end.
- The admin subscription view now distinguishes subscription grace from over-quota state, shows a minute-updated days-remaining counter and exact local-time deadline, reads the API's actual date field names, and removes unsupported claims about auto-renewal and a saved bKash/SSLCommerz payment method. The displayed default Pro rate now matches the accepted USD configuration.
- Added a focused API contract for the missing-subscription case (`status: unknown`) so the UI does not report a nonexistent subscription as active.
- Branch remains `codex/build-server-adr`. The configured `space` SSH connection works, but its account (`hunterlog2`, UID 1000) is the same as the organization Actions runner and its Docker daemon is rootful. The build-host isolation preflight correctly fails closed; this session cannot use `sudo` non-interactively, so no remote host settings were changed and no Space build/test result is claimed.
- Checks: scoped `git diff --check` passed. `rustfmt --edition 2021 --check crates/api/stheno-api-admin-platform/src/lib.rs` reports formatting drift in pre-existing blocks elsewhere in the already-modified file; the new subscription response block is formatted, and unrelated blocks were left untouched. The focused API test and admin typecheck are pending the isolated Space build path.

---

### [v2.29.23] — 2026-10-23 (ADR-032 courier preflight ordering)

**Summary:** Validate the social-order courier dispatch plan before conversion can reserve inventory.

**Modified Surfaces:**
- Staff confirmation now validates courier settings, the selected provider and shipping option, destination mapping, buyer address, COD amount, and fulfillment items against the draft before converting it. The draft ID remains the stable idempotency reference; the actual provider handoff still runs only after conversion succeeds.
- Expanded the chat-offer contract with overlapping enabled-offer rejection, quantity limits, currency precision, and inclusive-start/exclusive-end schedule boundary cases.
- Updated the ADR-032 phase plan with this ordering guarantee and the remaining focused regression and role-matrix gates.
- Corrected the brand-kit documentation link to the existing `branding/Vendryx_Brand_Kit/` source and aligned its asset inventory with that kit.

**Quality Gates:**
- `rustfmt --edition 2021 --check crates/api/stheno-api-admin/src/orders.rs`: PASSED.
- `rustfmt --edition 2021 --check crates/modules/stheno-ai/tests/chat_offer_contract.rs`: PASSED.
- `git diff --check -- crates/api/stheno-api-admin/src/orders.rs docs/ADR_IMPLEMENTATION_PLAN.md docs/BRAND.md docs/WORK_LOG.md`: PASSED.
- Brand-kit paths referenced by `docs/BRAND.md`: PASSED (checked against the working tree).
- `scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-api-admin -CargoTestTarget lib`: BLOCKED before source packaging; SSH host `space` reports runner/build-service UID `1000`. No Cargo compilation or test started.
- Added a focused admin API regression that checks invalid courier preflight leaves the social draft unconverted. Its execution is blocked by the same Space UID preflight.
- Focused `stheno-ai` offer contract execution is also pending the isolated Space runner; neither Cargo target was compiled locally.

---

### [v2.29.22] — 2026-10-23 (ADR-033 lifecycle and ADR-034 payment hardening)

**Summary:** Align new trial provisioning and trial lifecycle behavior with ADR-033 while retaining already-issued expiry timestamps.

**Modified Surfaces:**
- The shipped trial duration is seven days. Application startup reads the duration and start trigger from `config/tiers.toml`, validates the duration range and accepted submission trigger, and passes it to launch provisioning.
- New trial timestamps begin at the persisted launch-request submission time. Existing subscription expiry timestamps are not recalculated when configuration changes.
- Request-time entitlement resolution now checks persisted trial/term dates in UTC. Trial access ends at the exact `trial_ends_at` instant; a trial missing its end date fails closed to Free. Paid Pro access remains during the seven-day grace and resolves to Free at the exact grace boundary. The trial tier now grants the ADR-defined Pro trial feature set.
- A paginated 15-minute platform reconciliation persists trial expiry, enters `grace` at a paid-term boundary, then persists the Free downgrade; once Free is stored, a backward clock adjustment cannot restore Pro. The launch integration contract covers exact boundaries, replay, grace transitions, and backward-clock idempotence. Payment dunning now skips an expired contract term so its own ladder cannot shorten the seven-day grace.
- Payment dunning no longer touches Free or Trial subscriptions, and paid-term expiry clears stale retry state when downgrading. Focused contracts cover both non-billed tiers and preservation of the complete contract grace period.
- The trial lifecycle helper evaluates the stored expiry instant in UTC, keeps transitions monotonic across clock skew, and supports persisted legacy fourteen-day end dates.
- Dunning notification intents now use email and WhatsApp, removing the SMS channel that contradicted ADR-034.
- Manual invoice-proof submission validates the payment method, transaction ID, and proof reference; it refuses an already-paid invoice or a second proof while review is pending. A partial unique index closes the concurrent duplicate-proof race; a database trigger prevents a racing paid invoice from reverting to pending verification. The migration refuses to proceed if legacy data already violates the uniqueness invariant. Focused database contracts are authored.
- Updated public marketing, admin, storefront, and entitlement documentation copy to seven days. Lifecycle, configured override, and legacy-term regression tests are authored.
- Storefront cart IDs are now AES-256-GCM encrypted in the existing seven-day HttpOnly/SameSite cookie, authenticated with `COOKIE_SECRET`, and decryptable with `COOKIE_SECRET_PREVIOUS` during rotation. Legacy IDs are accepted only when they match the bounded Medusa ID alphabet; invalid ciphertext fails closed. Focused Vitest contracts cover round trip, nonce freshness, wrong key, tampering, malformed payload, weak key, and invalid ID.

**Quality Gates:**
- `npm --prefix apps/marketing run typecheck`, `npm --prefix apps/storefront run typecheck`, and `npm --prefix apps/admin run typecheck`: PASSED.
- Rust formatting check for `tenant_entitlements.rs`, the API/server `lib.rs` files, `tenant_layer.rs`, `stheno-platform/src/service.rs`, and focused platform tests: PASSED. Five trailing-space-only lines in the payment-proof section were removed while touching that code; no unrelated service formatting was changed.
- `config/tiers.toml` parse, `cargo metadata --no-deps --format-version 1`, and scoped `git diff --check`: PASSED.
- `npm --prefix apps/storefront run test -- tests/unit/cart-cookie-crypto.test.ts`: PASSED (3 focused tests). `npm --prefix apps/storefront run typecheck`: PASSED.
- `scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-api-core -CargoTestTarget lib`: BLOCKED before packaging because the Actions runner and build service both run as UID `1000`; no local or workspace-wide Cargo build was started.
- `scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-platform -CargoTestTarget launch_requests`: BLOCKED by the same UID preflight; Cargo did not start. `git diff --check` on the scoped ADR-033/034 files: PASSED. Whole-repository doc-sync remains blocked by pre-existing deleted `brand/kit/` assets linked from `docs/BRAND.md`.
- `scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-platform -CargoTestTarget payment_proof_truth`: BLOCKED by the same UID preflight; Cargo did not start.
- `python scripts/doc-sync-workflow.py check`: 4/5 checks PASSED; the link check fails because the pre-existing deleted `brand/kit/` directory is referenced by `docs/BRAND.md`.

---
### [v2.29.21] — 2026-10-23 (ADR-032 confirmation replay and authorization)

**Summary:** Align duplicate social confirmation with the idempotent courier handoff and cover the
server admin-auth boundary for conversion requests.

**Modified Surfaces:**
- Replayed confirmation of an already converted social draft now has an explicit flow test that
  expects the same pending order, allowing the downstream order-keyed fulfillment ledger to return
  the prior dispatch result.
- Added a server-router integration test proving missing/invalid credentials receive 401 and a valid customer credential receives 403
  before the social draft conversion handler runs.
- Added a focused `stheno-ai` contract test for disabled-by-default offers, explicit enablement, exact quote pricing, conversation/event binding, same-event replay, cross-event replay rejection, conversation mismatch, expiry rejection, and immediate invalidation after owner disable.
- Chat-offer dashboard creation now defaults disabled. Editing preserves the current enabled state,
  and owners must explicitly select activation; updating a disabled offer cannot silently turn it on.
- Updated Space `rust-package` so tests with a named integration target start the same bounded
  PostgreSQL/Redis service containers and receive `DATABASE_URL`/`REDIS_URL`; package checks without
  an integration target still avoid those services. A named `lib` target selects Rust unit tests,
  while other target names select one integration test; each path performs one compile/test command.

**Quality Gates:**
- Rust formatting, PowerShell parsing, embedded Bash syntax, the Space build-template fixture, and `npm --prefix apps/admin run typecheck` pass.
- Focused Space Cargo execution was attempted with `scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-ai -CargoTestTarget chat_offer_contract`; Space refused before packaging because the Actions runner and build service both run as UID `1000`. No local or workspace-wide Cargo build was started.
- `python scripts/doc-sync-workflow.py check` fails on a pre-existing missing `brand/kit/` target linked from `docs/BRAND.md`; those brand assets are already deleted in the worktree and are outside this ADR-032 change.

---
### [v2.29.20] — 2026-10-23 (ADR-032 dispatch recovery and scoped Space gates)

**Summary:** Close the final local child-write retry case in courier dispatch and give Space a
package-scoped Rust gate for the ADR-032 acceptance checks.

**Modified Surfaces:**
- Added focused recovery coverage for a courier label returned successfully but not persisted
  locally; replay is expected to restore the label from the idempotency ledger without another
  courier call.
- Provider credential redaction now recursively strips secret-bearing fields before both durable
  response persistence and request hashing, including nested and separator/case variants.
- Added `rust-package` to the Space build entrypoint and Store wrapper. It checks one Cargo package
  and can run one named integration-test target with the shared pinned image and Cargo caches.
  Existing host UID isolation and capacity gates still run before any build container starts.

**Quality Gates:**
- PowerShell parser: PASSED for the Store wrapper and Space build entrypoint.
- Scoped `rustfmt --edition 2024 --check` on the changed Rust files: PASSED.
- `cargo metadata --no-deps --format-version 1`: PASSED for selected package metadata.
- `git diff --check` on the scoped implementation/docs/migration files: PASSED (Git reports
  line-ending conversion notices only).
- `scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-fulfillment
  -CargoTestTarget distinct_shipments_are_not_deduped`: BLOCKED before build setup because Space
  reports Actions runner and build service both use UID `1000`. No Cargo build/test started.

### [v2.29.18] — 2026-10-23 (ADR-032 courier credential hardening)

**Summary:** Continue ADR-032 phase 32.5 by securing tenant courier credentials before wiring
staff-confirmed dispatch.

**Modified Surfaces:**
- Courier credential settings now reuse the existing AES-GCM envelope, binding ciphertext to the
  active tenant and courier provider. Existing plaintext records are encrypted on first access.
- Courier list responses fully mask secret values; credential writes reject unknown fields,
  invalid value types, and enabling an incomplete account. Probes decrypt only in the server and
  fail closed when the tenant key or stored envelope is invalid.
- Pathao, Steadfast, and RedX no longer ship with sample credentials. Shipment creation rejects
  absent recipient data, invoice references, and courier destination IDs, and no longer converts
  provider failures or missing IDs into fabricated tracking records. Pathao destination lookup
  errors no longer fall back to hard-coded sample city, zone, or area mappings.
- Added `create_fulfillment_idempotent` with tenant/provider-scoped deterministic fulfillment IDs,
  full-input payload binding, replay lookup of the local record, and rejection when a stable key is
  reused for changed shipment data.
- Added focused envelope encoding/decoding unit tests. Dispatch remains intentionally unwired until
  provider-specific credential resolution, tenant destination configuration, and the social staff
  confirmation hook are integrated.

**Quality Gates:**
- `rustfmt --edition 2024 crates/api/stheno-api-admin/src/couriers.rs`: PASSED.
- Focused courier envelope and provider ambiguity tests were added but not run; the Space build
  wrapper currently offers only workspace-wide Cargo build modes, which conflict with the
  repository's feature-scoped test rule. Direct builds are also guarded by the Space
  runner/build-service UID isolation check.
- `git diff --check`: BLOCKED by a trailing-space line in the pre-existing working diff for
  `docs/DECISIONS.md`; the focused implementation files were clean.

### [v2.29.19] — 2026-10-23 (ADR-032 staff-confirm courier dispatch)

**Summary:** Wire the courier handoff into human confirmation and keep tenant secrets out of
fulfillment persistence.

**Modified Surfaces:**
- The production admin order route now dispatches the selected Pathao, Steadfast, or RedX
  provider only after the social draft has passed its confirmation checks and conversion/reservation.
- Credentials are decrypted from tenant-scoped encrypted settings for that call; the fulfillment
  service supplies them ephemerally, strips secret fields from provider responses, and hashes only
  non-secret request details into its durable idempotency ledger.
- Owner courier settings now require Pathao store ID and explicit Pathao/RedX destination mappings.
  Mappings are keyed by country, province, and city so same-named cities cannot silently route to
  another location. RedX also requires a configured area name and parcel weight.
- Confirmation rejects unsupported countries/providers, disabled couriers, missing buyer/address
  fields, invalid money conversion, unmapped destinations, malformed mappings, and non-shippable
  or non-positive order lines. Repeated confirmation uses an order-derived stable key.
- Fulfillment retries resume deterministic address, fulfillment, item, and label records after
  partial local persistence. Focused tests cover malformed destination maps, credential redaction,
  recovery after address/fulfillment/item write failures, and holding an ambiguous provider result
  without issuing a second consignment. A stable order key is bound across provider IDs, and a
  tenant-scoped reference index prevents courier changes from opening a second provider namespace.

**Quality Gates:**
- `rustfmt --edition 2024` on changed Rust files: PASSED; `cargo metadata --no-deps --format-version 1`: PASSED.
- Focused Rust compile and tests were not run. Space's current wrapper exposes only workspace-wide
  Cargo modes, and its runner/build-service UID isolation guard must remain in force. No full
  workspace build was started.
- Documentation sync was not run. `git diff --check` is still known to hit an unrelated trailing
  whitespace line in the existing `docs/DECISIONS.md` diff.

### [v2.29.17] — 2026-10-23 04:42 UTC

**Summary:** Complete ADR-032 phase 32.3 with a bounded DeepSeek/Qwen-compatible provider and
read-only conversation orchestration.

**Modified Surfaces:**
- **Provider contracts:** `stheno-ai` now defines normalized chat messages, usage, tool schemas,
  responses, and the async provider boundary.
- **Provider adapter:** new `stheno-ai-openai-compatible` crate restricts production endpoints to
  approved HTTPS hosts, pins the configured model, bounds bodies and output tokens, uses a
  ten-second timeout, normalizes refusals, avoids retries, and omits response bodies from errors.
- **Conversation engine:** limits history, knowledge, request bytes, model/tool call counts, and
  reported token usage; frames user/merchant/product text as untrusted; exposes only four typed
  read-only commerce tools; rejects malformed and unsupported calls without executing them.
- **Phase ledger:** 32.3 complete; 32.4 Meta ingress and 32.5 admin/human confirmation remain.

**Quality Gates Verified:**
- `cargo test -p stheno-ai-openai-compatible`: PASSED (6 wiremock contract tests).
- `cargo test -p stheno-flows --lib ai_conversation::tests`: PASSED (5 orchestration tests).
- `cargo test -p stheno-flows --lib ai_tools::tests`: PASSED (7 typed tool tests).
- `cargo clippy -p stheno-ai-openai-compatible --all-targets -- -D warnings`: PASSED.
- `cargo clippy -p stheno-flows --lib -- -D warnings`: PASSED.
- `cargo fmt -p stheno-ai -p stheno-ai-openai-compatible -p stheno-flows -- --check`: PASSED.

Full workspace gates remain pending. Documentation sync still fails only because the user-deleted
`brand/kit/` directory is referenced by `docs/BRAND.md`.

---

### [v2.29.16] — 2026-10-23 04:42 UTC

**Summary:** Complete ADR-032 phase 32.2 with bounded read-only catalog, offer, and shipping
tools, and fail-closed shipping eligibility.

**Modified Surfaces:**
- **Live offer contract:** the tool now has a database test proving it reads a configured merchant
  price, hides drafts, and preserves unavailable stock as unknown.
- **Shipping quote tool:** validates country/currency codes, destination text, count, weight, and
  region; checks region currency; returns sorted current eligible rates with a bounded result and
  explicit truncation indicator. It has no write-capable service dependency.
- **Fulfillment eligibility:** country, province, city, and postal constraints are checked; missing
  destination data, unknown geo-zone kinds, and unsupported option rules fail closed. Missing or
  malformed flat amounts are errors instead of silently becoming zero.
- **ADR tracking:** phase 32.2 marked complete with focused evidence; phases 32.3–32.5 remain.

**Quality Gates Verified:**
- `cargo test -p stheno-flows --lib ai_tools::tests`: PASSED (6 tests).
- `cargo test -p stheno-flows --test ai_product_offer`: PASSED (1 live PostgreSQL contract test).
- `cargo test -p stheno-flows --test ai_shipping_quote`: PASSED (1 live PostgreSQL contract test).
- `cargo test -p stheno-fulfillment --test shipping_eligibility_fail_closed`: PASSED (1 live PostgreSQL contract test).
- `cargo clippy -p stheno-flows --lib -- -D warnings`: PASSED.
- `cargo clippy -p stheno-fulfillment --lib -- -D warnings`: PASSED.
- `cargo fmt -p stheno-flows -p stheno-fulfillment -- --check`: rerun after final formatting below.

Full workspace gates remain pending. The previously recorded doc-sync check still reports absent
`brand/kit/` assets linked by `docs/BRAND.md`; those user-owned deletions were not restored.

---

### [v2.29.15] — 2026-10-23 04:17 UTC

**Summary:** Close the OKF's NUL-character validation gap and continue ADR-032 phase 32.2 with
typed, read-only product offer and catalog search tools.

**Modified Surfaces:**
- **OKF validation:** reject embedded NUL in titles, categories, content, and search queries as
  caller validation errors instead of leaking a PostgreSQL text error.
- **AI commerce tool:** compose existing product, pricing, inventory, and region services behind
  `ProductOfferTool`; bound malformed input and quantity, reject unpublished products and
  currency/region mismatch, return only service-resolved price and availability, and apply a
  two-second overall timeout. The tool has no write-capable dependency.
- **Live catalog search:** add bounded, literal title-prefix lookup through the tenant-scoped
  product repository; only published products are returned, with wildcard input rejected and an
  eight-result cap. No search-index price fields are consumed.
- **ADR status and phase plan:** record that 32.1 is complete and 32.2 has started; the full
  ADR remains incomplete.

**Quality Gates Verified:**
- `cargo test -p stheno-ai`: PASSED (2 unit tests, 1 Postgres isolation/retrieval integration test).
- `cargo test -p stheno-product --test ai_catalog_search`: PASSED (Postgres search-boundary integration test).
- `cargo test -p stheno-flows --lib ai_tools::tests::rejects_malformed_or_unbounded_tool_arguments`: PASSED.
- `cargo fmt -p stheno-ai -p stheno-flows -- --check`: PASSED.
- `cargo clippy -p stheno-ai -p stheno-product -p stheno-flows --lib -- -D warnings`: PASSED.
- Module isolation, SQL boundary, module contract, and schema drift scripts: PASSED.
- `python scripts/doc-sync-workflow.py check`: FAILED because `brand/kit/` is currently absent while `docs/BRAND.md` links to it.
- `cargo test -p stheno-flows ai_tools::tests::rejects_malformed_or_unbounded_tool_arguments`: BLOCKED by pre-existing compile errors in `tests/platform_workflows.rs` (missing `is_private` fields in two input initializers).

The live-service contract tests, catalog search and shipping tools, provider/orchestrator, Meta
webhooks, human-confirmed admin handoff, and `/admin/ai` remain outstanding in ADR-032.

---

### [v2.29.14] — 2026-10-23 03:56 UTC

**Summary:** Implement ADR-032 phase 32.1 as a dedicated tenant-isolated knowledge module for
merchant-authored static policy and FAQ content.

**Modified Surfaces:**
- **`stheno-ai` module**: allowlisted DTOs, validated create/update inputs, bounded list and
  multilingual lexical search, deterministic ranking, duplicate protection, and soft deletion.
- **`ai.knowledge_entry` migration**: forced tenant RLS, generated `tsvector`, GIN search index,
  and active title/category uniqueness.
- **Architecture inventory**: module, schema, and entity counts synchronized.
- **ADR implementation plan**: phase 32.1 evidence recorded; AI provider, tools, webhooks, and
  admin configuration remain incomplete.

**Quality Gates Verified:**
- `cargo fmt -p stheno-ai -- --check`: PASSED
- `cargo test -p stheno-ai`: PASSED (2 unit tests, 1 Postgres RLS/retrieval integration test)
- `cargo clippy -p stheno-ai --all-targets -- -D warnings`: PASSED
- `scripts/check-module-isolation.ps1`: PASSED
- `scripts/check-sql-boundaries.ps1`: PASSED
- `scripts/check-module-contracts.ps1`: PASSED
- `scripts/check-schema-drift.ps1`: PASSED

The service deliberately accepts static merchant-authored content only. Future conversation
orchestration must treat retrieved content as untrusted and must never mix customer messages,
prices, stock, or other live commerce data into this table.

---

### [v2.29.13] — 2026-10-23 03:37 UTC

**Summary:** Establish a phased implementation plan for ADR-032 through ADR-036, and align the
existing social-order reference code with ADR-032's `VX-XXXX` reference format.

**Modified Surfaces:**
- **ADR implementation plan** (`docs/ADR_IMPLEMENTATION_PLAN.md`): ordered phases, dependencies,
  relevant failure cases, and acceptance evidence for ADRs 032–036.
- **ADR-032 social references**: generated codes now carry the fixed `VX-` prefix; storefront
  acceptance checks match the response format.

**Quality Gates Verified:**
- `cargo test -p stheno-flows-order social_message`: PASSED (4 passed, 0 failed).
- Storefront E2E was not run; the full remote/manual checkout environment is not part of this
  focused reference-format change.
- Full workspace gates were not run. ADR-032's OKF, AI provider, webhook, and admin configuration
  phases remain outstanding and are not claimed complete.

---

### [v2.29.12] — 2026-10-23 02:56 UTC

**Summary:** ADR-036 parity slice: add a Space `web-test` mode that mirrors Store's hosted Admin and
Storefront test/build commands with the isolated, lockfile-keyed npm cache path. Run safe scoped
orphan and snapshot cleanup before capacity rejection so stale build data can free required disk.

**Modified Surfaces:**
- **ADR-036**
- **Space web-test builder mode**
- **Space capacity guard and fixture**

**Quality Gates Verified:**
- `bash -n scripts/check-build-host-capacity.sh scripts/test-check-build-host-capacity.sh`: PASSED
- `scripts/test-check-build-host-capacity.sh`: PASSED (run with isolated WSL HOME; includes `web-test`)
- `Store scripts/build-on-space.ps1 PowerShell parser`: PASSED
- `Space build-on-space.ps1 PowerShell parser`: PASSED
- `pwsh -NoProfile -File scripts/test-build-on-space-template.ps1`: PASSED
- Embedded Bash fixture validates isolation → scoped cleanup → capacity check → source packaging order.
- `pwsh -NoProfile -File scripts/build-on-space.ps1 -Mode web-test`: safely rejected by the live
  separate-UID preflight before source packaging or cache setup
- `python scripts/doc-sync-workflow.py check`: FAILED on the existing missing `brand/kit/` target
- `git diff --check -- docs/DECISIONS.md docs/WORK_LOG.md`: FAILED on pre-existing trailing
  whitespace at `docs/DECISIONS.md:2190`

The new web-test commands have not run remotely; current runner/daemon isolation still rejects the
host before source packaging. Existing hosted parity failures remain unresolved and are not counted
as passing gates.

---

### [v2.29.11] — 2026-10-22 19:36 UTC

**Summary:** ADR-036 operations phase: measure persistent Cargo/npm cache usage before and after gates, enforce per-volume ceilings, and check disk, inode, and memory headroom before container startup.

**Modified Surfaces:**
- **ADR-036**
- **Space cache-size guard**
- **Space builder**
- **Space infrastructure docs**
- **SPACE-070**
- **Legacy cache volume inventory** (labels inspected; no rootful volume measured, relabeled, or removed)

**Quality Gates Verified:**
- `scripts/test-check-build-host-capacity.sh`: PASSED (Rust/web thresholds and disk/inode/memory failure paths)
- `scripts/test-check-build-cache-sizes.sh`: PASSED (Rust/npm ceilings, exact boundaries, over-limit rejection, measurement failure, and symlink rejection)
- `bash -n scripts/check-build-host-capacity.sh scripts/test-check-build-host-capacity.sh`: PASSED
- `bash -n scripts/check-build-cache-sizes.sh scripts/test-check-build-cache-sizes.sh`: PASSED
- `pwsh -NoProfile -File scripts/test-build-on-space-template.ps1`: PASSED
- `Space build-on-space.ps1 PowerShell parser`: PASSED
- `Store CI workflow YAML parse`: PASSED (run during the preceding ADR-036 verification)
- `legacy Docker volume ownership inspection`: PASSED (six legacy Store/Space volumes use the old manager label and have no project label)

---


### [v2.29.10] — 2026-10-22 19:31 UTC

**Summary:** ADR-036 operations phase: add dry-run-first, host-lock-protected recovery for stale labeled build containers and unused networks; invoke it only after the isolation guard and verify all fixtures without mutating the live daemon.

**Modified Surfaces:**
- **ADR-036**
- **Space build resource recovery**
- **Space builder**
- **Space infrastructure docs**
- **SPACE-070**

**Quality Gates Verified:**
- `build-resource recovery fixture`: PASSED
- `cache-volume fixture`: PASSED
- `cache-reset fixture`: PASSED
- `host-isolation fixture`: PASSED
- `workspace-retention fixture (isolated HOME)`: PASSED
- `Space Bash syntax`: PASSED
- `Space PowerShell parser`: PASSED
- `Store builder isolation rejection before source packaging`: PASSED

---


### [v2.29.9] — 2026-10-22 19:22 UTC

**Summary:** ADR-036 web static slice: pin Node 24.22.1/npm 11.19.1, enforce reviewed npm install scripts, add lockfile-keyed Space npm caching, and record the hosted and local parity failures that still block promotion.

**Modified Surfaces:**
- **ADR-036**
- **Store CI workflow**
- **Admin and Storefront npm policy**
- **Space web-static builder**
- **Space cache and isolation fixtures**

**Quality Gates Verified:**
- `strict npm ci (Admin and Storefront)`: PASSED
- `Admin typecheck`: PASSED
- `Admin production build`: PASSED
- `workflow YAML parse`: PASSED
- `Space cache-volume fixture`: PASSED
- `Space cache-reset fixture`: PASSED
- `Space host-isolation fixture`: PASSED
- `Space workspace-retention fixture (isolated HOME)`: PASSED
- `Space Bash syntax`: PASSED
- `Space PowerShell parser`: PASSED

**Parity blockers observed (not counted as passing gates):** Admin `npm test` finished with 167/168
tests passing; the failing route-smoke assertion expects an “Overview” heading absent from the current
home page. Storefront `npm run build` fails before compilation because `scripts/frontend.js` is
missing. Its root TypeScript check passes, but the workflow's `tsconfig.*.json` loop has no matching
theme config files in this checkout. The Space build-host invocation remains blocked before source
packaging by the shared runner/build-account UID check.

---


### [v2.29.8] — 2026-10-22 18:58 UTC

**Summary:** ADR-036 trust boundary hardening: the manual builder now rejects the shared host before source packaging; add a rootless-daemon and separate-UID preflight with failure fixtures. Confirm the existing runner's active LogBook Android build and document the unassigned Store/Space runner group.

**Modified Surfaces:**
- **ADR-036**
- **Space build host isolation**
- **Space runner group**
- **Space builder**

**Quality Gates Verified:**
- `workspace-retention fixture`: PASSED
- `cache-volume fixture`: PASSED
- `cache-reset fixture`: PASSED
- `build-host-isolation fixture`: PASSED
- `Bash syntax`: PASSED
- `PowerShell parser`: PASSED
- `live builder preflight rejection`: PASSED

---


### [v2.29.7] — 2026-10-22 18:54 UTC

**Summary:** ADR-036: create a selected-repository runner group for Store and Space while preserving hosted CI; document that the existing busy shared runner has not been moved. Add scoped, lock-protected cache-volume recovery and fixtures.

**Modified Surfaces:**
- **ADR-036**
- **Space runner isolation**
- **Space cache recovery**
- **build host operations docs**

**Quality Gates Verified:**
- `Space cache-reset fixture`: PASSED
- `cache-volume fixture`: PASSED
- `workspace-retention fixture`: PASSED
- `Bash syntax checks`: PASSED

---


### [v2.29.6] — 2026-10-22 18:52 UTC

**Summary:** ADR-036 cache recovery: add a dry-run-first, lock-protected, exact-scope cache reset command with ownership validation and mocked-Docker edge-case fixture.

**Modified Surfaces:**
- **ADR-036**
- **Space cache recovery**
- **build host operations docs**

**Quality Gates Verified:**
- `Space cache-reset fixture`: PASSED
- `Bash syntax check`: PASSED

---


### [v2.29.5] — 2026-10-22 18:50 UTC

**Summary:** ADR-036 cache isolation hardening: cache volumes now require matching purpose, builder, and project labels; mocked-Docker fixture covers cross-manager and cross-project rejection.

**Modified Surfaces:**
- **ADR-036**
- **Space build cache provisioning**
- **build host documentation**

**Quality Gates Verified:**
- `Space cache-volume fixture`: PASSED
- `Bash syntax check`: PASSED

---


### [v2.29.4] — 2026-10-22 18:45 UTC

**Summary:** ADR-036 Phase 1: the Space builder now cold-creates only missing exact cache volumes with ownership labels and rejects foreign or unverifiable volumes. Mocked-Docker fixture, cleanup retention fixture, PowerShell parser, and Bash syntax checks pass. Full host rust-ci remains unrun while the shared runner is busy.

**Modified Surfaces:**
- **ADR-036**
- **Space build scripts**
- **build host operations docs**

**Quality Gates Verified:**
- `cache-volume fixture`: PASSED
- `cleanup retention fixture`: PASSED
- `PowerShell parser`: PASSED
- `Bash syntax`: PASSED

---


### [v2.29.3] — 2026-10-22 18:44 UTC

**Summary:** ADR-036: add fail-closed cold-cache provisioning and scoped snapshot retention with isolated fixture checks. Full remote rust-ci, runner isolation, cache corruption recovery, volume limits, and orphan-resource cleanup remain pending; docs sync still fails on the missing brand/kit target.

**Modified Surfaces:**
- **ADR-036**
- **Space build scripts**
- **build host operations docs**

**Quality Gates Verified:**
- `cleanup retention fixture`: PASSED
- `cache-volume fixture`: PASSED
- `PowerShell parser`: PASSED
- `Bash syntax`: PASSED

---


### [v2.29.2] — 2026-10-22 18:41 UTC

**Summary:** ADR-036: add unique build snapshots and tested scoped retention (7 days or newest 20, skipping active builds); cleanup helper fixture passes. Full rust-ci, orphan-resource recovery, cache limits, and runner isolation remain pending; docs sync still reports the existing missing brand/kit link.

**Modified Surfaces:**
- **ADR-036**
- **Space build scripts**
- **build host operations docs**

**Quality Gates Verified:**
- `cleanup retention fixture`: PASSED
- `PowerShell parser`: PASSED
- `Bash syntax`: PASSED

---


### [v2.29.1] — 2026-10-22 18:32 UTC

**Summary:** ADR-036 Phase 1 hardening: isolate Store and Space cache namespaces by repository toolchain target and profile; add Store rust-ci builder mode with private service containers cleanup limits and bounded daemon calls. Full Rust CI execution remains pending because the shared host runner is occupied and Docker became unresponsive.

**Modified Surfaces:**
- **docs/DECISIONS.md**
- **docs/WORK_LOG.md**
- **scripts/build-on-space.ps1**
- **Space build host scripts**

**Quality Gates Verified:**
- `Isolated Store cold cargo check exit 0 in 4m36s; isolated warm cargo check exit 0 in 5m33s; cache volumes provisioned with ownership labels; PowerShell parser and Bash syntax checks passed`: PASSED

---


### [v2.29.0] — 2026-10-22 17:44 UTC

**Summary:** ADR-036 Phase 1: validate the Store workspace through Space's pinned Rust build image and record cold/warm cache evidence; document the staged runner trust boundary and edge-case plan.

**Modified Surfaces:**
- **docs/DECISIONS.md**
- **scripts/build-on-space.ps1**
- **docs/WORK_LOG.md**

**Quality Gates Verified:**
- `pwsh -NoProfile -File scripts/build-on-space.ps1 (cold cargo check 5m47s; warm cargo checks 2m21s and 2m50s; warm end-to-end 5m00s and 5m14s)`: PASSED

---


### [v1.0.4] — 2026-10-10 15:35 UTC

**Summary:** Repository declutter, ADR-031 alignment, and developer documentation suite

**Modified Surfaces:**
- **Documentation**
- **Storefront**
- **Admin**
- **Marketing**
- **Platform**
- **Engine**

**Quality Gates Verified:**
- `doc-sync-workflow check`: PASSED
- `check-brand-surface`: PASSED
- `check-no-mock-storage`: PASSED
- `storefront typecheck`: PASSED

---


### [v2.28.0] — 2026-09-09 01:35 UTC

**Summary:** Formalized and accepted ADR-031 ("Dynamic Multi-Tenant Visual Theming on a Unified Default Storefront Runtime"). Established the single-server Next.js storefront architecture that serves multi-tenant branded storefronts from the battle-tested, mature `frontend-default` DOM foundation without multi-server deployment sprawl. Defined dynamic runtime design token injection (CSS custom properties) for colors, typography, corner radii, and materials in `layout.tsx`, accompanied by component visual variant modifiers (`data-*`) for layout adaptivity. Standardized Apple Human Interface Guidelines (HIG) as the core aesthetic baseline (8pt spatial grid, generous whitespace, tabular pricing numerals, 16px squircles, pill buttons, frosted glass, fluid springs). Updated `docs/DECISIONS.md`, `docs/ARCHITECTURE.md`, and documentation indices.

**Modified Surfaces:**
- `docs/DECISIONS.md` (ADR-031 index entry and accepted architectural decision record)
- `docs/ARCHITECTURE.md` (Block 1 pluggable presentation layer updated with ADR-031 multi-tenant runtime)
- `docs/WORK_LOG.md` (Version history ledger)

**Quality Gates Verified:**
- `python scripts/doc-sync-workflow.py check`: Verified clean document sync and invariants.

---

### [v2.27.0] — 2026-09-08 17:30 UTC


**Summary:** Executed the complete Apple Human Interface Guidelines (HIG) functionalization and product truth remediation for the Vendryx Merchant Admin Dashboard (`apps/admin`) following the Backend-First protocol. Eradicated 100% of browser mock storage (`localStorage`), fake `setTimeout` connection handshakes, and hardcoded records across production routes (enforced by `scripts/check-no-mock-storage.ps1`). Built the server-side Courier Logistics Gateway & AES-256-GCM Credential Vault (`/admin/couriers`), live upstream gateway probes (Pathao, Steadfast, RedX), and zone rate engine (`/settings/couriers`). Built the live Merchant Onboarding Engine (`/admin/onboarding` & `/onboarding`) dynamically inspecting database tables. Restructured Settings into an Apple HIG Grouped Inset Settings Hub (`/settings`) with 7 categories and instant synonym search. Replaced the `/` redirect with an authentic Operational Home Cockpit (`/`) backed by `GET /admin/home/summary`. Hardened `/analytics` to dynamic store currencies (including BDT) and semantic labels. Created comprehensive Vitest route smoke and analytics test suites.

**Modified Surfaces:**
- `crates/api/stheno-api-admin/src/couriers.rs` (AES-256-GCM encrypted courier vault, live probes, rate zones)
- `crates/api/stheno-api-admin/src/onboarding.rs` (live table inspection for onboarding milestones)
- `crates/api/stheno-api-admin/src/home.rs` (aggregated operational cockpit metrics)
- `crates/api/stheno-api-admin/src/settings.rs` (JSONB store settings and policies)
- `apps/admin/src/routes/settings/settings-hub.tsx` (Apple HIG Settings Hub)
- `apps/admin/src/routes/couriers/` (logistics list, credential modal, real probes)
- `apps/admin/src/routes/onboarding/` (guided setup checklist wizard)
- `apps/admin/src/routes/home/` (owner operational cockpit)
- `apps/admin/src/routes/analytics/` (currency truth and state hardening)
- `apps/admin/src/routes/__tests__/route-smoke.spec.tsx` (automated route smoke suite)
- `apps/admin/src/routes/analytics/__tests__/analytics.spec.tsx` (analytics unit test suite)
- `scripts/check-no-mock-storage.ps1` (mock storage CI gate)

**Quality Gates Verified:**
- `check-no-mock-storage.ps1`: PASSED (0 mock storage violations)
- `npm --prefix apps/admin run typecheck`: PASSED (0 TypeScript errors)
- `npm --prefix apps/admin test -- --run`: PASSED (26 test files passed, 165 tests passed, 0 failures)
- `cargo test --lib -p stheno-api-admin`: PASSED (6 passed, 0 failures)
- `cargo check --workspace`: PASSED (33.30s, exit code 0)

---

### [v2.26.0] — 2026-09-08 11:11 UTC

**Summary:** Implemented all architectural decisions (ADR-020 through ADR-027), multi-tier zero-cost egress & zero-DB-hit in-memory catalog caching (ADR-024), transaction-mode PgBouncer connection pooling (ADR-025), core content routes (/about, /policies/*) (ADR-020), created master architecture specification (docs/ARCHITECTURE.md), and archived legacy documentation clutter into docs/archive/.

**Modified Surfaces:**
- **crates/api/stheno-api-store**
- **crates/stheno-server**
- **apps/storefront/packages/storefront-core**
- **docker-compose.yml**
- **docs/ARCHITECTURE.md**
- **docs/DECISIONS.md**
- **docs/archive/**

**Quality Gates Verified:**
- `cargo test -p stheno-api-store`: PASSED
- `cargo clippy -p stheno-api-store -p stheno-api-admin -p stheno-server`: PASSED
- `npm --prefix apps/storefront run typecheck`: PASSED
- `check-storefront-boundaries.sh`: PASSED
- `check-frontend-purity.sh`: PASSED
- `check-frontend-pin.sh`: PASSED
- `check-frontend-resolution.sh`: PASSED
- `check-frontend-switch.sh`: PASSED
- `check-frontend-not-forked.sh`: PASSED
- `check-module-isolation.sh`: PASSED
- `check-sql-boundaries.sh`: PASSED
- `check-shared-kernel.sh`: PASSED
- `check-no-domain-gates.sh`: PASSED
- `stheno-cli openapi --check`: PASSED
- `doc-sync-workflow.py check`: PASSED

---


### [v2.25.0] — 2026-09-08 15:45 UTC

**Summary:** Completed Phase P28 (Hardening, PCI re-attestation, General Availability) across all 5 tasks and passed omnibus Phase Gate P28 (`./scripts/gate-p28.sh`). This completes 95 of 95 tasks (100% Phase 3 GA) for Stheno / Vendryx Commerce. Verified gateway redirect and iframe CSP script manifest (P28-01), completed PCI DSS v4.0 SAQ-A re-attestation covering all five payment gateways (P28-02), built provider failure modes integration tests covering timeout ambiguity recovery, gateway 500 error handling, callback omission convergence, and courier down closed-failure (P28-03), built multi-tenant and multi-theme load test runner holding latency and 0-round-trip SSR branding budgets (P28-04), and authored production incident runbooks with automated verification gate (P28-05).

**Modified Surfaces:**
- **Gateway CSP Script Manifest (P28-01)**: Updated `docs/payment-page-scripts.md`, `scripts/check-checkout-scripts.sh`, and `scripts/check-checkout-scripts.ps1` to cover Stripe, bKash, Nagad, SSLCommerz, and COD.
- **PCI DSS v4.0 SAQ-A Attestation (P28-02)**: Updated `docs/PCI_SAQ_A.md` re-attesting compliance across all 5 gateways under Req 6.4.3 and 11.6.1; verified with `./scripts/check-pci-saq-a.sh`.
- **Provider Failure Modes Harness (P28-03)**: Implemented `crates/stheno-server/tests/provider_failure_modes.rs` covering gateway timeout with idempotency recovery, gateway 500 session preservation, callback omission convergence sweep, and courier API down fail-closed and deduplication prevention; verified with `cargo test --workspace --test provider_failure_modes`.
- **Multi-Tenant Load Suite (P28-04)**: Created `scripts/multi-tenant-load.js`, `scripts/load-suite.sh`, and `scripts/load-suite.ps1` asserting sub-50ms mean TTFB, 0 external render-path round trips (< 0.05ms SSR token resolution), and 100% tenant isolation across multiple concurrent themes in one process; verified with `./scripts/load-suite.sh --multi-tenant`.
- **Operational Incident Runbooks (P28-05)**: Authored `docs/runbooks/reconciliation-discrepancy.md`, `docs/runbooks/stuck-consignment.md`, and `docs/runbooks/gateway-outage.md`; built verification gates `scripts/check-runbooks.sh` and `scripts/check-runbooks.ps1`.
- **Omnibus Phase Gate P28**: Created `scripts/gate-p28.sh` and `scripts/gate-p28.ps1` verifying universal structural boundaries and all five P28 deliverables.

**Quality Gates Verified:**
- `./scripts/gate-p28.sh`: PASSED (All 5 stages + universal checks clean, exit code 0)
- `./scripts/check-checkout-scripts.sh`: PASSED
- `./scripts/check-pci-saq-a.sh`: PASSED
- `cargo test --workspace --test provider_failure_modes`: PASSED (4/4 tests clean)
- `./scripts/load-suite.sh --multi-tenant`: PASSED (1761 req/s, 13.89ms mean TTFB, 0 error rate)
- `./scripts/check-runbooks.sh`: PASSED
- `python scripts/doc-sync-workflow.py check`: PASSED (5/5 checks clean)

---

### [v2.24.0] — 2026-09-08 14:20 UTC

**Summary:** Completed Phase P27 (Theme packages & conformance) across all 10 tasks and passed Phase Gate P27 (`./scripts/gate-p27.sh`). Implemented theme conformance test runner verifying contract slots and shopping journey stages (browse → product → cart → checkout → order confirmed), conformance gate script, checkout ownership and skinning tests, core delta budget audit script, per-tenant themed deploy pipeline, bundler switch verifications, and comprehensive theme authoring guide.

**Modified Surfaces:**
- **Storefront Conformance Harness**: Added `test:conformance` script to `apps/storefront/package.json` and built `apps/storefront/scripts/test-conformance.js` verifying all 45 contract slots across frontends.
- **Storefront Conformance Gate**: Created `scripts/check-frontend-conformance.sh` and `scripts/check-frontend-conformance.ps1`.
- **Checkout Ownership Verification**: Built Vitest suite `apps/storefront/tests/unit/checkout-ownership.test.ts` asserting core checkout ownership, webpack request holding, and PCI SAQ A script compliance.
- **Core Delta Budget Audit**: Built `scripts/check-core-delta-budget.sh` and `.ps1` verifying contract changes remain strictly within budget.
- **Themed Deployment Pipeline**: Created `scripts/deploy-themed-storefront.sh` and `.ps1` supporting `--dry-run` and per-tenant target separation.
- **Documentation & Authoring**: Authored `docs-site/docs/frontend-authoring.md` with opening checkout rule (ADR-013), contract slot inventory, ADAPT vs SKIN rules, and time-to-new-theme benchmarks; created gate `scripts/check-frontend-docs.sh` and `.ps1`.
- **Phase Gate P27**: Created `scripts/gate-p27.sh` and `scripts/gate-p27.ps1` verifying all 10 stages clean with exit code 0.

**Quality Gates Verified:**
- `./scripts/gate-p27.sh`: PASSED (10/10 stages clean, exit code 0)
- `npm --prefix apps/storefront run test:conformance -- --frontend=default`: PASSED
- `./scripts/check-frontend-conformance.sh`: PASSED
- `npm --prefix apps/storefront run typecheck`: PASSED
- `npm --prefix apps/storefront run test:conformance -- --frontend=b`: PASSED
- `./scripts/check-checkout-scripts.sh`: PASSED
- `npm --prefix apps/storefront run test -- checkout-ownership`: PASSED (5/5 tests)
- `./scripts/check-core-delta-budget.sh`: PASSED
- `./scripts/deploy-themed-storefront.sh --dry-run`: PASSED
- `./scripts/check-frontend-switch.sh`: PASSED
- `./scripts/check-frontend-docs.sh`: PASSED
- `python scripts/doc-sync-workflow.py check`: PASSED (5/5 checks clean)

---

### [v2.23.0] — 2026-09-08 08:20 UTC

**Summary:** Implemented `@stheno/frontend-lustre` (cold-luxury watch and ornament storefront frontend) satisfying the complete 45-slot contract in `@stheno/storefront-core` according to `docs/FRONTEND_LUSTRE_PLAN.md` and `AGENTS.md §12`. Full greenfield implementation with 0 structural forks, 0 `@medusajs/ui` components, zero frontend data fetching or SDK calls, no unsourced figures, and complete Next.js App Router static/dynamic production build.

**Modified Surfaces:**
- **Storefront Packages**: Created `apps/storefront/packages/frontend-lustre` with 125 source files across 11 modules (`account`, `cart`, `categories`, `collections`, `common`, `fast-checkout`, `home`, `layout`, `order`, `products`, `shipping`, `skeletons`, `store`).
- **Storefront Core & Switching**: Registered `typecheck:lustre`, `build:lustre`, and `dev:lustre` in `apps/storefront/package.json`; configured `apps/storefront/tsconfig.lustre.json`; updated `scripts/check-frontend-switch.sh` to test `default`, `matrix`, and `lustre`.
- **Documentation**: Documented all 45 contract slots as `derived` (0 `forked`) in `docs/FRONTEND_LUSTRE_SLOTS.md`.

**Quality Gates Verified:**
- `npm --prefix apps/storefront run typecheck:lustre`: PASSED (0 errors)
- `npm --prefix apps/storefront run typecheck`: PASSED (0 errors)
- `npm --prefix apps/storefront run build:lustre`: PASSED (25/25 pages compiled)
- `./scripts/check-frontend-pin.sh`: PASSED (4/4 frontends pin core 0.1.0)
- `./scripts/check-frontend-not-forked.sh`: PASSED (125 files, 0 forks, 0 @medusajs/ui)
- `./scripts/check-frontend-purity.sh`: PASSED (0 reads, 0 server actions)
- `./scripts/check-storefront-boundaries.sh`: PASSED (0 core internal / SDK imports)
- `./scripts/check-frontend-resolution.sh`: PASSED (1 binding, 45 slots)
- `./scripts/check-frontend-switch.sh`: PASSED (default, matrix, and lustre all pass)
- `npm --prefix apps/storefront run test`: PASSED (16/16 unit tests)
- `python scripts/doc-sync-workflow.py check`: PASSED (5/5 checks clean)

---

### [v2.22.0] — 2026-09-08 05:45 UTC

**Summary:** Completed Phase P24 (COD Settlement & Courier Reconciliation) across all 12 tasks and passed Phase Gate P24 (`./scripts/gate-p24.sh`). Implemented courier settlement cycle and settlement lines migrations with RLS + FORCE, 3-amount order distinctness, courier-agnostic delivery-driven capture (BP6), return cancellation and net-zero expectation handling, idempotent settlement report import (BP7), 3-bucket reconciliation classification, exact decimal courier fee accounting (`sum(remitted) + sum(fees) == sum(collected)`) (BP8), deliberate mismatch discrepancy rows, aged unsettled line alerting, admin reconciliation routes, and admin reconciliation UI.

**Modified Surfaces:**
- **Database & Migrations**: Created migration `20260908000003_courier_settlement.sql` introducing `payment.courier_settlement_cycle` and `payment.courier_settlement_line` tables with tenant isolation, RLS enabled and forced.
- **stheno-order**: Implemented three distinct monetary amounts on orders (`order_total`, `collected`, `remitted`) in `models.rs` and database schema.
- **stheno-payment**: Implemented courier settlement engine in `crates/modules/stheno-payment/src/settlement/` featuring idempotent CSV/JSON import, reconciliation run bucket segregation, exact decimal fee balance identity validation, mismatch discrepancy logging, and aged unsettled line alerting.
- **stheno-api-admin**: Mounted admin reconciliation endpoints (`GET /admin/reconciliation/owed`, `GET /admin/reconciliation/matched`, `GET /admin/reconciliation/unmatched`, `POST /admin/reconciliation/runs`).
- **apps/admin**: Built merchant reconciliation UI with summary metrics, tabbed status filters, and discrepancy tracking in `apps/admin/src/routes/reconciliation/`.
- **End-to-End Testing & Phase Gates**: Created end-to-end COD lifecycle test (`cod_lifecycle.rs`), verification script (`scripts/cod-lifecycle.sh` / `.ps1`), and Phase Gate P24 (`scripts/gate-p24.sh` / `.ps1`).

**Quality Gates Verified:**
- `./scripts/gate-p24.sh`: PASSED (10/10 stages clean, exit code 0)
- `cargo test -p stheno-db --test rls_audit`: PASSED
- `cargo test -p stheno-order --test cod_amounts`: PASSED
- `cargo test -p stheno-payment --test cod_capture_on_delivery`: PASSED
- `cargo test -p stheno-payment --test cod_return_no_capture`: PASSED
- `cargo test -p stheno-payment --test settlement_import`: PASSED
- `cargo test -p stheno-payment --test reconciliation_buckets`: PASSED
- `cargo test -p stheno-payment --test settlement_balance`: PASSED
- `cargo test -p stheno-payment --test settlement_mismatch`: PASSED
- `cargo test -p stheno-payment --test settlement_alerting`: PASSED
- `cargo test -p stheno-api-admin --test reconciliation_routes`: PASSED
- `npm --prefix apps/admin run test -- reconciliation`: PASSED (5/5 tests)
- `./scripts/cod-lifecycle.sh`: PASSED (4/4 stages clean, exit code 0)
- `python scripts/doc-sync-workflow.py check`: PASSED (5/5 checks clean)

---

### [0.4.0] — 2026-09-07 21:21 UTC

**Summary:** Configured CORS allowlists, platform operator token, and registered tenant owner accounts for local hosts; updated login i18n brand strings

**Modified Surfaces:**
- **crates/stheno-server**
- **apps/admin**
- **apps/platform**
- **docs/WORK_LOG.md**

**Quality Gates Verified:**
- `doc-sync-workflow.py check`: PASSED

---

### [v2.21.0] — 2026-09-08 06:30 UTC

**Summary:** Completed Phase P26 (Branding tier & runtime tokens) across all 10 tasks and passed `./scripts/gate-p26.sh`. Implemented tenant branding migration with RLS + FORCE, Store API `/store/branding` endpoint with documented defaults, curated self-hosted font library, storefront core per-request zero-round-trip CSS token emission, boundary asset dimension sniffers in `stheno-file`, admin live preview UI, checkout branding compliance, and Playwright Chromium measured Lighthouse & Core Web Vitals runner.

**Modified Surfaces:**
- **Backend & Database**: Added tenant branding table migration (`20260908000002_tenant_branding`), RLS audit tests, and Store API `/store/branding` handler in `crates/api/stheno-api-store/`.
- **Asset Validation**: Implemented boundary type sniffing and dimension validation (PNG, JPEG, WebP) in `crates/modules/stheno-file/` (`validate_branding_asset`).
- **Storefront Core**: Implemented `resolveBrandingTokens` and `emitBrandingCss` in `apps/storefront/packages/storefront-core/src/lib/branding.ts`, inlined directly into RootLayout `<head>` with 0 render-path round trips.
- **Admin Dashboard**: Built `/settings/branding` configuration screen with live theme token preview and color pickers in `apps/admin/src/routes/branding/`.
- **Fonts & Licences**: Curated 7 approved self-hosted fonts with licence audit in `docs/FONTS.md` and verification script `scripts/check-font-licences.sh`.
- **Performance & Core Web Vitals**: Implemented real headless Chromium audit runner in `apps/storefront/tests/lighthouse/runner.js` measuring LCP, CLS, TTFB, and enforcing 0 external branding round trips.
- **Phase Gate**: Created `scripts/gate-p26.sh` and `scripts/gate-p26.ps1` bundling all 10 verification steps.

**Quality Gates Verified:**
- `./scripts/gate-p26.sh`: PASSED (10/10 checks clean)
- `cargo test -p stheno-db --test rls_audit`: PASSED (4/4 tests)
- `cargo test -p stheno-api-store --test branding_endpoint`: PASSED (3/3 tests)
- `cargo test -p stheno-file --test branding_asset_validation`: PASSED (11/11 tests)
- `npm --prefix apps/admin run test -- branding`: PASSED (6/6 tests)
- `npm --prefix apps/storefront run test -- branding-tokens`: PASSED (5/5 tests)
- `npm --prefix apps/storefront run test:lighthouse`: PASSED (Performance score 100/100, 0 round trips)
- `python scripts/doc-sync-workflow.py check`: PASSED (5/5 checks)

---

### [v2.20.0] — 2026-09-08 02:55 UTC

**Summary:** Full repository documentation synchronization and automated versioned workflow deployment. Reconciled pricing and quota invariants across marketing surfaces, updated SITEMAP.md and PHASE3_STATUS.md to reflect live payment provider & courier admin panels, aligned module inventories, and established automated documentation verification gates.

**Modified Surfaces:**
- **Marketing Public Metadata**: Synced `apps/marketing/public/llms.txt`, `llms-full.txt`, and `apps/marketing/index.html` with canonical `config/tiers.toml` quotas (Free: 100 products, 3 staff seats; Pro: ৳1,500/mo, 10 staff seats).
- **Phase 3 Status**: Updated `docs/PHASE3_STATUS.md §2` to record completion of payment provider credentials admin UI (`apps/admin/src/routes/payment-providers/`) and backend AES-256-GCM vault.
- **Site Map**: Updated `docs/SITEMAP.md` with `/demo`, `/privacy`, `/terms`, `/settings/payment-providers`, `/settings/entitlements`, `/help`, and `/release-notes`.
- **Data Model & TRD**: Added `demo` sandbox module and `order-change` to `docs/DATA_MODEL.md`.
- **Parity Matrix**: Updated `docs/PARITY_MATRIX.md §1` to record passing status for all 34 core modules.
- **Payment Gateways**: Updated `docs/providers/PAYMENT_GATEWAYS.md` credential matrix with live compiled badges and marked Nagad as planned.
- **Workflow Automation**: Created `scripts/doc-sync-workflow.py`, `doc-sync-workflow.ps1`, `doc-sync-workflow.sh`, and `docs/DOCS_WORKFLOW.md`.

**Quality Gates Verified:**
- `python scripts/doc-sync-workflow.py check`: PASSED (5/5 checks clean)
- `cargo run -p stheno-cli -- openapi --check`: PASSED (0 drift)
- `cargo run -p stheno-cli -- provider-docs --check`: PASSED (0 drift)
- `python scripts/check-no-fabricated-handlers.py`: PASSED (8 constants, 159 stubs)
- `npm.cmd --prefix apps/marketing run build`: PASSED (built in 6.25s)

---

### [v2.19.0] — 2026-09-07 20:45 UTC

**Summary:** Complete Vendryx product rebrand, interface adaptation, and reference-driven merchant administration pages across Phases A through J.

**Modified Surfaces:**
- **Brand System**: Established `docs/BRAND.md`, `brand/vendryx/brand.json`, `--vx-*` tokens in `index.css`, vector logos across marketing and admin.
- **Architecture**: Recorded ADR-028 in `docs/DECISIONS.md`.
- **Marketing Surface**: High-converting bento layout, interactive client-side sandbox demo (`/demo`), dedicated auth experience (`/login`, `/signup`, `/forgot-password`, `/welcome`).
- **Merchant Admin Panel**: Payment provider configuration (`/settings/payment-providers`), AES-256-GCM vault dialog, courier logistics (`/settings/couriers`), domains & TLS provisioning (`/settings/domains`), operator security & browser sessions (`/profile`), store policies editor (`/settings/store`), help desk & ticket tracker (`/help`), and release notes (`/release-notes`).
- **Screen Matrix**: Cataloged and verified 16 reference screens in `docs/VENDRYX_SCREEN_MATRIX.md`.

**Quality Gates Verified:**
- `npm.cmd --prefix apps/admin run build`: PASSED (33.98s)
- `npm.cmd --prefix apps/marketing run build`: PASSED (7.01s)
- `npm.cmd --prefix apps/platform run build`: PASSED (7.54s)
- `powershell -File scripts/check-brand-surface.ps1`: PASSED
- `cargo test -p stheno-payment`: PASSED (16/16 test suites)
- `cargo test -p stheno-api-admin --test provider_config_routes`: PASSED

---

### [v2.19.1] — 2026-09-08 04:40 UTC

**Summary:** Merchant Admin Dashboard Apple HIG Audit remediation and Backend-First functionalization. Eliminated mock storage, timers, and fabricated data; created real backend contracts and bridges; redesigned `/settings` into an Apple HIG-compliant Settings Hub with 7 functional categories, synonym search, and 44px touch targets.

**Backend Services & Contracts (100% Verified):**
- **RBAC Current User Permissions**: Added `GET /admin/rbac/me/permissions` returning real user scopes (`customer:*`, `product:*`, etc.) in `crates/api/stheno-api-admin/src/rbac.rs`.
- **Admin View Configurations & Columns**: Registered complete table column specifications for `return-reasons`, `refund-reasons`, `product-types`, `product-tags`, `stock-locations`, `locations`, `publishable-api-keys`, and `secret-api-keys` in `crates/api/stheno-api-admin/src/views.rs`.
- **Store Settings & Business Policy Engine**: Implemented `GET/POST /admin/settings/{key}` and `GET /admin/settings?category=...` backed by Postgres `settings.store_setting` with 64KB size guard and validation in `crates/api/stheno-api-admin/src/settings.rs`.
- **Tenant Custom Domains Bridge**: Implemented tenant-scoped `/admin/domains` CRUD and server-side DNS challenge verify endpoint in `crates/api/stheno-api-admin-platform/src/lib.rs`.
- **Tenant Billing Read Models**: Implemented `/admin/billing/subscription` and `/admin/billing/invoices` returning honest tenant subscription and invoice ledger records in `stheno-api-admin-platform`.

**Frontend Apple HIG Transformation & Truth Remediation:**
- **Zero Mock Storage Gate**: Created and enforced `scripts/check-no-mock-storage.ps1` and `.sh`. Removed all `localStorage`/`sessionStorage` and fake timers from production routes.
- **Route Collision Fixes**: Mounted `/orders/social` before `/orders/:id`, added `/profile` -> `/settings/profile` redirect, cleaned unmounted courier routes from nav menus.
- **Real Backend Bindings**:
  - `profile-password-section.tsx`: Hooked to `/auth/user/emailpass/update`.
  - `profile-sessions-section.tsx`: Real browser/device detection with `useLogout`.
  - `cod-rules-card.tsx`: Connected to `/admin/settings/cod_rules`.
  - `store-policies-section.tsx`: Connected to `/admin/settings/store_policies`.
  - `domains-view.tsx`: Connected to `/admin/domains` with server-side challenge verification.
  - `entitlements-detail.tsx`: Connected to `/admin/billing/subscription` and `/admin/billing/invoices`; eliminated fake invoices.
  - `help-center.tsx`: Replaced fake ticket simulator with verified Syed Systems engineering support channels.
- **Apple HIG Settings Hub (`/settings`)**:
  - Implemented 7 categories: Store & Organization, Commerce Policies, Financial & Currency, Shipping & Logistics, Developer & API, Security & Access, Platform & Billing.
  - Built Apple HIG primitives: `SettingsRow` (44px touch targets), `SettingsSection` (grouped inset cards), `StatusAccessory`, `AsyncState`.
  - Added synonym and keyword search (`vat`, `bkash`, `dns`, `rbac`, etc.) with instant results count.
- **Documentation**: Recorded ADR-029 in `docs/DECISIONS.md`, updated `docs/VENDRYX_SCREEN_MATRIX.md`.

**Quality Gates Verified:**
- `cargo check -p stheno-server`: PASSED (exit code 0)
- `cargo test -p stheno-api-admin-platform`: PASSED (10/10 tests)
- `npm --prefix apps/admin run typecheck`: PASSED (exit code 0)
- `npm --prefix apps/admin test -- --run`: PASSED (139/139 tests across 20 suites)
- `powershell -ExecutionPolicy Bypass -File scripts/check-no-mock-storage.ps1`: PASSED (zero mock storage)

---

### Session Log — 2026-09-08 (Space Standalone Control Plane Documentation Sync)
**Objective**: Synchronize all documentation across the repository to reflect that the company control plane has been extracted and updated as **Space**, a separate standalone project located at `D:\Syed Systems\Space`.

- **ADR-030 Recorded**: Added `ADR-030 — Company Control Plane extracted to standalone 'Space' project` to `docs/DECISIONS.md`.
- **Primary Plan Parity**: Added implementation note to `docs/SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md` referencing the standalone project root at `D:\Syed Systems\Space`.
- **Architecture & Status Alignment**:
  - `docs/CONTROL_PLANE.md`: Added architectural status update banner clarifying that Space is the company control plane, while in-repo `apps/platform` and `stheno-api-platform` serve as transitional surfaces during phased migration.
  - `docs/CONTROL_PLANE_PLAN.md`: Added transition banner for handler remediation as transitional baseline for Space connectors.
  - `docs/HANDOVER_CONTROL_PLANE.md`: Added developer handoff notice directing new control plane development to `D:\Syed Systems\Space`.
  - `docs/PLATFORM_COMPLETION_TASKS.md`: Clarified the 23 handlers as the in-repo transitional platform surface.
  - `docs/RUNBOOK_CONTROL_PLANE.md`: Noted operational scope for `stheno-api-platform` vs Space fleet operations.
  - `docs/PRD.md`: Clarified operator control plane is Space (`D:\Syed Systems\Space`) in Summary.
  - `docs/TRD.md`: Updated Section 1.2 and Section 9 tenancy and deployment notes to distinguish in-repo transitional routes from standalone Space.
  - `docs/PHASE3_STATUS.md`: Added architecture callout in Section 2b.
  - `docs/PHASE4_TASKS.md`: Added header note directing to Space tickets and standalone project.
  - `docs/VENDRYX_ADAPTATION_PLAN.md`: Updated operator control plane rows and Phase I guidance to reference Space.
  - `docs/DEPLOYMENT.md`: Clarified `/platform/*` credentialing relative to Space.
  - `docs/ENTITLEMENTS.md`: Noted Space as target multi-product billing and entitlement authority.
# Session Log — 2026-10-23 (ADR-033 deployed plan price)

- Wired the admin subscription endpoint to report exact configured plan amount, currency,
  and billing interval from the deployment's `config/tiers.toml`; it selects the persisted
  subscription tier when available and the request's effective tier otherwise.
- Removed the dashboard's hardcoded Pro price. The subscription screen now displays the
  server-provided exact amount and interval, and reports an unavailable price when config
  has no sellable Pro price.
- Added focused API coverage with a deployment price override. The Space package gate passed
  all 8 `entitlements` tests, including configured current-plan pricing and Pro teaser pricing
  for a Free tenant.
- The same focused compile exposed that the internal `ConversationInput` attempted to
  deserialize server-authorized chat offer quotes. Removed `Deserialize` from this
  server-assembled type rather than making the protected quote DTO deserializable.
- The first scoped admin web check uncovered missing tmpfs mountpoint directories in the
  disposable read-only Space workspace. Space's builder now creates those empty staging
  directories before Docker mounts them; its embedded Bash syntax and template contract
  checks pass. The mount also allows native install binaries to execute, as required by
  esbuild postinstall scripts.
- Added a dedicated `web-admin` Space mode, with a cache key based only on the admin lockfile,
  and only the admin app's `node_modules` tmpfs. It sets Node's heap to 2 GiB under the
  existing 3 GiB container limit. The admin typecheck passed. An initial check exposed
  Node's default ~1.5 GiB heap ceiling; the adjusted run passed.
- The upgrade modal now displays the deployment's configured Pro price even for Free-tier
  stores and disables the upgrade action when Pro has no price. Added modal semantics,
  focus entry/restoration, Tab trapping (including focus moving outside), and Escape dismissal.
  Removed hardcoded active bKash/card claims from payment-method management; it reports the
  actual missing method and directs users to the configured billing portal.
- A second focused API run passed all 8 `entitlements` tests after adding Pro teaser pricing
  to the response. The final admin-only typecheck passed after the modal and payment-method updates.
- `cargo fmt -p stheno-api-admin-platform -- --check` was not clean: rustfmt reports existing
  unrelated formatting throughout the modified API crate source. Those unrelated lines were
  left unchanged. The scoped admin typecheck and API test are the gates relevant to this slice.

### Session Log — 2026-10-23 (ADR-033 trial warning delivery)

- Added a scheduled 15-minute trial-warning job for day 3 and day 6. It resolves the verified
  merchant account owner through the platform owner binding, skips missing/unverified contacts,
  and records each milestone using the existing notification resource fields.
- Added a sent-event lookup so scheduler retries skip already successful milestones. A missed
  day-3 run that resumes on day 6 sends the final warning only.
- Added optional idempotent dispatch through notification providers and wired Resend's
  `Idempotency-Key` header. Stable keys make provider-accepted/database-write-ambiguous retries
  safe within Resend's documented 24-hour key retention window. Durable outbox and cross-provider
  guarantees remain open.
- Corrected two compile errors in the in-progress ADR-032 social confirmation path discovered
  by the focused server package check: `OrderDTO.items` is the line collection, and the quote ID
  is cloned before the async closure so it remains available for draft audit metadata. Made the
  admin auth request mutable where its verified claims are inserted into request extensions.
- Space checks: `stheno-notification-resend --test resend` passed (6/6); notification integration
  test passed (1/1) with the Space Postgres test service; `stheno-server` package check passed.
  A notification test run without test services correctly failed fast because `DATABASE_URL`
  was unset; rerunning with `-CargoTestServices` passed.
- `rustfmt --edition 2021 --check` passed for the notification provider/service/DTO, Resend tests,
  platform DTO/service, and scheduler files. No workspace-wide test or build was run.
- Added the admin trial banner using the persisted server expiry, a minute-updated remaining-day
  count, a missing-date state, and the existing billing portal/upgrade fallback. The focused Space
  `web-admin` run completed `npm run typecheck` with exit code 0.
- Tightened the trial warning payload for retry identity: its body is stable across retries,
  the merchant-controlled store name is HTML-escaped, and the tenant component of the provider
  idempotency key is SHA-256 encoded so the key stays bounded. A final Space rerun after these
  small edits could not connect: SSH to `syed-build@space` timed out at `100.105.191.63:22` on
  consecutive attempts. Therefore the successful server package check and admin typecheck above
  predate these last source edits; the current source still needs those two gates rerun when Space
  is reachable.

### Session Log — 2026-10-24 (ADR-033 renewal during grace)

- Collected subscription invoices now atomically mark collection, reset dunning, and advance
  `term_ends_at` to at least their covered `period_end`. Collecting an older invoice cannot shorten
  a newer paid-through date or commit payment without the renewal update.
- Added a focused platform package regression for renewal during grace and reconciliation at the
  old grace deadline. Documented the behavior and marked its Space execution pending.
- Enforced ADR-034's no-SMS rule at the shared notification dispatch boundary. SMS now fails with
  a stable validation error even if an SMS provider is registered; email, WhatsApp, and push
  routing remain provider-dependent.
- Corrected the ADR-033 router tier matrix to match the accepted model: Trial receives Pro
  features for its persisted term, merchant-owned custom domains are available on all tiers, and
  the Pro-only feature leak check applies to Free.
- Implemented the ADR-034 shopper payment-proof path: tenant-RLS order proof ledger, one pending
  reference per order, customer ownership and input validation, explicit `pending_verification`
  state, storefront submit/reload UI, and merchant review controls. Review and order payment-state
  updates share one transaction; verified becomes captured, rejected returns to unpaid. No test or
  build was run per instruction.
- As requested, no tests or builds were run. Space SSH was unavailable at `100.105.191.63:22` when
  last checked; the focused regression remains authored but unexecuted.

### Session Log — 2026-10-24 (ADR-033/034 durable notification intents)

- Added `notification.notification_outbox` with unique stable idempotency keys, provider selection,
  bounded attempts, delayed retry scheduling, delivery leases, and crash recovery. The enqueue SQL
  serializes duplicate keys and creates the notification audit row and durable intent together.
- Added a bounded outbox worker. It records successful delivery atomically with notification state;
  provider errors are stored as a fixed redacted category. Automatic retries are limited to
  providers that explicitly guarantee native idempotency, and ambiguous failures from other
  providers are dead-lettered to avoid duplicate customer messages. Delivery calls time out at
  30 seconds and the scheduled batch is bounded to keep its fleet lock from expiring mid-batch.
- Trial day-3/day-6 reminders now enqueue before delivery, and the server runs a one-minute worker.
  Resend declares native idempotency. Added an authored outbox contract test; no test or build was
  run as requested.
- WhatsApp provider/sender identity routing, consent checks, and broad customer-event migration
  remain implementation work; ADR-034 remains partial.

### Session Log — 2026-10-24 (ADR-034 tenant-aware notification routing groundwork)

- Added tenant identity to durable notification intents and the outbox delivery context. The
  provider contract now receives the trusted tenant ID for each queued send so a tenant-owned
  sender can resolve credentials without putting secrets in notification data or recipient fields.
- Customer order confirmations now enqueue with an event-stable idempotency key instead of sending
  inline. Trial reminders explicitly persist their subscription tenant from the platform worker.
- This is routing groundwork only: WhatsApp sender selection, consent persistence/enforcement, and
  migration of the remaining notification events are still open. No tests or builds were run.

### Session Log — 2026-10-24 (ADR-034 WhatsApp opt-in delivery path)

- Added a tenant-isolated consent ledger with explicit checkout/chat/settings sources, affirmative
  re-consent, and revocation state. The outbox refuses WhatsApp enqueue without active consent, and
  the provider checks consent again immediately before the external send.
- Added a Meta WhatsApp Cloud API provider. Free stores use central credentials; currently entitled
  Pro/trial stores use their single enabled, encrypted merchant account and do not fall back to
  central credentials when merchant setup is missing. Recipients, phone IDs, template names,
  languages, and template parameters are bounded and validated. Sends require approved-template
  metadata and use a bounded timeout; Meta calls are treated as non-idempotent.
- Checkout now offers an optional WhatsApp transactional-update opt-in and queues the order
  confirmation using deployment-configured approved template name/language. Inbound WhatsApp
  STOP/UNSUBSCRIBE/CANCEL/END messages revoke tenant-scoped consent before AI handling. The API
  response stays unchanged. Other transaction event types and template/WABA setup remain open.
  No tests or builds were run.

### Session Log — 2026-10-24 (ADR-034 COD/manual-transfer boundary)

- Added a separately registered `pp_manual` offline provider with required merchant account and
  transfer instructions stored through the existing encrypted payment credential vault. The
  provider returns its configured instructions with server-calculated amount/currency through
  both session creation and authorization; the existing `pp_system_default` provider remains COD.
- Checkout and order confirmation now display transfer information only for `pp_manual`. The
  order proof form no longer treats COD as a transfer or asks COD customers for a TrxID, and it
  continues to show the transfer destination after refresh while verification is pending. The
  transfer provider fails closed for non-BDT carts so a local ৳ account is never paired with a
  foreign-currency total.
- Added source-level provider contracts for COD/manual distinction and instruction persistence.
  No test, build, local gate, or Space gate was run per instruction.

### Session Log — 2026-10-24 (ADR-034 Free notification allowances)

- Added atomic per-tenant, per-channel daily usage and reservation ledgers. Free subscriptions are
  capped at 300 WhatsApp messages and 200 emails per UTC day across queued and direct delivery;
  Trial and Pro remain outside this Free-only cap.
- Concurrent outbox workers reserve under a transaction before provider dispatch. When full, an
  outbox intent remains pending until the next UTC day without consuming a retry attempt. Plan
  lookup or accounting failures defer delivery, so configuration or database faults cannot bypass
  the cost guard. Ambiguous provider outcomes retain their reservation conservatively.
- Documented the bounded cost exception in ADR-034 and the tier matrix. No tests, builds, lint,
  or remote gates were run, as requested; implementation remains unverified.

### Session Log — 2026-10-24 (ADR-034 durable account and invite notices)

- Account-created welcome emails and staff invitation emails now enter the tenant-scoped durable
  outbox with event-ID idempotency keys, preventing a subscriber replay from sending duplicates.
- Password-reset email remains immediate because its payload includes a time-sensitive reset token;
  the shared direct-send path still applies the Free email allowance.
- No tests, builds, lint, or remote gates were run, as requested.
- Customer manual-payment-proof submission now queues an idempotent email that clearly states
  “pending verification” and “not paid”; the handler verifies the order still belongs to the
  authenticated customer and does not change the proof response if email queueing fails.
- Merchant proof approval/rejection now queues an idempotent customer email after the persisted
  decision, accurately describing captured payment or the still-unpaid order without echoing the
  submitted reference or reviewer note.
- Admin order cancellation now queues one idempotent customer email and does not claim a refund
  or payment reversal that the cancellation path does not perform.
- Nested shipment and delivery routes queue a single email per fulfillment/status only after the
  state transition succeeds and all fulfillment line-item IDs are verified against the specified
  order. The direct fulfillment routes lack an order association, so they do not guess a recipient.

### Session Log — 2026-10-24 (ADR-034 customer WhatsApp revocation)

- Linked checkout WhatsApp consent to the authenticated customer id when the order has one, and
  added an authenticated `DELETE /store/customers/me/whatsapp-consent` endpoint. It revokes all
  active tenant-scoped consent associated with that identity, including consent attached to a
  previous phone number; repeated revocation is safe and does not accept a caller-supplied id or
  number.
- Added the customer lookup index and kept guest checkout consent nullable. No tests, builds, lint,
  or remote gates were run, as requested; this implementation remains unverified.
- Nested admin fulfillment cancellation now propagates cancellation failure and queues a stable
  customer email only after cancellation succeeds and the fulfillment items are verified against
  the route's order. The existing response JSON shape is retained. No tests, builds, lint, or remote
  gates were run, as requested.
- Added a customer profile control that calls the authenticated revocation action and explains
  that email notices remain enabled and checkout can request WhatsApp consent again. The account
  action reports success or failure accessibly without claiming consent status before the server
  confirms the change. No storefront checks were run, as requested.
- Store return create/list/get now require a valid customer token and verify every order against
  that token; list-all resolves only the caller's paginated orders. A return request email enters
  the durable outbox only after the return and its requested items are persisted, and says the
  return is pending store review rather than claiming a refund. The existing return response DTOs
  are unchanged. No tests, builds, or remote gates were run.
- Admin return cancellation routes now execute the persisted cancellation service, surface failed
  transitions, and enqueue one idempotent notice that does not imply a refund. The confirmation
  route remains unchanged because its current service path does not perform a verified refund.
- Removed the return service's synthetic hard-coded USD refund transaction and swallowed ledger
  error. Completing a return with a positive refund amount now fails closed so the operator must
  issue it through the payment service, which uses the payment's actual currency and provider
  idempotency boundary. Refund claims with a positive amount also fail closed until that provider
  refund has been issued; claim confirmation no longer fabricates a refund ledger entry.
  Updated ADR-034/035 readiness evidence. No tests, builds, lint, or remote gates were run, as
  requested.

### Session Log — 2026-10-24 (ADR-034 notification allowance visibility)

- Added a tenant-scoped notification usage read through the notification service. It uses the
  current UTC date, reports delivered and in-flight reservations, derives each channel's actual
  plan limit from the same quota policy used by dispatch, and fails closed when that policy or
  database read is unavailable.
- Mounted `GET /admin/notifications/usage` on the authenticated Admin API and added a dashboard
  view that refreshes periodically and distinguishes a missing cap (unlimited) from zero remaining.
  The response contains no recipient identifiers or message payloads.
- Updated ADR-034/035 readiness notes. No tests, builds, lint, or remote gates were run, as
  requested.

### Session Log — 2026-10-24 (ADR-034 capture and refund notices)

- Successful admin payment capture and refund actions now enqueue idempotent customer emails
  describing the exact amount and currency returned by the payment service.
- Admin-recorded payment-collection paid status now queues a distinct notice describing the
  store-recorded amount/currency without claiming a PSP capture.
- Before notifying, the handler resolves the payment's collection through the tenant's order-link
  ledger. Order-scoped actions verify that the URL order owns that collection; direct payment
  actions require exactly one linked order. Missing, mismatched, or ambiguous associations skip
  notification rather than guessing a recipient. Queue failures are logged after the payment
  action succeeds and do not change its response.
- No tests, builds, lint, or remote gates were run, as requested.

### Session Log — 2026-10-24 (ADR-035 Store Intelligence first slice)

- Defined Store Intelligence as token-free deterministic measurement first, followed by authorized
  source adapters for Meta/search, evidence-backed OKF/RAG retrieval, optional model ranking, and
  owner-approved styling recommendations. Added edge-case and acceptance phases 35.1–35.7,
  including source permissions/freshness, privacy, prompt injection, model abstention, accessibility,
  human approval, and rollback.
- Added tenant-scoped order-schema aggregates over current order snapshots for consecutive
  seven-day windows. Draft, canceled, deleted, and product-less records are excluded; no buyer
  identifiers or message content leave the order module. Product and category queries are
  independent so category totals are not derived from the product top-N list; both expose an
  explicit truncation state above 100 results. Added the named partial tenant/time index for this
  exact admin query.
- Added deterministic product/category direction, integer basis-point change, stable ordering, and
  explicit evidence-volume buckets; a zero historical baseline is represented without a fabricated
  percentage. The owner/admin endpoint and dashboard expose these signals and state clearly that
  social/search data are unavailable until authorized integrations exist.
- Extended product aggregation to return one result per product using the latest included line-item
  label, and changed both bounded SQL queries to fetch 101 rows so the API can accurately flag a
  100-row cap. Null/blank categories are grouped as `Uncategorized`, preserving their order volume.
- Added deterministic, capped next-step candidates for products/categories with at least five current
  period orders. Every candidate cites its first-party order evidence and requires owner review; the
  code does not call an inference provider, mutate commerce data, or alter chat prices.
- Added category-keyword archetype suggestions and a logo palette suggestion flow in Branding. Logo
  analysis is restricted to bounded same-origin raster content or small raster data URLs, rejects SVG
  and cross-origin sources, samples a downscaled canvas, and offers colors only as a preview which
  must be explicitly applied and then saved. The existing dashboard theme tokens remain the user
  override path. Archetype suggestions expose the `category-keywords-v1` rule-map version.
- ADR readiness now distinguishes these shipped source slices from outstanding social/search
  ingestion, OKF/RAG retrieval, model ranking, persisted archetype styling, audit/revert controls, and
  acceptance gates. No tests, builds, lint, or remote gates were run, per the instruction to skip
  testing for this implementation pass.
- Updated ADR-035 readiness and decision trust rules. No tests, builds, lint, or remote gates were
  run, as requested.

### Session continuation — 2026-10-24 (ADR-035 and ADR-036 code-only continuation)

- Added an hourly opt-in Space systemd health monitor that records disk/inode/memory state, optional
  runner-unit liveness, and only correctly labeled project cache sizes to bounded, private CSV files.
  Cache scans are time-bounded and run at low CPU/I/O priority; the unit caps memory at 128 MiB and
  CPU at 10%. Added a dry-run-by-default installer; it has not been run on Space.
- Instrumented the Space manual builder to record attempted container build mode, outcome, elapsed
  time, and a short source-archive hash, with no source or secret data in the metrics record.
- Created the corresponding Space changes on its own `codex/build-server-adr` branch; its existing
  dirty state was preserved. No hosted CI routing, runner registration/group change, or remote host
  installation was made. ADR-036 remains partial pending those trust/configuration operations and
  full job parity.
- The Space health collector now samples system memory/disk/inodes even when the idle rootless
  Docker daemon is stopped, records daemon state, and skips cache inspection without starting the
  daemon. The optional runner heartbeat check uses the user systemd service manager. The monitor
  remains opt-in and uninstalled.
- Added tenant-RLS persistence for Store Intelligence recommendation review states (`approved`,
  `dismissed`, `reverted`) with actor-attributed append-only audit and request idempotency. Owners
  and admins can record review from the dashboard; the action only records a decision and cannot
  change products, prices, content, or storefront state. Recommendation IDs are time-bucketed and
  bounded; no review decision is treated as authorization to execute commerce actions.
- ADR-035 remains partial: social/search adapters, multi-source normalization, model ranking,
  persisted storefront archetype controls, source permission/freshness controls, and audit-history
  UI are not present. Recommendation review now has a tenant-RLS current-state table and an
  append-only audit table. A bounded lexical evidence lookup adds up to three tenant-owned
  knowledge source references per recommendation; it returns explicit match/no-match/unavailable
  states and exposes titles/IDs/timestamps only, never knowledge content to a model. No tests,
  builds, formatting, or remote gates were run, as requested.

### Session continuation — 2026-10-24 (targeted ADR checks and build-memory reduction)

- Removed Diesel's workspace-wide `64-column-tables` feature after scanning every generated
  `diesel::table!` schema and confirming none exceeds 32 columns. Store product and notification
  crate checks pass without it. During the old-feature compile, a sampled `rustc` working set was
  about 2.5 GiB; during the reduced-feature compile it remained about 0.8 GiB. These are sampled
  working sets, not measured peak/RSS benchmarks.
- Space's manual Rust CI and service-backed package modes now run one Cargo job at a time and disable
  incremental artifacts and debug symbols for dev/test outputs. This fits the builder's 8 GiB host
  and 3 GiB job-container ceiling. Embedded-script syntax and cache/capacity/cleanup fixtures pass.
- Fixed compile and lint failures in the current ADR work: declared notification's `ulid` dependency,
  removed an unused outbox projection and `DateTime` clones, kept the notification compatibility
  trait shape explicit, and replaced APIs newer than the declared Rust 1.80 MSRV. The Admin API
  documents its deliberate Clippy `result_large_err` allowance because its established Axum handler
  contract returns `Response` directly.
- Added pure guardrail tests for recommendation review identity, day-window validity, actor/request
  bounds, decisions, and review-history limits. Added an owner/admin-only tenant-RLS history route
  capped at 50 events and an on-demand dashboard history view; decisions still cannot execute
  commerce changes.
- Focused checks passed: `cargo test -p stheno-ai --lib` (11 tests), `cargo check -p
  stheno-product --lib`, `cargo check -p stheno-notification --lib`, `cargo check -p
  stheno-api-admin --lib`, `cargo clippy -p stheno-ai --lib -- -D warnings`, and
  `cargo clippy -p stheno-api-admin --lib -- -D warnings`. Admin's `npm run typecheck` and the
  three logo-source boundary tests pass. Module isolation, SQL boundaries, shared-kernel,
  module-contract, domain-gate, rustfmt, and Space builder-template checks pass.
- Full workspace tests, full hosted CI, schema-drift against a live database, and remote Space jobs
  were not run. ADR-035 source adapters, cross-source normalization, persisted storefront style
  application, and acceptance/tenant-isolation gates remain open. ADR-036 trusted CI routing and
  full stage parity also remain open; no hosted job or remote resource was changed.

### CI follow-up — affected Cargo package gates

- Added `scripts/affected-cargo-gate.py`: PR Rust Clippy/tests now select changed Cargo packages
  plus reverse dependents; root manifests/toolchains and unowned Rust sources select the full
  workspace; docs-only changes skip Cargo compilation/tests. Metadata/diff failures fail closed to
  the full workspace.
- Added temporary-Git-workspace fixtures for dependent propagation, workspace fallback, docs-only
  skip, and unowned Rust-source fallback. The selector test suite passes (4 tests).
- Updated `.github/workflows/ci.yml` to fetch the comparison history and use the selector for the
  integration Clippy/test steps. Manual dispatch still runs the full workspace. The hosted workflow
  itself has not been dispatched from this checkout.
- Read-only GitHub inspection confirms the `Store and Space Build Host` organization runner group
  is scoped to both private repositories, and its `space` runner advertises `syed-build` and was
  online/busy at inspection time. I did not dispatch a job or change runner settings. The new gate
  therefore keeps PR execution hosted and preserves the group for a later parity-verified route.
- The latest visible main `Nightly` run (2026-10-23) failed at `Start stack` / `Restore drill`; the
  latest visible push-triggered CI run (2026-10-21) reported a workflow-file issue before any job
  started. The new workflow passes local YAML parsing and the repository CI configuration
  self-check, but it has not been pushed or executed by GitHub Actions.

### Session continuation — 2026-10-24 (focused gates and Nightly CI repair)

- Added four deterministic Store Intelligence report tests covering integer trend math, missing baselines, confidence boundaries, category archetype version/fallback behavior, row caps/truncation, and stable recommendation filtering/order. `cargo test -p stheno-ai --lib` passes 15 tests; package Clippy passes.
- Fixed the admin test fixture to initialize the optional notification service. `cargo test -p stheno-api-admin --lib` passes 10 tests with an isolated disposable Postgres service. The new test remains at the end of `orders.rs` to satisfy Clippy's item-order lint.
- Admin all-target Clippy surfaced a Rust 1.80 MSRV violation (`Option::is_none_or`) in the ADR-touched trial lifecycle code. Replaced it with the MSRV-compatible `map_or` form; `cargo clippy -p stheno-api-admin --all-targets -- -D warnings` now passes.
- Read-only hosted workflow inspection showed Nightly's start, restore, and cleanup steps execute scripts tracked as mode `100644` directly, causing exit 126. Updated all three steps to invoke Bash explicitly and extended `check-ci-postgres.sh` to guard these invocations.
- CI's affected-package selector remains covered by four temporary-workspace tests. Workflow YAML parsing, CI self-check, selector tests, and exact touched-file rustfmt check pass. No GitHub job was dispatched; hosted CI remains unverified from this unpushed branch.

### Session continuation — 2026-10-24 (ADR-034 fulfillment-start notice)

- Added the missing customer lifecycle notice when an admin creates an order fulfillment. The notice is queued only after the order status update succeeds, uses the durable outbox key `fulfillment/<fulfillment-id>/prepared`, and explicitly says the fulfillment has started but the order has not shipped.
- Unsupported fulfillment states remain silent. Focused tests cover the prepared copy and unsupported-state abstention.
- Updated ADR-034 readiness evidence. Other event types, provider/quota delivery gates, WABA/template deployment configuration, and the remaining ADR-032–036 acceptance work are still outstanding.
- Wired the remaining supported return receipt boundaries to real order-service transitions: receive, cancel receipt, receive confirmation, and request confirmation no longer return hard-coded fake state. Return receipt and cancellation notifications use tenant-scoped order resolution and stable return/status idempotency keys; confirmation wording explicitly does not promise a refund.
- Hardened return-receipt validation: unknown return lines, negative or inconsistent damaged quantities, duplicate-line overflow, and receipt totals above requested quantities now fail before any line is written. The receipt writes and status transition run under a tenant-scoped transaction with a return-row lock, and absolute receipt totals make a retried receive request idempotent. Updated stale return-flow regression expectations so positive refund confirmation must go through the payment service and cannot fabricate a USD ledger transaction. Added a database-free validation test plus database-backed flow assertions; the flow targets compile, but execution is blocked because the configured PostgreSQL endpoint is unavailable. The dashboard's staged receive-action persistence remains incomplete.

### Session continuation — 2026-10-24 (ADR-032 focused gate repair)

- Fixed the chat-offer contract test so all service calls use the explicit connection type, SQL fixture errors retain database context, and the disable/revoke assertion calls the existing audited soft-delete operation. Changed AI environment-test locks to async mutexes and moved the review row conversion/helper before the test module.
- Fixed compile blockers uncovered in the focused server auth-test dependency closure: cloned `store_name` before the social route consumes context, preserved the AI model name before moving provider configuration, and cloned the notification service before scheduler closure capture.
- `cargo check --locked -p stheno-ai --tests`: passed. `cargo clippy --locked -p stheno-ai --all-targets -- -D warnings`: passed. `cargo test --locked -p stheno-ai --lib`: passed (15 tests). `npm --prefix apps/admin run typecheck`: passed. `cargo check --locked -p stheno-server --test social_admin_confirmation_auth`: passed (one existing unused-variable warning in `stheno-api-admin-platform`).
- `cargo test --locked -p stheno-ai`: unit tests passed (15/15); the `chat_offer_contract` integration test could not connect to the configured PostgreSQL endpoint (`error connecting to server`). No test result is claimed for the database-backed contract. Space execution and remaining ADR-032–036 acceptance gates are still open.

### Session continuation — 2026-10-24 (ADR-032 server compile follow-up)

- The focused server test compile exposed ownership errors in the social assistant provider configuration, scheduler closure capture, and Store API route composition; fixed those moves. The same MSRV check found two Rust 1.82 `Option::is_none_or` calls in the new WhatsApp sender path despite the workspace's Rust 1.80 MSRV; replaced them with compatible `map_or` calls. The webhook composition function retains its explicit dependency list with a targeted Clippy allowance.
- `cargo check --locked -p stheno-server --test social_admin_confirmation_auth`: passed after the fixes (the dependency build prints one unrelated unused-variable warning in `stheno-api-admin-platform`). `cargo clippy --locked -p stheno-server --test social_admin_confirmation_auth --no-deps -- -D warnings`: passed.
- `cargo clippy --locked -p stheno-server --test social_admin_confirmation_auth -- -D warnings`: failed in local workspace dependencies with 57 `clippy::result_large_err` diagnostics plus an unused-variable diagnostic in `stheno-api-admin-platform`; no broad API lint suppression or response-type refactor was made. CI/Space toolchain parity remains to be checked.

### Session continuation — 2026-10-24 (ADR-034 staged return receipt)

- Replaced the admin receive/damaged-item placeholder handlers with tenant-scoped persisted `return_receive` order changes and item actions. Edit/remove validates the action belongs to that return and action kind; confirmation reconstructs absolute quantities and uses the existing locked, transactional return validator. Closing an unconfirmed receipt cancels its staged change. A retried confirmation repairs the order-change status if receipt persistence completed before interruption.
- The admin order preview now loads active order changes/actions and attaches receipt actions to matching line previews, so the dashboard reloads its staged values after navigation or refresh.
- `cargo check --locked -p stheno-api-admin --tests`: passed after the final service/API changes. `cargo clippy --locked -p stheno-api-admin --lib --no-deps -- -D warnings` and `cargo clippy --locked -p stheno-order --all-targets --no-deps -- -D warnings`: passed. `cargo test --locked -p stheno-order --lib`: passed (2 receipt validation tests). Admin TypeScript typecheck passed. Focused rustfmt check passes for the two Rust source files; the workspace-wide fmt check failed on Windows with OS error 206, and package fmt check finds formatting in `crates/modules/stheno-order/tests/cod_amounts.rs`.
- `python scripts/doc-sync-workflow.py check`: passed all five sections, including pricing/quota invariants, 34 module docs, provider registry, brand surfaces, and 567 relative links.
- Database-backed receipt workflow execution remains unverified. PostgreSQL is unreachable in this local environment; Space/CI delivery gates and WABA deployment configuration remain outstanding.

### Session continuation — 2026-10-24 (ADR-033 production quota wiring)

- Wired production Admin product creation and staff-seat creation through the existing quota workflow builders. Invite acceptance reserves a staff seat too, and invite acceptance now creates the user and consumes the invite token in one tenant-scoped transaction.
- Added exact tenant-scoped product and user counts in their owning modules (avoids the admin list page ceiling). Normal server startup reconciles counters before mounting/serving the app and fails closed on reconciliation errors; a daily 02:00 job repairs counter drift. Added the quota job schedule to the registered-cron test and updated ADR-033 implementation evidence.
- Product and staff deletes now use a single live-row soft-delete update, so concurrent repeated deletes cannot double-decrement; the production Admin handlers release the corresponding tenant counter after a successful delete.
- Closed the remaining Admin write bypasses: batch product creation/deletion and the first-user bootstrap now use the same quota reservation/release paths as the single-item routes.
- `cargo check --locked --offline -p stheno-api-admin -p stheno-server --tests`: passed after the final quota create/delete/reconciliation changes (one existing unused-variable warning in `stheno-api-admin-platform`). `cargo test --locked --offline -p stheno-api-router --test tier_matrix`: passed (10 tests). `cargo test --locked --offline -p stheno-server --lib scheduler::tests::registered_schedules_parse`: passed (1 test). Exact touched-file rustfmt check and documentation sync check passed. The repository SQL-boundary script was started but interrupted after it remained silent during its full module scan; the new raw SQL stays within each owning module's schema.
- After closing batch/bootstrap write paths, `cargo check --locked --offline -p stheno-api-admin -p stheno-server --tests` passed incrementally (same unrelated warning).
- Added production-handler regressions for product and staff create limits; `cargo check --locked --offline -p stheno-api-admin --test quota_routes` passed. `cargo test --locked --offline -p stheno-api-admin --test quota_routes` compiled, then both tests failed before exercising the handlers because `TestDb::migrated()` could not connect to PostgreSQL (`error connecting to server`).
- `cargo clippy --locked --offline -p stheno-api-admin -p stheno-server -p stheno-product -p stheno-user --lib --no-deps -- -D warnings`: passed. The corresponding `--all-targets` command fails only on the existing `clippy::items_after_test_module` layout in `stheno-server/src/lib.rs` (the trial-config test module precedes production items); the existing unrelated admin-platform unused-variable warning is also emitted from dependencies.

### Session continuation — 2026-10-24 (ADR-033/035 SSR branding)

- Storefront root layout now loads the tenant's saved `/store/branding` response without cache and forwards the incoming Host to the engine, which resolves tenant scope from Host. The response feeds SSR token emission; API failure falls back to platform defaults, and hostname mock themes remain development-only. CSS color, length, and material token allowlists prevent persisted values from injecting CSS or escaping the style element.
- Moved dev theme CSS options into core and passed them into the presentation-only theme switcher, removing its frontend-to-core internal import.
- `apps/storefront` local `tsc --noEmit`: passed. Focused `vitest` branding token suite: 6/6 passed. Storefront boundary, frontend purity, frontend resolution, and frontend-not-forked checks passed. The frontend pin/switch scripts could not run correctly under this WSL shell: Node is installed on Windows, but its stdin/path behavior is incompatible with those scripts; an attempted direct wrapper did not provide valid results. The standard Yarn invocation is also blocked by the existing lockfile missing the workspace package, so no dependency install/lockfile mutation was attempted.
- Still required: server-rendered response test, frontend pin/switch gate execution in a supported shell, persisted archetype/card-style selection and storefront application, and the rest of ADR-032–036 acceptance work.
- Still open in this workstream: the production route/database quota matrix. PostgreSQL is unavailable locally, so DB-backed workflow acceptance was not run. Space/CI gates and other ADR-032–036 acceptance items remain outstanding.

### Session continuation — 2026-10-24 (ADR-035 persisted storefront appearance)

- Added tenant-scoped archetype, product-card style, and image-aspect settings on the existing branding row, exposed via additive `/admin/store/appearance` and `/store/appearance` endpoints so frozen branding response shapes stay unchanged. Service allowlists and migration checks reject unsupported values.
- The admin branding screen now lets an operator explicitly save or reset these settings. SSR loads them per Host without caching and adds semantic attributes; the canonical frontend applies card/image variants in CSS and honors reduced-motion preferences. Category suggestions remain advisory and do not mutate the saved selection.
- `cargo check --locked --offline -p stheno-store -p stheno-api-admin -p stheno-api-store`: passed. `cargo test --locked --offline -p stheno-store --lib`: passed (2 appearance allowlist tests). Admin and storefront typechecks passed. Focused branding-token Vitest: 6/6 passed. Migration phase check and doc-sync workflow check passed.
- `cargo clippy --locked --offline -p stheno-store -p stheno-api-admin -p stheno-api-store --lib --no-deps -- -D warnings` failed on 57 existing `clippy::result_large_err` findings across unrelated Store API handlers; none point to the new appearance code. `scripts/check-schema-drift.sh` could not run because `DATABASE_URL` is unset. `cargo fmt --all -- --check` could not start on this Windows workspace (OS error 206); package-scoped format check also reported pre-existing formatting differences in unrelated Admin API files, so no whole-file formatter was applied to avoid reformatting user changes.
- ADR-035 source connection/freshness controls, database-backed endpoint authorization/tenant isolation, and broader ADR-032–036 acceptance gates remain open.

### Session continuation — 2026-10-24 (ADR-033 frontend pin/switch evidence)

- The WSL shell could not resolve the installed Windows `node.exe`, so the two frontend gate scripts could not run unchanged. Ran the pin script's manifest/version checks directly with native Node, and extracted/executed the switch gate's JavaScript directly with native Node. Both passed: default pins core `^0.1.0`, the fixture frontend also pins `^0.1.0`, and the default frontend binds all four checked module slots to the intended frontend/core paths with a unique `.next` directory.
- This supplies equivalent logic evidence, not a claim that the shell wrappers themselves ran. Server-rendered response verification and the remaining ADR-032–036 gates are still open.

### Session continuation — 2026-10-24 (ADR-036 selector fail-closed coverage)

- Added local tests proving manual `force_full` bypasses discovery and package-discovery failure launches the workspace test gate. The existing tests already cover reverse dependents, manifest-wide runs, docs-only skips, and unowned Rust paths.
- `python -m unittest discover -s scripts/tests -p 'test_affected_cargo_gate.py' -v`: passed (6 tests). This verifies selector behavior only; Space host trust isolation, exact hosted-gate parity, and remote restart/cancellation behavior remain unverified.

### Session continuation — 2026-10-24 (ADR-034 persisted return status)

- Replaced the fabricated `open` status in `GET /admin/returns/{id}` with the persisted return DTO status. Preserved the existing `{ return, data }` response keys and `{ id, status }` object shape; missing records return the established 404 error envelope.
- `cargo test --locked --offline -p stheno-api-admin --lib orders::return_lookup_error_tests`: passed (2 tests), covering 404 for a missing return and 500 for a database failure. This does not replace a database-backed handler contract test; return API authorization and tenant-isolation coverage remain open with schema/database gates.

### Session continuation — 2026-10-24 (ADR-035 owner/admin guard coverage)

- Extracted the existing Store Intelligence owner/admin role predicate into a directly testable helper and added a role allow/deny matrix for owner, admin, staff, manager, operator, customer, and empty roles. The production request guard still resolves the actor from tenant-scoped user service before applying that predicate.
- `cargo test --locked --offline -p stheno-api-admin --lib ai::authorization_tests`: passed (1 test). This is role-policy evidence only; it does not replace endpoint-level JWT/tenant database tests, which still require the database-backed acceptance environment.

### Session continuation — 2026-10-24 (ADR-035 local ranking failure coverage)

- Added Store Intelligence ranking regression cases for malformed and unknown-field JSON, unknown candidate IDs, invalid uncertainty, omitted candidates, and an oversized model response. Added an OpenAI-compatible provider fixture proving a response above 512 KiB fails without response-body echo or retry. Updated ADR-035.4 evidence accordingly.
- `cargo test --locked --offline -p stheno-ai --lib ranking_tests`: passed (3 tests).
- `cargo test --locked --offline -p stheno-ai-openai-compatible --lib tests::oversized_provider_response_is_rejected_without_echo_or_retry`: passed (1 test); first compile of this dependency closure took 1m31s.
- Broader ADR-035 acceptance remains open: source adapters/normalization, SQL tenant-isolation and endpoint authorization integration tests, DB-backed persistence tests, browser/dashboard journey, and deployment/Space gates.
- Expanded verification: `cargo test --locked --offline -p stheno-ai -p stheno-ai-openai-compatible --lib` passed (16 core + 7 provider tests); `cargo clippy --locked --offline -p stheno-ai -p stheno-ai-openai-compatible --all-targets -- -D warnings` passed. Package formatting check remains blocked by rustfmt differences in the pre-existing `stheno-ai/tests/configuration_secrets.rs` and unrelated formatting already present in the provider source; no whole-file formatter was applied to avoid broadening the diff.
- Closed the ADR-035.4 rank-slot behavior evidence gap by extracting the production semaphore acquisition helper and testing that a second request fails immediately while the first holds the slot, then succeeds after release. `cargo test --locked --offline -p stheno-api-admin --lib ai::authorization_tests`: passed (2 tests: role authorization and slot contention/recovery).

### Session continuation — 2026-10-24 (ADR-032 flow contracts)

- Ran focused Store Intelligence ranking/provider suites and API role/concurrency tests (recorded above in ADR-035.4 evidence).
- `cargo test --locked --offline -p stheno-flows --lib ai_`: passed (12 tests) covering bounded prompt context, untrusted retrieved text, typed read-only dispatch, malformed/parallel tool calls, refusal/provider failure, tool argument validation, unknown price/inventory, zero stock, currency/published-catalog checks, and offer timeout behavior. Updated ADR-032.2/.3 evidence; DB-backed live-service contracts, webhook route fixtures, provider behavior/cost limits, and Space gates remain open.

### Session continuation — 2026-10-24 (ADR-035 trend-window validation)

- Consolidated Store Intelligence order and category window validation into one pre-query validator and added tests for adjacent seven-day window acceptance, invalid comparison spacing, reversed/equal boundaries, and periods longer than seven days.
- `cargo test --locked --offline -p stheno-order --lib store_intelligence_window_tests`: passed (2 tests). Updated ADR-035.1 evidence; database aggregation/tenant-isolation fixture execution remains unverified.

### Session continuation — 2026-10-24 (ADR-032 / ADR-035 database acceptance)

- Confirmed the test database endpoint is loopback. Started an isolated, volume-free PostgreSQL 16 container for persistence tests; focused tests use `TestDb`, which creates uniquely named `stheno_test_*` databases and drops them at teardown.
- `cargo test --locked --offline -p stheno-order --test store_intelligence`: passed (1 DB integration test for tenant isolation, draft exclusion, product counts, and category totals). Window validator unit tests also pass (2).
- `cargo test --locked --offline -p stheno-ai`: passed (23 total: 16 unit tests and 7 persistence tests across chat offers, encrypted configuration, conversation memory, knowledge isolation, and webhook ordering). The first chat-offer fixture attempt violated the migration's `expires_at > issued_at` constraint; fixed the fixture to move both timestamps while preserving that invariant, then reran successfully.
- `cargo test --locked --offline -p stheno-api-store --test meta_webhooks`: passed (6 PostgreSQL-backed webhook route tests).
- `cargo test --locked --offline -p stheno-flows --test ai_product_offer --test ai_shipping_quote`: compilation first failed because C: had 0 free space. After verifying no Cargo/rustc process was active and measuring the shared generated target at 22.37 GiB, used `cargo clean -p stheno-flows` (Cargo removed 1.7 GiB; C: recovered 1.54 GiB) and retried. Compilation succeeded, but the first DB test remained idle for over four minutes while Docker Desktop stopped responding. Interrupted only that hung test process. Docker CLI removal/status calls also time out; port 55432 still reports a listener, so the temporary container may remain running. No additional Docker service was started. The flow DB fixtures remain unverified.
- Docker/container cleanup could not be confirmed because the Docker Desktop backend stopped responding; this must be resolved before further database-backed local tests. The volume-free test container has no mounted persistent data.
- After adding the database integration target, `rustfmt --check --edition 2021 crates/modules/stheno-order/tests/store_intelligence.rs` and `cargo clippy --locked --offline -p stheno-order --all-targets -- -D warnings` passed. The service source formatting check reports only existing import-order differences elsewhere in its file.

### Session continuation — 2026-10-24 (ADR-035 Search Console snapshots)

- Added a tenant-RLS current-observation table with a named query and additive Search Console kind constraint. Migration identifiers are unique and ordered after existing same-day migrations.
- Added `StoreIntelligenceSourceService` to validate tenant/source/property/metric identity and upsert provider observations by date window. The owner/admin refresh route persists the provider report; a bounded owner/admin snapshot route reloads the latest 100 observations from the past 400 days, and collection prunes older rows. Search Console disconnect removes the encrypted credentials and all saved observations in one transaction. The dashboard shows stored values, their date windows, coverage, and retrieval time.
- Extended the encrypted configuration integration fixture to persist and read two adjacent windows, then clear the observations with disconnect. The source-insights DB fixture compiles under all-target checks but cannot execute while `docker ps` times out; the test DB port responds at TCP level, which is not sufficient proof that the PostgreSQL service is usable.
- `cargo check --locked --offline -p stheno-ai -p stheno-api-admin --all-targets`: passed.
- `cargo test --locked --offline -p stheno-search-console`: passed (3 tests).
- `cargo test --locked --offline -p stheno-ai --lib`: passed (20 tests).
- `cargo clippy --locked --offline -p stheno-ai -p stheno-api-admin --all-targets -- -D warnings`: passed.
- `npm --prefix apps/admin run typecheck`: passed.
- Database-backed snapshot/disconnect execution, schema-drift acceptance, source audit fixture execution, other source adapters, and Space acceptance remain open.

### Session continuation — 2026-10-24 (ADR-035 source connection audit)

- Added tenant-RLS `ai.source_connection_audit` with an indexed owner-history query. Owner/admin Search Console saves record connect/update/disable actions, and disconnect records the actor while deleting credentials and observations in the same transaction. The bounded owner/admin audit endpoint and dashboard history contain no credential values.
- Extended the PostgreSQL configuration fixture to assert two-window history, tenant isolation for observations and audit records, actor attribution for connect/update/disable/disconnect, and disconnect cleanup. It compiles through all-target checks but remains unexecuted because Docker Desktop commands hang.
- `cargo test --locked --offline -p stheno-ai --lib`: passed (20 tests).
- `cargo test --locked --offline -p stheno-search-console`: passed (3 tests).
- `cargo clippy --locked --offline -p stheno-ai -p stheno-api-admin --all-targets -- -D warnings`: passed.
- `npm --prefix apps/admin run typecheck`: passed.
- Actor audit and source observation PostgreSQL execution, schema-drift checks, OAuth consent/callback, other source adapters, and Space acceptance remain open.

### Session continuation — 2026-10-24 (ADR-035 adjacent-window comparison)

- Added a deterministic exact-Decimal comparison for retained source observations. It selects only complete, equal-length adjacent windows with matching tenant, property, metric definition, unit, timezone, and denominator/sample shape; it reports no relative change for zero baselines or partial windows. Search Console snapshot responses now include these comparable changes, and the dashboard labels them as basis points.
- `cargo test --locked --offline -p stheno-ai --lib`: passed (21 tests), including matching, mismatched, partial, and adjacent-window comparison cases.
- `cargo test --locked --offline -p stheno-search-console`: passed (3 provider fixtures).
- `cargo clippy --locked --offline -p stheno-ai -p stheno-api-admin --all-targets -- -D warnings`: passed.
- `npm --prefix apps/admin run typecheck`: passed.
- PostgreSQL-backed history/audit fixture and schema-drift remain unexecuted due the Docker Desktop hang.

### Session continuation — 2026-10-24 (ADR-035 checked aggregation and source compatibility)

- Hardened Search Console Decimal aggregation and weighted-position math with checked operations so extreme provider values return a bounded malformed-response error instead of panicking.
- Added a shared cross-source aggregation helper restricted to complete additive counts with matching normalized metric definitions, units, windows, timezone, tenant, and distinct scope partitions. Duplicate source/account partitions, incompatible definitions, partial data, non-count units, and Decimal overflow fail closed.
- Documented the source unit and attribution rules in ADR-035: ratios and weighted averages are not directly additive, and providers must establish partition disjointness before aggregation.
- `cargo fmt --package stheno-search-console -- --check`: passed.
- `cargo test --locked --offline -p stheno-search-console`: passed (4 tests), including overflow rejection.
- `cargo clippy --locked --offline -p stheno-search-console --all-targets -- -D warnings`: passed.
- `cargo test --locked --offline -p stheno-ai --lib`: passed (22 tests), including disjoint cross-source counts and rejection cases.
- `cargo clippy --locked --offline -p stheno-ai --all-targets -- -D warnings`: passed.
- Meta adapters, OAuth consent/callback, adapter integration for common metric definitions, DB/schema acceptance, and Space gates remain open.

### Session continuation — 2026-10-24 (ADR-035 Meta account insights)

- Added `stheno-meta-insights` for read-only Facebook Page and Instagram professional-account aggregates, reusing the existing tenant-encrypted Messenger/Page and Instagram channel credentials. Facebook requests only `page_post_engagements` and `page_media_view`; Instagram requests only `total_interactions` and `reach`. Provider-specific metric definitions remain separate.
- Requests use a bounded 90-day window, 12-second timeout, 1 MiB response limit, up to five cursor pages, bearer authorization, and Meta `appsecret_proof`. The Graph API version is configurable with `STHENO_META_INSIGHTS_API_VERSION` and defaults to `v26.0`. Empty results are unavailable, missing metrics and incomplete pagination are partial, and provider errors return stable error classes.
- Added tenant-safe credential loading, owner/admin fetch and saved-history routes, and an admin panel for on-demand fetch, history, and adjacent-window changes. No posts, messages, demographics, or customer identities are collected.
- Official source contracts: [Facebook Page Insights](https://developers.facebook.com/docs/graph-api/reference/page/insights/), [Instagram Account Insights](https://developers.facebook.com/documentation/instagram-platform/api-reference/instagram-user/insights), [Meta permissions](https://developers.facebook.com/docs/permissions/), and [Meta secure Graph API calls](https://developers.facebook.com/docs/graph-api/guides/secure-requests/).
- `cargo test --offline -p stheno-meta-insights`: passed (5 tests), including pagination, empty results, tenant mismatch, permission denial, oversized bodies, and overflow.
- `cargo check --locked --offline -p stheno-api-admin --all-targets`: passed.
- `cargo clippy --locked --offline -p stheno-meta-insights -p stheno-ai -p stheno-api-admin --all-targets -- -D warnings`: passed; a follow-up Meta-only clippy pass also passed after adding overflow/body-limit coverage.
- `npm --prefix apps/admin run typecheck`: passed.
- Meta OAuth consent/callback and permission grant, consented WhatsApp aggregates, deployment app-review/scope configuration, API authorization fixtures, PostgreSQL persistence/schema acceptance, and Space gates remain open.


### Session continuation — 2026-10-24 (ADR-035 Search Console OAuth)

- Added owner/admin OAuth initiation and completion using the `webmasters.readonly` scope. Signed callback state binds tenant, actor, property, and issue time; a tenant-RLS nonce table permits one completion within ten minutes and prunes expired state on new attempts.
- The server exchanges authorization codes against Google's token endpoint with a 12-second client timeout and 1 MiB response bound, requires an offline refresh token, and saves client credentials plus refresh token through the existing encrypted configuration and actor audit transaction. The UI handles the Google callback on the authenticated AI settings page.
- The deployment must set `STHENO_SEARCH_CONSOLE_CLIENT_ID`, `STHENO_SEARCH_CONSOLE_CLIENT_SECRET`, and `STHENO_SEARCH_CONSOLE_REDIRECT_URI` to the exact HTTPS admin callback page (HTTP localhost is allowed for development). Helm now supports references to operator-created Kubernetes Secrets through `secretEnv`; credential values are not stored in chart defaults.
- Updated `MODULE.toml`, the AI table inventory, physical table count, and implementation/decision evidence for the new OAuth-state table.
- `cargo check --offline -p stheno-api-admin --all-targets`: passed; this also updated the existing lockfile dependency metadata, and the subsequent locked run passed.
- `cargo test --locked --offline -p stheno-search-console --lib`: passed (5 tests), including authorization-code exchange and refresh-token requirement.
- `cargo test --locked --offline -p stheno-ai --lib`: passed (22 tests).
- `cargo clippy --locked --offline -p stheno-api-admin -p stheno-search-console -p stheno-ai --all-targets -- -D warnings`: passed.
- `npm --prefix apps/admin run typecheck`: passed.
- `cargo test --locked --offline -p stheno-api-admin --lib search_console_oauth_state_is_signed_and_redirect_uri_is_trusted`: passed (1 focused test).
- `cargo check --locked --offline -p stheno-ai --all-targets`: passed, compiling the new PostgreSQL tenant-isolation and single-use OAuth state fixture without executing it.
- `python scripts/doc-sync-workflow.py check`: passed all five checks; 567 relative links resolve.
- `bash scripts/check-module-contracts.sh`, `bash scripts/check-module-isolation.sh`, `bash scripts/check-sql-boundaries.sh`, and `bash scripts/check-shared-kernel.sh`: passed.
- Helm rendering was skipped because Helm is not installed; the chart change only adds optional `secretKeyRef` environment values and has no default secret values.
- PostgreSQL-backed one-time-state/RLS migration execution, Google live consent/property acceptance, Meta-specific scopes/controls, consented WhatsApp aggregates, export/delete integration, and Space acceptance remain open.


### Session continuation — 2026-10-24 (ADR-035 consented WhatsApp aggregates)

- Added a cross-schema read in `stheno-query`, the only permitted query boundary, to match a separate explicit anonymous-analytics opt-in ledger to hashed inbound webhook sender keys. This ledger is distinct from transactional order-update consent. The optional checkout checkbox records analytics consent only when selected; authenticated customer revocation and WhatsApp STOP withdraw both consent types. The aggregate excludes revoked consent, events before consent, unprocessed events, and users without analytics consent. Only aggregate counts leave the query layer; phone numbers and hashes are not returned.
- The owner/admin route requires a configured WhatsApp account, limits requests to 92 past UTC days and 5,000 active consent records, records only partial coverage because webhook delivery completeness is unknown, and persists source-scoped observations. The dashboard exposes on-demand aggregate message and distinct-sender counts.
- Added an index on processed WhatsApp event account/time windows with the served query named in the migration comment. The PostgreSQL fixture covers active, revoked, pre-consent, and unknown senders, and ran successfully on Space.
- To honor the Space-only build/test requirement, removed the generated local Cargo target using `cargo clean` after confirming there were no active Cargo/rustc processes. It removed 59,155 generated files (28.7 GiB) from `C:\Users\Mahi\rust-target\debug`; C: recovered 28.49 GiB free. No source files or dependency stores were removed. All subsequent builds/tests below used `scripts/build-on-space.ps1` and its persistent remote target volume.
- Space `rust-package` checks passed for `stheno-api-admin` and `stheno-api-store`.
- Space `cargo test --locked -p stheno-query --test consented_whatsapp`: passed 2/2, including the PostgreSQL aggregate fixture for consent boundaries and processed events.
- Space library tests passed: `stheno-meta-insights` 5/5, `stheno-search-console` 5/5, and `stheno-ai` 22/22.
- Space `stheno-notification` integration target initially exposed malformed `INSERT ... SELECT ... ON CONFLICT` SQL, missing explicit tenant IDs on notification/delivery rows, and a race between DB fixtures creating the shared test role. Rewrote the outbox statements as multiline SQL, persisted tenant IDs explicitly, and serialized this integration target's DB fixtures. The final target passed 3/3: durable outbox idempotency, separate analytics-consent revocation, and template routing.
- Space `web-static` stopped during the Admin TypeScript check with Node V8 out-of-memory (exit 134) under the Space builder's 3 GiB web-container cap. Storefront typecheck and frontend static gates did not run in that invocation.
- Rust formatting passed on the final touched Rust files. The attempted documentation sync invoked `cargo run -p stheno-cli` locally; I stopped it immediately and ran the supported `cargo clean`, removing its 2,005 generated files (542.6 MiB) from the local C: target. No Cargo/rustc process remains. Documentation sync and static architecture boundary checks still need a Space-only execution path. PostgreSQL consent/RLS and outbox delivery behavior are exercised on Space; schema-drift acceptance, live Meta permission setup, export/delete integration, production acceptance, and remaining ADR phase gates remain open.

### Session continuation — 2026-10-25 (ADR-032 social draft conversion replay)

- Fixed order conversion to clear `is_draft_order` when a draft becomes pending. Social confirmation retries now return the already-pending social order before re-entering validation/conversion; this also covers older records created while the draft flag was left stale.
- Serialized test database initialization in the shared `TestDb` helper to prevent concurrent fixtures racing to create the same application role.
- Space `stheno-flows-order --test social_draft`: passed 10/10.
- Space `stheno-flows-order --test social_draft_convert`: passed 6/6, including repeated confirmation, source/channel preservation, missing-channel rejection, and non-draft rejection.
- The first wrapper attempt used the Space default login and correctly refused because it shares UID with the Actions runner and uses rootful Docker. Reran through the isolated `syed-build` account; UID/rootless isolation passed. Persistent Space target cache was retained.
- Rustfmt and `git diff --check` passed for the touched Rust changes. Remaining ADR acceptance work is tracked in the plan; this does not complete ADR-032 or the full implementation plan.

### Session continuation — 2026-10-25 (ADR-033 production quota routes)

- Ran `stheno-api-admin --test quota_routes` through the isolated Space builder with PostgreSQL/Redis services: passed 2/2. Both production product creation and staff creation enforce atomic tenant quota limits against PostgreSQL.
- The focused route matrix does not cover batch creation, first-user bootstrap, invite acceptance, or the quota-reconciliation repair job; those remain explicit acceptance gaps.

### Session continuation — 2026-10-25 (ADR-033 renewal recovery)

- Ran `stheno-platform --test billing_run` on Space with PostgreSQL/Redis services. The initial run failed all three cases because the shared fixture omitted the required `USD/BDT` FX row. Seeded a deterministic FX rate via `PlatformService::upsert_fx_rate` in test setup.
- Reran the same target: passed 3/3, including renewal payment during grace, a paid-through date beyond the former deadline, and no downgrade when the old grace deadline is reconciled.
- `rustfmt` and `git diff --check` passed for the test fixture change.

### Session continuation — 2026-10-25 (ADR-032 staff authorization)

- Fixed the social confirmation authorization fixture to enable the buyer-account entitlement for its isolated test tenant before registering a customer. The previous 404 correctly reflected disabled buyer accounts and did not reach the intended role assertion.
- Corrected the authenticated staff request to use POST, matching the conversion route; GET returned 405 and had not tested handler access.
- Space `stheno-server --test social_admin_confirmation_auth --CargoTestServices`: passed 1/1. Missing/invalid staff credentials return 401, a customer token returns 403, and authenticated staff reaches the conversion handler (404 for the intentionally nonexistent order).

### Session continuation — 2026-10-25 (ADR-034 receipts and manual proof)

- Space `stheno-flows --test return_receive --CargoTestServices`: passed 1/1 for receive, cancel, and lifecycle behavior.
- Space `stheno-flows --test return_partial --CargoTestServices`: passed 1/1 for partial receipt and damaged quantity handling.
- Space `stheno-platform --test payment_proof_truth --CargoTestServices` initially exposed an invoice status constraint mismatch: service logic persisted `pending_verification`, while the database check constraint rejected it. Added migration `20261025000001_invoice_pending_verification_status`, with a guarded down migration that refuses rollback while pending invoices exist.
- Reran the proof target on Space: passed 2/2 for pending proof status, rejection of malformed references, one pending proof per invoice, and prevention of paid-invoice reopening.
- Refund provider integration and broader receipt concurrency/authorization acceptance remain open.

### Session continuation — 2026-10-25 (ADR-032 courier preflight)

- Space `stheno-api-admin --lib --CargoTestServices`: passed 19/19, including courier credential envelope validation, exact destination mapping checks, and the DB-backed invalid-courier preflight asserting that the social draft remains unconverted.
- Combined with the server authorization target, staff/customer/missing-token gates have executable evidence. Provider handoff ambiguity recovery and successful staff dispatch through the DB-backed route remain unverified.

### Session continuation — 2026-10-25 (ADR-034 Store API parity)

- Space `stheno-api-store --test return_parity --CargoTestServices`: passed 1/1.
- Space `stheno-api-store --test order_parity --CargoTestServices`: passed 1/1.
- Both frozen Store API response fixture targets remain green for the affected order and return endpoints.

### Session continuation — 2026-10-25 (ADR-032 AI persistence contracts)

- Space `stheno-ai --test chat_offer_contract --CargoTestServices`: passed 1/1 for conversation-bound quotes and revocation after offer change.
- Space `stheno-ai --test knowledge_isolation --CargoTestServices`: passed 1/1 for tenant isolation, deduplication, and soft deletion.
- Space `stheno-ai --test conversation_memory --CargoTestServices`: passed 1/1 for encryption, bounded history, deduplication, and tenant isolation.
- Space `stheno-ai --test configuration_secrets --CargoTestServices`: passed 2/2 for fail-closed configuration and encrypted/redacted/rotatable provider and Meta secrets.
- Space `stheno-ai --test webhook_event_ordering --CargoTestServices`: passed 2/2 for tenant binding, malformed event IDs, deduplication, and suppression of out-of-order messages.

### Session continuation — 2026-10-25 (ADR-032 Meta webhook ingress)

- Space `stheno-api-store --test meta_webhooks --CargoTestServices`: passed 6/6. Coverage includes bad signatures, account/tenant binding, duplicate and out-of-order events, WhatsApp/Instagram channel/account IDs, GET verification, retryable handoff failure, and malformed/unsupported/oversized deliveries.
- Rate-limit and live provider acceptance remain open.

### Session continuation — 2026-10-25 (ADR-032 typed AI tools and provider)

- Space `stheno-ai-openai-compatible --lib`: passed 7/7 for typed tool-call parsing, refusal normalization, timeout/malformed response handling, 429/5xx, size bounds, no retry, and no response-body/secret echo.
- Space `stheno-flows --test ai_product_offer --CargoTestServices`: passed 1/1 for live configured price authority and unknown stock handling.
- Space `stheno-flows --test ai_shipping_quote --CargoTestServices`: passed 1/1 for current eligible rates and region/currency validation.

### Session continuation — 2026-10-25 (ADR-032/033 focused evidence reconciliation)

- Space `stheno-ai-openai-compatible --lib`: passed 7/7 for parsing, refusal, timeout/malformed response, 429/5xx, size bounds, and no retry/body echo.
- Space `stheno-flows --test ai_product_offer --CargoTestServices`: passed 1/1 for pricing authority, published catalog behavior, and unknown stock.
- Space `stheno-flows --test ai_shipping_quote --CargoTestServices`: passed 1/1 for eligible live rates and region/currency checks.
- Space `stheno-api-admin --test quota_routes --CargoTestServices`: passed 2/2 for product and staff quota routes.
- Space `stheno-platform --test billing_run --CargoTestServices`: passed 3/3, including renewal after the former paid-term deadline.
- Updated the implementation plan to replace stale pending Space claims and record these executed gates. Browser journeys, the full quota matrix, successful DB-backed courier dispatch/ambiguity recovery, deployment credentials, and remaining phase acceptance are still open.
- Attempted Space `stheno-flows --test quota_boundary --CargoTestServices`; the wrapper stopped before starting the container because available Space memory was 4,086,204 kB and the configured minimum is 4,352 MiB. No test ran. Later reads fell to 3,246,424 kB and then 1,377,392 kB, so I left host processes and caches untouched; retry when Space has enough headroom.
- Resumed once Space recovered: `stheno-flows --test quota_boundary --CargoTestServices` passed 1/1 on Space, rejecting the 11th product at the exact limit of 10. The persistent target cache was reused and increased from about 7.03 GB to 7.04 GB; no volume, image, dependency cache, or target cleanup occurred.
- The next Space `stheno-flows --test quota_reconciliation --CargoTestServices` invocation stopped before container start on the wrapper's transient managed-network cleanup race. Read-only inspection confirmed no managed container/network remained; available memory then fell below the 4,352 MiB launch threshold, so the test did not run and was not retried while the host was short on memory.
- Extending the shared Space `quota_routes` target produced 4 passes and one test-assumption failure: the test used a zero quota, which the entitlement contract defines as unlimited. Changed the test to seed a full one-seat counter instead. The added bootstrap, invite-acceptance, batch-create, and delete-counter cases otherwise passed in that run.
- Reviewing the delete paths exposed a real atomicity gap: product/staff soft-delete and counter decrement were separate transactions. Added quota-aware service operations that perform both writes inside `with_tenant_conn`'s transaction, routed individual and batch product deletes plus staff deletes through them, and added trigger-injected rollback checks. The updated `quota_routes` target needs a Space rerun; the host currently has less than the 4,352 MiB container minimum, so these latest changes are not yet verified.
- Later Space checks also timed out over SSH while the host reported under 2 GiB available. No local build was substituted.

### Session continuation — 2026-10-25 (ADR-033 transactional quota deletion)

- Reran `stheno-api-admin --test quota_routes --CargoTestServices` on Space after moving product and staff soft-delete plus quota decrement into one tenant transaction.
- Passed 5/5: bootstrap is idempotent and fails closed at a full seat limit; invite acceptance consumes one seat and preserves a second invite when full; batch product creation enforces per-item quota; production product/staff routes enforce limits; trigger-injected counter-decrement failures roll back deletion, while successful deletes release the counter.
- The Space Rust build used its persistent Cargo target cache; no cleanup was performed. Cargo finished the dev profile in 3m39s and the five DB-backed tests in 68.66s.
- Retried `stheno-flows --test quota_reconciliation --CargoTestServices` on Space after capacity recovered: passed 1/1 in 4.55s, verifying drift detection and exact reconciliation. Its dev profile finished in 57.22s using the persistent target cache. The earlier wrapper cleanup race occurred before test startup and did not recur.

### Session continuation — 2026-10-25 (ADR-033/035 focused database acceptance)

- Space `stheno-order --test store_intelligence --CargoTestServices`: passed 1/1 for tenant separation, draft-order exclusion, exact product counts, and independent category totals.
- Space `stheno-flows --test seat_quota --CargoTestServices`: passed 1/1 for staff-seat creation through the quota-check flow.
- Updated the implementation plan to record only the acceptance evidence executed on Space. Endpoint authorization matrices, release/browser journeys, and other explicitly listed ADR gates remain open.

### Session continuation — 2026-10-25 (ADR-032 courier idempotency)

- Added `stheno-fulfillment --test social_dispatch_idempotency` to exercise the DB-backed provider boundary: successful dispatch persists one fulfillment, confirmation replay reuses it, and an ambiguous response after provider acceptance blocks a second call.
- First Space run passed the ambiguous-retry case and exposed a missing shipping-option fixture in the success case. Made the optional shipping option absent in this service-level fixture and reran on Space: passed 2/2 (32.38s).
- Corrected malformed whitespace in the unsettled-provider error message. The target verifies fail-closed duplicate prevention; it does not yet provide the operator/provider reconciliation action for resolving an unknown request. Authenticated route-level dispatch and that recovery path remain open.

### Session continuation — 2026-10-25 (ADR-032 unknown courier reconciliation)

- Added a tenant-scoped operator resolution API for an unknown courier request. `accepted` stores only bounded canonical consignment/tracking fields; `not_created` enables a retry by claiming the same durable request row. Both resolutions use a compare-and-swap from `unknown` and record staff actor, evidence reference, outcome, and timestamp. The staff actor uses `actor_id` when present and the authenticated JWT subject otherwise.
- Added DB-backed fulfillment cases for accepted-result recovery without a second provider call, verified absence followed by one retry/replay, and concurrent conflicting resolution. Space `stheno-fulfillment --test social_dispatch_idempotency --CargoTestServices` passed 5/5.
- Space `stheno-server --test social_admin_confirmation_auth --CargoTestServices` first failed because the staff JWT omitted optional `actor_id`; after falling back to its non-empty subject, it passed 1/1. The test covers 401 without credentials, 403 for a customer, and staff reaching the order lookup.
- Added an Admin API integration fixture that creates a social order and unknown courier request, resolves an accepted consignment over HTTP, and verifies a repeated settlement is rejected. Space `stheno-api-admin --test social_courier_reconciliation --CargoTestServices` passed 1/1 after correcting the fixture to use a prefixed ULID request ID.
- The Space Cargo target cache was reused (about 7.19 GB); no cache, volume, image, or target cleanup occurred. The actual staff draft-conversion/courier-dispatch HTTP path still needs a successful route-level fixture; live courier confirmation remains an operational acceptance item.

### Session continuation — 2026-10-25 (ADR-035 Meta rate-limit behavior)

- Added a WireMock regression confirming Meta HTTP 429 maps to `RateLimited` and the adapter makes exactly one provider request.
- Space `stheno-meta-insights --lib` passed 6/6. Rust formatting ran locally; no local build or test was run.
- Updated ADR-035.2 evidence. Separate Meta insights authorization/connection controls, export/delete workflow integration, browser, and deployment acceptance remain open.

### Session continuation — 2026-10-25 (ADR-032 courier rate limits)

- Added HTTP 429 shipment-create fixtures for Pathao, RedX, and Steadfast; each verifies the provider returns an error without retrying the shipment request.
- Space `stheno-fulfillment-pathao --test consignment`, `stheno-fulfillment-redx --test provider`, and `stheno-fulfillment-steadfast --test provider` each passed 4/4.
- Updated ADR-032.5 and the total closure plan. The remaining courier acceptance requires live provider credentials and deployment configuration.

### Session continuation — 2026-10-25 (ADR-033 concurrent staff seats)

- Added a database-backed race test that runs two staff-seat quota sagas concurrently at a limit of one and verifies exactly one user is created, the counter remains one, and the other request gets the seat-limit conflict.
- Space `stheno-flows --test seat_quota --CargoTestServices` passed 2/2 (including the existing exact-cap test).
- Updated ADR-033. A real browser journey and broader release acceptance remain open.

### Session continuation — 2026-10-25 (ADR-034 concurrent receipt)

- Extended the DB-backed return receipt lifecycle test with two simultaneous identical absolute receipt writes. Both requests succeed and the persisted total remains one received unit.
- Space `stheno-flows --test return_receive --CargoTestServices` passed 1/1, including the new concurrent path and prior invalid-batch, replay, cap, refund-truth, and quantity validations.
- Updated ADR-034 evidence. The broader receipt authorization matrix remains open.

### Session continuation — 2026-10-25 (ADR-034 Stripe refund reconciliation)

- Extended the payment-provider trait with backward-compatible defaults for keyed refund requests and provider-specific reconciliation. Providers without a proven lookup still fail closed on ambiguous refunds.
- Stripe refund creation now sends the durable `Idempotency-Key` and stores that opaque key in refund metadata. Reconciliation lists refunds for the PaymentIntent with a bounded response, then accepts exactly one result only when its key, payment, minor-unit amount, lowercase currency, and status (`pending` or `succeeded`) match. Missing, duplicate, mismatched, failed, or unreadable evidence leaves the request unresolved.
- Added Stripe wiremock tests for keyed creation and exact lookup matching, plus payment-ledger tests that recover an ambiguous refund, cache/replay the reconciled result, and retain fail-closed behavior when a provider cannot reconcile.
- Space `stheno-payment-stripe --test refund_reconciliation` passed 2/2. Space `stheno-payment --test ambiguous_timeout --CargoTestServices` passed 3/3.
- Based on Stripe's official [idempotent requests](https://docs.stripe.com/api/idempotent_requests) and [list refunds](https://docs.stripe.com/api/refunds/list) API behavior. Live Stripe account acceptance remains open.

### Session continuation — 2026-10-26 (ADR-034 return-receipt authorization)

- Added a server-level auth regression for staged return receipt confirmation. Missing credentials return 401, a customer token returns 403, and a staff token passes the admin auth layer and reaches the handler.
- Space `pwsh -NoProfile -File scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-server -CargoTestTarget return_receipt_auth -CargoTestServices` passed 1/1. The first source snapshot was 358 MB and took about 15 minutes to transfer; compilation took 9m 10s with the existing Space Cargo target cache. The cache grew from about 7.63 GB to 7.76 GB. No cache cleanup was performed.
- Future Space runs in this pass will use a D: temp staging folder to keep source snapshots out of C:.

### Session continuation — 2026-10-26 (ADR-035 separate Meta Insights credentials)

- Meta Insights now uses dedicated encrypted configuration kinds for Facebook Page and Instagram metrics instead of reusing messaging channel credentials. Owner/admin save, disable, fetch, audit, and disconnect paths are wired; disconnect removes retained observations and credentials in one transaction.
- The admin page identifies required read-only permission sets, keeps the insights token separate from messaging, and exposes enable/disable, disconnect, and connection history controls.
- Space `pwsh -NoProfile -File scripts/build-on-space.ps1 -Mode rust-package -CargoPackage stheno-ai -CargoTestTarget configuration_secrets -CargoTestServices` passed 2/2, covering the separate credential boundary, disable/re-enable, audit actor, and disconnect cleanup.
- Space source staging for this run used `D:\Syed Systems\BuildStaging`; Cargo build/test ran on Space and no C: build artifacts or cache cleanup were used.
- Admin API compile and dashboard typecheck are still pending this session.

### Session continuation — 2026-10-26 (ADR-035 tenant export and erasure)

- The platform tenant export now includes redacted Store Intelligence source configuration, all retained source observations, connection audit, recommendation review decisions and append-only review audit, and hashed WhatsApp analytics consent. Credentials and transient OAuth state are excluded. Platform purge invokes a cross-module workflow that erases the source, review, OAuth-state, and analytics-consent rows under tenant RLS before deleting the tenant.
- Added a server integration target covering anonymous/customer/staff/owner authorization, secret redaction, cross-tenant export isolation, review/consent/source export, and post-purge erasure. Updated the platform purge regression to the cleanup callback.
- On Space, `cargo check --locked -p stheno-server --target x86_64-unknown-linux-gnu --profile dev` passed. `cargo test --locked -p stheno-server --test meta_insights_authorization --no-run --target x86_64-unknown-linux-gnu --profile dev` and `cargo test --locked -p stheno-platform --test tenant_purge --no-run --target x86_64-unknown-linux-gnu --profile dev` both passed. The workspace lockfile required three updated local-package dependency edges before the locked check passed.
- Database-backed execution is pending the Space services preflight: latest available memory was 4,280 MiB, below its 4,352 MiB requirement. Source staging used `D:\Syed Systems\BuildStaging`; no build or test ran on C:, and no caches were cleared.

### Session continuation — 2026-10-27 (ADR-035 Space preflight)

- The Space database-service preflight stopped before creating a build container: 4,214 MiB was available against the 4,352 MiB requirement. The authorization/export/purge and platform-purge test targets remain compiled but not executed against PostgreSQL.
- No database service was started or stopped, and no cache was cleared. No build or test ran on C:.
- On Space, module isolation, SQL boundaries, module contracts, DTO separation, migration phasing, and domain-gating checks passed via bash. The provider documentation check passed with 0 drift. The relative-link check passed for 566 links after replacing a broken link to a separate Space-repository document with a plain cross-project reference.
- Strict Clippy stopped in unchanged stheno-db on five clippy::uninlined_format_args errors under -D warnings, before the changed server target was reached.
- The full documentation-sync script remains incomplete: Space host Python cannot find Cargo, and the separate brand PowerShell check is unavailable on that Linux host. Its pricing/quota and module inventory checks passed in the initial run; provider drift and relative links passed separately.

### Session continuation — 2026-10-27 (Space host recheck)

- A read-only check through the Space SSH alias confirmed Docker uses `/var/lib/docker` and reports no `name=rootless` security option. The shared builder correctly rejects this daemon at its isolation preflight; the host is not currently ready for isolated build execution.
- Space reported 4,172 MiB available, below the 4,352 MiB database-service threshold. Existing containers (including the long-running PostgreSQL service) were left running; no service was started or stopped and no cache or workspace was cleaned.
- ADR-036's implementation status now records that the dedicated account does not by itself prove rootless daemon isolation. Trusted routing and complete host gate parity remain off/pending.

### Session continuation — 2026-11-13 (ADR-037 Cloudflare edge readiness)

- Implemented tenant-prefixed catalog cache tags, vendor-neutral country header handling, host-only admin session cookies, engine admin response headers, a static Caddy/Tunnel Compose and Helm path, origin/cache-safety test targets, an S3-compatible signed provider with separate public/private buckets, and the Cloudflare edge runbook. The runbook rate-limit and callback paths were read from the current route definitions.
- Final WP3 review added the tenant listing tag for `/[country]/products`, `/[country]/collections`, and `/[country]/categories` index paths; the middleware test covers the product listing path. The focused Vitest command remains unrun because Space execution failed before the remote checks started.
- WP1 criterion 7 is interpreted as preventing a storefront-host `/admin/*` request from reaching the engine or admin upstream; the required parity 404 is rendered by the storefront fallback. A literal rule that `/admin/*` reach no upstream conflicts with criterion 4 unless the proxy can reproduce the storefront's unknown-route body.
- Durable tag/hostname purge delivery is incomplete. The existing leased outbox is `notification.notification_outbox`; it is notification-specific and there is no generic catalog event/outbox path in the authorized module scopes. No purge subscriber or retry/dead-letter worker was added. The current product L1 cache is tenant-keyed with a five-minute TTL and has no catalog edit invalidation; Next.js route cache tags remain generic and no invalidation hook was found. WP2 criteria 12–15 are unmet; criterion 16 has mock tests for success, hostname, 429/5xx, timeout, no-op and batching, but they were not run and tenant concurrency, coalescing, retries and dead-lettering are absent.
- The admin cookie is `__Host-stheno-admin-session`; existing bearer sessions remain accepted until token expiry. The engine applies CSP to `/admin/*`, but the static Vite dashboard is a separate upstream and does not receive that engine middleware.
- Work-package ladder: WP2 rung 2 (reuse the cache helper; the existing outbox is not a generic purge outbox); WP3 rung 2 (reuse the existing middleware and response-header path); WP4 rung 2 (reuse Host-scoped JWT verification); WP1 rung 7 (no existing reverse-proxy/Tunnel configuration); WP5 rung 2 (reuse `TestApp`); WP6 rung 7 using already-declared HTTP and HMAC/hash dependencies (the existing S3 provider was only an in-memory stub); WP7 rung 2 (reuse ADR-037 and current route definitions).
- The required remote package check did not start: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File D:\Syed Systems\Store\scripts\build-on-space.ps1 -SpaceRepositoryRoot D:\Syed Systems\Space -Mode rust-package -CargoPackage stheno-cdn-cloudflare` returned `You cannot call a method on a null-valued expression` from `D:\Syed Systems\Space\scripts\build-on-space.ps1` before SSH/build execution. No Cargo, TypeScript, or test command ran on this laptop.
- `docker compose config --quiet` was attempted as a non-build config validation and failed with Docker config access denied plus `unknown flag: --quiet`. `helm lint deploy/chart` was not available (`spawn helm ENOENT`). Rust formatting was applied only to the changed S3 provider source and its test file.
- The listed lightweight bash checks (`check-module-isolation.sh`, `check-sql-boundaries.sh`, `check-migration-phases.sh`, and `check-docs-links.sh`) could not start because the Windows Bash service returned `Access is denied` (`Bash/Service/CreateInstance/E_ACCESSDENIED`). Schema drift was not run because its script invokes Cargo tests and requires a database.
- No index, commit, stash, reset, or cache was modified; only scoped working-tree files changed.
- The standalone `docker-compose -f deploy/proxy/compose.check.yml config -q` check passed. `docker-compose config -q` also exited 0 with only the expected missing runtime-variable warnings for the admin suffix and Tunnel token.
- `Cargo.lock` was not changed because it is outside the listed scopes. Its current `stheno-file-s3` entry predates the new direct dependencies, and it has no `stheno-cdn-cloudflare` entry; a Cargo gate will require resolving that scope conflict.
- Existing dashboard registrar-help text still names Cloudflare in `apps/admin/src/routes/domains/domains-view.tsx:80,469,547-575`, `apps/admin/src/routes/help/help-center.tsx:66`, and `apps/admin/src/routes/release-notes/release-notes-view.tsx:45`; those files are outside WP4's allowed scope and were not changed.
- Graphify CLI query/update could not run (`ModuleNotFoundError: No module named graphify`); graph.json was read manually for navigation, and `graphify-out/` is outside the task's allowed file scopes.
