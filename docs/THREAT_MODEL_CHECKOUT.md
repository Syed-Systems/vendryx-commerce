# Threat Modelling Session — Checkout

**Format:** one facilitated 3-hour session · **When:** end of P3, before P4 writes payment code
· **Output:** a ranked findings list with owners and phases, appended to [SECURITY.md](SECURITY.md)

Checkout is where money, inventory, PII, and a third party all meet. It is the only flow worth a
dedicated session — everything else is covered by the control set in SECURITY.md.

This is a **run sheet**, not a report. Someone facilitates it, a room fills the gaps, and the
output is real findings rather than a document that gets filed.

---

## 1. Before the session

### Who is in the room

Six people, maximum eight. Past that, people stop talking.

| Role | Why they are essential |
|---|---|
| **Facilitator** | Runs the clock, asks the prompts, does not defend the design |
| Rust lead | Knows the workflow engine and what compensation actually does |
| Backend engineer (payments) | Will write the Stripe provider |
| Frontend engineer | Owns what the browser does and where Stripe Elements sits |
| Ops / infra | Knows the network, secrets, and what is actually deployed |
| **Someone who has run a store** | The one who says "customers do X" and is right |

That last seat is the one teams skip and shouldn't. Fraud patterns are operational knowledge, not
engineering knowledge.

### Pre-reading — sent 48 hours ahead

- [WORKFLOWS §2.3](WORKFLOWS.md) — the 11-step `complete-cart` workflow
- [PAYMENTS §6, §7, §11](PAYMENTS.md) — idempotency, webhooks, the 34-case atlas
- [SECURITY §3, §8](SECURITY.md) — threat model, business-logic abuse
- This document

**Rule:** anyone who has not read the pre-reading contributes anyway. Excluding them punishes the
wrong thing, and the naive question is often the good one.

### Room setup

The data flow diagram (§2) on a wall or shared canvas, large. Trust boundaries drawn in a
different colour. A visible parking lot for out-of-scope items — most derailments are someone
raising a real issue at the wrong moment.

---

## 2. The system under discussion

Everything inside the dashed boxes is a trust boundary crossing. Threats live on the crossings,
not inside the boxes.

```
        ┌────────────────────────────────────────────────────────────┐
        │ BROWSER (fully attacker-controlled)                        │
        │   Next.js storefront    ·    Stripe Payment Element        │
        └───────┬────────────────────────────────┬───────────────────┘
                │                                │
     ═══════════▼═══════ TB1 ═══════             ╚═══ TB2 ═══► Stripe
     (public internet → Store API)                (card data — never enters Stheno)
                │
        ┌───────▼──────────────────────────────────────────────────┐
        │ STORE API   /store/carts/*                               │
        │   publishable key → sales-channel scope                  │
        │   customer JWT (optional) · cart cookie + cart_secret    │
        └───────┬──────────────────────────────────────────────────┘
                │
        ┌───────▼──────────────────────────────────────────────────┐
        │ complete-cart WORKFLOW  (durable, compensating)          │
        │   1 validate_cart          7 consume_campaign_budget     │
        │   2 validate_shipping      8 create_order                │
        │   3 validate_promotions    9 create_order_links          │
        │   4 recalculate_totals    10 mark_cart_completed         │
        │   5 reserve_inventory     11 [parallel] events · email   │
        │   6 authorize_payment ◄────────────┐    · index sync     │
        └───────┬──────────────────┬─────────┼────────────────────┘
                │                  │         │
     ═══ TB4 ═══▼═══        ═══ TB5 ▼═══     │ (async resume)
        ┌───────────────┐   ┌──────────────┐ │
        │  POSTGRES     │   │ STRIPE API   │ │
        └───────────────┘   └──────┬───────┘ │
                                   │         │
                            ═══ TB3 ▼════════╧═══
                            ┌─────────────────────┐
                            │ /hooks/payment/     │  ← untrusted until
                            │   stripe            │    signature verified
                            └─────────────────────┘

        ┌──────────────────────────────────────────┐
        │ OPERATOR ── TB6 ──► ADMIN API ──► order  │
        └──────────────────────────────────────────┘
```

| Boundary | Crossing | Assumption being made |
|---|---|---|
| **TB1** | Internet → Store API | Everything from the browser is hostile |
| **TB2** | Browser → Stripe | Card data never enters our systems. **This is the PCI SAQ-A claim** |
| **TB3** | Stripe → webhook | Untrusted bytes until HMAC verification passes |
| **TB4** | App → Postgres | Network is trusted; the app's SQL is not automatically safe |
| **TB5** | App → Stripe | Outbound calls are ambiguous on failure |
| **TB6** | Operator → Admin | Authenticated but possibly compromised or malicious |

---

## 3. Agenda

| Time | Activity |
|---|---|
| 0:00–0:15 | Facilitator walks the diagram. **Corrections only** — no threats yet |
| 0:15–0:30 | Attacker personas (§4). Pick the two most relevant to this business |
| 0:30–1:15 | **STRIDE pass** on TB1, TB2, TB3 (§5) |
| 1:15–1:25 | Break |
| 1:25–2:00 | **STRIDE pass** on TB4, TB5, TB6 |
| 2:00–2:35 | **Abuse-case round** (§6) — the un-STRIDE-able business logic |
| 2:35–2:55 | Rank, assign owners and phases (§7) |
| 2:55–3:00 | Close: what got parked, when the next session is |

**Discipline that makes it work:** the facilitator writes down every threat without debating it.
Assessment happens at 2:35, not during discovery. Ten minutes arguing whether something is
exploitable kills the twenty threats that would have surfaced instead.

---

## 4. Attacker personas

From [SECURITY §3](SECURITY.md). Pick two and keep them named on the wall — "what would the bot do
here" produces better output than "are there threats here."

| Persona | Motivation | Typical opener |
|---|---|---|
| **Bot** | Card testing, credential stuffing, scraping | Automate the cheapest endpoint that returns a signal |
| **Fraud customer** | Free or cheap goods | Manipulate price, stack promos, abuse returns |
| **Competitor** | Business intelligence | Enumerate orders, scrape prices, watch inventory |
| **Compromised operator** | Theft | Refund to their own card, export the customer list |
| **Supply-chain attacker** | Card skimming | Get one script onto the checkout page |

For most stores the honest top two are **Bot** and **Fraud customer**. Supply-chain attacker
outranks both if you are large enough to be targeted.

---

## 5. STRIDE prompts

Time-boxed to ~7 minutes per boundary. The prompts are starters, not a checklist to complete.

### TB1 — Internet → Store API

| | Prompt |
|---|---|
| **S**poof | Can I act as another customer? Guess or steal a `cart_id`? Reuse a leaked JWT after a password change? |
| **T**amper | Can I set a price, a quantity that overflows, a negative amount, a different currency, another region's tax? |
| **R**epudiate | If a customer denies placing an order, what do we hold? |
| **I**nfo disclosure | Does an error tell me an email exists? Can I read another cart? Do IDs leak volume? |
| **D**oS | Cheapest request that costs us the most? Cart creation? Promo validation? A pathological filter? |
| **E**levation | Any store endpoint that reaches admin behaviour? Can `metadata` reach somewhere privileged? |

### TB2 — Browser → Stripe (the PCI boundary)

| | Prompt |
|---|---|
| **T**amper | What executes on the checkout page? Who can add a script? Would we know within a day? |
| **I**nfo disclosure | Could card data reach our origin by any path — an error handler, analytics, a form autofill, a session replay tool? |
| **Repudiate** | Do we have a script manifest, and does CI enforce it? |

**This boundary decides the SAQ-A claim.** If anything found here means card data could touch our
servers, the compliance posture changes and that is a business-level escalation, not a backlog
item.

### TB3 — Stripe → webhook

| | Prompt |
|---|---|
| **S**poof | Can a forged event be accepted? Is the comparison constant-time? Is the raw body used? |
| **T**amper | Replayed old event? Amount differing from our record? Test-mode event in production? |
| **D**oS | Unsigned flood forcing HMAC computation? Is there a limit *before* verification? |
| **Ordering** | What breaks if `charge.refunded` arrives before `payment_intent.succeeded`? |

### TB4 — App → Postgres

| | Prompt |
|---|---|
| **T**amper | Any dynamic SQL? Sort or filter fields reaching a query unallowlisted? |
| **I**nfo disclosure | What does a connection-string leak give? Are backups encrypted? |
| **D**oS | Can one request exhaust the pool? Hold a lock long enough to stall checkout? |
| **E**levation | Does the app role have more grants than it needs — `DELETE` on `audit_log`? |

### TB5 — App → Stripe

| | Prompt |
|---|---|
| **T**amper | Is any outbound URL configurable? Where does it resolve? |
| **R**epudiate | After a timeout, can we prove whether the charge happened? |
| **D**oS | Stripe slow or down — does checkout fail safely, and is inventory released? |
| **Ambiguity** | Walk E1 and E2 from [PAYMENTS §11](PAYMENTS.md) out loud. Does the room agree on the outcome? |

### TB6 — Operator → Admin

| | Prompt |
|---|---|
| **S**poof | CSRF on a state-changing admin call? Session fixation? |
| **R**epudiate | Who refunded this, from where, and can they erase the record? |
| **I**nfo disclosure | What can the lowest-privileged role export? |
| **E**levation | Can a role grant itself a permission? Create an API key exceeding its own scope? |

---

## 6. Abuse-case round

STRIDE finds protocol flaws. It does not find "the promotion works exactly as designed and costs
us €40,000." Run these as scenarios, not categories.

Each is stated as a goal. The room describes the cheapest path.

1. **Get the product free.** Any combination — promos, price manipulation, credit lines, currency.
2. **Order without paying.** Complete a cart while authorization fails or is voided after.
3. **Get paid twice.** Order, receive, dispute, and also return.
4. **Empty the inventory** without buying anything.
5. **Take over an abandoned cart** and read its addresses.
6. **Find every promo code** in under an hour.
7. **Learn their monthly order volume** without an account.
8. **Test 10,000 stolen cards** through checkout.
9. **As a junior operator, extract the full customer list** without tripping anything.
10. **Make one order unfulfillable forever** — stuck in a state nobody can clear.

Scenario 10 finds a different class of bug from the other nine: not theft, but a state machine with
a trap. Those cost support hours rather than money and are almost never designed against.

---

## 7. Ranking and output

At 2:35, each finding gets:

| Field | Values |
|---|---|
| **Likelihood** | High / Medium / Low — *for this business*, not in the abstract |
| **Impact** | Money · PII · Availability · Compliance · Trust |
| **Existing control?** | Already covered by SECURITY.md or PAYMENTS.md → note the reference and move on |
| **Owner** | A person |
| **Phase** | P4 for checkout-path items; later only with a stated reason |

Anything that is **money or compliance impact with an existing control of "none"** goes into P4
regardless of likelihood. Everything else is ranked normally.

Output format, appended to [SECURITY §2](SECURITY.md) as findings F30+:

```
F31 | Promo code brute-force via cart API | High | Money
     Control: none → rate limit per cart + per IP, constant-time compare
     Owner: <name> | Phase: P4
```

---

## 8. Findings already known

Seed the wall with these so the session starts warm and nobody spends twenty minutes rediscovering
them. Their presence also calibrates the room on what a finding looks like.

| Boundary | Known finding | Status |
|---|---|---|
| TB1 | Cart id is a bearer credential | F6 → `cart_secret` binding |
| TB1 | Promo code enumeration | F10 → rate limits |
| TB1 | Filter DoS via leading-wildcard `ILIKE` | F11 → pattern limits |
| TB1 | User enumeration on register/reset | F14 → uniform responses |
| TB1 | Anonymous cart flooding | F22 → TTL + rate limit |
| TB2 | PCI SAQ-A script-attack criterion | F25 → manifest + PSP confirmation |
| TB3 | Unsigned webhook flood | F12 → pre-verification rate limit |
| TB3 | Replay, reorder, duplicate | PAYMENTS E7–E10 |
| TB5 | Ambiguous timeout → double charge | PAYMENTS §6 |
| TB6 | CSRF on cookie auth | **F1 — launch blocker** |
| TB6 | No audit log | **F3 — launch blocker** |

**If the session produces nothing beyond this table, it failed** — either the wrong people were in
the room or the facilitator defended the design instead of running the clock.

---

## 9. How these sessions fail

Named so the facilitator can catch them live.

| Failure | Correction |
|---|---|
| Designer defends instead of listening | The facilitator is not the designer. Say "noted" and write it down |
| Debating exploitability during discovery | Park it. Assess at 2:35 |
| Only engineers in the room | Fraud is operational knowledge. Get the operator seat filled |
| Threats without owners | An unowned finding is a wish |
| One session, never repeated | Re-run when the flow changes materially — new PSP, B2B, subscriptions |
| Output lives in a slide deck | It goes into SECURITY.md as F30+, or it did not happen |
| Scope creeps to the whole system | This session is checkout. Everything else is the parking lot |

---

## 10. Follow-up

- Findings appended to [SECURITY §2](SECURITY.md) within 48 hours, while the reasoning is fresh.
- Each becomes a test in the P4 suite — the atlas in [PAYMENTS §11](PAYMENTS.md) is the format.
- **Re-run when the flow changes materially.** A second PSP, B2B ordering, subscriptions, or a
  marketplace model each invalidate parts of this diagram.
- The diagram in §2 is maintained. A stale data flow diagram is worse than none, because people
  trust it.
