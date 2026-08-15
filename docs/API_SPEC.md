# API Specification

**Store and Admin APIs — both frozen, both Medusa v2 wire-compatible · Auth — Medusa-compatible shape**

Route inventory measured from `medusajs/medusa@b733400`: **324 route files** — ~46 store, ~278
admin and auth. Per [ADR-002](DECISIONS.md#adr-002), **both surfaces are compatibility contracts.**
The admin dashboard is forked from Medusa and depends on the admin API matching byte for byte.

---

## 1. Conventions

### 1.1 Base

```
https://api.example.com
  /store/*      public commerce      auth: publishable API key (+ optional customer JWT)
  /admin/*      operator + machine   auth: operator JWT (cookie or bearer) or secret API key
  /auth/*       authentication       auth: varies by endpoint
  /hooks/*      provider webhooks    auth: provider signature verification
```

### 1.2 Authentication

| Surface | Mechanism |
|---|---|
| Store, anonymous | `x-publishable-api-key: pk_…` — **required**; resolves the sales-channel scope |
| Store, customer | `Authorization: Bearer <jwt>` — 15 min TTL, `actor_type: customer` |
| Admin, operator | `__stheno_jwt` httpOnly cookie, or `Authorization: Bearer <jwt>` |
| Admin, machine | `Authorization: Basic <base64(sk_…:)>` |

Refresh tokens rotate with reuse detection: presenting a rotated token invalidates the whole
family and forces re-authentication.

### 1.3 Field selection (both surfaces)

```
?fields=id,title,*variants,*variants.calculated_price,-description
```

- bare name — include that field
- `*relation` — expand a relation with its default fields
- `*relation.field` — expand and select within
- `-field` — exclude from the default set

Depth limit 5, node limit 30. Over the limit is `400`, never a slow query. An undeclared field is
`400` — the allowlist is per entity ([TRD §7](TRD.md)).

### 1.4 Pagination

**Both surfaces — offset**, matching Medusa exactly:

```json
{ "products": [...], "count": 1234, "offset": 0, "limit": 20 }
```

The top-level key is the resource name (`products`, `orders`, `customers`), not a generic
envelope — that is what the forked dashboard destructures.

Cursor pagination was specified and then **withdrawn** with [ADR-002](DECISIONS.md#adr-002). Offset
paging can skip or duplicate rows while data changes underneath; that flaw is inherited
deliberately. A future additive `/admin/v3/*` surface is the place to fix it.

### 1.5 Filtering

```
?status=published                       implicit eq
?created_at[$gte]=2026-01-01
?id[$in][]=prod_1&id[$in][]=prod_2
?$or[0][title][$ilike]=shirt&$or[1][description][$ilike]=shirt
```

Operators: `$eq $ne $in $nin $gt $gte $lt $lte $like $ilike $exists $and $or`. Each endpoint
declares its filterable fields and permitted operators; anything else is `400` with the offending
path. Resolved by the `FilterSpec` macro ([TRD §5.3](TRD.md)).

### 1.6 Errors

**Store** — Medusa's shape, byte-compatible:

```json
{ "type": "not_found", "message": "Product with id prod_01J was not found" }
```

Types: `not_found` 404 · `invalid_data` 400 · `unauthorized` 401 · `not_allowed` 403 ·
`duplicate_error` 409 · `invalid_argument` 400 · `unexpected_state` 500.

**Admin** — the same Medusa shape. RFC 9457 was withdrawn with
[ADR-002](DECISIONS.md#adr-002); the forked dashboard parses `{ type, message }`.

Validation detail is not lost — it is rendered into `message` in Medusa's format:

```json
{ "type": "invalid_data", "message": "title: Title is required, variants[0].prices[0].amount: Must be >= 0" }
```

An additive `X-Stheno-Trace-Id` response header carries the trace id on every response. Additive
headers are safe: the dashboard ignores what it does not read.

### 1.7 Idempotency (FR-PLUS-2)

All mutating endpoints accept `Idempotency-Key`. The key maps to a workflow `transaction_id`
([WORKFLOWS §1.7](WORKFLOWS.md)); a replay returns the original response with
`Idempotency-Replayed: true`. Keys are retained 24 hours. On cart completion the key is optional —
the cart id already provides it.

### 1.8 Versioning

- Store is versioned by **compatibility**, not by URL. Breaking it requires a written exemption
  and a parity-suite update.
- Admin is versioned by **compatibility** too. A future redesign lands as an additive `/admin/v3/*`
  surface alongside the compatible one, never as a change to `/admin/*`.
- `Stheno-Version` response header carries the build.

---

## 2. Store API — frozen contract

46 route files. Request and response shapes are **byte-compatible with Medusa v2** except for
documented additions that are strictly additive.

### 2.1 Endpoints

| Method | Path | Notes |
|---|---|---|
| `GET` | `/store/products` | `q`, `handle`, `id`, `collection_id`, `category_id`, `tag_id`, `type_id`, `region_id`, `sales_channel_id`, `order`, `limit`, `offset` |
| `GET` | `/store/products/:id` | id or handle |
| `GET` | `/store/product-variants` · `/store/product-variants/:id` | |
| `GET` | `/store/product-categories` · `/:id` | `include_descendants_tree`, `include_ancestors_tree` |
| `GET` | `/store/collections` · `/:id` | |
| `GET` | `/store/product-tags` · `/:id` · `/store/product-types` · `/:id` · `/store/product-options` · `/:id` | |
| `GET` | `/store/regions` · `/:id` | |
| `GET` | `/store/currencies` · `/:code` | |
| `GET` | `/store/locales` | supported locales |
| `POST` | `/store/carts` | create |
| `GET` `POST` | `/store/carts/:id` | retrieve, update |
| `POST` | `/store/carts/:id/line-items` | add |
| `POST` `DELETE` | `/store/carts/:id/line-items/:line_id` | update, remove |
| `POST` `DELETE` | `/store/carts/:id/promotions` | apply, remove |
| `POST` | `/store/carts/:id/shipping-methods` | add |
| `POST` | `/store/carts/:id/taxes` | recalculate |
| `POST` | `/store/carts/:id/customer` | associate |
| `POST` | `/store/carts/:id/complete` | **idempotent** → order or `requires_more` |
| `GET` | `/store/shipping-options?cart_id=` | eligible options |
| `POST` | `/store/shipping-options/:id/calculate` | calculated pricing |
| `GET` | `/store/payment-providers?region_id=` | |
| `POST` | `/store/payment-collections` | |
| `POST` | `/store/payment-collections/:id/payment-sessions` | |
| `POST` | `/store/customers` | register |
| `GET` `POST` | `/store/customers/me` | |
| `GET` `POST` | `/store/customers/me/addresses` | |
| `GET` `POST` `DELETE` | `/store/customers/me/addresses/:address_id` | |
| `GET` | `/store/orders` · `/store/orders/:id` | customer-scoped |
| `POST` | `/store/orders/:id/transfer/request` · `/accept` · `/decline` · `/cancel` | |
| `POST` | `/store/returns` | request a return |
| `GET` | `/store/return-reasons` · `/:id` | |

### 2.2 The contract that must not break

```http
GET /store/products?fields=*variants.calculated_price&region_id=reg_01J&limit=2
x-publishable-api-key: pk_01J…
```

```json
{
  "products": [{
    "id": "prod_01JABC…",
    "title": "Merino Crew",
    "handle": "merino-crew",
    "status": "published",
    "variants": [{
      "id": "variant_01JABC…",
      "title": "M / Navy",
      "calculated_price": {
        "id": "price_01J…",
        "is_calculated_price_price_list": false,
        "is_calculated_price_tax_inclusive": true,
        "calculated_amount": 89.00,
        "raw_calculated_amount": { "value": "89", "precision": 20 },
        "is_original_price_price_list": false,
        "is_original_price_tax_inclusive": true,
        "original_amount": 120.00,
        "raw_original_amount": { "value": "120", "precision": 20 },
        "currency_code": "usd",
        "calculated_price": { "id": "price_01J…", "price_list_id": null,
                              "price_list_type": null, "min_quantity": 1, "max_quantity": null },
        "original_price": { "id": "price_01J…", "price_list_id": null,
                            "price_list_type": null, "min_quantity": 1, "max_quantity": null }
      }
    }]
  }],
  "count": 42, "offset": 0, "limit": 2
}
```

`raw_calculated_amount` exists only for Medusa compatibility — Stheno's source of truth is
`numeric(20,6)` ([DATA_MODEL §0](DATA_MODEL.md)). It is emitted, never consumed.

### 2.3 Compatibility enforcement

Every Store endpoint has recorded request/response fixtures from a seeded Medusa v2 instance.
CI replays them and diffs, with an allowlist for ids and timestamps only. A diff fails the build.
The corpus is built in **P0**, before any endpoint exists ([TRD §14.1](TRD.md)).

---

## 3. Admin API — frozen contract

~278 route files across 51 resource groups, matching Medusa's admin surface exactly. The forked
dashboard is the primary client and it is unmodified, so this contract is as frozen as the Store API.

### 3.1 What is and is not allowed to differ

| Aspect | Rule |
|---|---|
| Response shapes, key names, status codes | **Identical to Medusa.** Verified by the parity corpus |
| Pagination | Offset, Medusa's shape. Inherited warts included |
| Errors | `{ type, message }`, Medusa's types and codes |
| Filters, sub-resource nesting, batch semantics | Match Medusa, inconsistencies and all |
| **Additive** query params, endpoints, response **headers** | Allowed — the dashboard ignores what it does not read |
| **Changed** response body on an existing endpoint | **Forbidden.** Requires a written exemption and a parity-corpus update |

Bulk operations are still workflow-backed internally (FR-PLUS-8), giving progress, partial success,
and cancellation — but the **response shape stays Medusa's**. The improvement is in the engine, not
in the wire format.

### 3.2 Resource groups

**Catalog** — `products`, `product-variants`, `product-options`, `product-tags`, `product-types`,
`collections`, `product-categories`, `inventory-items`, `reservations`

**Pricing** — `price-lists`, `price-preferences`

**Orders** — `orders`, `draft-orders`, `order-edits`, `order-changes`, `returns`, `claims`,
`exchanges`, `refund-reasons`, `return-reasons`

**Fulfillment** — `fulfillments`, `fulfillment-sets`, `fulfillment-providers`, `shipping-options`,
`shipping-option-types`, `shipping-profiles`, `stock-locations`

**Payments** — `payments`, `payment-collections`

**Customers** — `customers`, `customer-groups`

**Marketing** — `promotions`, `campaigns`

**Config** — `regions`, `currencies`, `sales-channels`, `stores`, `tax-rates`, `tax-regions`,
`tax-providers`, `locales`, `translations`

**Access** — `users`, `invites`, `api-keys`, `rbac` (roles, policies)

**Platform** — `uploads`, `notifications`, `workflows-executions`, `search`, `views`, `layouts`,
`property-labels`, `plugins`, `feature-flags`, `index`

### 3.3 Uniform resource pattern

```
GET    /admin/products              list    → { products: [...], count, offset, limit }
POST   /admin/products              create  → { product: {...} }
GET    /admin/products/:id          get     → { product: {...} }
POST   /admin/products/:id          update  → { product: {...} }
DELETE /admin/products/:id          delete  → { id, object: "product", deleted: true }
POST   /admin/products/batch        batch   → Medusa's batch shape
```

The top-level key is the **resource name**, singular for single resources and plural for lists —
Medusa's convention, reproduced exactly. It is less uniform than a generic `{data}` envelope, and
that is the cost accepted in [ADR-002](DECISIONS.md#adr-002).

### 3.4 Order operations — the ones that carry the domain

Every order mutation is preview-then-commit, backed by the order-change model
([DATA_MODEL §2.4](DATA_MODEL.md)):

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/admin/orders/:id/edits` | open an edit → returns `order_change` |
| `POST` | `/admin/orders/:id/edits/items` · `/shipping-methods` | append actions |
| `GET` | `/admin/orders/:id/edits/preview` | **computed, no writes** — the delta the operator sees |
| `POST` | `/admin/orders/:id/edits/confirm` | apply, bump version, settle payment delta |
| `DELETE` | `/admin/orders/:id/edits` | discard |
| `POST` | `/admin/returns` · `/:id/request-items` · `/:id/receive` · `/:id/confirm` | return flow with per-item received/damaged |
| `POST` | `/admin/claims` · `/:id/inbound-items` · `/:id/outbound-items` · `/:id/confirm` | claim flow |
| `POST` | `/admin/exchanges` · `/:id/inbound-items` · `/:id/outbound-items` · `/:id/confirm` | exchange flow |
| `POST` | `/admin/orders/:id/fulfillments` · `/:fid/shipments` · `/:fid/cancel` | fulfillment |
| `POST` | `/admin/orders/:id/capture` · `/refund` · `/mark-paid` | payment |
| `POST` | `/admin/orders/:id/cancel` · `/archive` · `/complete` | lifecycle |

`GET …/preview` returning a fully computed order projection with **zero writes** is the endpoint
the admin UI is built around. It is what lets an operator see the refund amount before committing.

### 3.5 Workflow observability (FR-PLUS-1) — additive

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/admin/workflows-executions` | list, filterable by workflow, state, date |
| `GET` | `/admin/workflows-executions/:id` | full step timeline with inputs, outputs, errors, compensations |
| `POST` | `/admin/workflows-executions/:id/retry` | resume from the failed step |
| `POST` | `/admin/workflows-executions/:id/cancel` | cancel and compensate |
| `POST` | `/admin/workflows/:name/run` | run a workflow directly (RBAC-gated) |

Medusa has `GET /admin/workflows-executions`; the retry, cancel, and run endpoints plus the deep
step timeline are **additive extensions**. The forked dashboard's existing workflow screen keeps
working, and we deepen it.

---

## 4. Auth API

Medusa-compatible shape, so existing auth flows port unchanged.

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/auth/:actor_type/:provider` | authenticate → JWT |
| `POST` | `/auth/:actor_type/:provider/register` | register → JWT |
| `POST` | `/auth/:actor_type/:provider/callback` | OAuth/OIDC callback |
| `POST` | `/auth/:actor_type/:provider/reset-password` | request reset |
| `POST` | `/auth/:actor_type/:provider/update` | set new password (reset token) |
| `POST` | `/auth/token/refresh` | rotate refresh token |
| `POST` | `/auth/session` · `DELETE` | cookie session set/clear |
| `POST` | `/auth/mfa/factors` · `/factors/:id/verify` · `/challenges` · `/challenges/:id/verify` | MFA |
| `GET` | `/auth/verification/:token` | email verification |

`actor_type` ∈ `{user, customer}`, extensible by crate. `provider` ∈ `{emailpass, google, github,
oidc}`.

**Rate limits:** 5 attempts / 15 min per IP+identifier on authenticate and reset, with exponential
backoff lockout. Applied by middleware, not per handler.

---

## 5. Webhooks

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/hooks/payment/:provider` | PSP events |
| `POST` | `/hooks/fulfillment/:provider` | carrier events |

Every webhook: verify signature → persist raw event → return `200` → process asynchronously.
Duplicate provider event ids are dropped. Never process inline — a slow handler causes provider
retries and duplicate side effects.

---

## 6. OpenAPI

Generated from handler annotations via `utoipa`, served at `/openapi.json` and `/docs`. CI
regenerates and fails on an uncommitted diff, so the spec cannot drift from the code.

**Both frontends use `@medusajs/js-sdk` directly.** No Stheno-specific client is generated or
shipped for 1.0 — using Medusa's own SDK unmodified is precisely what proves compatibility by
construction rather than by assertion.

The generated OpenAPI spec is still published for third-party integrators, and CI fails on an
uncommitted diff so it cannot drift from the handlers.

---

## 7. Rate limits and quotas

| Surface | Default | Header |
|---|---|---|
| Store, anonymous | 100 req/min per IP | `RateLimit-*` (RFC 9331) |
| Store, customer | 300 req/min per customer | |
| Auth | 5 req/15 min per IP+identifier | |
| Admin | 1,000 req/min per actor | |
| Webhooks | unlimited, signature-gated | |

Body limit 2 MB, except uploads (configurable, default 20 MB). All configurable; defaults chosen
to be safe rather than generous.
