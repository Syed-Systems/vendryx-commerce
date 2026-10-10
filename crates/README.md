# Vendryx Core Commerce Engine Workspace

> The high-performance, modular monolith commerce engine powering Vendryx Commerce, written in Rust. Governed by [AGENTS.md](../AGENTS.md), [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md), and [docs/DECISIONS.md](../docs/DECISIONS.md).

---

## 1. Architectural Philosophy

The Vendryx engine (`stheno-*`) is designed as a **modular monolith** with build-enforced boundaries:
- **Compile-Time Extensibility ([ADR-001](../docs/DECISIONS.md#adr-001)):** Modules and providers are compiled Rust crates wired into the application root at compile time. No dynamic runtime plugin containers.
- **Postgres Multi-Tenancy with RLS ([ADR-012](../docs/DECISIONS.md#adr-012)):** A single database instance serves all merchants. Isolation is enforced at the database layer via `FORCE ROW LEVEL SECURITY` and `tenant_id` scoping on every table.
- **Durable SAGA Engine on Postgres ([ADR-003](../docs/DECISIONS.md#adr-003)):** Multi-module transactions run on a custom Postgres SAGA orchestrator with compensation stacks and idempotency guarantees.
- **Diesel Compile-Time Persistence ([ADR-004](../docs/DECISIONS.md#adr-004)):** Type-safe persistence with zero runtime reflection across 127 entities.
- **Exact Decimal Money ([ADR-006](../docs/DECISIONS.md#adr-006)):** Money arithmetic uses `rust_decimal::Decimal`. No floating-point numbers (`f32`/`f64`) anywhere in domain calculations.

---

## 2. Workspace Organization

```
crates/
├── core/                                 # Shared kernel and infrastructure primitives
│   ├── stheno-types/                     # Starved shared kernel (Money, Id<T>, Metadata)
│   ├── stheno-db/                        # Diesel connection pooling & tenant RLS scoping
│   ├── stheno-events/                    # Outbox event publisher & subscriber traits
│   ├── stheno-workflow/                  # SAGA workflow engine & step execution context
│   ├── stheno-links/                     # Cross-module association link tables
│   ├── stheno-query/                     # Cross-module query engine via links
│   ├── stheno-http/                      # Axum HTTP utilities, auth extractors, owned DTOs
│   └── stheno-registry/                  # Compile-time service and provider registry
│
├── modules/                              # 33 leaf domain modules (independent schemas)
│   ├── stheno-product/                   # Products, variants, collections, categories
│   ├── stheno-cart/                      # Cart state machine, shipping methods, adjustments
│   ├── stheno-payment/                   # Payment sessions, collections, ledger & refunds
│   ├── stheno-order/                     # Order lifecycle, line items, fulfillment state
│   ├── stheno-inventory/                 # Stock levels, reservations, warehouse channels
│   ├── stheno-pricing/                   # Price sets, money rules, currency conversions
│   ├── ...                               # customer, auth, tax, region, rbac, file, etc.
│
├── flows/                                # Inter-module SAGA business workflows
│   ├── stheno-flows-cart/                # Complete cart, add item, apply promotion
│   └── stheno-flows-order/               # Order placement, capture payment, reserve stock
│
├── subscribers/                          # Asynchronous domain event handlers
│   └── stheno-subscribers/               # Outbox event consumers (search indexing, audit)
│
├── providers/                            # External service adapters
│   ├── stheno-payment-stripe/            # Stripe Elements & payment intent provider
│   ├── stheno-payment-sslcommerz/        # SSLCommerz hosted gateway adapter
│   ├── stheno-payment-bkash/             # bKash tokenized payment provider
│   ├── stheno-payment-nagad/             # Nagad mobile financial service adapter
│   ├── stheno-payment-offline/           # Cash-on-Delivery (COD) provider
│   ├── stheno-fulfillment-pathao/        # Pathao Courier consignment dispatch
│   ├── stheno-fulfillment-steadfast/     # Steadfast Courier integration
│   ├── stheno-fulfillment-redx/          # RedX parcel delivery adapter
│   └── stheno-fulfillment-manual/        # Standard manual fulfillment provider
│
├── api/                                  # HTTP route crates
│   ├── stheno-api-store/                 # Frozen Medusa v2 Storefront API routes
│   ├── stheno-api-admin/                 # Medusa v2 Admin API parity routes
│   ├── stheno-api-platform/              # Transitional operator platform routes
│   └── stheno-api-router/                # Main Axum router mounting all sub-APIs
│
├── stheno-server/                        # Composition root & production HTTP server binary
└── stheno-cli/                           # Developer CLI (migrations, openapi, doc sync)
```

---

## 3. Four Inviolable Architectural Boundaries ([AGENTS.md §2](../AGENTS.md#2-the-rules-that-fail-the-build))

1. **Module Isolation:** A crate in `crates/modules/` must **never** depend on another crate in `crates/modules/`. Modules are leaves.
   - Cross-module writes go through **Workflows** (`crates/flows/`).
   - Cross-module reads go through **Query Engine** (`stheno-query`).
   - Cross-module events go through **Subscribers** (`crates/subscribers/`).
   - Cross-module entity associations go through **Links** (`stheno-links`).
2. **Schema Isolation:** Each module owns one Postgres schema. Raw SQL (`sql_query`) may never cross module schema boundaries.
3. **Starved Shared Kernel:** `stheno-types` has zero dependencies and holds primitives only. Domain entities (`Product`, `Order`) never live in the kernel.
4. **No Feature Gates in Domain Code:** Entitlements and plan tiers are enforced at route mounting time, never by branching inside domain services.

---

## 4. Common Developer Commands

All commands are run from the repository root:

```bash
# Verify workspace compiles
cargo check --workspace

# Run complete workspace unit and integration test suite
cargo test --workspace

# Run Clippy linter with zero warnings tolerance
cargo clippy --workspace --all-targets -- -D warnings

# Verify formatting
cargo fmt --all -- --check

# Run boundary enforcement checks
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-module-contracts.ps1

# Start the local development server (port 9000)
cargo run -p stheno-server
```
