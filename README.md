# Stheno — a Rust commerce engine with Medusa-grade capability

> **Stheno** was Medusa's immortal sister. Same family, different constitution.
> The name is a placeholder — rename freely, it appears only in crate prefixes and the binary name.

Stheno is a headless commerce engine: a **Rust** backend (the engine) and **Next.js**
frontends (storefront + admin dashboard). It targets full feature parity with
[medusajs/medusa](https://github.com/medusajs/medusa) `develop`, and beats it on latency,
memory, deploy footprint, and type-safety.

## What this repository is

This is the **specification package**, not the implementation. It contains everything an
engineering team (human or agent) needs to build the engine without re-deriving Medusa's
domain model from scratch.

| Document | What it answers |
|---|---|
| [docs/PRD.md](docs/PRD.md) | Who it's for, what it must do, how success is measured |
| [docs/SITEMAP.md](docs/SITEMAP.md) | Every storefront and admin page, with URL + data contract |
| [docs/TRD.md](docs/TRD.md) | Architecture, crate graph, engine internals, NFRs, infra |
| [docs/DATA_MODEL.md](docs/DATA_MODEL.md) | All 127 entities + 22 link tables, per module |
| [docs/API_SPEC.md](docs/API_SPEC.md) | Store and Admin APIs, both Medusa-compatible |
| [docs/WORKFLOWS.md](docs/WORKFLOWS.md) | The SAGA engine spec and the 249-workflow catalog |
| [docs/PAYMENTS.md](docs/PAYMENTS.md) | Provider trait, money boundary, idempotency, webhooks, 34-case edge atlas |
| [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) | 11 phases, gates, staffing, sequencing |
| [docs/TASKS.md](docs/TASKS.md) | 247 tasks with exit commands, and the build-loop protocol |
| [docs/PRODUCT.md](docs/PRODUCT.md) | Onboarding, social checkout, storefront, notifications, reporting |
| [docs/ENTITLEMENTS.md](docs/ENTITLEMENTS.md) | Feature gating, plan tiers, quotas — how you sell this as a service |
| [docs/SECURITY.md](docs/SECURITY.md) | Compliance obligations, threat model, controls, 29 audit findings |
| [docs/THREAT_MODEL_CHECKOUT.md](docs/THREAT_MODEL_CHECKOUT.md) | Run sheet for the facilitated checkout threat-modelling session |
| [docs/PARITY_MATRIX.md](docs/PARITY_MATRIX.md) | Line-by-line checklist against Medusa |
| [docs/DECISIONS.md](docs/DECISIONS.md) | ADRs — the four load-bearing choices and their costs |
| [AGENTS.md](AGENTS.md) | Build rules for coding agents. Read before writing code |

## The four decisions everything else hangs on

1. **Extensibility is compile-time.** Modules and providers are Rust crates wired into a
   generated registry. No WASM host, no dynamic loading, no DI container. Customising Stheno
   means depending on it as a library and recompiling. → [ADR-001](docs/DECISIONS.md#adr-001)
2. **Full wire compatibility with Medusa v2 — Store and Admin.** Existing storefronts,
   `@medusajs/js-sdk`, and **Medusa's own admin dashboard** work unmodified. The dashboard and
   storefront are forked rather than rebuilt. Improvements are additive only.
   → [ADR-002](docs/DECISIONS.md#adr-002)
3. **Durable workflows run on Postgres.** A custom SAGA orchestrator ports Medusa's semantics
   exactly — compensation stacks, async steps, retries, timeouts — with no extra infrastructure.
   → [ADR-003](docs/DECISIONS.md#adr-003)
4. **Diesel + diesel-async for persistence.** Compile-time schema verification across all
   127 entities; `sql_query` escape hatch for the pricing and promotion hot paths.
   → [ADR-004](docs/DECISIONS.md#adr-004)

## Scale of the target

Measured from `medusajs/medusa@b733400` (`develop`), not from memory:

| Surface | Count |
|---|---|
| Module packages | 34 (25 with persistence) |
| Admin dashboard | forked, not rebuilt |
| Entity models | 127 |
| Link tables | 22 writable + 14 read-only |
| Provider packages | 16 |
| Workflows | 249 |
| Named workflow steps | 288 |
| HTTP route files | 324 (≈46 store, ≈278 admin + auth) |
| Admin dashboard route groups | 38 |

This is a large port. [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) estimates
**~58 engineer-weeks sequential, ~32 calendar weeks with three parallel tracks.** Anyone
promising a quarter is reading a different document.

## Repository layout once implementation starts

**A modular monolith:** one binary, one database, hard internal boundaries enforced by CI.

```
stheno/
  Cargo.toml               # workspace
  crates/
    stheno-server/         # the binary — the only crate that sees everything
    core/                  # types, db, workflow, query, links, events, http, audit
    modules/               # 34 LEAF crates. Zero lateral edges between them
    providers/             # 16 integrations, one module each
    flows/                 # write composition, split by domain (8 crates)
    subscribers/           # ALL event handlers live here, never in a module
    api/                   # HTTP composition, split by domain + a router crate
  migrations/              # one dir per module; one Postgres schema per module
  apps/
    storefront/            # forked from Medusa's Next.js starter
    admin/                 # forked from Medusa's dashboard (React + Vite),
                           #   Enterprise paths stripped and rebuilt
  tests/parity/            # golden-file tests vs a live Medusa instance
```

Modules never call each other. Cross-module interaction happens through exactly four channels —
**workflows** (writes), **query** (reads), **events** (notification), **links** (association) — and
`scripts/check-module-isolation.sh` fails the build on anything else. See
[TRD §3](docs/TRD.md).

## Getting started

```bash
docker compose up -d postgres redis
export DATABASE_URL=postgres://stheno:stheno@localhost:5432/stheno
cargo run -p stheno-server -- migrate
cargo run -p stheno-server -- seed --demo
cargo run -p stheno-server                    # :9000
cd apps/admin && npm run dev                  # :5173  merchant dashboard
cd apps/storefront && npm run dev             # :8000  shopper storefront
cd apps/platform && npm run dev               # :5174  operator control plane
```

### The control plane

`apps/platform` is the operator's view: every merchant, what plan each is on, what each
plan includes, and the negotiated exceptions that apply to one merchant alone
([ENTITLEMENTS.md 5d](docs/ENTITLEMENTS.md), [ADR-016](docs/DECISIONS.md#adr-016)).

It is gated on `STHENO_PLATFORM_TOKEN`, which the server reads at boot. **Unset means the
whole `/platform/*` surface 404s** — off by default rather than open by default:

```bash
STHENO_PLATFORM_TOKEN=dev-platform-token cargo run -p stheno-server
```

`./scripts/dev-tenants.sh up` sets that token and stands up three tenants on three plans
against one server, which is the fastest way to see plan changes take effect.

### Running the tests

`DATABASE_URL` must point at a real Postgres. A test run without it proves
nothing, so the suite fails rather than falling back to in-memory storage:

```bash
docker compose up -d postgres
export DATABASE_URL=postgres://stheno:stheno@localhost:5432/stheno
cargo test --workspace
```

CI provisions the same Postgres major version as a service container; see
[.github/workflows/ci.yml](.github/workflows/ci.yml). `./scripts/check-ci-postgres.sh`
verifies the two stay in step.
