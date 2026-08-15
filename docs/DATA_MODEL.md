# Data Model — Stheno Commerce Engine

**127 entities across 25 persisting modules · 22 writable link tables · 14 read-only link definitions · 3 Stheno-only payment-safety tables · 152 physical tables**

Derived from `medusajs/medusa@b733400` by enumerating `packages/modules/*/src/models/` and
`packages/modules/link-modules/src/definitions/`. This is the target schema, not an approximation.

---

## 0. Universal conventions

**One Postgres schema per module** ([TRD §3.4](TRD.md)). Tables below are unqualified for
readability; in the schema they are `product.product`, `order.order_item`, `pricing.price`, and so
on. Link tables live in the `links` schema.

Every entity table carries these columns unless explicitly noted:

| Column | Type | Notes |
|---|---|---|
| `id` | `text` PRIMARY KEY | `<prefix>_<ULID>` — wire-identical to Medusa |
| `created_at` | `timestamptz NOT NULL DEFAULT now()` | |
| `updated_at` | `timestamptz NOT NULL DEFAULT now()` | trigger-maintained |
| `deleted_at` | `timestamptz NULL` | soft delete; partial index `WHERE deleted_at IS NULL` |
| `metadata` | `jsonb NULL` | user-extensible, on customer-facing entities |

**Money columns** are `numeric(20,6)` paired with a `currency_code text` column. Never float,
never integer minor units. See [TRD §5.2](TRD.md).

**Cross-module references** are plain `text` columns with **no FK constraint** — that is what makes
the module boundary real. A cross-schema FK would recreate the coupling the boundary exists to
prevent. Intra-module (same-schema) references are true FKs with `ON DELETE` semantics.

**ID prefixes** must match Medusa exactly, or imported data breaks: `prod_`, `variant_`, `cart_`,
`order_`, `cus_`, `reg_`, `sc_`, `pcol_`, `pay_`, `promo_`, `pset_`, `price_`, `ful_`, `so_`,
`sloc_`, `iitem_`, `ilev_`, `res_`, `user_`, `apk_`, `tax_`, `ret_`, `claim_`, `exch_`, `ordch_`.

---

## 1. Module entity inventory

Counts are exact. Names map 1:1 to Medusa model files.

| Module | Entities | Models |
|---|---:|---|
| **order** | 23 | `order`, `order_item`, `line_item`, `line_item_adjustment`, `line_item_tax_line`, `order_shipping_method`, `shipping_method`, `shipping_method_adjustment`, `shipping_method_tax_line`, `order_summary`, `order_change`, `order_change_action`, `transaction`, `credit_line`, `address`, `return`, `return_item`, `return_reason`, `claim`, `claim_item`, `claim_item_image`, `exchange`, `exchange_item` |
| **fulfillment** | 12 | `fulfillment`, `fulfillment_item`, `fulfillment_label`, `fulfillment_provider`, `fulfillment_set`, `service_zone`, `geo_zone`, `shipping_option`, `shipping_option_rule`, `shipping_option_type`, `shipping_profile`, `address` |
| **product** | 12 | `product`, `product_variant`, `product_option`, `product_option_value`, `product_product_option`, `product_product_option_value`, `product_image`, `product_variant_product_image`, `product_category`, `product_collection`, `product_tag`, `product_type` |
| **cart** | 9 | `cart`, `line_item`, `line_item_adjustment`, `line_item_tax_line`, `shipping_method`, `shipping_method_adjustment`, `shipping_method_tax_line`, `credit_line`, `address` |
| **payment** | 8 | `payment_collection`, `payment_session`, `payment`, `payment_provider`, `capture`, `refund`, `refund_reason`, `account_holder` |
| **promotion** | 7 | `promotion`, `promotion_rule`, `promotion_rule_value`, `application_method`, `campaign`, `campaign_budget`, `campaign_budget_usage` |
| **auth** | 6 | `auth_identity`, `provider_identity`, `auth_mfa_factor`, `auth_mfa_recovery_code`, `auth_password_reset_token`, `auth_verification` |
| **pricing** | 6 | `price_set`, `price`, `price_rule`, `price_list`, `price_list_rule`, `price_preference` |
| **rbac** | 5 | `rbac_role`, `rbac_policy`, `rbac_role_policy`, `rbac_role_inheritance`, `rbac_role_parent` |
| **customer** | 4 | `customer`, `customer_group`, `customer_group_customer`, `address` |
| **index** | 4 | `index_data`, `index_metadata`, `index_relation`, `index_sync` |
| **settings** | 4 | `view_configuration`, `layout_configuration`, `property_label`, `user_preference` |
| **tax** | 4 | `tax_region`, `tax_rate`, `tax_rate_rule`, `tax_provider` |
| **inventory** | 3 | `inventory_item`, `inventory_level`, `reservation_item` |
| **store** | 3 | `store`, `currency`, `locale` |
| **translation** | 3 | `translation`, `locale`, `settings` |
| **notification** | 2 | `notification`, `notification_provider` |
| **region** | 2 | `region`, `country` |
| **search** | 2 | `search_index`, `search_index_sync` |
| **stock-location** | 2 | `stock_location`, `stock_location_address` |
| **user** | 2 | `user`, `invite` |
| **api-key** | 1 | `api_key` |
| **currency** | 1 | `currency` |
| **sales-channel** | 1 | `sales_channel` |
| **workflow-engine** | 1 | `workflow_execution` |
| | **127** | |

Modules without persistence (9): `analytics`, `cache-inmemory`, `cache-redis`, `caching`,
`event-bus-local`, `event-bus-redis`, `file`, `locking`, `link-modules`. The 16 provider packages
under `providers/` are counted separately.

---

## 2. Core domain detail

The eight areas below carry the domain complexity. The rest are conventional CRUD and follow
directly from §1.

### 2.1 Product

```
product ──< product_variant ──< product_variant_product_image >── product_image
   │                │
   │                └──< product_product_option_value >── product_option_value
   ├──< product_product_option >── product_option ──< product_option_value
   ├──> product_collection      (nullable)
   ├──> product_type            (nullable)
   └──<> product_tag            (many-to-many)

product_category ──> parent_category_id (self, ranked, arbitrary depth)
                 ──<> product           (many-to-many)
```

| Table | Key columns |
|---|---|
| `product` | `title`, `handle` (unique), `subtitle`, `description`, `status` (`draft\|proposed\|published\|rejected`), `thumbnail`, `weight`, `length`, `height`, `width`, `hs_code`, `origin_country`, `mid_code`, `material`, `discountable`, `external_id`, `type_id`, `collection_id` |
| `product_variant` | `product_id`, `title`, `sku` (unique), `barcode`, `ean`, `upc`, `allow_backorder`, `manage_inventory`, `variant_rank`, dimensions, `hs_code`, `origin_country` |
| `product_option` | `product_id`, `title` |
| `product_option_value` | `option_id`, `value` |
| `product_product_option_value` | variant ⇄ option-value join. **This defines variant identity** |
| `product_category` | `name`, `handle`, `parent_category_id`, `rank`, `is_active`, `is_internal`, `mpath` |
| `product_collection` | `title`, `handle` (unique) |
| `product_image` | `url`, `rank` |
| `product_variant_product_image` | per-variant media assignment |

**Constraints.** `product_variant` unique on the full option-value set within a product — enforced
by a unique index on a generated hash of sorted option-value ids, since Postgres cannot express
"unique across a join set" directly. `product_category.mpath` is a materialised path for subtree
queries without recursive CTEs on the read path.

### 2.2 Pricing

The most performance-critical read path in the system.

```
price_set ──< price ──< price_rule
     ▲                        │
     │ (link)                 └──> attribute + operator + value
product_variant                     e.g. region_id = reg_01J…
shipping_option

price_list ──< price          (price.price_list_id, nullable)
           ──< price_list_rule
price_preference               (attribute + value → is_tax_inclusive)
```

| Table | Key columns |
|---|---|
| `price_set` | id only — the anchor a link table points at |
| `price` | `price_set_id`, `currency_code`, `amount` `numeric(20,6)`, `raw_amount` jsonb, `min_quantity`, `max_quantity`, `rules_count`, `price_list_id` |
| `price_rule` | `price_id`, `attribute`, `operator`, `value`, `priority` |
| `price_list` | `title`, `type` (`sale\|override`), `status` (`active\|draft`), `starts_at`, `ends_at`, `rules_count` |
| `price_list_rule` | `price_list_id`, `attribute`, `value[]` |
| `price_preference` | `attribute`, `value`, `is_tax_inclusive` |

**Resolution algorithm** — must be reproduced exactly, in one query:

1. Collect candidate prices for the price set matching `currency_code` and `quantity` within
   `[min_quantity, max_quantity]`.
2. Discard any price whose rules are not **all** satisfied by the calculation context
   (`region_id`, `customer_group_id`, `sales_channel_id`, custom attributes).
3. Rank survivors by `rules_count DESC` (specificity), then `amount ASC`.
4. Apply price lists: an active `override` list wins outright; a `sale` list wins if lower.
5. Return `calculated_amount`, `original_amount`, `calculated_price.price_list_id`, and the
   matched rule set.

Steps 1–3 are one CTE with a `LATERAL` join and a `NOT EXISTS` unsatisfied-rule check. Fetching
candidates and ranking in application code is the mistake that makes this endpoint slow.

**Indexes:** `price(price_set_id, currency_code)`, `price_rule(price_id)`,
`price(price_list_id) WHERE price_list_id IS NOT NULL`.

### 2.3 Cart

```
cart ──< line_item ──< line_item_adjustment      (promotions)
     │            └──< line_item_tax_line
     ├──< shipping_method ──< shipping_method_adjustment
     │                   └──< shipping_method_tax_line
     ├──< credit_line                             (gift cards, store credit)
     ├──> address (billing) ──> address (shipping)
     └──> region_id, sales_channel_id, customer_id, currency_code   (cross-module, no FK)
```

| Table | Key columns |
|---|---|
| `cart` | `region_id`, `customer_id`, `sales_channel_id`, `email`, `currency_code`, `shipping_address_id`, `billing_address_id`, `completed_at` |
| `line_item` | `cart_id`, `title`, `subtitle`, `thumbnail`, `quantity`, `variant_id`, `product_id`, `product_title`, `variant_sku`, `unit_price`, `is_tax_inclusive`, `is_discountable`, `is_custom_price`, `compare_at_unit_price` |
| `line_item_adjustment` | `item_id`, `amount`, `promotion_id`, `code`, `description` |
| `line_item_tax_line` | `item_id`, `rate`, `code`, `tax_rate_id`, `provider_id` |
| `shipping_method` | `cart_id`, `name`, `amount`, `is_tax_inclusive`, `shipping_option_id`, `data` jsonb |
| `credit_line` | `cart_id`, `amount`, `reference`, `reference_id` |

**Denormalisation is intentional.** `line_item` copies `product_title`, `variant_sku`, and
`unit_price` at add-to-cart time. Renaming a product must not retroactively change a placed order,
and the cart must render without joining the product module.

**Totals are computed, never stored on `cart`.** One pure function over the loaded graph
(FR-PLUS-7). Adjustments and tax lines *are* stored, because they are decisions, not derivations.

### 2.4 Order and the order-change model

The hardest part of the port. Risk R1 in the PRD; build it first (P1).

```
order ──< order_item >── line_item ──< line_item_adjustment
     │                            └──< line_item_tax_line
     ├──< order_shipping_method >── shipping_method
     ├──< transaction              (money in/out ledger)
     ├──< credit_line
     ├──< order_summary            (computed totals snapshot)
     └──< order_change ──< order_change_action

return ──< return_item          claim ──< claim_item ──< claim_item_image
exchange ──< exchange_item
```

| Table | Key columns |
|---|---|
| `order` | `display_id` (sequential, per-store), `region_id`, `customer_id`, `sales_channel_id`, `email`, `currency_code`, `status`, `version`, `is_draft_order`, `canceled_at` |
| `order_item` | `order_id`, `item_id`, `version`, `quantity`, `fulfilled_quantity`, `shipped_quantity`, `return_requested_quantity`, `return_received_quantity`, `return_dismissed_quantity`, `written_off_quantity` |
| `order_summary` | `order_id`, `version`, `totals` jsonb — paid, refunded, credit, outstanding, difference |
| `transaction` | `order_id`, `amount`, `currency_code`, `reference`, `reference_id` — the money ledger |
| `order_change` | `order_id`, `version`, `change_type` (`edit\|claim\|exchange\|return\|transfer`), `status` (`pending\|requested\|confirmed\|declined\|canceled`), `requested_by`, `confirmed_by` |
| `order_change_action` | `order_change_id`, `action` (`ITEM_ADD`, `ITEM_UPDATE`, `ITEM_REMOVE`, `SHIPPING_ADD`, `RETURN_ITEM`, `RECEIVE_RETURN_ITEM`, `WRITE_OFF_ITEM`, `SHIP_ITEM`, `FULFILL_ITEM`, `CANCEL_ITEM_FULFILLMENT`, …), `reference`, `reference_id`, `amount`, `details` jsonb, `applied` |

**The mechanism, stated precisely** — get this wrong and edits, returns, claims, and exchanges all break:

1. An order at `version = N` has `order_item` rows at that version.
2. A mutation opens an `order_change` with `status = pending` and appends `order_change_action` rows.
3. **Preview** = apply the pending actions to the version-N snapshot in memory. No writes. This is
   what powers "show the customer the refund amount before confirming."
4. **Confirm** applies the actions, increments `order.version`, writes new `order_item` rows at
   `N+1`, recomputes `order_summary`, and appends the payment or refund `transaction`.
5. **Decline or cancel** marks the change and discards it. Version does not move.
6. The full timeline is `order_change_action` rows in order. Never delete them.

Returns, claims, and exchanges are all `order_change` subtypes with their own item tables carrying
per-item disposition — `damaged_quantity`, `received_quantity`, `reason_id`.

### 2.5 Promotion

```
promotion ──> application_method ──< promotion_rule (target_rules)
          │                     └──< promotion_rule (buy_rules)
          ├──< promotion_rule ──< promotion_rule_value      (eligibility rules)
          └──> campaign ──> campaign_budget ──< campaign_budget_usage
```

| Table | Key columns |
|---|---|
| `promotion` | `code` (unique), `type` (`standard\|buyget`), `is_automatic`, `campaign_id`, `status` |
| `application_method` | `type` (`fixed\|percentage`), `target_type` (`items\|shipping_methods\|order`), `allocation` (`each\|across`), `value`, `currency_code`, `max_quantity`, `apply_to_quantity`, `buy_rules_min_quantity` |
| `promotion_rule` | `attribute`, `operator` (`eq\|ne\|in\|nin\|gt\|gte\|lt\|lte`), `description` |
| `promotion_rule_value` | `promotion_rule_id`, `value` |
| `campaign` | `name`, `campaign_identifier` (unique), `starts_at`, `ends_at` |
| `campaign_budget` | `type` (`spend\|usage`), `limit`, `used` |
| `campaign_budget_usage` | per-order consumption record — makes budget reversal on cancel possible |

**Budget consumption must be atomic.** `UPDATE campaign_budget SET used = used + $1 WHERE id = $2
AND (limit IS NULL OR used + $1 <= limit) RETURNING *`. Zero rows returned means the budget is
exhausted — reject. Read-then-write here allows over-spend under concurrency.

### 2.6 Inventory and fulfillment

```
inventory_item ──< inventory_level ──> location_id     (cross-module)
              └──< reservation_item

fulfillment_set ──< service_zone ──< geo_zone
                                └──< shipping_option ──< shipping_option_rule
                                                    ├──> shipping_option_type
                                                    ├──> shipping_profile
                                                    └──> (link) price_set
fulfillment ──< fulfillment_item
            └──< fulfillment_label
```

| Table | Key columns |
|---|---|
| `inventory_item` | `sku`, `hs_code`, `origin_country`, `requires_shipping`, dimensions |
| `inventory_level` | `inventory_item_id`, `location_id`, `stocked_quantity`, `reserved_quantity`, `incoming_quantity` |
| `reservation_item` | `inventory_item_id`, `location_id`, `quantity`, `line_item_id`, `allow_backorder` |
| `geo_zone` | `type` (`country\|province\|city\|zip`), `country_code`, `province_code`, `city`, `postal_expression` jsonb |
| `shipping_option` | `name`, `price_type` (`flat\|calculated`), `service_zone_id`, `shipping_profile_id`, `provider_id`, `data` jsonb |
| `shipping_option_rule` | `attribute`, `operator`, `value` |
| `fulfillment` | `location_id`, `provider_id`, `shipping_option_id`, `delivery_address_id`, `packed_at`, `shipped_at`, `delivered_at`, `canceled_at`, `data` jsonb |

**`available_quantity = stocked_quantity − reserved_quantity`.** Never store it. Reservation
creation takes `SELECT … FOR UPDATE` on the `inventory_level` row — this is FR-INV-6, and the
concurrency test in the CI gate exists specifically to prove it.

### 2.7 Payment

Entities are 1:1 with Medusa, plus three tables Medusa does not have. Each closes a specific
silent-failure hole; full rationale in [PAYMENTS.md](PAYMENTS.md).

```
payment_collection ──< payment_session ──> (provider_id, data jsonb)
                   └──< payment ──< capture
                                └──< refund ──> refund_reason
customer ──(link)── account_holder
```

| Table | Key columns |
|---|---|
| `payment_collection` | `amount`, `currency_code`, `status`, `authorized_amount`, `captured_amount`, `refunded_amount` |
| `payment_session` | `provider_id`, `amount`, `currency_code`, `status` (7 states), `data` jsonb, `authorized_at`, `context` jsonb |
| `payment` | `provider_id`, `amount`, `captured_at`, `canceled_at`, `data` jsonb — created **only** on successful authorization |
| `capture` / `refund` | `payment_id`, `amount`, `created_by`; refund adds `refund_reason_id`, `note` |
| `account_holder` | `provider_id`, `external_id`, `data` jsonb |

**Stheno-only (3):**

| Table | Purpose |
|---|---|
| `payment_provider_request` | Outbound idempotency ledger. `idempotency_key` unique, `request_hash`, `state` (`in_flight\|succeeded\|failed\|unknown`). Makes an ambiguous timeout resolvable instead of a coin flip |
| `payment_webhook_event` | Raw event store. Unique `(provider_id, provider_event_id)` — the replay guard. `provider_created_at` drives ordering, never our clock |
| `payment_reconciliation_run` | Nightly drift detection against the provider. `drifts` jsonb; never auto-corrects |

**Ledger invariants — database constraints, not application checks:**

```
0 ≤ Σ captures ≤ payment.amount
0 ≤ Σ refunds  ≤ Σ captures
```

Every capture and refund re-derives the aggregate under `SELECT … FOR UPDATE` on the payment row.
Over-capture and over-refund are impossible rather than merely validated.

### 2.8 Auth, user, RBAC

```
auth_identity ──< provider_identity        (emailpass, google, github, oidc)
              ├──< auth_mfa_factor
              ├──< auth_mfa_recovery_code
              ├──< auth_password_reset_token
              └──< auth_verification

user ──(link)── rbac_role ──< rbac_role_policy ──> rbac_policy
              └──< rbac_role_inheritance / rbac_role_parent
invite ──(link)── rbac_role
```

| Table | Key columns |
|---|---|
| `auth_identity` | `app_metadata` jsonb — holds `user_id` / `customer_id` binding |
| `provider_identity` | `entity_id`, `provider`, `user_metadata` jsonb, `provider_metadata` jsonb (password hash lives here) |
| `auth_mfa_factor` | `type` (`totp`), `secret` (encrypted at rest), `verified_at` |
| `rbac_role` | `name`, `description` |
| `rbac_policy` | `resource`, `action`, `effect` (`allow\|deny`), `conditions` jsonb |
| `rbac_role_inheritance` | transitive closure, maintained on write |
| `api_key` | `token` (hashed), `redacted`, `title`, `type` (`publishable\|secret`), `last_used_at`, `revoked_at` |

**Authorization evaluation:** collect policies over the role-inheritance closure; **deny beats
allow** at any depth. `rbac_role_inheritance` stores the closure rather than walking parents per
request, so a permission check is one indexed read.

---

## 3. Link tables

Link tables are how modules compose without depending on each other. Each has `id`, both foreign
ids, `deleted_at`, and a unique index on the pair.

### 3.1 Writable links (22 — real tables)

| Link table | Left | Right |
|---|---|---|
| `product_variant_price_set` | product.variant | pricing.price_set |
| `product_variant_inventory_item` | product.variant | inventory.inventory_item |
| `product_sales_channel` | product.product | sales-channel |
| `product_shipping_profile` | product.product | fulfillment.shipping_profile |
| `shipping_option_price_set` | fulfillment.shipping_option | pricing.price_set |
| `cart_payment_collection` | cart | payment.payment_collection |
| `cart_promotion` | cart | promotion |
| `order_cart` | order | cart |
| `order_payment_collection` | order | payment.payment_collection |
| `order_claim_payment_collection` | order.claim | payment.payment_collection |
| `order_exchange_payment_collection` | order.exchange | payment.payment_collection |
| `order_promotion` | order | promotion |
| `order_fulfillment` | order | fulfillment |
| `order_return_fulfillment` | order.return | fulfillment |
| `customer_account_holder` | customer | payment.account_holder |
| `region_payment_provider` | region | payment.payment_provider |
| `sales_channel_location` | sales-channel | stock-location |
| `fulfillment_set_location` | fulfillment.fulfillment_set | stock-location |
| `fulfillment_provider_location` | fulfillment.provider | stock-location |
| `publishable_api_key_sales_channel` | api-key | sales-channel |
| `user_rbac_role` | user | rbac.role |
| `invite_rbac_role` | user.invite | rbac.role |

### 3.2 Read-only links (14 — no table)

Declared relationships the query engine can traverse using ids already present on an entity. No
join table, no writes.

`cart_customer` · `cart_product` · `cart_region` · `cart_sales_channel` · `cart_shipping_option` ·
`order_customer` · `order_product` · `order_region` · `order_sales_channel` ·
`inventory_level_stock_location` · `line_item_adjustment_promotion` · `product_translation` ·
`store_currency` · `store_locale`

### 3.3 Rust representation

```rust
pub struct LinkDefinition {
    pub name:        &'static str,
    pub left:        EntityRef,          // module + entity + field
    pub right:       EntityRef,
    pub table:       Option<&'static str>,   // None ⇒ read-only link
    pub cardinality: Cardinality,
    pub cascade:     CascadeBehaviour,
}

pub static LINKS: &[LinkDefinition] = &[ /* 36 entries */ ];
```

The query planner consults `LINKS` to route a field path across a module boundary. Adding a link is
a one-line addition here plus, for writable links, a migration.

---

## 4. Indexing strategy

Beyond primary keys and the `deleted_at IS NULL` partial indexes:

| Table | Index | Serves |
|---|---|---|
| `product` | `(status, created_at DESC) WHERE deleted_at IS NULL` | store listing |
| `product` | unique `(handle)` | handle lookup |
| `product_variant` | `(product_id)`, unique `(sku)` | expansion, SKU lookup |
| `price` | `(price_set_id, currency_code)` | the pricing hot path |
| `price_rule` | `(price_id)` | rule satisfaction check |
| `cart` | `(customer_id)`, `(completed_at) WHERE completed_at IS NULL` | active carts |
| `line_item` | `(cart_id)` | cart load |
| `order` | `(display_id)` unique, `(customer_id, created_at DESC)`, `(status, created_at DESC)` | admin list, account history |
| `order_item` | `(order_id, version)` | versioned snapshot load |
| `order_change_action` | `(order_change_id, created_at)` | timeline |
| `inventory_level` | unique `(inventory_item_id, location_id)` | availability |
| `reservation_item` | `(line_item_id)`, `(inventory_item_id)` | release on cancel |
| `promotion` | unique `(code)` | code entry |
| `api_key` | unique `(token)` | auth on every request |
| `payment_provider_request` | unique `(idempotency_key)` | outbound dedupe on every provider call |
| `payment_webhook_event` | unique `(provider_id, provider_event_id)` | webhook replay guard |
| `capture`, `refund` | `(payment_id)` | ledger aggregate under lock |
| `workflow_execution` | `(state, run_at) WHERE state IN ('invoking','compensating')` | the worker claim query |
| `index_data` | GIN on `data` jsonb | cross-module filtering |
| `search_index` | GIN on `tsvector` | full-text search |

**Rule:** every index in this table exists because a named query needs it. Adding an index without
naming the query it serves is not allowed — see [AGENTS.md](../AGENTS.md).

---

## 5. Schema generation and verification

- `diesel print-schema` output is **checked in** per module as `schema.rs`. Not generated at build.
- CI runs migrations against a clean Postgres, regenerates, and **fails on any diff**. Schema and
  code cannot drift.
- Every model struct derives `Queryable`, `Selectable`, `Identifiable`; every insert struct derives
  `Insertable`. `#[diesel(check_for_backend(Pg))]` on all of them, so a column type change breaks
  the build at the model rather than at runtime.
- Migration numbering is timestamp-based; conflicts surface at merge, not at deploy.
