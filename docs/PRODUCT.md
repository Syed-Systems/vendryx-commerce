# Product Layer — Onboarding, Social Checkout, Storefront, Reporting

**What sits on top of the engine.** [TRD.md](TRD.md) specifies the commerce engine;
this specifies the product merchants actually use.

Adopted from a prior Medusa-based product plan whose engineering research was verified against
Medusa's source. **The tenancy model from that plan was deliberately not adopted** — see
[ADR-009](DECISIONS.md#adr-009). Everything below is engine-agnostic and applies to Stheno as
written.

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
- **Per-merchant theming. One storefront design for everyone**
- A mobile app, native or otherwise. Web push covers the notification need (§6)
- Subscriptions, digital goods, bookings, services
- Redesigning the forked dashboard's product/order/inventory screens ([ADR-002](DECISIONS.md#adr-002))

---

## 4. Onboarding

**Target: signup → first product live in under 20 minutes**, unassisted, with the merchant's own
messy spreadsheet and no edits to their file.

```
1. Sign in with Google       no password, no email verification, no form
2. Store name + WhatsApp     two fields. Subdomain auto-suggested, editable,
                             uniqueness checked live
3. Upload spreadsheet        .xlsx or .csv — or skip and add products manually
4. Map your columns          their headers shown beside our fields; they match them up.
                             Mapping is saved to the merchant
5. Preview first 5 rows      as they will appear → confirm → import runs in background
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
3. Median onboarding time (signup → first product live) **under 20 minutes**
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
