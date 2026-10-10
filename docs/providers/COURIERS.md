# Bangladesh Couriers — integration contracts

**Date:** 2026-08-21 · **For:** [P23](../archive/PHASE3_TASKS.md) and [P24](../archive/PHASE3_TASKS.md) · **Governed by:** [ADR-013](../DECISIONS.md#adr-013) decision 6

Companion to [PAYMENT_GATEWAYS.md](PAYMENT_GATEWAYS.md). Same rule: credentials are tenant
data, entered in the admin panel and encrypted at rest.

**Confidence is lower here than for payments.** None of the three publishes a public developer
portal. Base URLs and endpoint paths below are consistent across multiple independent SDK
implementations, which makes them reliable enough to design against — but field-level details
must be confirmed against the merchant-portal documentation before P23 implementation, not
during it.

---

## 1. Credential matrix

| Courier | Fields a merchant supplies |
|---|---|
| **Pathao** | `client_id`, `client_secret`, `username`, `password`, `store_id`, sandbox flag |
| **Steadfast** | `api_key`, `secret_key` |
| **RedX** | `access_token` (JWT), sandbox flag |

Pathao is the awkward one: it wants the merchant's **portal login password**, not just a
machine credential. That raises the stakes on P21-12 — this is a reusable human credential, and
it must never appear in a log line or an audit row.

---

## 2. Pathao Courier (Merchant API)

| Environment | Base URL |
|---|---|
| Sandbox | `https://courier-api-sandbox.pathao.com` |
| Production | `https://api-hermes.pathao.com` |

**Auth:** `POST /aladdin/api/v1/issue-token` with `client_id`, `client_secret`,
`grant_type: "password"`, `username`, `password` → `access_token`.

**Surfaces:** stores (create/list), location hierarchy (**cities → zones → areas**), price-plan
calculation, order creation, order status. Webhooks are supported.

The three-level city/zone/area hierarchy is the important structural fact. A delivery address
is not a free-text string to Pathao — it resolves to an `area_id`, and rates depend on where
that area sits. This is what P23-12 has to express through existing shipping options, and what
P23-03's inside-city / outside-city / sub-city tiers are derived from.

---

## 3. Steadfast Courier

**Base URL:** `https://portal.packzy.com/api/v1`
**Auth:** `Api-Key` and `Secret-Key` request headers.

| Endpoint | Purpose |
|---|---|
| `POST /create_order` | Create a consignment |
| `POST /create_order/bulk-order` | Bulk create |
| `GET /status_by_cid/{consignment_id}` | Status by consignment id |
| `GET /get_balance` | Account balance |

Create request: `invoice`, `recipient_name`, `recipient_phone`, `recipient_address`,
`cod_amount`; optional `recipient_email`, `alternative_phone`, `item_description`, `total_lot`,
`delivery_type`.

Create response: `consignment_id`, `invoice`, `tracking_code`, `recipient_name`,
`recipient_phone`, `recipient_address`, `cod_amount`, `status` (e.g. `"in_review"`).

`get_balance` is the reconciliation hook. P24's settlement import needs a source of truth for
what the courier holds versus what it has remitted, and this is Steadfast's.

Note Steadfast has **no rate/price endpoint** in the documented surface — unlike Pathao. If that
holds, `calculate_price` for Steadfast is merchant-configured pricing, not a live lookup, and
`FulfillmentProvider` must be able to express that honestly rather than inventing a rate.

---

## 4. RedX

**Base URL:** `https://openapi.redx.com.bd/v1.0.0-beta`
**Auth:** header `API-ACCESS-TOKEN: Bearer <jwt>`

| Endpoint | Purpose |
|---|---|
| `POST /parcel` | Create a parcel |
| `GET /parcel/{parcel_id}` | Parcel detail |
| `GET /parcel/track/{tracking_id}` | Tracking |
| `PUT /parcel/{parcel_id}/cancel` | Cancel |
| `GET /areas` | Delivery areas |
| pickup stores | Create/list/detail |

Create fields: `customer_name`, `customer_phone`, `delivery_area`, `delivery_area_id`,
`customer_address`, `merchant_invoice_id`, `cash_collection_amount`, `parcel_weight`,
`instruction`, `value`, `pickup_store_id`, parcel details.

`cash_collection_amount` is RedX's COD field, `cod_amount` is Steadfast's, and Pathao has its
own. **Three names for the money a courier collects on our behalf** — P24-02 requires order
total, collected, and remitted to be three distinct stored numbers, and this is where the
mapping to each courier's vocabulary belongs.

Note the base URL says `v1.0.0-beta`. A beta API is a moving contract; P23-13's recorded
fixtures are the defence against it changing under us.

---

## 5. The gap that actually matters for P23

Every courier above exposes **create, cancel and track**. None of them publishes a complete
**status vocabulary** in the sources available — the list of every state a parcel can be in,
including the ugly ones: partial delivery, return-in-transit, hold, failed-delivery-attempt.

That is precisely what P23-01 and P23-11 are about, and why P23-11 is a `[BP]`:

> An unrecognised status is a logged, alerting failure, never a silent no-op.

The failure mode is an order stuck in a state the engine does not know exists, discovered by a
customer. Since the vocabulary cannot be enumerated from documentation, the engine must be
built to be **loud about the unknown** rather than complete about the known — the
`Unknown(String)` arm carries the courier's raw status so an unmapped value is captured, not
discarded.

**Action before P23-05/07/09:** each merchant account gives access to the courier's own status
list in their portal. Collect real webhook payloads per courier and record them as fixtures
(P23-13). Documentation will not supply this; observation will.

---

## 6. Confidence summary

| Fact | Confidence |
|---|---|
| Base URLs, auth headers, endpoint paths | **Medium-high** — consistent across independent SDKs |
| Create-order field names | **Medium** — from SDK wrappers, confirm against portal docs |
| Status vocabularies | **Unknown** — not published; must be observed |
| Rate/price surfaces | **Pathao yes, Steadfast unclear, RedX unclear** |

Sources: [Pathao merchant SDK (Node/TS)](https://github.com/Sifat07/pathao-merchant-sdk) ·
[Pathao courier Laravel package](https://github.com/enuenan/pathao-courier) ·
[Pathao courier Python SDK](https://pypi.org/project/pathao-courier-api/) ·
[Steadfast Laravel package (vendor-published)](https://github.com/steadfast-it/SteadFast-Courier-Laravel-Package) ·
[Steadfast API plugin](https://wordpress.org/plugins/steadfast-api/) ·
[RedX Laravel package](https://github.com/codeboxrcodehub/redx-courier) ·
[RedX shipping documentation](https://www.nop-station.com/redx-shipping-documentation?categoryId=9)
