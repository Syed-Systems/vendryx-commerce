# Vendryx — Commerce Operating System by Syed Systems

> **Vendryx** is the customer-facing commerce operating system for merchants.
> The repository retains the internal `stheno-*` identifiers for engine and wire compatibility
> (see [ADR-028](docs/DECISIONS.md#adr-028) for the brand boundary definition).

> **Current status:** [docs/STATUS.md](docs/STATUS.md) is the single source of truth for
> completion state. Older phase and handover documents are evidence or task context, not the
> live project dashboard.

## What we are building

A **hosted, multi-tenant commerce platform for small social-commerce sellers** — merchants
running a shop out of a Facebook page, an Instagram profile, or a WhatsApp inbox. They get a
dashboard, a themed storefront on a subdomain, COD orders, and — on the paid tier — payment
gateways, courier dispatch, and buyer fraud scoring.

Four sentences that fix the shape of everything else:

- **We host it. Merchants do not.** One binary and one Postgres serve every merchant; isolation
  is a database policy (`tenant_id` on every row, `FORCE ROW LEVEL SECURITY`), not a deployment
  boundary. → [ADR-012](docs/DECISIONS.md#adr-012)
- **We sell it as a subscription** — indefinite Free, ৳1500/mo Pro on a 6- or 12-month term —
  and we invoice and collect it ourselves, because no billing provider collects BDT from bKash
  or Nagad. → [ADR-017](docs/DECISIONS.md#adr-017)
- **We operate it from the control plane** (`apps/platform`), not from a merchant's dashboard.
  Provisioning, plans, billing, oversight and support all live there. → [CONTROL_PLANE.md](docs/CONTROL_PLANE.md)
- **The engine is Medusa-v2 wire-compatible on the Admin API. That is an implementation detail
  and never a product claim.** It buys us a forked dashboard instead of a rebuilt one. Nothing
  customer-facing mentions Medusa. → [ADR-027](docs/DECISIONS.md#adr-027)

**Read [docs/PRODUCT.md](docs/PRODUCT.md) for who this is for and
[docs/ENTITLEMENTS.md](docs/ENTITLEMENTS.md) for how it is sold.** Those two answer "what" and
"why"; everything below answers "how".

## What this repository is

The **implementation and the specs it is built from**, together: the Rust engine, three
frontends (`apps/storefront`, `apps/admin`, `apps/platform`), the migrations, and `docs/`.

Current stage is tracked in one place: [docs/STATUS.md](docs/STATUS.md). In short, the
commerce product is GA complete through Phase 3; the transitional control plane is the
remaining partially complete surface, while new company-wide control-plane work has moved to
Space (`D:\Syed Systems\Space`).

### Where the docs disagree, this is the order of precedence

The doc set was written over three phases and the product changed underneath it. When two
documents conflict, **the higher row wins** — and the loser is a bug to be fixed, not a
nuance to be interpreted.

| | Authority | Why it wins |
|---|---|---|
| 1 | [docs/DECISIONS.md](docs/DECISIONS.md) | The ADR log. Every architectural reversal is recorded here first, with its supersession |
| 2 | [docs/STATUS.md](docs/STATUS.md) | The current completion ledger. This is the source for "how much is done" |
| 3 | [docs/PRODUCT.md](docs/PRODUCT.md) · [docs/ENTITLEMENTS.md](docs/ENTITLEMENTS.md) · [docs/CONTROL_PLANE.md](docs/CONTROL_PLANE.md) | The product as sold and operated. Written after the hosted-SaaS turn |
| 4 | [docs/PRD.md](docs/PRD.md) · [docs/TRD.md](docs/TRD.md) | Requirements and architecture. Correct on the engine, superseded wherever they describe self-hosting |

An ADR marked **Proposed** records a decision taken in conversation that the orchestrator has
not accepted. It is not yet binding, and no document may cite it as settled.

### What each document answers

#### Start here — what we want to do

| Document | What it answers |
|---|---|
| [docs/STATUS.md](docs/STATUS.md) | Current project status, completion counts, and the active source-of-truth rules |
| [docs/PRODUCT.md](docs/PRODUCT.md) | Who the merchant is, what it costs them today, onboarding, checkout, notifications |
| [docs/ENTITLEMENTS.md](docs/ENTITLEMENTS.md) | The three tiers, the rule that decides what is free, how a gate is enforced |
| [docs/CONTROL_PLANE.md](docs/CONTROL_PLANE.md) | The operator surface — provisioning, billing, oversight, support |
| [docs/PRD.md](docs/PRD.md) | Engine requirements, performance budgets, success measures |

#### How we want to do it

| Document | What it answers |
|---|---|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Master architecture specification — two-block topology, zero-cost egress, multi-tier caching, tenancy |
| [docs/DECISIONS.md](docs/DECISIONS.md) | 28 ADRs — every load-bearing choice, its alternatives, and its cost |
| [docs/BRAND.md](docs/BRAND.md) | Vendryx brand guidelines, design system tokens (`--vx-*`), typography |
| [docs/TRD.md](docs/TRD.md) | Architecture, crate graph, tenancy, engine internals, infra |
| [docs/DATA_MODEL.md](docs/DATA_MODEL.md) | All 127 entities + 22 link tables, per module |
| [docs/API_SPEC.md](docs/API_SPEC.md) | Store and Admin API shapes |
| [docs/WORKFLOWS.md](docs/WORKFLOWS.md) | The SAGA engine spec and the 249-workflow catalog |
| [docs/PAYMENTS.md](docs/PAYMENTS.md) | Provider trait, money boundary, idempotency, webhooks, 34-case edge atlas |
| [docs/SITEMAP.md](docs/SITEMAP.md) | Every storefront, admin, and marketing page, with URL + data contract |
| [docs/SECURITY.md](docs/SECURITY.md) | Compliance obligations, threat model, controls, 29 audit findings |
| [docs/DOCS_WORKFLOW.md](docs/DOCS_WORKFLOW.md) | Documentation sync and version logging workflow protocol |
| [docs/WORK_LOG.md](docs/WORK_LOG.md) | Chronological version-by-version work and increment ledger |
| [AGENTS.md](AGENTS.md) | Build rules for coding agents. Read before writing code |

#### Where the work is

| Document | What it answers |
|---|---|
| [docs/STATUS.md](docs/STATUS.md) | Current status across commerce, transitional platform, and Space |
| [docs/PHASE3_STATUS.md](docs/PHASE3_STATUS.md) | Phase 3 evidence archive; current summary lives in STATUS.md |
| [docs/PHASE3_TASKS.md](docs/PHASE3_TASKS.md) | P21–P28: providers, gateways, couriers, COD settlement, themes |
| [docs/PHASE4_TASKS.md](docs/PHASE4_TASKS.md) | P29-P36: control-plane acceptance criteria and task order |
| [docs/PLATFORM_COMPLETION_TASKS.md](docs/PLATFORM_COMPLETION_TASKS.md) | The 23 control-plane handlers that answer 200 with hardcoded values |
| [docs/TASKS.md](docs/TASKS.md) | P0–P10 and the build-loop protocol every later phase still follows |

## The decisions everything else hangs on

Six, not four. The list grew when the product became a hosted service; the full record and the
rejected alternatives are in [docs/DECISIONS.md](docs/DECISIONS.md).

1. **Tenancy is a database policy.** One database, `tenant_id` on every row, `FORCE ROW LEVEL
   SECURITY`, tier resolved per request. One binary deploy and one migration run moves every
   merchant — engine updates do not scale with customers.
   → [ADR-012](docs/DECISIONS.md#adr-012), superseding [ADR-009](docs/DECISIONS.md#adr-009)
2. **Extensibility is compile-time.** Modules and providers are Rust crates wired into a
   generated registry. No WASM host, no dynamic loading, no DI container.
   → [ADR-001](docs/DECISIONS.md#adr-001)
3. **Durable workflows run on Postgres.** A custom SAGA orchestrator — compensation stacks,
   async steps, retries, timeouts — with no extra infrastructure.
   → [ADR-003](docs/DECISIONS.md#adr-003)
4. **Diesel + diesel-async for persistence.** Compile-time schema verification across all
   127 entities; `sql_query` escape hatch for the pricing and promotion hot paths.
   → [ADR-004](docs/DECISIONS.md#adr-004)
5. **Subscription billing is first-party.** No provider collects BDT from bKash or Nagad, so
   invoicing and collection live in the `platform` schema, on the gateways already integrated.
   → [ADR-017](docs/DECISIONS.md#adr-017)
6. **Admin wire compatibility with Medusa v2 is retained as an internal economy, never marketed.**
   It is what lets the dashboard be forked rather than rebuilt.
   → [ADR-002](docs/DECISIONS.md#adr-002), amended by [ADR-027](docs/DECISIONS.md#adr-027)

**Proposed and not yet binding:** ADR-020–027 — content pages, AI-authored storefront content as
data, Host-as-tenant-identity, one build per theme, CDN over origin, transaction-mode pooling,
Compose-before-Kubernetes. They are how the storefront fleet is intended to work; none has been
accepted, and nothing is built on them.

## Scale of the engine

The domain surface being ported, measured from `medusajs/medusa@b733400` (`develop`),
not from memory. This is the size of the engine, not the size of the product:

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

This is a large port. [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) estimated
**~58 engineer-weeks sequential, ~32 calendar weeks with three parallel tracks.** Anyone
promising a quarter is reading a different document.

## Repository layout

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
Its full scope — what it is for, and how much of it exists — is
[CONTROL_PLANE.md](docs/CONTROL_PLANE.md).

**It is off unless a credential is configured.** `STHENO_PLATFORM_TOKEN` is read at boot;
unset means the whole `/platform/*` surface 404s. Off by default, never open by default:

```bash
STHENO_PLATFORM_TOKEN=dev-platform-token cargo run -p stheno-server
```

`./scripts/dev-tenants.sh up` sets that token and stands up three tenants on three plans
against one server, which is the fastest way to see plan changes take effect.

#### Two kinds of credential

| | Bootstrap token | Named operator |
|---|---|---|
| Where it lives | `STHENO_PLATFORM_TOKEN`, in the environment | a `platform.operator` row, hashed |
| Who it identifies | nobody — it is a shared secret | one person |
| Audited as | `op_bootstrap` | their name |
| Revoke one holder | impossible; rotating it locks everyone out | `DELETE /platform/operators/{id}` |

**Use the bootstrap token to issue the first named credential, then stop using it.** Every
control-plane write is recorded, and a write recorded against `op_bootstrap` cannot tell
you which of three people made it.

```bash
# Issue the first operator. The token in the response is shown once and is not
# recoverable afterwards -- only its SHA-256 is stored.
curl -sX POST localhost:9000/platform/operators   -H "Authorization: Bearer $STHENO_PLATFORM_TOKEN"   -H 'Content-Type: application/json'   -d '{"name":"Ayesha"}'

# From then on, use that token.
curl -s localhost:9000/platform/tenants -H "Authorization: Bearer op_..."

# Who did what, newest first. Cross-tenant on purpose.
curl -s localhost:9000/platform/audit -H "Authorization: Bearer op_..."
```

**Rotation is issue-then-revoke, in that order**, and it disturbs nobody else:

```bash
curl -sX POST localhost:9000/platform/operators   -H "Authorization: Bearer op_old" -d '{"name":"Ayesha"}'
curl -sX DELETE localhost:9000/platform/operators/op_01J... -H "Authorization: Bearer op_new"
```

The bootstrap token stays valid after named credentials exist, deliberately: revoking the
last named credential must not be able to lock the owner out of their own control plane.
Retiring it once the team is established is [PHASE4_TASKS.md](docs/PHASE4_TASKS.md) P36-05.

A revoked credential stops working immediately and stays listed, so an access review can
tell "revoked" apart from "never issued". The audit rows it wrote keep naming it.

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
