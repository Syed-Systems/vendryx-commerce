# Payments setup — what to supply, and where

**Status:** Operational guide · **Date:** 2026-09-02 · **Companion to:** [PAYMENTS.md](PAYMENTS.md)

Everything below is verified against a running deployment. The goal of this document is that
a gateway goes live by entering credentials in the dashboard and registering one URL — no
code change, no redeploy.

---

## 1. One-time, per deployment

### `STHENO_CREDENTIAL_KEY` — required

Gateway credentials are encrypted with AES-256-GCM before they are stored. This variable is
the key. **Without it every credential save fails**, and the failure surfaces far from its
cause: the dashboard reports a save error, and checkout later reaches the payment step with
no method to select.

```
STHENO_CREDENTIAL_KEY=<64 hex characters, or 32 raw bytes>
```

Generate one with `openssl rand -hex 32`. It is not currently listed in
[DEPLOYMENT.md](DEPLOYMENT.md); treat it as mandatory alongside `DATABASE_URL`.

Two properties worth knowing before you rotate it:

- The key is bound to the tenant and provider through AAD, so a ciphertext cannot be moved
  between merchants or between gateways.
- Changing it makes **existing stored credentials unreadable**. Re-enter them after a
  rotation; there is no re-encryption path yet.

Local development sets a fixed development key automatically in
`scripts/dev-tenants.sh`. Never use that value anywhere real.

### Storefront variables

The storefront reaches the engine with these two. Both were recently renamed off the Medusa
names, and a deployment still carrying the old ones fails closed at boot rather than
silently: the env check refuses to start, and the region middleware answers 500.

```
STHENO_BACKEND_URL=https://<the engine's host>
NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY=pk_...
```

| Old name | Current name |
|---|---|
| `MEDUSA_BACKEND_URL` | `STHENO_BACKEND_URL` |
| `NEXT_PUBLIC_MEDUSA_PUBLISHABLE_KEY` | `NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY` |

The storefront's Playwright config reads `STHENO_BACKEND_URL` too. Point it at the loopback
address rather than a `*.localhost` name when running tests: browsers resolve `*.localhost`
internally, Node does not, so the API-only tests fail to connect while the browser ones pass.

---

## 2. Per gateway — the fields to enter

Enter these in the dashboard: **Settings → Payment Providers → Setup Keys**. Fields marked
`*` are required; the form refuses to save without them, and an unknown field is rejected
rather than silently stored.

### bKash — `pp_bkash_bkash`

| Field | Type | |
|---|---|---|
| `base_url` | url | `*` sandbox or production host |
| `app_key` | secret | `*` |
| `app_secret` | secret | `*` |
| `username` | secret | `*` |
| `password` | secret | `*` |

### SSLCommerz — `pp_sslcommerz_sslcommerz`

| Field | Type | |
|---|---|---|
| `store_id` | secret | `*` |
| `store_passwd` | secret | `*` |
| `is_sandbox` | bool | optional — sandbox when true |
| `base_url` | url | optional — overrides the host |

### Stripe — `pp_stripe_stripe`

| Field | Type | |
|---|---|---|
| `api_key` | secret | `*` |
| `webhook_secret` | secret | `*` — the `whsec_…` signing secret |
| `base_url` | url | optional |
| `livemode` | bool | optional |

### Manual / Cash on delivery — `pp_system_default`

No fields. It is configured by saving an empty credential, which the seed already does for
every tenant it creates.

---

## 3. Enable it for a region

**A configured gateway does not appear at checkout until it is linked to a region.** This
is the step most easily missed: credentials save, the provider shows "Configured", and the
storefront still offers nothing.

In the dashboard: **Settings → Payment Providers → Regions → tick the region → Save**.

`GET /store/payment-providers?region_id=…` is the check. It lists a provider only when both
are true for that tenant: a stored credential **and** a region link.

---

## 4. Register the callback URL with the gateway

```
https://<the merchant's own domain>/hooks/payment/<provider_id>
```

For example `https://acme.stheno.shop/hooks/payment/pp_bkash_bkash`.

**The host must be the merchant's domain, not a shared one.** One server serves every
merchant and tells them apart by `Host` (ADR-012), so the host is what routes a callback to
the right merchant's credentials. A callback sent to the wrong host reaches the wrong
merchant's keys and fails signature verification.

The endpoint is deliberately strict:

- The raw request body reaches the provider byte-for-byte. Signatures are computed over
  exact bytes, so nothing parses or re-serialises it first.
- Verification happens before any database write, so a forged callback costs one hash.
- A signature failure answers **401**, not 400, so a gateway stops retrying a payload that
  will never be accepted.
- Repeat deliveries of the same `(provider, event id)` are recognised and answered `200`
  without reprocessing, which is what gateways expect from an idempotent receiver.

### How each gateway is authenticated

| Gateway | Mechanism |
|---|---|
| Stripe | HMAC over the raw body against `webhook_secret`, from the `Stripe-Signature` header |
| bKash | callbacks carry no signature, so the event is confirmed against bKash's payment-query API before it is accepted |
| SSLCommerz | IPNs carry no signature, so the event is validated against `validationserverAPI` using `val_id` before it is accepted |

For bKash and SSLCommerz this means the gateway itself is the authority, not the payload.
An event that cannot be tied to a real payment is refused. Validating against the gateway is
also stronger than recomputing SSLCommerz's `verify_sign` hash: it still holds if the store
password leaks, because the answer comes from SSLCommerz rather than from the caller.

---

## 5. Order of operations

1. Set `STHENO_CREDENTIAL_KEY` and deploy.
2. Settings → Payment Providers → **Setup Keys** for the gateway.
3. Same row → **Regions** → tick the regions that may use it.
4. Register the callback URL above in the gateway's own dashboard.
5. Confirm with `GET /store/payment-providers?region_id=…` — the provider should be listed.
6. Place one real low-value order end to end.

Step 5 is worth doing explicitly: it is the only check that covers both halves, and the
storefront failure mode for a missing region link looks identical to a missing credential.

---

## 6. What is verified, and what to watch

**Verified on a running deployment:** credentials encrypt and store; the dashboard's payment
provider screen reads and writes them; region enablement propagates to
`/store/payment-providers`; checkout completes end to end against the manual provider from a
freshly seeded database; inbound callbacks reach the right tenant and are recorded, with
each merchant's callbacks landing only in that merchant's rows.

**Known limits, honestly:**

- The three real gateways are integrated against their live APIs, but no transaction has
  been run against a real bKash, SSLCommerz or Stripe account from this codebase. First
  contact with production credentials is the remaining unknown, and it is the reason for
  step 6 above.
- Bangladesh subscription collection is intentionally manual first: invoice in USD/BDT,
  collect proof by transaction ID and screenshot, then activate Pro only after operator
  verification. The source-of-truth workflow is
  [COMMERCIAL_IMPLEMENTATION_PLAN.md](COMMERCIAL_IMPLEMENTATION_PLAN.md).
- `credential_health` reports `unconfigured` until a provider has actually been called;
  a configured gateway with no traffic is not a failing one.
- Rotating `STHENO_CREDENTIAL_KEY` invalidates stored credentials, as noted in §1.
- The outbound reconciliation sweep exists and can be triggered by hand from the control
  plane, but nothing schedules it yet. Until it is scheduled, an ambiguous timeout stays
  ambiguous until someone runs it.
