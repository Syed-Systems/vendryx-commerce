# Vendryx Commerce — Master Architecture Specification

> **Authority:** This document is the definitive master architecture specification for the Vendryx Commerce platform (engine repository `stheno-*`). It codifies the architectural decisions established in [docs/DECISIONS.md](DECISIONS.md) (ADR-001 through ADR-037) and the engineering laws defined in [AGENTS.md](../AGENTS.md).

---

## 1. Executive Summary & System Philosophy

Vendryx Commerce is a high-performance, multi-tenant commerce platform tailored for social-commerce and modern retail merchants. The system is engineered around four core architectural principles:

1. **Two-Block Symmetrical Architecture**: The presentation tier (storefront) and the commerce engine (backend) are completely decoupled, symmetrical blocks communicating exclusively over clean, versioned HTTP APIs. Any storefront design can be plugged in without modifying the engine, and the engine can be upgraded or scaled without impacting storefront rendering.
2. **Zero-Cost Egress & Zero-DB-Hit Navigation**: Public catalog browsing incurs zero egress costs and zero database hits during normal shopper navigation. Read traffic is terminated at the CDN cache edge and origin in-memory cache layers.
3. **True Multi-Tenancy in a Single Database**: Every merchant shares a single PostgreSQL database instance and a single Rust backend binary, strictly isolated via PostgreSQL Row-Level Security (`FORCE ROW LEVEL SECURITY`) and transaction-scoped tenant context (`SET LOCAL app.current_tenant_id`).
4. **Enforced Modularity & Correctness**: The engine kernel is composed of 34 isolated module crates with zero lateral dependencies, strict per-module SQL schemas, SAGA workflows with compensations, and zero-float decimal money arithmetic.

```
                                  [Shopper Browser]
                                         │
                   ┌─────────────────────┴─────────────────────┐
                   ▼                                           ▼
      [Page Navigation: Tier 0]                  [Cache Miss / First Load]
      Next.js Router Cache & Prefetch                          │
      (0 Network Requests)                                     ▼
                                                    [Edge CDN: Tier 1]
                                                    Cloudflare / Fastly
                                                    Cache-Control: public, s-maxage=3600
                                                    (0 Egress Cost)
                                                               │
                                                   Cache Miss / Revalidate
                                                               │
                                                               ▼
                                                  [Reverse Proxy / Ingress]
                                                               │
                                       ┌───────────────────────┴───────────────────────┐
                                       ▼                                               ▼
                             [Storefront SSR: Node]                           [Engine: Axum/Rust]
                             Next.js Server Components                        stheno-server (:8080)
                             (Full Route Cache / ISR)                                  │
                                       │                                               ▼
                                       └───────────────────────────────► [Origin In-Memory L1: Tier 2]
                                             /store/* (Internal)         CacheService (Local RAM)
                                                                         (0 DB Hits)
                                                                                       │
                                                                                   Cache Miss
                                                                                       │
                                                                                       ▼
                                                                           [PgBouncer Pooler: Tier 3]
                                                                           pool_mode = transaction (:6432)
                                                                                       │
                                                                                       ▼
                                                                               [PostgreSQL 16]
                                                                               Row Level Security (RLS)
```

---

## 2. The Two-Block Symmetrical Architecture

The platform separates commerce logic from presentation into two independent, symmetrical building blocks:

### Block 1: Unified Storefront Presentation Layer
- **Location:** `apps/storefront/packages/frontend-default`
- **Core Engine:** Next.js App Router providing standard URL routing (`/[countryCode]/(main)/*`), server data loaders, and regional cart management.
- **Unified Multi-Tenant Runtime (ADR-031):**
  - **Single Next.js Server:** All merchant stores run on a single Next.js application cluster without deploying separate servers per tenant or theme.
  - **Host-Based Resolution:** Request hostnames are resolved in `<1ms` by `middleware.ts`, injecting tenant context into server components.
  - **Dynamic Design Tokens (CSS Custom Properties):** The root layout (`layout.tsx`) dynamically injects tenant theme tokens into `<style id="stheno-brand-tokens">:root { ... }</style>` on SSR. Tokens control colors, typography (SF Pro, Inter, Geist), corner radii (squircles, pills), and materials (frosted glass blur).
  - **Config-Driven Visual Modifiers:** Mature `frontend-default` components expose semantic variant attributes (`data-card-style="apple-minimal"`, `data-buy-box="sticky"`, `data-image-aspect="portrait"`), enabling layout and styling variety across stores while preserving battle-tested accessibility and form validation.
  - **Apple HIG Baseline:** The platform defaults to Apple Human Interface Guidelines: 8pt spatial rhythm, generous whitespace, tabular pricing numerals, 16px squircle containers, 9999px pill buttons, and fluid spring curves (`cubic-bezier(0.16, 1, 0.3, 1)`).
- **Independence Guarantee:** Frontend modules **present data only; they never fetch directly** (enforced by `./scripts/check-frontend-purity.sh`). Data arrives as strongly typed props from server components.
- **No Forking Policy:** Modifying the Next.js storefront package to create a standalone visual theme for a specific merchant is an architectural violation. All visual differentiation must happen via CSS design tokens and data attribute variant modifiers.

### Block 2: High-Performance Engine (`stheno-server`)
- **Location:** `crates/`
- **Technology:** Rust, Axum, Tokio, Diesel Async, deadpool.
- **API Surfaces:**
  - `/store/*`: Public shopper API (catalog, cart, checkout, customer accounts).
  - `/admin/*`: Merchant administration API (products, orders, inventory, pricing, promotions).
  - `/platform/*` (or Space): Fleet orchestration and provisioning.
- **Symmetry:** The engine has zero awareness of the visual styling or theme tokens used by the storefront. Brand injection happens purely in the Next.js runtime.

---

## 3. Multi-Tier Zero-Cost Egress & Caching Pipeline (ADR-024)

Repeated database queries and bandwidth egress on page navigation represent the single largest operational expense for hosted commerce platforms. Vendryx implements a strict 4-tier caching hierarchy to ensure **zero egress costs** and **zero database queries** on public page navigation:

### Tier 0: Client Browser Router Cache (0 Network Requests)
- Next.js App Router `<Link prefetch>` warms RSC payloads in browser memory.
- Client-side transitions between visited or prefetched catalog pages (e.g. browsing from homepage to collection to product detail) execute instantly without triggering any network requests to the CDN or origin.

### Tier 1: Global Edge CDN (0 Egress Cost)
- A compliant CDN (Cloudflare, Fastly, AWS CloudFront) sits in front of the origin.
- **CDN-Ready HTTP Contract:**
  - **Headers:** Cacheable catalog endpoints emit `Cache-Control: public, s-maxage=3600, stale-while-revalidate=86400`.
  - **Zero `Set-Cookie` on Cacheable Routes:** Public catalog responses (`/store/products*`, `/store/collections*`, `/store/categories*`, `/store/regions*`) **never set cookies**. `middleware.ts` excludes public routes from session cookie generation, preventing CDNs from bypassing cache storage.
  - **Vary Header:** Emits `Vary: Accept-Encoding`, strictly avoiding `Vary: Cookie` which would fragment cache keys per shopper.
  - **Cache Key:** Built strictly from `Host + Path + Query`. Because Host represents the tenant identity ([ADR-022](DECISIONS.md#adr-022)), multi-tenant cache separation is native without custom cache key logic.
  - **Purge Tags:** All responses emit dual purge tags (`Surrogate-Key` and `Cache-Tag`), enabling instant invalidation upon product catalog updates.

### Tier 2: Origin Application In-Memory L1 Cache (0 DB Hits)
- When a CDN edge encounters a cache miss or initiates background revalidation (`stale-while-revalidate`), the request reaches `stheno-server`.
- `stheno-api-store` leverages `stheno_caching::CacheService` backed by an in-memory TTL provider (`InMemoryCacheProvider`).
- Serialized product listings, category trees, collections, and regional details are cached in origin RAM (default TTL: 300s).
- Origin cache hits resolve in **sub-millisecond time** directly from memory, eliminating SQL queries to PostgreSQL.

### Tier 3: Database & Connection Pooling (ADR-025)
- When cold cache misses must query the database, connections pass through **PgBouncer** in `pool_mode = transaction` on port 6432.
- Protocol-level prepared statements are tracked via `max_prepared_statements = 100` (PgBouncer >= 1.21).
- Transaction pooling ensures database connection count remains decoupled from auto-scaling web application replica count.
- Direct database connections (port 5432) are reserved strictly for session-level advisory locks (`PostgresAdvisoryLockProvider`) and database schema migrations.

---

## 4. Multi-Tenancy & Same-Origin Routing (ADR-012, ADR-022, ADR-023)

### Database Multi-Tenancy
- All tenants reside within a single PostgreSQL database.
- Isolation is enforced at the database engine level via PostgreSQL Row-Level Security (`FORCE ROW LEVEL SECURITY`).
- Every query executes within a transaction where the connection context is initialized with:
  ```sql
  SET LOCAL app.current_tenant_id = '<tenant-uuid>';
  ```
- Any attempt to access data belonging to another tenant is blocked by the database engine, preventing application-level IDOR vulnerabilities.

### Same-Origin Host Routing
To eliminate CORS preflight overhead, protect against third-party cookie blocking in modern browsers (Safari ITP, Chrome Privacy Sandbox), and streamline DNS:
- The reverse proxy maps each incoming domain (e.g., `store.brand.com`) to the tenant:
  - `store.brand.com/` → Storefront SSR container (`apps/storefront`)
  - `store.brand.com/store/*` → Backend Engine (`stheno-server`)
- The storefront and the Store API share that origin, so shopper cookies flow without cross-origin credentials configuration.
- The Merchant Dashboard (`apps/admin`) and `/admin/*` are **not** served on storefront hostnames. Each store has its own admin hostname on a domain that carries no merchant-authored content, and `/admin/*` returns 404 everywhere else ([ADR-037](DECISIONS.md#adr-037) decision 9). The static proxy rules live in [`deploy/proxy/Caddyfile`](../deploy/proxy/Caddyfile) and are also packaged by the Helm chart; see [DEPLOYMENT.md §8](DEPLOYMENT.md#8-cloudflare-edge) for edge setup.

---

## 5. Backend Modularity & Clean Architecture

The engine is built strictly according to the module isolation rules specified in [AGENTS.md](../AGENTS.md) §2:

### The Module Isolation Rule
- **No Module May Depend on Another Module:** `crates/modules/*` are strictly leaf crates. Lateral dependencies between modules are forbidden and rejected by CI (`scripts/check-module-isolation.sh`).
- **Four Channels of Cross-Module Interaction:**
  1. **Workflows (`crates/flows/*`):** Orchestrate multi-module write operations using the SAGA pattern with compensation steps.
  2. **Queries (`stheno-query`):** Resolve cross-module reads through explicit entity link definitions.
  3. **Events (`crates/subscribers/*`):** Handle asynchronous cross-module side effects. Modules emit events inside transactions; subscribers live outside modules.
  4. **Links (`stheno-links`):** Manage many-to-many associations between entities owned by different modules.

### SQL Schema Isolation
- Each module owns exactly one PostgreSQL schema (e.g., `product.*`, `cart.*`, `order.*`, `pricing.*`).
- Cross-schema raw SQL joins are strictly forbidden (checked by `scripts/check-sql-boundaries.sh`).

### Financial Correctness
- Floating-point numbers (`f32`, `f64`) are denied across all domain crates.
- All monetary operations use `stheno_types::Money` backed by fixed-point `rust_decimal::Decimal`.
- Allocations, taxes, and prorations are verified via property-based testing.

---

## 6. External Control Plane Separation (ADR-030)

Fleet management and multi-store orchestration are decoupled from individual store data planes:
- **Data Plane (This Repository):** Hosts the tenant's store operations, product catalogs, customer checkouts, and merchant dashboards.
- **Control Plane (`Space` at `D:\Syed Systems\Space`):** Dedicated fleet management system handling:
  - Store provisioning and lifecycle management.
  - Subscription billing, invoicing, Bangladesh Bank USD/BDT conversion, and manual payment
    verification (bKash, Nagad, Rocket, bank transfers) per
    [COMMERCIAL_IMPLEMENTATION_PLAN.md](COMMERCIAL_IMPLEMENTATION_PLAN.md).
  - Custom domain SSL orchestration and DNS verification.
  - Global merchant support and oversight.

---

## 7. Infrastructure & Deployment (ADR-026)

- **Local Development & Single-Node Production:** Managed via `docker-compose.yml`:
  - `web`: `stheno-server` Rust binary (:8080)
  - `storefront`: Next.js SSR frontend (:3000)
  - `admin`: Vite merchant dashboard (:5173)
  - `postgres`: PostgreSQL 16 (:5432)
  - `pgbouncer`: Transaction-mode connection pooler (:6432)
  - `redis`: Shared cache and event broker (:6379)
  - `localstack`: S3-compatible asset storage (:4566)
- **High-Scale Production:** Deployed using Kubernetes Helm charts (`deploy/helm/stheno`), leveraging Horizontal Pod Autoscaling (HPA) backed by PgBouncer transaction pooling.

---

## 8. Architectural Decision Record Reference

For comprehensive rationale, alternatives considered, and trade-off analyses, consult [docs/DECISIONS.md](DECISIONS.md):

| ADR | Title | Status |
|---|---|---|
| [ADR-001](DECISIONS.md#adr-001) | Rust for the engine, Next.js for the storefront, React for admin | Accepted |
| [ADR-002](DECISIONS.md#adr-002) | Full Medusa v2 wire compatibility on `/store/*` and `/admin/*` | Accepted |
| [ADR-012](DECISIONS.md#adr-012) | Multi-tenant single database via Postgres Row-Level Security | Accepted |
| [ADR-013](DECISIONS.md#adr-013) | Pluggable storefront frontends via contract slot system | Accepted |
| [ADR-020](DECISIONS.md#adr-020) | Core-owned content routes (`/about`, `/policies/*`) | Accepted |
| [ADR-021](DECISIONS.md#adr-021) | Symmetrical Two-Block Topology: Pluggable Frontend & Engine | Accepted |
| [ADR-022](DECISIONS.md#adr-022) | Host-based multi-tenancy with same-origin reverse proxy routing | Accepted |
| [ADR-023](DECISIONS.md#adr-023) | Multi-tenant database migrations via single RLS-enabled schema | Accepted |
| [ADR-024](DECISIONS.md#adr-024) | Multi-tier zero-cost egress and in-memory origin caching | Accepted |
| [ADR-025](DECISIONS.md#adr-025) | Transaction-mode PgBouncer connection pooling with prepared statements | Accepted |
| [ADR-026](DECISIONS.md#adr-026) | Containerized Docker Compose baseline with Kubernetes at scale | Accepted |
| [ADR-027](DECISIONS.md#adr-027) | Medusa compatibility as internal economy, never a product claim | Accepted |
| [ADR-028](DECISIONS.md#adr-028) | Vendryx brand boundary: customer-facing Vendryx, internal stheno-* | Accepted |
| [ADR-030](DECISIONS.md#adr-030) | Space Control Plane as independent repository for fleet ops | Accepted |
