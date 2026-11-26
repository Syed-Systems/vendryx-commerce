# Vendryx — Commerce Operating System by Syed Systems

> **Vendryx** is the modern, multi-tenant commerce operating system designed for social commerce sellers and growth merchants.
> The repository retains internal `stheno-*` identifiers for engine crates and wire compatibility (governed by [ADR-028](docs/DECISIONS.md#adr-028)).
>
> **Project Completion Authority:** [docs/STATUS.md](docs/STATUS.md) is the single source of truth for project completion state. Historical phase logs are archived in [docs/archive/](docs/archive/).

---

## 1. What We Are Building

A **hosted, multi-tenant commerce platform for social-commerce and modern retail sellers** — merchants selling across Facebook, Instagram, TikTok, WhatsApp, and their own branded online storefront.

Merchants receive an Apple Human Interface Guidelines (HIG) dashboard, a customized storefront on their custom domain or subdomain, Cash on Delivery (COD) order management, payment gateways (Stripe, bKash, Nagad, SSLCommerz), courier automated dispatch (Pathao, Steadfast, RedX), and fraud scoring.

### Core Architectural Principles

- **We host it. Merchants do not.** A single compiled Rust binary and PostgreSQL cluster serve every merchant. Multi-tenant isolation is enforced at the database engine level (`tenant_id` on every row, `FORCE ROW LEVEL SECURITY`), not as separate container deployments. → [ADR-012](docs/DECISIONS.md#adr-012)
- **First-party subscription billing.** Indefinite Free tier (100 products, 3 staff seats) and Pro tier ($20.00 USD/mo or local ৳1500 BDT, 10% flat discount on 12+ months) collected directly. → [ADR-017](docs/DECISIONS.md#adr-017), [COMMERCIAL_IMPLEMENTATION_PLAN.md](docs/COMMERCIAL_IMPLEMENTATION_PLAN.md)
- **Unified Multi-Tenant Storefront ([ADR-031](docs/DECISIONS.md#adr-031)).** A single Next.js 15 App Router runtime serves all merchant storefronts. Styling and layout variations are driven by dynamic SSR design token injection and semantic variant modifiers, eliminating separate frontend container fleets.
- **Merchant Admin Truth Contract ([ADR-029](docs/DECISIONS.md#adr-029)).** Zero mock storage and zero fake timers across admin routes. Every card and toggle binds directly to a verified Rust backend API.
- **Enterprise Control Plane extracted to Space ([ADR-030](docs/DECISIONS.md#adr-030)).** Permanent cross-product enterprise SaaS operations live in `D:\Syed Systems\Space`. In-repo `apps/platform` provides the transitional commerce console and connector baseline.
- **Wire compatibility with Medusa v2 Admin API ([ADR-002](docs/DECISIONS.md#adr-002), [ADR-027](docs/DECISIONS.md#adr-027)).** An internal engineering economy that allows forking the dashboard rather than rebuilding it from scratch. Public surfaces never mention legacy codenames.

---

## 2. High-Level System Architecture

```mermaid
flowchart TD
    subgraph Clients["Presentation Layer (Clients & Frontends)"]
        SF["Unified Storefront<br/>(Next.js 15 / React 19)<br/>:8000"]
        ADM["Merchant Admin Dashboard<br/>(Vite / React 18 / Apple HIG)<br/>:5173"]
        MKT["Public Marketing Site<br/>(Vite / Tailwind 4)<br/>:5175"]
        PLT["Transitional Platform Console<br/>(Vite / Operator Hub)<br/>:5174"]
    end

    subgraph Ingress["Ingress & Edge"]
        PROXY["Host Header Resolver & Ingress Proxy"]
    end

    subgraph CoreEngine["Vendryx Commerce Engine (:9000)"]
        ROUTER["Axum HTTP Router (stheno-api-router)"]
        API_STORE["Store API (stheno-api-store)"]
        API_ADMIN["Admin API (stheno-api-admin)"]
        API_PLATFORM["Platform API (stheno-api-platform)"]
        
        SAGA["Postgres SAGA Engine (stheno-workflow)"]
        MODULES["34 Leaf Domain Modules (crates/modules/*)"]
        LINKS["Link Association Engine (stheno-links / stheno-query)"]
        OUTBOX["Transactional Outbox (stheno-events / subscribers)"]
    end

    subgraph Providers["Provider Integrations"]
        PG["Payments: Stripe, bKash, Nagad, SSLCommerz, COD"]
        CR["Couriers: Pathao, Steadfast, RedX"]
    end

    subgraph Storage["Persistence Layer"]
        PGDB[("PostgreSQL 16 (Row-Level Security)")]
        REDIS[("Redis (Cache & Rate Limiting)")]
    end

    subgraph External["Company SaaS Control Plane"]
        SPACE["Space Control Plane (Standalone Project)<br/>D:\\Syed Systems\\Space"]
    end

    Clients --> PROXY
    PROXY --> ROUTER
    ROUTER --> API_STORE
    ROUTER --> API_ADMIN
    ROUTER --> API_PLATFORM
    
    API_STORE --> SAGA
    API_ADMIN --> SAGA
    API_PLATFORM --> SAGA
    
    SAGA --> MODULES
    MODULES --> PGDB
    MODULES --> PG
    MODULES --> CR
    LINKS --> PGDB
    OUTBOX --> PGDB
    API_PLATFORM -.->|Connector API| SPACE
```

---

## 3. Monorepo Structure

```
├── apps/                                 # Frontend client applications
│   ├── storefront/                       # Unified Next.js 15 Storefront Runtime (ADR-031)
│   ├── admin/                            # Merchant Admin Dashboard (Apple HIG, Vite)
│   ├── marketing/                        # Public marketing site and demo simulator
│   └── platform/                         # Transitional operator console
│
├── crates/                               # Rust modular monolith engine (34 modules, 25 providers)
│   ├── core/                             # Kernel primitives, db pooling, saga workflow engine
│   ├── modules/                          # 34 leaf domain crates (product, cart, order, ai, etc.)
│   ├── flows/                            # Inter-module SAGA transaction orchestrations
│   ├── providers/                        # 25 payment, courier, AI, analytics, and CDN providers
│   ├── api/                              # HTTP API crates (store, admin, platform, router)
│   ├── stheno-server/                    # Composition root and production server binary
│   └── stheno-cli/                       # CLI for migrations, OpenAPI sync, and scaffolding
│
├── docs/                                 # Master technical documentation repository
│   ├── README.md                         # Documentation navigation hub
│   ├── DECISIONS.md                      # Architecture Decision Records (ADR-001 to ADR-037)
│   ├── ADR_IMPLEMENTATION_PLAN.md        # Phased closure plan & evidence for ADR-032 to ADR-036
│   ├── OKF_PLAN.md                       # AI Social Commerce (OKF) completion plan
│   ├── STATUS.md                         # Authoritative project completion ledger
│   ├── BRAND.md                          # Corporate brand identity and design tokens
│   └── archive/                          # Historical phase evidence logs and retired plans
│
├── Vendryx_Approved_Complete_Brand_Package/  # Final approved Vendryx Brand Kit v1.0 (vectors, tokens, guidelines)
├── config/                               # Tiers, quotas, and runtime configurations
├── deploy/                               # Deployment manifests, Docker Compose, PgBouncer configs
├── migrations/                           # Diesel SQL migrations (partitioned per module schema)
└── scripts/                              # Automated CI quality gates, conformance, and load tests
```

---

## 4. Local Development & Service Port Matrix

| Service / Application | Directory | Port | URL | Description |
|---|---|---:|---|---|
| **Commerce Engine** | `crates/stheno-server` | `9000` | `http://127.0.0.1:9000` | Axum HTTP server hosting Store, Admin, and Platform APIs |
| **Merchant Admin** | `apps/admin` | `5173` | `http://127.0.0.1:5173` | Merchant dashboard with Apple HIG Information Architecture |
| **Unified Storefront** | `apps/storefront` | `8000` | `http://localhost:8000` | Multi-tenant customer storefront (Next.js App Router) |
| **Platform Console** | `apps/platform` | `5174` | `http://127.0.0.1:5174` | Operator control plane for tenant provisioning |
| **Marketing Site** | `apps/marketing` | `5175` | `http://127.0.0.1:5175` | Public marketing site & interactive store simulator |
| **PostgreSQL** | `deploy/` | `5432` | `localhost:5432` | Multi-tenant PostgreSQL database with RLS policies |
| **Redis** | `deploy/` | `6379` | `localhost:6379` | Ephemeral caching and connection rate limiting |

---

## 5. Quickstart Guide

### Prerequisites
- **Rust:** 1.80+ (`rustup default stable`)
- **Node.js:** 20+ (`npm` / `npx`)
- **Docker & Docker Compose:** Docker Desktop or Linux Docker Engine
- **PostgreSQL Client:** `psql` (optional, for direct inspection)

### Step 1: Start Infrastructure
```bash
docker compose up -d postgres redis
```

### Step 2: Configure Environment
```bash
export DATABASE_URL="postgres://stheno:stheno@localhost:5432/stheno"
export STHENO_PLATFORM_TOKEN="dev-platform-token"
```

### Step 3: Run Database Migrations
```bash
cargo run -p stheno-cli -- db migrate
cargo run -p stheno-server -- seed --demo
```

### Step 4: Run Services
In separate terminal sessions:

```bash
# 1. Start the Rust commerce engine (:9000)
cargo run -p stheno-server

# 2. Start the Merchant Admin dashboard (:5173)
cd apps/admin && npm run dev

# 3. Start the Unified Storefront (:8000)
cd apps/storefront && npm run dev

# 4. Start the Marketing application (:5175)
cd apps/marketing && npm run dev
```

---

## 6. Architecture Decision Records (ADRs)

Key architectural decisions governed by [docs/DECISIONS.md](docs/DECISIONS.md):

| ADR | Title | Status | Impact |
|---|---|---|---|
| [ADR-001](docs/DECISIONS.md#adr-001) | Compile-time extensibility | Accepted | Zero dispatch cost; modules and providers compile as Rust crates. |
| [ADR-002](docs/DECISIONS.md#adr-002) | Wire compatibility with Medusa v2 | Accepted | Permits forking the admin dashboard; amended by ADR-027. |
| [ADR-003](docs/DECISIONS.md#adr-003) | Durable Postgres SAGA engine | Accepted | Compensation stacks, step retries, and timeouts on Postgres. |
| [ADR-004](docs/DECISIONS.md#adr-004) | Diesel persistence | Accepted | Compile-time verification across all 127 entities. |
| [ADR-006](docs/DECISIONS.md#adr-006) | Exact Decimal Money | Accepted | Zero floating-point arithmetic in money calculations. |
| [ADR-012](docs/DECISIONS.md#adr-012) | Postgres Row-Level Security Tenancy | Accepted | One database; `tenant_id` on every row enforced by RLS. |
| [ADR-027](docs/DECISIONS.md#adr-027) | Medusa compatibility is implementation detail | Accepted | Implementation detail only; never mentioned to customers. |
| [ADR-028](docs/DECISIONS.md#adr-028) | Brand boundary: Vendryx | Accepted | Public brand is **Vendryx**; internal identifiers remain `stheno-*`. |
| [ADR-029](docs/DECISIONS.md#adr-029) | Merchant Admin Truth Contract & Apple HIG | Accepted | Zero mock storage; Apple HIG 7 Settings categories. |
| [ADR-030](docs/DECISIONS.md#adr-030) | Space Standalone Control Plane | Accepted | Enterprise SaaS extracted to standalone `D:\Syed Systems\Space`. |
| [ADR-031](docs/DECISIONS.md#adr-031) | Unified Multi-Tenant Storefront Runtime | Accepted | Single Next.js runtime; dynamic SSR tokens & variant modifiers. Supersedes ADR-013, 020, 023. |

---

## 7. Quality Gates & Automated Verification

Run these verification commands before submitting pull requests:

```bash
# Documentation sync, pricing invariants, and link integrity
python scripts/doc-sync-workflow.py check

# Customer-facing brand surface verification (ADR-028)
powershell -ExecutionPolicy Bypass -File scripts/check-brand-surface.ps1

# Merchant admin zero-mock-storage verification (ADR-029)
powershell -ExecutionPolicy Bypass -File scripts/check-no-mock-storage.ps1

# Storefront TypeScript typecheck (ADR-031)
npm --prefix apps/storefront run typecheck

# Rust boundary and kernel isolation checks
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-module-contracts.ps1

# Engine clippy and test suites
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 8. Documentation Index

For detailed guides and specifications, visit the [docs/](docs/README.md) directory:
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — Master Architecture Document
- [docs/STATUS.md](docs/STATUS.md) — Single Source of Truth for Status
- [docs/BRAND.md](docs/BRAND.md) — Authoritative Brand Guidelines
- [docs/COMMERCIAL_IMPLEMENTATION_PLAN.md](docs/COMMERCIAL_IMPLEMENTATION_PLAN.md) — Pricing & Settlement
- [docs/SECURITY.md](docs/SECURITY.md) — Security Controls & Threat Model
- [AGENTS.md](AGENTS.md) — Rules for AI and human contributors working on the codebase
