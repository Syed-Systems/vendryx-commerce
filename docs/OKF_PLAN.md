# OKF Completion Plan — AI Social Commerce

**Status:** Plan dated 2026-11-14. The three ADR-032 changes it depends on were accepted by the
owner on 2026-11-14 and are recorded in the ADR-032 amendment. Decisions 4 to 8 in section 12
are still open. Nothing in this file is implemented unless section 2 says so.
**Extends:** [ADR-032](DECISIONS.md#adr-032) and its phases in
[ADR_IMPLEMENTATION_PLAN.md](ADR_IMPLEMENTATION_PLAN.md). Where this plan changes an ADR-032
decision, section 12 lists the amendment the owner must accept first.
**Scope:** the AI assistant that answers buyers on WhatsApp, Messenger and Instagram and turns
conversations into draft orders. Store Intelligence (ADR-035) is out of scope.

---

## 1. Goal

A Bangladeshi merchant who sells through chat connects their page in a few clicks, watches the
assistant draft replies for a few days, then lets it answer on its own. Buyers get correct
answers in Bangla, Banglish or English at any hour. Staff confirm every order. The merchant can
see what the assistant said, what it sold, and can stop it instantly.

The plan is finished when a merchant with no developer knowledge can reach that state without
help from us, and when nothing the assistant says can commit the merchant to a price, a stock
level, a delivery promise or an order the merchant did not approve.

---

## 2. What exists today

Verified against the code on 2026-11-14 by reading source, not by running it.

| Capability | State | Where |
|---|---|---|
| Tenant knowledge base with RLS, keyword search, 8-result cap | Built | `crates/modules/stheno-ai`, migration `ai_knowledge_entry` |
| Read-only tools for catalog, stock, shipping, chat offers | Built | `crates/flows/stheno-flows/src/ai_tools.rs` |
| Bounded conversation loop: 12 turns, 12,000 characters, 4 tool calls, 5 provider calls, 4,096 tokens | Built | `crates/flows/stheno-flows/src/ai_conversation.rs` |
| Meta webhook ingress with signature check, dedup, ordering, 60 per minute per hook | Built | `crates/api/stheno-api-store/src/social_webhook.rs` |
| Outbound reply to Meta | Built | `crates/stheno-server/src/ai_social.rs` |
| Draft order intake, Social Queue, staff confirmation, courier dispatch | Built | ADR-032 phases 32.5 and 32.6 |
| Owner-authored chat price offers | Built | `crates/modules/stheno-ai/src/offers.rs` |
| Encrypted conversation turns, expiring after 24 hours | Built | migration `ai_channel_conversation` |
| `STOP` / `UNSUBSCRIBE` consent revocation | Built | ADR-034 |
| Single safe fallback sentence when the model cannot answer | Built | `safe_fallback()` |

Not found in the code:

| Missing | Evidence |
|---|---|
| Conversation states, human takeover, staff notification | no match for handoff, takeover or paused in the AI module, flows or webhook |
| Suggest-then-approve mode | no match |
| Kill switch, per-store or platform | no match |
| Per-store or per-sender budget | only the per-conversation token cap exists |
| Merchant-visible transcript | turns are deleted after 24 hours and no read route was found |
| Buyer-facing disclosure that the assistant is automated | no match |
| Order status lookup tool | no match |
| One-click channel connection | merchants paste Meta credentials into `/admin/ai` |
| Evaluation set | no fixture of real conversations |

Live acceptance for everything in the first table is still open: no real Meta account, courier
account or model provider has been exercised end to end.

---

## 3. Rules that do not bend

These hold for every workstream. A design that needs to break one is rejected.

1. **The model commits to nothing.** Price, stock, shipping cost, discount, order creation,
   payment and dispatch are decided by server code. This is ADR-032 and it stays.
2. **A human can always take over, and when they do the assistant stops.** No conversation has
   two speakers on the merchant's side.
3. **The assistant never claims an order is confirmed, paid, shipped or guaranteed.** Those
   sentences are written by the server from order state, not generated.
4. **Tenant isolation is enforced by Postgres RLS**, not by prompt wording or application
   filters.
5. **Failure is silence plus a human, never a guess.** Provider down, budget exhausted,
   unsupported message, low confidence: the conversation is handed to staff.
6. **The buyer knows they are talking to an assistant**, and can ask for a person at any point.
7. **The merchant can read what was said and can switch it off in one action.**
8. **Buyer personal data sent to a model provider is the minimum the task needs.**

---

## 4. Conversation lifecycle

Every conversation is in exactly one state. This is the centre of the plan; most gaps in
section 2 are the absence of this state machine.

| State | Who replies | Enters when | Leaves when |
|---|---|---|---|
| `assistant` | Assistant sends replies | New conversation on a store in autonomous mode | Any handoff trigger, or closure |
| `suggesting` | Staff, with an assistant draft shown | New conversation on a store in suggest mode | Staff takes over, or closure |
| `human` | Staff only. Assistant is silent and produces no draft | Handoff trigger, or staff sends any message | Staff releases it, or the release timer expires |
| `closed` | Nobody | No activity for the idle period, or staff closes it | A new inbound message opens a new conversation |

**Handoff triggers** (any one moves `assistant` or `suggesting` to `human`):

- The buyer asks for a person, in Bangla, Banglish or English. Matched by a fixed phrase list
  first; the model may also request handoff through a dedicated tool.
- The assistant returned the safe fallback twice in one conversation.
- An inbound message is not text: voice note, image, video, document, location, sticker.
- The model provider failed, timed out or refused.
- The store or sender budget in section 6.7 is exhausted.
- A staff member sends a message in that conversation from any surface, including the Meta
  inbox. Meta echoes page-sent messages back through the webhook; that echo is the signal.
- The conversation mentions a complaint, refund or damaged item (fixed phrase list).

**On handoff:** the buyer receives one fixed, server-written line saying a team member will
reply. Staff receive a notification through the existing web push and email path. The
conversation appears at the top of the inbox with the reason.

**Release:** staff press "return to assistant", or the conversation returns automatically after
a merchant-set period with no staff reply (default: never during business hours, 12 hours
otherwise). Auto-release never happens on a complaint handoff.

**Concurrency:** the state is a column updated with compare-and-swap. A reply generated while
the state changed to `human` is discarded before sending, not after.

---

## 5. Rollout modes per store

A store is in one mode, set by the owner. New stores start in `off`.

| Mode | Behaviour |
|---|---|
| `off` | Messages are stored for the inbox. No model call is made. |
| `suggest` | The assistant drafts a reply for each inbound message. Staff send, edit or discard it. |
| `autonomous` | The assistant replies on its own inside the limits of this plan. |

The dashboard only offers `autonomous` after the store has sent at least 20 suggested replies,
and it shows how many of those staff sent unedited. That number is the merchant's own evidence
for trusting it. The threshold is configuration, not a constant.

A platform operator can force any store, or all stores, to `off`. That is the kill switch. It
takes effect on the next inbound message with no cache to expire, the same way entitlement
changes do.

---

## 6. Workstreams

Each has a decision, the design, and acceptance criteria. Criteria are written to be tested.

### 6.1 Shared inbox and handoff

**Decision.** Build a minimal inbox inside the merchant admin. Staff must not need the Meta
inbox to do their job, but using it must not break anything.

**Design.** Conversation list with state, last message, reason for handoff and unread count.
Conversation view with the transcript, the suggested draft when one exists, a reply box, and
take over / return / close. Replies sent here go out through the existing Meta send path.

**Acceptance.**
1. A conversation is in exactly one state; the column has a database check constraint.
2. Each trigger in section 4 moves the conversation to `human` and has its own test.
3. A staff reply from the Meta inbox moves the conversation to `human` through the echo event.
4. A model reply produced after a state change to `human` is never sent. Proven by a
   concurrency test.
5. Staff receive a notification within the outbox's normal delivery time on handoff.
6. Staff roles: owner and staff may reply; nobody outside the tenant can read a conversation.

### 6.2 Truth contract for assistant wording

**Decision.** Extend the ADR-034 truth contract to the assistant.

**Design.** After `create_social_draft_order` succeeds, the buyer-facing message is a fixed
server template: the request was received, reference `VX-XXXX`, the shop will confirm. The
model's own text for that turn is not sent. A post-generation filter blocks any model reply
that contains order-commitment language (confirmed, booked, shipped, guaranteed, and the Bangla
and Banglish equivalents) and substitutes the safe fallback plus handoff.

**Acceptance.**
7. After a draft is created the buyer receives the fixed template and only that.
8. A reply containing a blocked phrase is not sent and triggers handoff.
9. The phrase list covers English, Bangla script and Banglish, and is data, not code.
10. Templates are localised by conversation language, defaulting to the store's language.

### 6.3 Knowledge base

**Decision.** Replace keyword retrieval with sending the whole knowledge base, up to a cap.
Keep search only as the path for stores above the cap.

**Why.** A merchant's policies are a few pages. Keyword search on the PostgreSQL `simple`
configuration cannot match "dam koto" to a policy written as "price" or in Bangla script. The
whole text fits in the prompt and removes the retrieval failure entirely.

**Design.** Cap by characters. Under the cap, all enabled entries are sent in a stable order.
Over the cap, the dashboard tells the merchant and the existing search is used. Starter
templates for delivery areas and times, return policy, payment methods, business hours and
sizing are offered to a store with an empty knowledge base; the assistant cannot be set to
`autonomous` while the knowledge base is empty.

**Acceptance.**
11. A store under the cap sends every enabled entry on every call, in the same order.
12. A Banglish question about a policy written in Bangla script is answered correctly in the
    evaluation set (6.8).
13. Knowledge text is framed as untrusted reference data, as today.
14. `autonomous` cannot be selected with zero enabled entries.

**Open cost question.** Sending the whole knowledge base raises input tokens per call. The
current 4,096-token conversation budget will not hold it. The new budget must be chosen with
the provider's prompt caching price in hand; see section 12.

### 6.4 Channel connection

**Decision.** Become a Meta Tech Provider and use Embedded Signup for WhatsApp and Facebook
Login for Business for Pages and Instagram. Keep manual credential entry as a fallback.

**Why.** Asking a clothing seller to create a Meta developer app is the largest single barrier
to adoption in this plan.

**Design.** "Connect" opens Meta's popup; the merchant picks the page or number; the token
exchange and webhook subscription happen server side; credentials are stored through the
existing encrypted configuration path. Disconnect revokes the subscription and deletes the
credentials in one transaction, as Search Console disconnect does today.

**Acceptance.**
15. A merchant connects a Page with no credential typed or pasted.
16. Tokens are stored only through the existing encrypted path and never returned by any API.
17. Token expiry or revocation moves the store to `off`, notifies the owner, and is not retried
    in a loop.
18. Disconnect removes credentials and the webhook subscription atomically.

**Lead time.** Business verification and app review are external and slow. This workstream
starts first and finishes last. Meta's current requirements must be read before committing.

### 6.5 Messaging window and policy

**Design.** Meta allows free-form replies only inside a window after the buyer's last message.
Every outbound message checks the window. Outside it, the assistant sends nothing; staff are
told the window is closed and that only an approved template can be sent. The first assistant
message of a conversation states that it is an automated assistant and how to reach a person.

**Acceptance.**
19. No free-form message is sent outside the window; the attempt is recorded as blocked.
20. The first assistant message in every conversation carries the disclosure line.
21. `STOP` handling from ADR-034 still runs before any model call.

The exact window lengths per channel and the human-agent extension are Meta policy and must be
confirmed from Meta's documentation at implementation time, not assumed.

### 6.6 Buyer data sent to the model provider

**Decision.** The model receives message text and tool results. It does not receive the
buyer's phone number, full address or platform sender id.

**Design.** The sender is represented to the model by a per-conversation random label. When
the buyer gives a name, phone and address for an order, those are captured by the draft tool's
structured arguments, validated server side, and replaced in stored and replayed history with
placeholders. The provider is chosen only if its terms state no training on API data and it
offers deletion; the chosen provider and its data location are disclosed in the merchant
terms. The model provider is a configuration value per deployment.

**Acceptance.**
22. No request to the provider contains the sender id or the store's credentials. Asserted by
    a test that inspects the outbound request body.
23. Replayed history contains placeholders, not the phone or address captured earlier.
24. The provider name and data handling are stated in merchant-facing terms before launch.

**Known limit.** A buyer may type a phone number into free text before the assistant asks.
That text reaches the model. Pattern-based masking of phone numbers before the provider call
reduces this; it does not eliminate it. The plan accepts that limit and states it.

### 6.7 Cost, abuse and failure limits

**Design.**

- **Store daily budget** in tokens, set by tier. At the limit the store behaves as `suggest`
  with no draft, which is `human`, until the next UTC day. The owner is notified once.
- **Sender limit**: messages per sender per hour. Over it, the conversation goes to `human`.
- **Trial budget** is lower than Pro. ADR-033 leaves trial uncapped for notifications; model
  spend must not inherit that.
- **Provider failure**: one attempt, no retry of a mutating tool, handoff on failure. A
  circuit breaker opens after repeated failures and sends all conversations to `human` until
  it closes. A second provider may be configured as fallback, but only for reply generation.
- **Counters** are atomic per tenant, like the ADR-034 daily notification caps.

**Acceptance.**
25. Two concurrent conversations at the budget boundary cannot both exceed it.
26. At the budget limit no model call is made and staff are notified.
27. With the provider returning errors, every conversation is handed to staff and no buyer
    receives an error message.
28. One sender cannot cause more than the hourly limit of model calls.

### 6.8 Evaluation

**Decision.** No model, prompt or knowledge-format change ships without passing a fixed set.

**Design.** 150 to 200 conversations collected with consent from pilot merchants, with
personal data removed, covering Bangla script, Banglish, English and mixed. Each has expected
behaviour: correct answer, correct tool call, handoff, refusal, or no price invention. The set
lives in the repository as fixtures. A runner replays it against a configured provider and
reports a pass rate per category. It is run by hand before a change and is not a CI gate,
because it costs money and needs a provider key.

**Launch thresholds.**

| Category | Minimum |
|---|---|
| Invented price, stock or delivery promise | zero cases |
| Order-commitment wording | zero cases |
| Handoff when a person is requested | every case |
| Correct policy answer | set after the first baseline run |
| Correct tool selection | set after the first baseline run |

The two bottom thresholds are not guessed here. They are fixed after the first run shows what
the candidate models can do.

**Acceptance.**
29. The set exists with at least 150 conversations and at least 40 in Banglish.
30. The runner produces a per-category report and exits non-zero on any zero-tolerance failure.
31. The chosen model's report is recorded in `docs/WORK_LOG.md` before launch.

### 6.9 Transcripts and retention

**Decision.** Keep conversation transcripts for 30 days, visible to the merchant, then delete.
Model context still uses only the recent window.

**Why.** With a 24-hour expiry a merchant cannot check what the assistant told a buyer when a
dispute arrives two days later.

**Design.** Separate two things that are one table today: the short model context and the
transcript. The transcript is encrypted at rest with the existing tenant key path, covered by
RLS, included in tenant export and erased by tenant purge. Each assistant message records the
tools called and the knowledge entries supplied, so a wrong answer can be traced to its cause.

**Acceptance.**
32. A merchant can read a conversation from 29 days ago and cannot read one from 31 days ago.
33. Transcripts appear in the tenant export and are removed by tenant erasure.
34. A buyer's `STOP` does not delete the transcript but stops all further processing.
35. Each assistant message links to the tool calls and knowledge entries behind it.

Retention length is a legal and trust decision; 30 days is the proposal, not a requirement.

### 6.10 Order status

**Design.** A read-only tool that takes an order reference and returns status only when the
phone number on the order matches the sender's number on WhatsApp, or the buyer supplies the
phone number on Messenger and Instagram and it matches. It returns status, courier and
tracking reference. It never returns the address, items or amount. A mismatch returns the same
answer as an unknown reference.

**Acceptance.**
36. A correct reference with a wrong phone returns the same response as an unknown reference.
37. The response contains no address, item or amount.
38. Lookups are rate limited per sender.

### 6.11 Fake order defence

**Design.** At staff confirmation the Social Queue shows, for the buyer's phone number within
this store only: previous orders, how many were delivered, returned or refused. No cross-store
data. A first-time number with a cash-on-delivery order is flagged, not blocked.

**Acceptance.**
39. History shown is limited to the current tenant. Proven by an isolation test.
40. A flag never blocks confirmation; it informs it.

### 6.12 Value dashboard

**Design.** Per store, for a chosen period: conversations handled, share resolved without
handoff, drafts created, drafts confirmed, confirmed order value, median first-reply time,
handoff reasons. Deterministic SQL over existing rows, no model call.

**Acceptance.**
41. Every number is reproducible from stored rows and matches a hand count in a fixture.
42. Confirmed order value counts only orders staff confirmed.

---

## 7. Data model additions

Additive only, following the expand phase rules in [DEPLOYMENT.md](DEPLOYMENT.md). All tables
are in the `ai` schema with forced tenant RLS.

| Change | Purpose |
|---|---|
| `state`, `state_reason`, `state_changed_at`, `assigned_user_id` on the conversation | Section 4 |
| `mode` on AI configuration, with a platform override | Section 5 |
| `ai.conversation_message`: direction, author kind, encrypted body, sent status, expiry | Transcript, 6.9 |
| `ai.message_trace`: tool calls and knowledge entry ids per assistant message | Traceability |
| `ai.usage_counter`: tenant, UTC day, tokens, calls | Budgets, 6.7 |
| `ai.sender_window`: hashed sender, hour, count | Sender limit |
| `ai.suggestion`: draft text, outcome (sent, edited, discarded) | Suggest mode |

Sender identifiers are stored hashed with a tenant-scoped key, as quote binding does today.

---

## 8. Packaging

The assistant stays a Pro and trial feature under the existing `ai_assistant` entitlement.
Two changes to [ENTITLEMENTS.md](ENTITLEMENTS.md) are needed and are the owner's decision:

- A daily model budget per tier, lower for trial.
- Whether the inbox without the assistant (`off` mode: read and reply by hand) is available
  on Free. It costs no model spend and is a strong reason to stay on the platform.

---

## 9. Order of work

Each phase ends at a gate. A later phase does not start because an earlier one is nearly done.

| Phase | Work | Gate to leave the phase |
|---|---|---|
| 0 | Start the Meta Tech Provider application (6.4). Collect evaluation conversations (6.8). Accept the amendments in section 12. | Application submitted; 150 conversations collected |
| 1 | Lifecycle and inbox (4, 6.1), modes and kill switch (5), truth contract (6.2), disclosure and window (6.5), budgets and failure handling (6.7) | Criteria 1 to 10 and 19 to 28 pass; one internal store runs in `suggest` |
| 2 | Knowledge base change (6.3), data minimisation (6.6), transcripts (6.9), evaluation runner (6.8) | Zero-tolerance categories pass on the chosen model |
| 3 | Pilot with a small number of real merchants in `suggest` mode only | Merchants send most suggestions unedited; no zero-tolerance incident |
| 4 | `autonomous` for pilot merchants, then general availability | Two weeks with no commitment or isolation incident |
| 5 | Order status (6.10), fake order defence (6.11), value dashboard (6.12), embedded signup once Meta approves | Each workstream's criteria |

Phase 1 is the minimum before any real buyer talks to the assistant. Phases 1 and 2 are the
minimum before charging for it.

---

## 10. Edge cases

| Case | Required behaviour |
|---|---|
| Buyer sends five messages in ten seconds | Coalesce into one model call after a short debounce; reply once |
| Staff and assistant reply at the same moment | State check before send discards the assistant reply |
| Webhook delivered twice or out of order | Existing dedup and ordering; state transitions are idempotent |
| Buyer writes in a language the store has no policy for | Answer from tools if possible, otherwise handoff; never translate a policy that was not written |
| Buyer pastes instructions to ignore rules | Treated as data; tool allowlist and server price authority are the control, not the prompt |
| Product goes out of stock mid-conversation | Tools return live state; confirmation revalidates, as today |
| Token revoked by Meta | Store moves to `off`, owner notified, no retry loop |
| Merchant downgrades from Pro during a conversation | Next inbound message makes no model call; conversation becomes `human` |
| Budget exhausted mid-conversation | Current reply completes; next message is handed off |
| Provider returns a reply after its timeout | Discarded; the conversation was already handed off |
| Buyer asks about another buyer's order | Order status tool returns the unknown-reference answer |
| Two staff reply to the same conversation | Both sent; the conversation shows who sent each |
| Store suspended | No model call, no outbound message |
| Buyer says STOP | Consent revoked before any model call; conversation closed |
| Assistant is asked for medical, legal or financial advice | Declines and offers a person |
| Voice note or image | Handoff with the reason shown to staff |

---

## 11. What this plan does not do

- No autonomous order confirmation, payment capture or dispatch. ADR-032 stands.
- No model-chosen discounts. Owner-authored chat offers remain the only path.
- No voice or image understanding. Those messages go to staff.
- No outbound marketing or broadcast messages.
- No comment-to-inbox automation on Facebook posts. It is common in this market and is the
  natural next feature, but it has its own Meta permissions and policy.
- No fine-tuning and no shared corpus across merchants.
- No cross-store buyer reputation. It would be useful against fake orders and it would break
  tenant isolation.

---

## 12. Decisions the owner must make

| # | Decision | Proposal |
|---|---|---|
| 1 | Amend ADR-032 decision 2: whole knowledge base in the prompt instead of retrieval | Accepted 2026-11-14 |
| 2 | Amend ADR-032: add conversation states, modes and the kill switch | Accepted 2026-11-14 |
| 3 | Transcript retention, replacing the 24-hour expiry for merchant-visible history | Accepted 2026-11-14: 30 days |
| 4 | Model provider | Decided 2026-11-14: a low-cost Chinese model (DeepSeek or Qwen class, as ADR-032 decision 3 already says). Which one is picked by the first evaluation run (6.8) and a read of its API data terms (6.6); the provider stays a configuration value so it can be swapped |
| 5 | Daily model budget per tier | Decide with real token prices and the new prompt size |
| 6 | Apply to become a Meta Tech Provider | Start now |
| 7 | Inbox without the assistant on the Free tier | Owner's call; no model cost either way |
| 8 | Default mode for a new Pro store | `suggest` |

Decisions 1 to 3 are recorded in the [ADR-032](DECISIONS.md#adr-032) amendment.

---

## 13. Not verified

- Meta's current messaging window lengths, the human-agent extension, Tech Provider
  requirements, Embedded Signup availability for Bangladesh, and review timelines.
- Model quality in Banglish. No candidate model has been measured.
- Token cost of the whole-knowledge-base prompt and whether the provider's prompt caching
  applies to it.
- Whether Meta echoes page-sent messages for every channel in the form section 4 relies on.
- The legal position on sending buyer messages to a model provider outside Bangladesh.
- Section 2 was established by reading and searching source. No part of the assistant was run.

Each item is a check to perform, with its result recorded in
[WORK_LOG.md](WORK_LOG.md), before the phase that depends on it.
