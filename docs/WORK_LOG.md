# System Work Log & Version History

Chronological ledger of platform increments, surface adaptations, and documentation
synchronizations. Enforced by `python scripts/doc-sync-workflow.py`.

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


---