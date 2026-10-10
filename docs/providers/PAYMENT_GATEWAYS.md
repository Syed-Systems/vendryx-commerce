# Bangladesh Payment Gateways — integration contracts

**Date:** 2026-08-21 · **For:** [P22](../archive/PHASE3_TASKS.md) · **Governed by:** [ADR-013](../DECISIONS.md#adr-013) decisions 6 and 7

Pulled from vendor documentation so P22 is implementable without anyone re-reading a
gateway's website. Where a fact could not be confirmed from the vendor's own docs, it is
marked and the gap is named rather than filled with a plausible guess.

**Credentials are tenant data, not platform data.** No key below belongs to us. Each merchant
brings their own, entered in the admin panel and encrypted at rest (P21-11, P21-12). Base URLs
are included in that set because bKash and Nagad issue them per merchant at onboarding — they
are configuration, never build constants.

---

## 1. Credential matrix — what the admin panel must accept

This is the input P21-11 and P21-12 are built against.

| Gateway | Status | Fields a merchant supplies | Notes |
|---|---|---|---|
| **bKash** | **Compiled & Live** | `base_url`, `app_key`, `app_secret`, `username`, `password` | Base URL issued at onboarding; remote grant token probe |
| **SSLCommerz** | **Compiled & Live** | `store_id`, `store_passwd`, `is_sandbox` | Base URL is fixed per environment (Sandbox/Live) |
| **Stripe** | **Compiled & Live** | `api_key`, `webhook_secret`, `base_url`, `livemode` | Direct credit card processing; remote balance probe |
| **Nagad** | **Planned (Blocked on PDF)** | `base_url`, `merchant_id`, **merchant private key (PEM)**, **Nagad public key (PEM)** | Two multi-line PEM blobs; awaiting vendor portal spec |

**Nagad breaks the "paste an API key" assumption.** It needs an RSA keypair: the merchant
signs with their private key and Nagad encrypts responses with the public key the merchant
uploaded to Nagad's portal. The admin UI needs a multi-line PEM field and validation that the
key parses, and P21-12's encryption-at-rest has to cover a private key, which is a materially
worse thing to leak than an API key.

---

### Compiled Provider Catalog (Generated)

<!-- BEGIN GENERATED PROVIDER CATALOG -->
| Provider ID | Provider Name | Credential Fields | Environments | Verification Strategy |
|---|---|---|---|---|
| `pp_system_default` | Offline / Cash on Delivery | *(none)* | Built-in | Immediate / Available |
| `pp_stripe_stripe` | Stripe | `api_key` (secret, required), `webhook_secret` (secret, required), `base_url` (url, optional), `livemode` (bool, optional) | Live, Sandbox | Remote account retrieve |
| `pp_sslcommerz_sslcommerz` | SSLCommerz | `store_id` (secret, required), `store_passwd` (secret, required), `is_sandbox` (bool, optional), `base_url` (url, optional) | Live, Sandbox | Configured, awaiting first transaction |
| `pp_bkash_bkash` | bKash | `base_url` (url, required), `app_key` (secret, required), `app_secret` (secret, required), `username` (secret, required), `password` (secret, required) | Live, Sandbox | Remote grant token probe |
| `pp_nagad_nagad` | Nagad | `base_url` (url, required), `merchant_id` (secret, required), `merchant_private_key` (secret, required), `nagad_public_key` (secret, required) | Live, Sandbox | RSA keypair validation probe |
<!-- END GENERATED PROVIDER CATALOG -->

---

## 2. bKash — Tokenized Checkout (v1.2.0-beta)

Source: [developer.bka.sh](https://developer.bka.sh/docs/checkout-process-overview). Confidence: **high**, vendor docs.

### Token lifecycle

`POST {base_url}/tokenized/checkout/token/grant` — `app_key`, `app_secret` in the body;
`username`, `password` as headers. Returns `id_token`.

| Rule | Value |
|---|---|
| Token lifetime | 3600s |
| Refresh window | before the 50th–55th minute |
| Refresh calls | **max 2 per hour** — exceeding it blocks the API for an hour |

That rate limit is a design constraint, not a footnote: a naive "refresh on every 401" retry
loop locks the merchant out of their own gateway for an hour. The token cache must be shared
and refreshed on a schedule, not per request (P22-02).

### Create payment

`POST {base_url}/tokenized/checkout/payment/create`

Headers: `Content-Type: application/json`, `Accept: application/json`,
`Authorization: <id_token>`, `X-App-Key`.

Request: `mode` (`"0011"` for checkout), `payerReference`, `callbackURL`, `amount` (**String**),
`currency` (**`"BDT"` only**), `intent` (`"authorization"`), `merchantInvoiceNumber`,
optional `merchantAssociationInfo`.

Response: `statusCode` (`"0000"` = Successful), `statusMessage`, `paymentID` (**24h expiry**),
`paymentCreateTime`, **`bkashURL`**, `callbackURL`, `successCallbackURL`, `failureCallbackURL`,
`cancelledCallbackURL`, `transactionStatus` (initially `"Initiated"`), `amount`, `currency`,
`intent`, `merchantInvoiceNumber`. Errors: `errorCode`, `errorMessage`.

**`bkashURL` is the redirect target** — this is what `ProviderCapabilities.redirect` and the
`InitiateOutput.data.redirect_url` convention (P21-06, P21-07) exist to carry.

### Execute payment

`POST {base_url}/tokenized/checkout/execute/` with the paymentID appended to the path.
Headers: `Accept`, `Authorization`, `X-App-Key`. No body.

Response adds `trxID`, `agreementID`, `customerMsisdn`, `paymentExecuteTime`, and
`transactionStatus` = `"Completed"`.

Two hard rules from the docs:

- **A paymentID may be executed once.** Reuse after any response — success or failure — is
  prohibited.
- **A paymentID expires after 24 hours**, returning *"The session of this payment has timed
  out."*

Execute is therefore **not idempotent and not safely retryable**. A timeout mid-execute leaves
the merchant not knowing whether money moved, and retrying is forbidden. Query Payment is the
convergence path — this is exactly what P22-04 means by *"capture via execute + query-payment
convergence"*, and it is the single most likely place to silently lose or double-count a payment.

### Webhooks — AWS SNS, not an HMAC header

This is the surprise. bKash delivers callbacks through **Amazon SNS**:

- Headers: `x-amz-sns-message-type`, `x-amz-sns-message-id`, `x-amz-sns-topic-arn`
- Body carries `Signature`, `SignatureVersion`, `SigningCertURL` — verification is SNS
  certificate validation, **not** a shared secret
- Subscription must first be confirmed by calling the `SubscribeURL` in a
  `SubscriptionConfirmation` message
- Notifications are sent **only for successfully completed payments**
- Payload: `dateTime`, `debitMSISDN`, `creditOrganizationName`, `trxID`, `transactionStatus`,
  `transactionType`, `amount`, `currency`, `transactionReference`, `merchantInvoiceNumber`

Consequences for the engine:

1. `signature_header()` returns **`None`** — the signature is in the body. The trait already
   allows this (P21-01), and bKash is the reason it must.
2. The hooks route has to handle a message *type* that is not a payment event at all
   (`SubscriptionConfirmation`), which no current code path anticipates.
3. `SigningCertURL` must be validated against an Amazon host allowlist before fetching it.
   Fetching an attacker-supplied URL and trusting the certificate it returns is the standard
   way this integration is broken.

Not documented by the vendor: expected HTTP response code, retry behaviour, IP allowlist.

Remaining pages to pull when P22 starts: `refund-api-overview`, `refund-transaction`,
`refund-status`, `error-codes`.

---

## 3. SSLCommerz

Source: [developer.sslcommerz.com/doc/v4](https://developer.sslcommerz.com/doc/v4/). Confidence: **high**, vendor docs.

| Environment | Base URL |
|---|---|
| Sandbox | `https://sandbox.sslcommerz.com` |
| Live | `https://securepay.sslcommerz.com` |

### Session

`POST /gwprocess/v4/api.php` — `store_id`, `store_passwd`, `total_amount` (10.00–500,000.00),
`currency`, `tran_id`, `success_url`, `fail_url`, `cancel_url`, `cus_name`, `cus_email`,
`cus_phone`, `product_category`, `product_name`, `product_profile`; optional `ipn_url`,
`value_a`–`value_d`, `emi_option`.

Response: `status` (`SUCCESS`/`FAILED`), `sessionkey`, **`GatewayPageURL`** (the redirect
target), `gw` (available methods), `storeLogo`, `storeBanner`.

### IPN and validation

IPN POSTs to `ipn_url` with `status` (`VALID`, `FAILED`, `CANCELLED`, `EXPIRED`,
`UNATTEMPTED`), `tran_id`, `val_id`, `amount`, `store_amount`, `bank_tran_id`, `card_type`,
`card_no`, `card_brand`, `card_issuer`, `card_issuer_country`, `currency_type`,
`currency_amount`, `risk_level`, `risk_title`, `verify_sign`, `verify_key`.

**The IPN is not trusted on its own.** After receiving it, the merchant must call the Order
Validation API and believe that instead:

`GET /validator/api/validationserverAPI.php?val_id=&store_id=&store_passwd=&format=json`
→ `VALID`, `VALIDATED` (already validated), or `INVALID_TRANSACTION`.

So SSLCommerz's `verify_webhook` returns `signature_header() = None` and authenticates by
calling back to SSLCommerz — the `None` return is a claim that the body is self-authenticating
*because it is re-verified server-side*, which is the only acceptable reading of that default.

Note `store_amount` ≠ `amount`: the merchant nets less than the customer paid. That is the same
three-distinct-amounts problem P24 has for COD, arriving early.

### Refunds

Initiate: `GET /validator/api/merchantTransIDvalidationAPI.php` with `bank_tran_id`,
`refund_trans_id`, `refund_amount`, `refund_remarks`, `store_id`, `store_passwd`
→ `success`, `failed`, `processing`.

Query: same endpoint with `refund_ref_id` → `refunded`, `processing`, `cancelled`.

`refund_amount` is a parameter, so **`partial_refund: true`** is honest here — but P21-09
requires that claim be exercised, not asserted.

---

## 4. Nagad

Confidence: **low on specifics.** The signing model is confirmed; exact field lists are not.
The authoritative document is the *Nagad Online Payment API Integration Guide* (v3.3 /
v1.5 MFI-M2C), distributed through the merchant portal rather than a public developer site.
Copies circulate on document-sharing sites; those are not a source this repo should build
against.

What is confirmed:

- Checkout initialise: `POST /remote-payment-gateway-1.0/api/dfs/check-out/initialize/{merchantId}/{orderId}`,
  with a locale parameter, followed by a complete-payment call.
- Base URL form: `https://{host}:{port}/{context-path}/{api-path}` — issued per merchant.
- **RSA both ways.** The merchant generates a keypair, uploads the public key to Nagad, and
  signs sensitive data with the private key. Nagad publishes its own public key in the merchant
  portal; the merchant encrypts sensitive data with it. Signature verification uses
  **SHA1withRSA**.
- Payloads carry `sensitiveData`, `signature`, and a merchant callback URL.

**SHA1withRSA is worth flagging.** SHA-1 is broken for collision resistance. It is what the
gateway specifies, so it is what we implement — but it should be recorded as a known weakness
in the threat model rather than discovered later, and it is a reason not to reuse this code
path for anything else.

**Blocking gap:** the exact request/response field names for initialise and complete. Get the
integration guide PDF from the Nagad merchant portal before P22-08.

---

## 5. What this changes about the plan

The plan assumed sandbox credentials were the critical path. They are not — they are per-tenant
data that arrives from each merchant. What is actually on the critical path is **documentation
access**, and it is now largely resolved: bKash and SSLCommerz are fully specified above, Nagad
needs one PDF from a merchant portal.

Three findings that change implementation rather than schedule:

1. **bKash execute is one-shot and non-idempotent.** Convergence via query-payment is not a
   nice-to-have; without it a network timeout is an unresolvable payment.
2. **bKash webhooks are SNS**, requiring subscription confirmation and certificate validation —
   a shape no current provider has.
3. **Nagad needs PEM keypairs in the admin UI**, not an API key field.

Sources: [bKash developer portal](https://developer.bka.sh/docs/checkout-process-overview) ·
[bKash create payment](https://developer.bka.sh/docs/create-payment.md) ·
[bKash execute payment](https://developer.bka.sh/docs/execute-payment.md) ·
[bKash webhooks](https://developer.bka.sh/docs/webhooks.md) ·
[bKash token management](https://developer.bka.sh/docs/token-management-overview-1.md) ·
[SSLCommerz v4 docs](https://developer.sslcommerz.com/doc/v4/) ·
[SSLCommerz developer portal](https://developer.sslcommerz.com/)
