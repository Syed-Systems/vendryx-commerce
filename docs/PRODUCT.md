# Product Layer — Onboarding, Social Checkout, Storefront, Reporting

**What sits on top of the engine.** [TRD.md](TRD.md) specifies the commerce engine;
this specifies the product merchants actually use.

Adopted from a prior Medusa-based product plan whose engineering research was verified against
Medusa's source. **The tenancy model from that plan was deliberately not adopted** — see
[ADR-009](DECISIONS.md#adr-009). Everything below is engine-agnostic and specifies the Vendryx
commercial product, powered by the internal Stheno engine (see [ADR-028](DECISIONS.md#adr-028)).

---

## 1. Who this is for

**Primary — the store owner.** Runs a small commerce business out of a Facebook page, Instagram
profile, or WhatsApp inbox. Typically 20–300 products. Manages orders today in a chat inbox and a
spreadsheet. **Low tolerance for setup work. Will not pay before experiencing value.** Android
phone; desktop only sometimes.

**Secondary — store staff.** Answers buyer messages and processes orders. Needs order and product
access. Must not see billing or settings.

**Not a user — the buyer.** Buyers never create accounts (§2).

### The problems, in the order they cost money

| Problem | Today | Cost |
|---|---|---|
| No catalog | Posts scroll away; "is this available?" | Lost sales, repeated answering |
| Orders live in chat | Screenshots, notebooks, memory | Missed and duplicated orders |
| Inventory is guesswork | A mental count | Overselling, refunds, reputation |
| No record | Nothing to review | Cannot tell what sells |
| Courier entry is manual | Retyping every order into a courier panel | ~1 hour/day |
| Fake COD orders | Buyer refuses delivery | Seller pays return freight |

The last two are the acute daily losses, and they are the **paid tier**
([ENTITLEMENTS §4](ENTITLEMENTS.md)).

---

## 2. Buyers have no accounts

Deliberate, and it propagates further than it looks. Account creation is the single largest
friction point at checkout, and a buyer has no reason to want an account for a one-off purchase
from a small seller.

Consequences, accepted:

- **No buyer-facing order history.** Order status is communicated by the seller, in chat.
- **Repeat buyers are recognised by phone number only**, and only in the dashboard.
- **All carts are guest carts.**
- No wishlists, no saved addresses, no buyer login.

This is a **product-wide decision, not a tier gate.** It never turns on at any price.

**Engine consequence:** `POST /store/customers` (register) and the customer auth routes are not
mounted. Guest checkout is the only path. FR-AUTH-9's guest-to-registered upgrade is retained in
the engine but unused by this product.

---

## 3. Non-goals

Explicitly out of scope. Do not build, do not design for.

- Buyer accounts, login, order history, wishlists
- A marketplace — no cross-merchant browsing, no shared cart, no platform-wide search
- Merchant-uploaded storefront code or a recolored `frontend-default`. Free and Trial receive the
  supported brand kit; Pro bespoke storefronts are developer-built independent frontend modules.
- A mobile app, native or otherwise. Web push covers the notification need (§6)
- Subscriptions, digital goods, bookings, services
- Redesigning the forked dashboard's product/order/inventory screens ([ADR-002](DECISIONS.md#adr-002))

---

## 4. Onboarding

**Target: account → submitted store request in under 20 minutes**, unassisted. A tenant dashboard
is not created until the request is reviewed and provisioned.

```
1. Create merchant account   Google or email/password. Email accounts verify before submission.
2. Save optional business    Business name, owner contact, WhatsApp, category, and intended plan
                             are resumable account-level draft fields.
3. Create store request      Store name, unique subdomain, logo, accent color, and approved icon.
4. Choose service            Free/Trial receive the brand kit. Pro confirms payment before a
                             bespoke-theme brief and assets enter the developer queue.
5. Track launch              Submitted → In review → Provisioning → Online, or Needs changes.
                             Expected activation is within one hour.
```

**Acceptance:** a seller with a real, messy spreadsheet completes this without assistance and
without editing their file.

### 4.1 Spreadsheet import — the mapping layer

The import engine itself is engine-side ([PRD FR-CAT-7](PRD.md)). What the product adds is
**only the mapping layer**:

```
upload .xlsx/.csv  →  parse headers
                   →  mapping UI: their headers ↔ our fields
                   →  DROP every unmapped column        ← required
                   →  INJECT sales channel on every row ← required
                   →  emit normalised CSV
                   →  hand to the import workflow
                   →  preview + confirm
```

**Two requirements that are not obvious and both cause silent failure:**

1. **Unmapped columns must be dropped, never passed through.** The importer's column set is a
   strict allowlist — one stray header ("Notes", "Supplier") aborts the entire import. A real
   merchant spreadsheet always has extra columns. **This is the normal case, not the edge case.**

2. **Inject the merchant's sales channel on every row.** Without it, imported products are
   **invisible on the merchant's own storefront** — storefront queries filter by sales channel.
   The seller sees products in the dashboard and an empty shop, with no error anywhere. Added by
   us, never mapped by the seller.

Import rules the mapping layer must respect:

- Headers are **case-insensitive**; the column set is a **strict allowlist**.
- Each row needs a **handle or an id**. Handle only → create. Id present → update. That is the
  create/update switch; there is no separate flag.
- **One row = one variant.** Rows sharing a handle merge into one product.
- Options are paired `Option N Name` / `Option N Value`, 1-based and unbounded. Value is required
  whenever Name is present.
- Category, tag, and sales-channel columns take **IDs, not names**.
- Status defaults to `draft` — include it explicitly or nothing goes live.

**Accept `.xlsx` directly.** Telling a seller to "save as CSV first" is exactly the friction this
feature exists to remove.

Persist the mapping on the merchant so the second upload skips step 4. **Report errors in the
seller's own column names**, never ours.

### 4.2 CSV export

Write with a **UTF-8 BOM**. Without it, Excel on Windows renders non-Latin product names as
mojibake and the merchant concludes the software is broken.

---

## 5. Storefront and checkout

**One design for everyone.** No theming, no theme marketplace. Branding is name, logo, and colour.

The product page offers exactly two actions.

### 5.1 Order Now — the primary path

```
address form (name, phone, address, area)
  → payment section shows Cash on Delivery only
    (Pro adds gateway options — ENTITLEMENTS §4)
  → creates a real order, status pending
  → web push fires to the seller
  → appears in the dashboard order list
```

### 5.2 Social checkout — the habit-forming path

```
generates a pre-filled message identifying the product unambiguously
  → opens the chosen app
  → creates a DRAFT order in a separate "Social" queue
  → NO notification (the seller is already getting the chat message)
  → staff talks to the buyer, then accepts → converts to a real order
```

**Why the form is primary:** courier dispatch (Pro) needs a structured address. A social order
arrives as free text and can never be auto-dispatched. **The free tier's job is to build the habit
that the paid tier monetises.**

### 5.3 Message format

```
Order from <Store Name>
─────────────────
1. <Product Title>
   Code: <SKU>
   <Option>: <Value> · <Option>: <Value>
   Qty <n> × <price> = <line total>
─────────────────
Total: <total>
Ref: #<draft short code>
```

`Ref` is **the only link between the chat conversation and the queued draft order.** Without it,
staff cannot match an incoming message to a dashboard row. Generate it short and unambiguous, and
**avoid characters confused when read aloud** — no `0`/`O`, no `1`/`I`.

### 5.4 Platform constraint — read before implementing

**Only WhatsApp supports pre-filled message text via a plain deep link** (`wa.me/<number>?text=`).

Messenger (`m.me`) and Instagram **do not**. Injecting text into Messenger requires a Facebook App,
a Messenger bot, page access tokens, and app review — an external dependency with approval risk,
which violates the free tier's zero-marginal-cost rule ([ENTITLEMENTS §3](ENTITLEMENTS.md)).

| Channel | Behaviour |
|---|---|
| WhatsApp | Real pre-fill via `wa.me?text=` |
| Messenger | **Copy to clipboard, then open `m.me`.** Button reads "Copy order & open Messenger" |
| Instagram | Same clipboard pattern |

**Do not spend time trying to make `m.me` pre-fill. It does not.**

### 5.5 Three traps in the draft-order path

Verified against Medusa's source. All apply to Stheno, which ports the same workflows.

**Trap 1 — the draft-order admin endpoint will not create a customer-less order.**
`order.customer_id` is nullable, but the endpoint's validator refuses a request carrying neither
`email` nor `customer_id`. Supplying `email` alone silently **creates a guest customer**, which is
then reused for every later order with that address.

> **Decision:** do not use `POST /admin/draft-orders` for social checkout. Invoke the
> order-creation workflow directly from our own route with `{ status: draft, is_draft_order: true,
> region_id, currency_code, sales_channel_id, items }` and **no email or customer_id**. Buyer name
> and phone go in `metadata`. This is the path that honours "buyers never have accounts" (§2).

**Trap 2 — a draft without `sales_channel_id` fails at conversion, not at creation.**
Conversion runs inventory reservation, and reservation location resolution reads
`sales_channel_id` off the order to follow stock-location ↔ sales-channel links.

A social order can therefore sit in the queue looking healthy for days and **explode the moment
staff accepts it** — the worst possible failure timing, in front of a waiting buyer.

> **Decision:** `sales_channel_id` must be set **at draft creation**, and the merchant's stock
> location must be linked to that channel at provisioning. Covered by a test that creates a draft
> and converts it.

**Trap 3 — conversion fires `order.placed`, so naive push logic double-notifies.**
Converting a social draft emits the same event as a real checkout. A subscriber pushing on
`order.placed` would notify the merchant about an order **their own staff just accepted by hand.**

> **Decision:** stamp `metadata.source = "social"` at draft creation; the push subscriber skips
> orders carrying it. Test both paths explicitly — Order Now must push, social conversion must not.

### 5.6 Order lifecycle

```
Order Now ─────────────► pending ──► confirmed ──► fulfilled ──► completed
                            │                                        ▲
Social checkout ─► draft ───┘                                        │
                    │                                                │
                    └──► cancelled ◄───────────────────────────────┘
```

Staff can move an order forward, cancel it, and edit quantities before confirmation.

### 5.7 Source attribution

`?ref=fb` on the storefront URL → cart `metadata` → order `metadata` → `GROUP BY`. **Not a
subsystem** — three assignments and a query.

---

## 6. Notifications

| Event | Channel | Marginal cost |
|---|---|---|
| Order Now placed | Web push + in-dashboard badge | 0 |
| Social draft queued | **In-dashboard badge only, no push** | 0 |
| Trial ending (day 23, 29) | Web push + email | 0 |
| Low stock | In-dashboard only | 0 |

**Web push is the primary channel:** a browser standard, no vendor, no per-message cost, and on
Android Chrome **no app install**. iOS Safari requires add-to-home-screen — accepted, since the
pilot market is overwhelmingly Android.

Implementation is a subscriber in `crates/subscribers/` on `order.placed` plus VAPID keys in env.
Subscriptions are stored per staff user (`endpoint`, `p256dh`, `auth`).

The subscriber **must skip `metadata.source == "social"`** (§5.5, trap 3).

> `ponytail:` a subscriber, not a notification provider. Fine for one channel. Promote it to a
> `NotificationProvider` only if per-merchant preferences across push/email/SMS are ever needed.

---

## 7. Reporting

**Correcting my own estimate:** I previously called this a missing 2–3 week module. It is not.
It is **four SQL aggregates**, and the honest version stops there.

| Report | Query |
|---|---|
| Orders by status | `COUNT(*) GROUP BY status` |
| Revenue | `SUM(total)` over confirmed orders only |
| Top products | `SUM(quantity) GROUP BY product, ORDER BY … LIMIT 10` |
| Orders by source | `COUNT(*) GROUP BY metadata->>'source'` |

**No pipeline, no warehouse, no event stream, no ML.** Add a materialised view only when a query
is *measurably* slow — not in anticipation.

**The tier split is a window gate, not a feature gate** ([ENTITLEMENTS §4.2](ENTITLEMENTS.md)):
Free sees a 30-day horizon, Pro sees full history. Same four queries, one bound parameter. Cost
scales with the window, which is what makes it honest under the marginal-cost rule.

This replaces the "reporting module — unscheduled" row in the parity matrix. It lands in **P8** as
a small set of tasks, not a phase of its own.

---

## 8. Success criteria

The gate before any Pro feature is built:

1. **20 merchants** who have placed ≥ 1 real order through the platform
2. **10 merchants** still active in week 4 after signup
3. Median onboarding time (account → submitted store request) **under 20 minutes**
4. **At least 5 merchants ask, unprompted, for a paid feature** — and the record of *which* feature
   decides what gets built first

**Criterion 4 is the important one.** Courier dispatch is the current hypothesis for the Pro
anchor. It is a market read, not a measurement. **Do not build Pro features before the pilot
answers this.**

---

## 9. Standing rules

Adopted verbatim, because each encodes a failure someone already paid for:

1. **Never delete or hide a merchant's existing data to enforce a limit.**
2. **The free tier makes no external API calls.** This is the line that decides tier questions.
3. **Move production off trial/student credit the day the first merchant pays.**
4. Mark deliberate shortcuts with a `ponytail:` comment naming the ceiling and the upgrade path.

---

## 10. Exploration — conversational order intake

**Status: proposal for validation; not approved for implementation.** This explores social inbox
automation for Facebook Messenger, Instagram inbox, and WhatsApp inbox. Agents answer product
questions, share links to product pages on the merchant's website, gather order details in the
social conversation, and create a pending order in the merchant dashboard. The store owner
verifies and confirms it before normal order processing begins.

The website is a product-information and link destination only. Its existing storefront checkout
continues to create website orders; the agent does not handle website conversations or replace
that checkout. This is a candidate Pro capability because social messaging and model calls have
marginal cost ([ENTITLEMENTS §3](ENTITLEMENTS.md)); the tier decision still needs merchant
validation under §8.

### 10.1 Product shape

The assistant handles the repetitive intake work. It does not accept an order on the merchant's
behalf.

```text
buyer asks about a product or places an order in a connected social inbox
  → assistant identifies the product and requested options
  → assistant checks current catalog, price, and available stock through Stheno
  → assistant answers product questions and may share the matching website product page
  → assistant asks for missing details (quantity, name, phone, delivery address)
  → assistant summarizes the request and tells the buyer it is awaiting merchant confirmation
  → a pending order appears in the dashboard's Social queue with source and conversation reference
  → store owner verifies, edits, confirms, or rejects it
  → confirmation uses the normal order processing and inventory workflow
```

The store owner remains responsible for the final commitment. The first version must require
human approval for every agent-created order. The assistant may not reserve stock, confirm an
order, take payment, change a price, or dispatch a courier. Rejection, timeout, or an unavailable
channel leaves no active order behind. Website checkout remains unchanged and does not require
agent approval.

### 10.2 Boundaries and sources of truth

| Information or action | Authoritative source / rule |
|---|---|
| Product, variant, price, sales channel | Stheno catalog and pricing; the assistant may only choose among returned records |
| Stock availability | Live inventory query through Stheno; describe it as a check at that moment, not a guarantee |
| Stock reservation | Existing order workflow, after the owner confirms the pending order |
| Shipping, returns, store policies, product FAQs | Merchant-approved knowledge sources with source and freshness metadata |
| Customer/order data | Explicit fields collected for the draft; minimum necessary, with existing retention and erasure rules |
| Final order status | Existing order lifecycle; never the model's conversational statement |

A retrieval or “knowledge base” service can answer store-policy questions, but must not be the
source for price, stock, order state, or customer identity. The Google product/vendor is unresolved:
choose a provider only after it passes data residency, tenant isolation, deletion, source citation,
freshness, cost, and portability checks. Keep the integration replaceable and do not upload a
merchant's customer records to a general-purpose knowledge corpus.

### 10.3 Delivery choices to validate

| Choice | Options | Current recommendation | Decision criteria |
|---|---|---|---|
| Social channel rollout | Facebook Messenger; Instagram inbox; WhatsApp inbox | Support the three target inboxes, but pilot one connector first and add the others in measured stages. | Merchant demand, official API availability, app review/setup burden, per-conversation cost, webhook reliability, message-window rules |
| Intake mode | Suggest reply; collect details and draft; auto-confirm | Collect details and create a draft; require merchant approval | Time saved, correction rate, buyer clarity, merchant trust, risk of a false commitment |
| Knowledge source | Merchant FAQs/policies; catalog feed; external search | Merchant-approved FAQs/policies plus live catalog tools | Answer accuracy, source visibility, freshness, setup effort, tenant isolation |
| Product match | Exact catalog selection; fuzzy suggestions; free-form item | Exact variant selection or ask a clarifying question; never invent a catalog item | Wrong-item rate, ambiguity rate, catalog quality |
| Plan packaging | Included in Pro; usage-limited Pro add-on; later tier | Pro feature with a usage limit set after provider-cost measurement | Marginal cost per conversation, support cost, willingness to pay, fair-use experience |

The current manual social flow remains available. A merchant must be able to pause the assistant
per social channel and take over a conversation without losing the pending order or conversation
reference. Website checkout remains its own path throughout.

### 10.4 Safety and trust requirements

- Ask for confirmation when the product/variant, quantity, address, or delivery terms are ambiguous.
- Never infer a price, stock count, delivery window, warranty, discount, or policy answer. If the
  live source is missing or stale, say that staff will confirm it.
- Show a clear “automated assistant” identity and a human handoff path. Do not imply a draft is an
  accepted order.
- Deduplicate provider webhook retries and repeated buyer submissions. A retry must not create a
  second draft for the same channel conversation and request.
- Validate all model output against an allowlisted draft-order DTO. The model cannot call arbitrary
  URLs, issue database queries, or invoke payment/refund/admin tools.
- Keep tenant scope on every channel credential, conversation, knowledge source, and draft. Apply
  existing PII minimization, access control, retention, audit, and erasure requirements
  ([SECURITY.md](SECURITY.md)). Do not retain raw conversation content longer than the feature
  needs; define retention before implementation.
- If the model, channel, catalog, or inventory lookup is unavailable, fall back to the merchant's
  existing manual inbox workflow. Never fail open into an order confirmation.

### 10.5 Architecture fit if validated

This should compose existing order, inventory, workflow, and entitlement capabilities rather than
introduce a second commerce path. Social channel adapters normalize inbound messages; a bounded
assistant orchestration layer may use read-only catalog/inventory queries and a single pending
order-creation action; the existing dashboard order queue and normal confirmation workflow remain
responsible for merchant review and order processing. Website checkout stays separate and
unchanged. The assistant must not write module tables directly or make one module depend on
another.

Gate the merchant-facing capability at the API/router and provider registration boundary under
[ENTITLEMENTS §6](ENTITLEMENTS.md). Keep entitlement checks out of domain services. Any mutating
workflow remains idempotent, and inventory is reserved only by the existing accepted-order path.
Do not change Store API response shapes to support this; expose a scoped, authenticated admin
workflow surface if existing APIs are insufficient.

### 10.6 Pilot criteria

Before committing implementation scope, interview pilot merchants and measure:

1. Whether merchants request conversational intake without being prompted, alongside the Pro
   feature signal in §8.
2. Conversations that produce a complete, reviewable draft without staff retyping the details.
3. Merchant edit/reject rate and the reasons for corrections, especially product, variant,
   quantity, price, and address errors.
4. Duplicate-draft rate under webhook retries and repeated messages.
5. Buyer understanding that the order is pending merchant approval.
6. Per-merchant and per-completed-draft provider cost, including messaging, model, storage, and
   support; use this to set any Pro usage limit rather than inventing a quota now.

Do not set numeric launch thresholds until a pilot baseline exists. A pilot is successful only if
it reduces staff re-entry without increasing incorrect orders, oversells, duplicate drafts, or
buyer confusion.

### 10.7 Suggested rollout

1. **Concierge validation:** staff manually observe a small set of social inbox conversations and
   record missing fields, catalog ambiguity, and merchant correction reasons. No autonomous
   customer-facing agent.
2. **Pending-order pilot:** one social channel, explicit assistant disclosure, live
   catalog/inventory reads, product-page links, and human verification for every pending order.
   Measure cost and correction rates.
3. **Controlled expansion:** add channels only after each connector meets the same privacy,
   idempotency, handoff, and approval requirements. Consider reducing approval only as a separate
   product decision backed by evidence.

Open decisions before implementation: which social channel to pilot first; provider and
data-processing terms; transcript retention; merchant control over FAQs and human escalation;
usage-limit unit; and whether each channel's setup belongs in self-serve onboarding or assisted
provisioning. Website ordering is out of scope for this automation.
