# Vendryx Architecture & Documentation Hub

Welcome to the **Vendryx Commerce** documentation repository. This directory contains the authoritative technical specifications, architectural decision records, wire contracts, compliance audits, and execution logs for the platform.

---

## 1. Document Authority & Precedence Rules

When specifications conflict or requirements appear ambiguous, **the higher level in this hierarchy strictly supersedes the lower** (per [AGENTS.md](../AGENTS.md)):

| Tier | Source of Authority | Description |
|---|---|---|
| **Level 1** | [DECISIONS.md](DECISIONS.md) | **Architecture Decision Records (ADRs).** All technical decisions and their reversals are settled here first. |
| **Level 2** | [STATUS.md](STATUS.md) | **Live Completion Ledger.** The sole source of truth for phase completion and task metrics. |
| **Level 3** | [PRODUCT.md](PRODUCT.md) · [ENTITLEMENTS.md](ENTITLEMENTS.md) · [BRAND.md](BRAND.md) | **Commercial & Product Truth.** Governs pricing, feature tier gating, customer-facing brand, and operator surfaces. |
| **Level 4** | [ARCHITECTURE.md](ARCHITECTURE.md) · [TRD.md](TRD.md) · [PRD.md](PRD.md) | **Technical Engine Specifications.** Defines rust engine topology, performance budgets, and module boundaries. |

---

## 2. Documentation Directory Map

```
docs/
├── README.md                                 # This navigation hub
│
├── 1. Core Architecture & Requirements
│   ├── ARCHITECTURE.md                       # Master system architecture & two-block topology
│   ├── DECISIONS.md                          # Architecture Decision Records (ADR-001 through ADR-031)
│   ├── TRD.md                                # Technical Requirements Document & crate graph
│   ├── PRD.md                                # Product Requirements Document & engine budgets
│   └── PRODUCT.md                            # Merchant personas, social commerce & workflows
│
├── 2. Domain Models & Wire Contracts
│   ├── DATA_MODEL.md                         # 127 entities, link tables & schema ownership
│   ├── API_SPEC.md                           # Store and Admin API wire shapes (Medusa v2 parity)
│   ├── WORKFLOWS.md                          # Postgres durable SAGA engine & 249-workflow catalog
│   ├── PAYMENTS.md                           # Payment traits, idempotency protocols & edge atlas
│   ├── ENTITLEMENTS.md                       # Plan tier quotas & route-level entitlement gating
│   └── SITEMAP.md                            # Complete URL taxonomy across Storefront, Admin, Marketing
│
├── 3. Brand & Frontend Presentation
│   ├── BRAND.md                              # Authoritative corporate identity & design tokens
│   ├── BRAND_GUIDELINES.md                   # Brand guidelines quick-reference
│   ├── FONTS.md                              # Typography licensing & font stack packaging
│   ├── FRONTEND_VERSIONING.md                # Storefront release & compatibility rules
│   └── VENDRYX_SCREEN_MATRIX.md              # UI screen catalog & navigation maps
│
├── 4. Security, PCI & Compliance
│   ├── SECURITY.md                           # Threat model, controls, 29 audit findings & authz matrix
│   ├── PCI_SAQ_A.md                          # PCI DSS SAQ-A merchant compliance verification
│   ├── PEN_TEST.md                           # Penetration testing plan & remediation reports
│   ├── THREAT_MODEL_CHECKOUT.md              # Checkout attack surface analysis
│   ├── DISCLOSURE_POLICY.md                  # Responsible vulnerability disclosure policy
│   └── payment-page-scripts.md               # Authorized script inventory & CSP manifest
│
├── 5. Operations, Commercial & Execution
│   ├── STATUS.md                             # Single source of truth for project completion
│   ├── COMMERCIAL_IMPLEMENTATION_PLAN.md     # Subscription pricing, FX rates, and courier settlement
│   ├── CONTROL_PLANE.md                      # Operator console specification (D1-D10 domains)
│   ├── PAYMENTS_SETUP.md                     # Gateway onboarding & credential encryption runbook
│   ├── MIGRATION_PLAYBOOK.md                 # Postgres migration & schema verification playbook
│   ├── DEPLOYMENT.md                         # Infrastructure topology & container orchestration
│   ├── DOCS_WORKFLOW.md                      # Documentation sync protocol & verification gates
│   ├── WORK_LOG.md                           # Chronological version-by-version release ledger
│   ├── providers/                            # Provider integration specs (bKash, Nagad, Pathao, RedX)
│   │   ├── PAYMENT_GATEWAYS.md               # Payment provider contracts
│   │   └── COURIERS.md                       # Courier provider contracts
│   └── runbooks/                             # Operational incident runbooks
│       ├── gateway-outage.md                 # Gateway outage resolution
│       ├── reconciliation-discrepancy.md     # Courier discrepancy resolution
│       └── stuck-consignment.md              # Stranded parcel resolution
│
├── 6. Active Tasks & Backlog
│   ├── PHASE4_TASKS.md                       # Phase 4 platform control plane task order
│   ├── PLATFORM_COMPLETION_TASKS.md          # 23 transitional platform handler remediation tasks
│   └── TASKS.md                              # Phase 0-10 historical baseline & §1 Loop Protocol
│
└── 7. Historical Archive (docs/archive/)
    ├── PHASE2_PLAN.md                        # Phase 2 persistence & durability plan
    ├── PHASE2_TASKS.md                       # Phase 2 tasks (P11–P20)
    ├── PHASE3_PLAN.md                        # Phase 3 providers & hardening plan
    ├── PHASE3_STATUS.md                      # Phase 3 evidence ledger (95/95 complete)
    ├── PHASE3_TASKS.md                       # Phase 3 tasks (P21–P28)
    └── ...                                   # Legacy frontend audits and exploration plans
```

---

## 3. Engineering Reading Order by Role

### 🛠️ Backend / Rust Engineers
1. [AGENTS.md](../AGENTS.md) — Fundamental build rules, module isolation, and connection conventions.
2. [ARCHITECTURE.md](ARCHITECTURE.md) — Master topology and crate dependency graph.
3. [DECISIONS.md](DECISIONS.md) — ADR-001 (Compile-time extensibility), ADR-003 (SAGA workflows), ADR-012 (Postgres RLS tenancy).
4. [DATA_MODEL.md](DATA_MODEL.md) & [WORKFLOWS.md](WORKFLOWS.md) — Module schemas and transaction orchestrations.
5. [PAYMENTS.md](PAYMENTS.md) — Gateway implementations, idempotency keys, and money arithmetic.

### 🎨 Frontend Engineers (Storefront, Admin, Marketing)
1. [AGENTS.md §12](../AGENTS.md#12-frontends-visual-theming) — Visual theming rules and forbidden frontend forks.
2. [DECISIONS.md ADR-031](DECISIONS.md#adr-031) — Dynamic Multi-Tenant Visual Theming on the Unified Default Storefront.
3. [DECISIONS.md ADR-029](DECISIONS.md#adr-029) — Merchant Admin Truth Contract & Apple HIG Information Architecture.
4. [BRAND.md](BRAND.md) — Authoritative brand tokens, typography, and asset inventory.
5. [API_SPEC.md](API_SPEC.md) & [SITEMAP.md](SITEMAP.md) — Wire contracts and URL taxonomy.

### 🔒 Security, Compliance & DevOps Engineers
1. [SECURITY.md](SECURITY.md) — Authorization matrix, threat model, and cryptographic boundaries.
2. [PCI_SAQ_A.md](PCI_SAQ_A.md) & [payment-page-scripts.md](payment-page-scripts.md) — Iframe/redirect compliance and CSP rules.
3. [DEPLOYMENT.md](DEPLOYMENT.md) — Production Docker Compose, PgBouncer pooling, and CDN caching.
4. [STATUS.md](STATUS.md) & [WORK_LOG.md](WORK_LOG.md) — Operational verification ledgers.

---

## 4. Documentation Verification Gate

All documentation must remain strictly synchronized with code and configuration. Before committing, run:

```bash
python scripts/doc-sync-workflow.py check
```
*(On Windows PowerShell: `.\scripts\doc-sync-workflow.ps1 check`)*

This gate asserts:
1. **Pricing Invariants:** Pro ($20/mo) and Free (100 products / 3 seats) match `config/tiers.toml`.
2. **Module Coverage:** All 33 Rust domain modules are documented in `TRD.md` or `DATA_MODEL.md`.
3. **Provider Synchronization:** Compiled providers match `docs/providers/PAYMENT_GATEWAYS.md`.
4. **Brand Surfaces:** All public surfaces present **Vendryx** per ADR-028.
5. **Link Integrity:** 100% of relative Markdown links resolve cleanly to active files.
