# Security and Compliance Specification

**Status:** Draft v1.0 · **Scope:** the whole spec package — engine, Store and Admin APIs, forked dashboard, forked storefront

This document exists because an audit of the other ten documents found **29 gaps**, three of them
severe enough to block an EU launch. The original package specified "OWASP ASVS Level 2" and
listed eight bullet points. That is a posture statement, not a security design.

Findings are in §2. Everything after §3 is the specification that closes them.

The one flow that warrants more than a control set is checkout — money, inventory, PII, and a
third party all cross there. [THREAT_MODEL_CHECKOUT.md](THREAT_MODEL_CHECKOUT.md) is the run sheet
for that session, scheduled at the end of P3.

---

## 1. Compliance obligations

These are legal or contractual, not aspirational. Each names what it forces on the build.

| Standard | Applies when | What it forces |
|---|---|---|
| **PCI DSS v4.0.1** | Always — you take card payments | SAQ-A posture (§10). Note the 2025 eligibility change below |
| **GDPR** (EU 2016/679) | Any EU customer | Erasure, portability, retention limits, breach notice in 72 h, DPA records (§11) |
| **UK GDPR / DPA 2018** | Any UK customer | Materially the same as GDPR |
| **CCPA/CPRA** | California residents, over thresholds | Deletion, opt-out of sale/sharing, disclosure |
| **European Accessibility Act** | **Enforceable since 28 June 2025** for EU e-commerce | EN 301 549 / WCAG 2.1 AA minimum. Exempt only if under 10 staff **and** under €2M revenue |
| **PSD2 / SCA** | EEA + UK card payments | 3DS step-up — already specified in [PAYMENTS §8.4](PAYMENTS.md) |
| **OWASP ASVS 4.0 Level 2** | Self-imposed baseline | The control set in §4–§14 |
| **EU consumer law** | EU B2C | 14-day withdrawal right — a returns-flow requirement, not just a UI one |

### 1.1 The PCI change that catches people out

PCI DSS v4.0.1 SAQ A (r1), effective **31 March 2025**, **removed** requirements 6.4.3 and 11.6.1
from the SAQ A control list — but added an **eligibility criterion** in their place: the merchant
must confirm their site *is not susceptible to script attacks that could affect the e-commerce
system*.

So the obligation did not disappear, it moved. You satisfy it one of two ways:

1. Implement the techniques described in 6.4.3 and 11.6.1 — script inventory, authorisation,
   integrity assurance, and change detection on the payment page; **or**
2. Obtain written confirmation from your PSP that their embedded form, implemented per their
   instructions, protects the payment page from script attacks.

**It applies to embedded forms (iframes / Payment Element), not to full redirects.** Since the
storefront uses Stripe Payment Element, it applies. §10 specifies the controls.

Getting this backwards — assuming SAQ A now has *fewer* obligations — is the common error.

---

## 2. Audit findings

Against the spec package as written before this document. Severity is the impact if it reached
production unaddressed.

### Critical — blocks launch

| # | Finding | Why it is critical |
|---|---|---|
| **F1** | **No CSRF defence.** [API_SPEC §1.2](API_SPEC.md) authenticates the admin with a `__stheno_jwt` cookie. Cookie auth without CSRF controls means any site an operator visits can issue authenticated admin requests | Full admin takeover via a link. The single most exploitable gap in the package |
| **F2** | **No data-erasure design.** GDPR/CPRA absent entirely. Compounded by universal soft delete (`deleted_at`) — data is never actually removed | Illegal to operate in the EU. Fines to 4% of global turnover |
| **F3** | **No audit log.** Order timelines and workflow executions exist, but nothing records *who* changed a price, issued a refund, or exported customers | PCI DSS req 10, SOC 2 CC7. No forensics after an incident |

### High

| # | Finding | Consequence |
|---|---|---|
| **F4** | Mass assignment unaddressed — nothing forbids deserialising a request into an entity struct | Client sets `id`, `amount`, `status`, `is_custom_price` |
| **F5** | IDOR not a structural rule. "Customer-scoped" appears as prose on `/store/orders`, enforced nowhere | Read any customer's orders by id |
| **F6** | Cart ownership undefined. A `cart_id` is a bearer credential with no binding to a session | Cart takeover; PII disclosure from addresses |
| **F7** | No security headers. HSTS, CSP, `X-Content-Type-Options`, `Referrer-Policy` all absent | XSS, clickjacking, protocol downgrade |
| **F8** | File uploads have a size cap and nothing else — no type allowlist, no content sniffing | Stored XSS via SVG; malware distribution from your domain |
| **F9** | SSRF unguarded. OIDC discovery URLs, S3 endpoints, and webhook targets are admin-configurable | Cloud metadata theft (IMDS), internal network scanning |
| **F10** | Promotion codes have no brute-force protection while auth endpoints do | Discount-code enumeration; margin loss |
| **F11** | Filter grammar has field allowlists but no value-count or pattern limits | `$in` with 100k values; `$ilike` `%a%` full scans → DoS |
| **F12** | Webhooks listed as "unlimited, signature-gated" in the rate-limit table | Unauthenticated HMAC computation flood → CPU exhaustion |
| **F13** | No session invalidation on password/MFA change | Stolen token survives the victim's remediation |
| **F14** | User enumeration unaddressed on register, login, and password reset | Customer list harvesting; credential stuffing target list |

### Medium

| # | Finding | Consequence |
|---|---|---|
| **F15** | `metadata` jsonb on every entity: unbounded, and rendered in the admin | Storage exhaustion; stored XSS sink |
| **F16** | **`payment_webhook_event` (added by me in PAYMENTS.md) stores raw provider payloads with PII and card metadata, with no retention limit** | Self-inflicted PII lake |
| **F17** | **`workflow_execution.context` jsonb stores step inputs — addresses, emails, names — indefinitely** | PII outside every erasure path |
| **F18** | [TRD §8.2](TRD.md) calls publishable-key scope resolution "the store-side security boundary". A publishable key is *public* | Overstated control; invites treating it as authentication |
| **F19** | Rate limiting is per-IP with no trusted-proxy configuration | Trivially bypassed; or spoofed via `X-Forwarded-For` |
| **F20** | Secret API keys are all-or-nothing | No least privilege for integrations |
| **F21** | `display_id` is sequential and customer-visible | Leaks order volume to competitors |
| **F22** | Anonymous carts are unauthenticated, unbounded, and never expire | Storage DoS |
| **F23** | Encryption at rest specified only for MFA secrets | Provider tokens, `data` jsonb unprotected |
| **F24** | Backups: PITR mentioned, encryption not | Plaintext PII in backup storage |
| **F25** | PCI SAQ-A script-attack eligibility criterion unaddressed (§1.1) | SAQ A ineligibility → SAQ D |
| **F26** | EAA not named; WCAG treated as a quality goal | Legal exposure in the EU |
| **F27** | Soft delete conflicts directly with erasure | "Deleted" customers remain in full |

### Low

| # | Finding |
|---|---|
| **F28** | No sanctions/OFAC screening consideration for international orders |
| **F29** | No security disclosure policy, `security.txt`, or CVE process |

### Checkout Threat Model Findings (P3-19 / THREAT_MODEL_CHECKOUT.md)

| # | Finding | Impact | Mitigation / Control | Owner | Phase |
|---|---|---|---|---|---|
| **F30** | Card-testing bot abuse on payment session endpoints | Money / Fees | Pre-session velocity limits per IP and device fingerprint, progressive CAPTCHA | Payments Lead | P4 (P4-27) |
| **F31** | Promo code brute-force / enumeration via `/store/carts/:id/promotions` | Money / Margin | Max 5 failed attempts/cart/hr, per-IP rate limit, constant-time compare | Promotions Lead | P4 (P4-15) |
| **F32** | Cart hijacking / IDOR via guessable cart IDs | PII / Trust | Cryptographic `cart_secret` issued in HttpOnly cookie, verified on every mutation | Cart Lead | P4 (P4-06) |
| **F33** | Race condition in campaign budget depletion across parallel checkouts | Money | Atomic budget consumption under Postgres row lock (`SELECT FOR UPDATE`) | Promotions Lead | P4 (P4-13, P4-14) |
| **F34** | Ambiguous PSP timeouts causing double charges or orphaned authorizations | Money / Trust | Idempotent provider wrapper with deterministic `idempotency_key` and `retrieve()` reconciliation | Payments Lead | P4 (P4-17, P4-18) |
| **F35** | Inventory depletion lockup without purchase (reservation hoarding) | Availability | Reservations created only upon checkout step 5 with strict TTL and compensation rollback | Inventory Lead | P4, P5 |
| **F36** | Client-side price / totals tampering during cart completion | Money | Step 4 of `complete-cart` always recomputes line totals, taxes, discounts from source tables server-side | Engine Lead | P4 (P4-02) |
| **F37** | Webhook replay or out-of-order execution race | State Corruption | `payment_webhook_event` store with monotonic convergence reducer and timestamp ordering | Payments Lead | P4 (P4-19, P4-20) |
| **F38** | Unauthorized admin refund / capture manipulation | Money / Theft | Immutable transaction ledger constraints, two-man rule on high-value refunds, strict audit logging | Security Lead | P4 (P4-21, P4-22) |

---

## 3. Threat model

Attackers this design assumes, in descending order of likelihood.

| Actor | Goal | Primary vectors |
|---|---|---|
| **Opportunistic bot** | Credential stuffing, card testing, scraping | Login, register, promo codes, payment sessions, catalog |
| **Fraudulent customer** | Free or cheap goods | Price manipulation, promo abuse, refund/return abuse, chargebacks |
| **Competitor** | Business intelligence | Order-volume enumeration, price scraping, inventory levels |
| **Compromised operator** | Theft, sabotage | Excess RBAC, missing audit trail, bulk export |
| **Malicious integrator** | Lateral movement | Provider config (SSRF), API key scope, webhook forgery |
| **Supply-chain attacker** | Card skimming (Magecart) | Third-party scripts on the payment page |

The two highest-value assets are **card data in transit through the browser** (Magecart, §10) and
**the customer PII store** (§11). Everything else is recoverable.

---

## 4. Transport and headers — closes F7

TLS 1.2+ terminated ahead of the app; the app refuses plaintext except on `/health/live`.

Emitted by `tower-http` middleware on every API response:

```
Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
X-Content-Type-Options: nosniff
Referrer-Policy: strict-origin-when-cross-origin
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Resource-Policy: same-site
Permissions-Policy: geolocation=(), camera=(), microphone=(), payment=(self)
Cache-Control: no-store        # on every authenticated response
```

**CSP** — both Next.js apps, nonce-based, no `unsafe-inline`:

```
default-src 'self';
script-src  'self' 'nonce-{random}' https://js.stripe.com;
frame-src   https://js.stripe.com https://hooks.stripe.com;
connect-src 'self' https://api.stripe.com;
img-src     'self' data: https://{cdn-host};
style-src   'self' 'nonce-{random}';
object-src  'none'; base-uri 'none'; frame-ancestors 'none';
form-action 'self'; upgrade-insecure-requests;
report-uri  /csp-report
```

`frame-ancestors 'none'` on the admin. CSP violations are reported and monitored — the report
endpoint is also the Magecart tripwire (§10).

**CORS** is an explicit origin allowlist per surface (`STORE_CORS`, `ADMIN_CORS`, `AUTH_CORS`).
Wildcard with credentials is rejected at boot, not at request time.

---

## 5. Authentication — closes F13, F14

Baseline from [PRD §5.10](PRD.md) stands. Additions:

**Anti-enumeration (F14).** Register, login, and password reset return **identical responses and
timing** whether or not the account exists. Reset always answers "if an account exists, we sent an
email." Registration collision is reported by email, never in the HTTP response. Timing is
equalised by always performing a hash operation, including against a dummy hash on miss.

**Credential-change invalidation (F13).** Every `auth_identity` carries a `token_version`, embedded
in issued JWTs. Password change, MFA enrolment or removal, and email change all increment it,
invalidating every outstanding access and refresh token. Access-token TTL is 15 minutes, so the
window is bounded even before the check.

**Account-change notification (F28).** Password, email, MFA, and payment-method changes email the
**previous** address. This is what turns a silent takeover into a report.

**Brute force.** Per-identifier and per-IP, exponential backoff, lockout at 10 failures in 15
minutes with an operator-visible unlock. Lockout is per identifier, not global, so it is not itself
a denial-of-service lever.

**Password policy.** Minimum 12 characters, checked against a breached-password list (k-anonymity
range query, no plaintext leaves the process). No composition rules, no forced rotation — current
NIST guidance.

**MFA.** Mandatory for any operator holding a role with refund, export, or user-management
permissions. Optional for customers. TOTP secrets and recovery codes are encrypted at rest (§13).

---

## 6. Sessions and CSRF — closes F1

**The critical finding.** The admin authenticates with a cookie, which means every state-changing
admin request is cross-site forgeable unless defended.

**Constraint from [ADR-002](DECISIONS.md#adr-002):** the forked dashboard authenticates the way
Medusa's does. Our CSRF controls must therefore be **additive** — they may require a header the
dashboard can be patched to send, but they may not change the login response shape. The fork is
ours, so adding a header to its API client is a small, acceptable change; changing the wire
contract is not.

Cookie attributes:

```
__stheno_jwt: HttpOnly; Secure; SameSite=Lax; Path=/; Domain=<exact>
__stheno_csrf: Secure; SameSite=Lax; Path=/          # readable by JS, double-submit
```

`SameSite=Lax` alone is **not sufficient** — it permits top-level `GET` navigation, and any
misclassified state-changing `GET` remains exploitable. Three controls, all required together:

1. **Double-submit token.** `__stheno_csrf` must be echoed in an `X-Stheno-CSRF` header. The pair
   is compared in constant time. The token is bound to the session, not global.
2. **Origin/Referer validation.** Every state-changing request must carry an `Origin` (or
   `Referer`) matching the configured admin origin. Absent or mismatched → 403.
3. **Custom-header requirement.** Cookie authentication is accepted **only** with the custom
   header present, which forces a CORS preflight that a cross-site form cannot satisfy.

**Bearer-token requests are exempt** — a token in an `Authorization` header cannot be attached
ambiently by a browser, so there is nothing to forge.

Enforced in middleware ahead of every handler, so a new route cannot forget it. A route may opt out
only via an explicit `#[csrf_exempt]` marker that CI reports in the PR.

**Store API** is not cookie-authenticated for mutations — the customer JWT is a Bearer token — so
it is structurally exempt. The cart cookie (§8) is the exception and gets the same treatment.

**Fork task:** the dashboard's API client must be patched to send `X-Stheno-CSRF` on every
mutating request. One interceptor, tracked as a task in [TASKS.md](TASKS.md).

---

## 7. Authorization — closes F5, F20

**Ownership is a type, not a convention (F5).** Store handlers cannot receive a bare id:

```rust
// ❌ impossible to construct in a store route
async fn get_order(Path(id): Path<OrderId>) -> Result<Json<Order>>

// ✅ the extractor resolves and verifies ownership, or 404s
async fn get_order(Owned(order): Owned<Order, ByCustomer>) -> Result<Json<Order>>
```

`Owned<T, Scope>` performs the scoped fetch. A store route that queries by raw id is a CI failure,
alongside the module-isolation check. **404, never 403**, on ownership failure — a 403 confirms the
object exists.

**Sales-channel scope** is applied by the router, not handlers ([TRD §8.2](TRD.md)).

**Correction to F18:** a publishable API key is *public* — it ships in browser JavaScript. It is a
**scoping** mechanism, not authentication, and must never be the only thing between a request and
data. Store endpoints are public by design; the key narrows which channel's data is visible. The
docs are corrected to say so.

**Secret key scoping (F20).** `api_key` gains a `scopes text[]` column. A key for an ERP
integration gets `orders:read, inventory:write` — not the ability to create admin users. Scopes are
checked alongside RBAC, and the intersection wins.

**RBAC** per [PRD FR-AUTH-7](PRD.md): deny beats allow across the inheritance closure. Additionally,
permissions that move money or export PII (`refund:create`, `customer:export`, `user:create`,
`api_key:create`) are flagged **sensitive** — they require MFA on the session and are always
audit-logged (§12).

---

## 8. Business-logic abuse — closes F6, F10, F21, F22

E-commerce is attacked through logic far more often than through injection.

**Cart ownership (F6).** A `cart_id` is currently a bearer credential. Bind it:

- Cart id lives in an `HttpOnly; Secure; SameSite=Lax` cookie, never a URL.
- A cart with a `customer_id` is accessible **only** with that customer's JWT.
- An anonymous cart is bound to a server-issued `cart_secret` (128-bit) held in the cookie; the id
  alone is insufficient.
- Claiming an anonymous cart on login transfers ownership and rotates the secret.

**Cart lifecycle (F22).** Anonymous carts expire after 30 days, authenticated after 90; a scheduled
job deletes them. Cart creation is rate-limited per IP. An abandoned cart holds no inventory —
reservations only occur at completion ([WORKFLOWS §2.3](WORKFLOWS.md)) — which is a deliberate
design property, not an accident.

**Price manipulation.** The Store API **never** accepts a price. `unit_price` and `is_custom_price`
are admin-only fields, rejected by the store request DTO at the type level (§9). Totals are always
recomputed server-side. This is already the design; it is now stated as a rule so it cannot erode.

**Promotion abuse (F10).**
- Code attempts rate-limited to 10 per cart and 20 per IP per hour; failures are counted, and
  enumeration triggers backoff.
- Codes are compared in constant time, and generated codes use ≥ 64 bits of entropy.
- Per-customer usage limits enforced under the same atomic pattern as campaign budgets
  ([DATA_MODEL §2.5](DATA_MODEL.md)).
- Campaign budget consumption is already race-free; that is what stops parallel-order budget
  overrun.

**Order enumeration (F21).** `display_id` stays sequential — operators need it and Medusa parity
requires it — but it is **never** an API lookup key. All lookups use the ULID. Customer-facing
order URLs use the ULID, so the sequential number is visible only to someone who already has the
order.

**Refund and return abuse.** Return windows enforced server-side. Refund totals bounded by the
ledger invariants in [PAYMENTS §3.3](PAYMENTS.md). Repeat-refund customers surfaced to operators
rather than auto-blocked.

**Card testing.** Payment-session creation is rate-limited per IP and per cart; a burst of failed
authorizations from one source trips a temporary block. This is the attack that gets a merchant
account terminated, so it is a launch requirement, not a hardening nicety.

---

## 9. Input handling — closes F4, F11, F15

**Mass assignment (F4).** Request DTOs and entity structs are **separate types, always**. An entity
never derives `Deserialize`. There is no `impl From<Json> for Product`. The DTO names every
accepted field explicitly; anything else is ignored or rejected. A handler taking an entity as its
body type is a CI failure.

Store and admin DTOs are distinct even for the same entity, because the admin may set fields the
store must not (`unit_price`, `status`, `metadata`).

**Filter DoS (F11).** Beyond the per-field allowlists already specified:

| Limit | Value |
|---|---|
| `$in` / `$nin` array length | 100 |
| Filter tree depth | 3 |
| Total filter predicates | 20 |
| `$like` / `$ilike` | Trailing-wildcard only (`foo%`). Leading `%` rejected |
| Free-text `q` length | 128 chars |
| Sort fields | Indexed columns only, per an allowlist |
| Page size | 100 (store), 200 (admin) |

Leading-wildcard `ILIKE` is the single worst offender — it defeats every index. Free-text search
routes to the search provider, which is what it is for.

**`metadata` (F15).** Capped at 8 KB serialised, 50 keys, depth 3. Keys match
`^[a-zA-Z0-9_.-]{1,64}$`. Values are strings, numbers, or booleans — no nested objects. The admin
renders metadata as **text, never HTML**, and the CSP blocks inline execution regardless.

**Injection.** SQL injection is structurally prevented by Diesel's builder; `sql_query` sites use
bound parameters only and carry a review marker ([AGENTS §2](../AGENTS.md)). Postgres identifiers
are never interpolated from user input — sort and filter fields resolve through allowlists to
static column references.

**Body limits.** 2 MB default, 20 MB uploads, 1 MB webhooks. Enforced before parsing.

---

## 10. Payments and PCI — closes F25

Controls are in [PAYMENTS.md](PAYMENTS.md). What this section adds is the PCI posture.

**Scope.** Card data never touches Stheno. Stripe Payment Element collects it in a Stripe-hosted
iframe; we see tokens and intent ids. This is the **SAQ A** posture.

**The eligibility criterion (§1.1).** Because the storefront uses an embedded form, we must confirm
the payment page is not susceptible to script attacks. Both routes are taken:

1. **Written PSP confirmation** — obtain and retain Stripe's statement that Payment Element,
   implemented per their instructions, protects against script attacks. Filed with the compliance
   record, refreshed annually.
2. **The techniques anyway**, because a Magecart skimmer is the highest-value attack on this system
   and the confirmation does not protect against *our own* supply chain:
   - **Script inventory.** Every script on the checkout route is enumerated, justified, and
     recorded in `docs/payment-page-scripts.md`. Adding one requires review.
   - **Integrity.** Subresource Integrity on all third-party scripts; nonce-based CSP with no
     `unsafe-inline`; `script-src` on checkout limited to `'self'` and `js.stripe.com`.
   - **Change detection.** CSP violation reports monitored and alerted. CI fails if the built
     checkout bundle's script set differs from the manifest.
   - **No tag managers, analytics, chat widgets, or A/B tools on checkout routes.** This is the
     rule that actually prevents Magecart, and it is a product constraint as much as a technical
     one — marketing will ask, and the answer is no.

**Cardholder-data hygiene.** No PAN, CVV, or track data may be logged, stored, or placed in
`metadata`. The log redaction allowlist (§13) treats any 13–19 digit sequence as suspect and
redacts it.

**Annual obligations.** SAQ A self-assessment, quarterly ASV scans if in scope, and an annual
penetration test ([IMPLEMENTATION_PLAN P10](IMPLEMENTATION_PLAN.md)).

---

## 11. Data protection and privacy — closes F2, F16, F17, F27

**The second critical finding.** The package had no erasure design, and universal soft delete makes
the problem worse.

### 11.1 Data inventory

Every store of personal data must be known, or erasure is a guess:

| Store | Personal data | Erasure |
|---|---|---|
| `customer`, `address` | name, email, phone, address | Pseudonymise |
| `order`, `order.address` | shipping/billing identity | **Pseudonymise, retain the record** |
| `cart` | email, addresses | Delete |
| `auth_identity`, `provider_identity` | email, hashes, provider ids | Delete |
| `notification` | email/phone, rendered body | Delete after retention |
| `payment.data`, `payment_session.data` | provider payloads, last4, name | Redact to non-identifying fields |
| **`payment_webhook_event.payload`** | full provider payload | **90-day retention, then purge (F16)** |
| **`workflow_execution.context`** | step inputs: addresses, emails, names | **Retention window + erasure cascade (F17)** |
| `index_data` | denormalised projections of the above | Rebuild after erasure |
| `search_index` | names, emails | Reindex after erasure |
| Logs, traces | request context | Redaction at write; 30-day retention |
| Backups | everything | Documented lag; erasure replays on restore |

`payment_webhook_event` and `workflow_execution.context` are **self-inflicted** — both were
introduced by this spec package. Any new jsonb column that can hold request data must be added to
this table in the same PR.

### 11.2 Erasure, done correctly

Financial records must be retained (EU tax law commonly 7–10 years) while identity must be
removable. These are reconciled by **pseudonymisation, not deletion**:

```
erase_customer(customer_id):
  1. auth_identity, provider_identity          → hard delete
  2. customer, address                         → PII columns overwritten:
                                                    email → sha256(email+pepper)@erased.invalid
                                                    name, phone, lines → tombstone
                                                 customer.erased_at = now()
  3. order.*_address                           → tombstone; order, items, totals, transactions kept
  4. cart (incomplete)                         → hard delete
  5. notification                              → hard delete
  6. payment.data / session.data               → redact to {last4, brand, provider_ref}
  7. workflow_execution.context                → redact where it references the subject
  8. index_data, search_index                  → rebuild for affected entities
  9. record an erasure_request row (id, requested_at, completed_at, actor) — itself audit evidence
```

The hashed email allows fraud correlation ("has this person been refunded before") without holding
an identifier — the standard reconciliation of erasure with legitimate interest.

**Soft delete (F27).** `deleted_at` is a *business* delete, not a privacy one. Erasure is a
separate, destructive path. Documenting the difference prevents the common failure of assuming one
implies the other.

### 11.3 Retention

| Data | Retention | Basis |
|---|---|---|
| Order and financial records | 10 years | Tax/accounting law |
| Customer profile | Until erasure, or 3 years after last order | Legitimate interest |
| Incomplete carts | 30 / 90 days | Necessity |
| `payment_webhook_event` | 90 days | Dispute window |
| `payment_provider_request` | 30 days | Idempotency window |
| `workflow_execution` | 30 days (90 for order flows) | Operations |
| Logs and traces | 30 days | Operations |
| Audit log | 2 years | PCI req 10.5 minimum is 1 year |
| Backups | 35 days | RPO |

Enforced by scheduled jobs, not by intention. A retention job that has not run in 48 hours alerts.

### 11.4 Other subject rights

- **Access / portability** — `GET /store/customers/me/export` produces machine-readable JSON of all
  personal data within 30 days, rate-limited and re-authenticated.
- **Rectification** — existing profile endpoints.
- **Objection / opt-out** — marketing consent stored with timestamp and source; analytics
  (`stheno-analytics`) respects it and is off by default.
- **Breach notification** — 72-hour obligation; the runbook in §15 is what makes that achievable.

### 11.5 Data residency

EU customer data stays in EU regions when the deployment declares `DATA_REGION=eu`. This is a
deployment constraint the docs must state, since Postgres, S3, the OTLP endpoint, and the email
provider must all agree.

---

## 12. Audit logging — closes F3

**The third critical finding.** Required by PCI DSS req 10 and SOC 2, and the only thing that makes
an incident investigable.

New table `audit_log`, append-only, no `UPDATE` or `DELETE` grant to the application role:

| Column | Purpose |
|---|---|
| `id`, `occurred_at` | |
| `actor_type`, `actor_id`, `actor_email` | who — email denormalised so the record survives erasure |
| `action` | `order.refund`, `product.price_update`, `user.role_grant`, `customer.export` |
| `resource_type`, `resource_id` | what |
| `before`, `after` jsonb | redacted diff |
| `ip`, `user_agent`, `trace_id` | context |
| `outcome` | `success \| denied \| error` |

**Logged always:** authentication (success and failure), authorization denials, any RBAC or API-key
change, any money movement (refund, capture, manual payment), price and promotion changes, bulk
exports, customer erasure, configuration changes, and every use of a **sensitive** permission (§7).

**Not logged:** ordinary reads. Volume without signal is how audit logs stop being read.

Exposed at `GET /admin/audit-log`, filterable, permission-gated, and **exportable but never
mutable**. Shipped to an external sink where the deployment provides one — an audit log an attacker
with database access can rewrite is not evidence.

---

## 13. Secrets, crypto, and logging — closes F19, F23, F24

**Secrets.** Environment variables only; never in the database, never in `metadata`. `JWT_SECRET`
and `COOKIE_SECRET` at least 32 bytes, boot fails otherwise (already specified). Rotation is
supported via `JWT_SECRET_PREVIOUS`, honoured for verification during an overlap window — otherwise
rotation logs every operator out.

**Encryption at rest (F23).** Beyond full-disk: application-level AEAD (XChaCha20-Poly1305) with a
key from `ENCRYPTION_KEY`, for MFA secrets, recovery codes, provider credentials, `account_holder.data`,
and any `data` jsonb holding provider tokens. Key rotation re-encrypts in a background job.

**Backups (F24).** Encrypted at rest and in transit, access-logged, restore tested quarterly.
Backups contain pre-erasure data — the retention window is documented as a known lag, and erasure
requests are replayed after any restore.

**Log redaction.** Deny-by-default on structured fields: `password`, `token`, `secret`,
`authorization`, `cookie`, `card`, `cvv`, `iban`, `email`, `phone`, `address`. Any 13–19 digit
sequence is masked. Redaction is a `tracing` layer, so it cannot be bypassed by a careless
`info!()`.

**Rate limiting (F19).** Client IP is derived from `X-Forwarded-For` **only** for a configured
`TRUSTED_PROXIES` CIDR list; otherwise the socket peer is used. Without this, the header is
attacker-controlled and every per-IP limit is decorative. Limits are per-IP **and** per-actor, and
sensitive endpoints (auth, promo codes, payment sessions, exports) get their own buckets.

---

## 14. Outbound requests and supply chain — closes F9

**SSRF (F9).** Any admin-configurable URL — OIDC discovery, S3 endpoint, webhook target, image
source — passes an egress guard before use:

- Scheme `https` only.
- DNS resolved **and pinned**, then the resolved IP checked against a denylist: RFC 1918,
  loopback, link-local (**169.254.0.0/16 — cloud metadata**), CGNAT, IPv6 ULA and mapped
  equivalents. Resolve-then-connect on the same address defeats DNS rebinding.
- Redirects not followed by default; when required, each hop is re-checked.
- 5-second timeout, 5 MB response cap.
- Outbound egress runs from a network context with no access to the metadata endpoint. Defence in
  depth, since the guard is code and code has bugs.

**File uploads (F8).** Presigned URLs are not a security control.

- Extension **and** sniffed content type must both be in the allowlist:
  `jpeg, png, webp, avif, gif, pdf, csv`.
- **SVG is not allowed.** It is an XSS vector, and no product image needs it.
- Content sniffed from magic bytes server-side; a mismatch is rejected. Images are re-encoded,
  which strips embedded payloads and EXIF.
- Filenames are generated, never taken from the client — no path traversal, no double extensions.
- Served from a **separate origin** with `Content-Disposition: attachment` for non-images and
  `X-Content-Type-Options: nosniff`, so a stored payload cannot execute in the app's origin.
- Size caps per type; per-actor upload rate limits.

**Dependencies.** `cargo audit` and `cargo deny` gate CI; npm dependencies scanned in the same
pass. Lockfiles committed. New dependencies follow [AGENTS §9](../AGENTS.md). An SBOM is generated
per release. Container images are rebuilt weekly for base-image CVEs even without a code change.

---

## 15. Security testing and operations

| Activity | Cadence | Gate |
|---|---|---|
| SAST (`clippy`, `semgrep`) | Every PR | Blocking |
| Dependency audit | Every PR + daily | Blocking on high/critical |
| Secret scanning (pre-commit + CI) | Every PR | Blocking |
| DAST (ZAP) against a seeded instance | Nightly | Blocking on high |
| Abuse-case tests (§8 scenarios) | Every PR | Blocking |
| Authorization matrix test — every role × every endpoint | Every PR | Blocking |
| **Checkout threat-modelling session** | End of P3, then on material change | Findings appended as F30+ before P4 payment code |
| Penetration test | Annual + before 1.0 | No high/critical to launch |
| Access review | Quarterly | — |
| Restore drill | Quarterly | — |
| Breach-response tabletop | Annual | — |

**The authorization matrix test deserves emphasis:** generate every (role, endpoint, ownership)
combination and assert the expected allow/deny. This is the only reliable defence against the
authorization gaps that creep in as endpoints multiply — and with ~278 admin routes, they will.

**Disclosure (F29).** `/.well-known/security.txt`, a published policy with a 90-day disclosure
window, and a `SECURITY.md` in the repository. Advisories issued as GitHub Security Advisories with
CVEs.

**Incident response.** Documented runbook: detect → contain → eradicate → recover → review.
Named on-call owner. GDPR's 72-hour clock starts at *awareness*, so the detection path — CSP
reports, audit-log anomalies, reconciliation drift ([PAYMENTS §9](PAYMENTS.md)), failed-auth
spikes — is part of the control, not an afterthought.

---

## 16. Where this lands in the plan

| Phase | Security deliverables |
|---|---|
| **P0** | Headers, CORS, CSRF middleware, rate limiting with trusted proxies, log redaction, secret validation, `Owned<T>` extractor, DTO/entity separation rule + CI check, filter limits, SAST/audit/secret-scanning CI |
| **P1** | Audit log table and writer; workflow context retention |
| **P2** | Upload pipeline with sniffing and re-encoding; egress guard |
| **P4** | Card-testing and promo rate limits; PCI script manifest; payment abuse tests |
| **P7** | Anti-enumeration, `token_version` invalidation, breached-password check, MFA for sensitive permissions, API-key scopes, authorization matrix test |
| **P8** | Erasure pipeline, retention jobs, export endpoint, data inventory verification |
| **P9** | Admin CSP, audit-log UI, sensitive-action MFA prompts, EAA/WCAG audit |
| **P10** | Penetration test, SAQ A completion, PSP script-attack confirmation, disclosure policy, restore drill |

**F1, F2, and F3 are launch blockers.** The rest are ordered by the phase that introduces the
surface they protect.
