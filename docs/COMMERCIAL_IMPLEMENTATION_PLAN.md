# Commercial Implementation Plan

**Status:** Source of truth for subscription pricing, regional payment collection, provider ownership, courier dispatch, CDN media, and customer experience planning.  
**Date:** 2026-09-13  
**Authority:** This file extends [ENTITLEMENTS.md](ENTITLEMENTS.md), [PAYMENTS.md](PAYMENTS.md), [PAYMENTS_SETUP.md](PAYMENTS_SETUP.md), and [ARCHITECTURE.md](ARCHITECTURE.md). If a plan document repeats these business rules, update it to point here instead.

---

## 1. Product Rule

Vendryx charges merchants for platform access only. It does not charge a buyer transaction fee, a merchant transaction fee, a courier markup, or a payment-provider markup.

Merchants bring their own payment-provider and courier-provider accounts. Provider costs belong to the merchant, buyer, or provider contract outside Vendryx. Any partner or aggregator is acceptable only when it creates no setup fee, monthly fee, or per-transaction liability for Vendryx itself.

---

## 2. Pro Pricing

| Field | Rule |
|---|---|
| Base price | USD 20.00 per month |
| Minimum purchase | 1 month |
| Maximum purchase | Unlimited; the merchant chooses any positive whole number of months |
| Long-term discount | 10% flat discount when `months >= 12` |
| Domain registration | Not included |
| Domain mapping | Included for Pro when the merchant brings their own domain |
| Transaction fee | 0% charged by Vendryx |

The calculation is exact decimal money:

```text
subtotal_usd = 20.00 * months
discount_usd = months >= 12 ? subtotal_usd * 0.10 : 0.00
total_usd = subtotal_usd - discount_usd
```

Examples:

| Months | Formula | Total |
|---:|---|---:|
| 1 | 20.00 * 1 | USD 20.00 |
| 3 | 20.00 * 3 | USD 60.00 |
| 6 | 20.00 * 6 | USD 120.00 |
| 12 | 240.00 - 10% | USD 216.00 |
| 24 | 480.00 - 10% | USD 432.00 |

Implementation rule: the 10% discount applies to the whole prepaid invoice once the selected term is at least 12 months.

---

## 3. Bangladesh Local Pricing

The base commercial price remains USD. Bangladesh merchants pay the BDT equivalent using the Bangladesh Bank published USD/BDT rate.

Invoice creation stores:

| Field | Required |
|---|---|
| `base_currency` | `USD` |
| `base_monthly_price` | `20.00` |
| `selected_months` | positive integer |
| `discount_percent` | `10.00` when months >= 12, otherwise `0.00` |
| `subtotal_usd` | exact decimal |
| `discount_usd` | exact decimal |
| `total_usd` | exact decimal |
| `presentment_currency` | `BDT` |
| `fx_source` | `bangladesh_bank` |
| `fx_rate_kind` | `WAR` unless finance/legal chooses another Bangladesh Bank rate |
| `fx_rate_date` | Bangladesh Bank rate date |
| `fx_rate` | exact decimal USD/BDT |
| `payable_bdt` | rounded whole taka |
| `rate_locked_until` | invoice expiry timestamp |

The rate is frozen when the invoice is issued. If the invoice expires, a new invoice uses the current Bangladesh Bank rate.

This follows the same industry pattern as Google Play regional pricing: keep a base price, convert to local currency, apply country-specific pricing rules, and store the converted amount for the purchase.

---

## 4. Bangladesh Manual Payment Flow

Initial Bangladesh payments are manual and human verified.

```text
Merchant selects term
  -> invoice issued in USD and BDT
  -> merchant pays by bKash, Nagad, Rocket, or bank transfer
  -> merchant uploads transaction ID and screenshot/proof
  -> Space/operator reviews proof
  -> invoice becomes paid, needs_more_info, failed, expired, or refunded
  -> entitlement changes only after paid
  -> email notification is sent
```

Invoice states:

| State | Meaning |
|---|---|
| `draft` | Created but not issued |
| `issued` | Amount and FX rate locked |
| `pending_payment` | Waiting for merchant payment |
| `pending_verification` | Proof uploaded; awaiting human review |
| `needs_more_info` | Operator needs a better screenshot, corrected transaction ID, or support action |
| `paid` | Verified; entitlement may activate or extend |
| `failed` | Rejected proof or failed payment |
| `expired` | Rate lock/payment window expired |
| `refunded` | Payment reversed after verification |

Entitlements:

- Uploading proof does not activate Pro.
- Approval of a paid invoice activates Pro.
- A new purchase before expiry extends `term_ends_at` from the current paid period end.
- A purchase after expiry starts a new paid period from the approval timestamp.

Security:

- Payment screenshots are private documents, never public CDN assets.
- Transaction IDs are unique per merchant and payment method unless an operator links a duplicate to an already-reviewed payment.
- Every operator action writes an audit log with actor, timestamp, before state, after state, and reason.

---

## 5. International Merchant Payments

International merchants pay through their own supported payment rails or direct subscription payment routes. Vendryx should prioritize providers that do not make Vendryx pay fixed fees.

| Priority | Provider model | Vendryx fee policy | Notes |
|---:|---|---|---|
| 1 | Stripe merchant-owned credentials | Vendryx pays nothing | Merchant bears Stripe fees; we store encrypted credentials and provider health |
| 2 | PayPal merchant-owned credentials | Vendryx pays nothing | Add only if target merchants need it |
| 3 | Manual bank/Wise-style subscription proof | Vendryx pays only its own receiving costs if chosen | Operationally slower; useful before full automation |
| Later | Aggregators | Allowed only if no cost liability to Vendryx | Must be optional and merchant-billed |

International presentment may use USD directly. If a local-currency display is later required, use a provider-localization pattern: store base USD, rate source, rate date, presented amount, and settlement assumptions.

---

## 6. Payment Provider Module Comparison

| Provider | Current role | Current status | Required next implementation |
|---|---|---|---|
| COD / offline | All tiers, order placed unpaid | Existing system/manual provider | Keep free; separate from Pro gateway entitlement |
| Stripe | International card/wallet payments | Integrated and locally tested; needs real credential boundary test | Credential verification endpoint must fail only on missing/invalid API key |
| SSLCommerz | Bangladesh gateway | Integrated and locally tested; needs merchant credential test | Keep as Pro gateway for merchants with their own account |
| bKash | Bangladesh gateway | Integrated and locally tested; needs merchant credential test | Keep as Pro gateway for merchants with their own account |
| Nagad | Bangladesh gateway | Integrated and locally tested; needs merchant credential test | Keep as Pro gateway for merchants with their own account |
| Rocket | Bangladesh manual payment first | Not implemented as API provider | Add manual payment method label now; API later only if real merchant-owned credentials exist |

Provider verification must be strict. A credential probe must not use mock fallback data. Missing API key, invalid API key, bad webhook secret, bad merchant ID, wrong sandbox/live mode, or region mismatch must be the only expected failures before live credentials exist.

---

## 7. Courier Dispatch Module Comparison

| Courier | Current role | Current status | Required next implementation |
|---|---|---|---|
| Manual fulfillment | All tiers | Existing/manual model | Remains free |
| Pathao | Bangladesh dispatch | Integrated and locally tested | Credential verification must fail loudly; no fallback in verification mode |
| RedX | Bangladesh dispatch | Integrated and locally tested | Add merchant-owned credential boundary test |
| Steadfast | Bangladesh dispatch | Integrated and locally tested | Add merchant-owned credential boundary test |
| DHL | International dispatch | Not built | Add only as direct merchant-owned account integration |
| FedEx | International dispatch | Not built | Add only as direct merchant-owned account integration |
| UPS | International dispatch | Not built | Add only as direct merchant-owned account integration |
| USPS | US dispatch | Not built | Add only as direct merchant-owned account integration |
| EasyPost / Shippo-like aggregators | Optional fallback | Not built | Allowed only if merchant-billed and no Vendryx fixed fee |

Dispatch fee rule: Vendryx does not collect a courier markup. Shipping charges shown to buyers belong to the merchant's configured shipping policy and courier account.

---

## 8. CDN And Media

Public product media:

```text
merchant upload
  -> validate type, size, dimensions, malware risk
  -> store original in object storage
  -> generate optimized variants
  -> serve public variants through Cloudflare CDN
  -> purge by product/media tag on replacement
```

Private operational documents:

- Payment screenshots
- Identity/KYC documents, if added
- Provider credential attachments
- Dispute evidence

These must not be cached publicly and must be accessed only through authenticated, time-limited URLs.

Product image quotas should be explicit before launch. The first implementation can set conservative limits per store and raise them by operator override.

---

## 9. Customer Experience

Merchant subscription flow:

1. Merchant sees Pro price as USD 20/month.
2. Merchant chooses 1 or more months.
3. If choosing 12 or more months, merchant sees the 10% discount before invoice creation.
4. Bangladesh merchant sees BDT amount, Bangladesh Bank rate, rate date, and invoice expiry.
5. International merchant sees USD or supported provider-local amount.
6. Merchant pays through their selected available method.
7. Bangladesh manual payments show `pending verification`.
8. Merchant receives email on proof received, approved, failed, or needs more info.
9. Pro features become visible only when entitlement is active.

Buyer checkout:

- Buyer payment and courier fees are merchant/provider concerns.
- Vendryx does not add a buyer-facing platform transaction fee.
- Provider failures are shown as payment or shipping method errors, not as subscription errors.

Operator experience in Space:

- Verification queue for manual payments.
- Provider credential health dashboard.
- Courier credential health dashboard.
- Invoice and entitlement timeline.
- Audit log for all manual decisions.
- Retry/reupload link generation for failed or incomplete proof.

---

## 10. Mechanical Implementation Plan

### Phase C1: Commercial config

- Add Pro discount config: 10% from 12 months.
- Keep `term_months = [1]` as the minimum, not the allowed maximum.
- Add tests asserting 12 and 24 month totals.

### Phase C2: Invoice calculator

- Implement exact-decimal subscription invoice calculation.
- Inputs: tier, selected months, billing country.
- Outputs: USD subtotal, discount, total, presentment currency, FX metadata, rounded due amount.
- Reject zero, negative, or non-integer months.

### Phase C3: Bangladesh FX source

- Add Bangladesh Bank rate fetch/import job.
- Store source, rate type, rate date, and rate.
- Allow audited operator override only when the official source is unavailable.
- Freeze rate at invoice issue.

### Phase C4: Manual payment proof

- Add proof upload endpoint and private file bucket path.
- Require transaction ID, payment method, invoice ID, and proof asset.
- Prevent duplicate transaction IDs per merchant/method.

### Phase C5: Space verification

- Add operator queue: pending, approve, reject, request more info.
- Send email on each terminal or action-required state.
- Activate or extend entitlement only when approved.

### Phase C6: Provider credential verification

- Add strict credential probes for Stripe, SSLCommerz, bKash, Nagad, Pathao, RedX, and Steadfast.
- Lock expected failure points when API keys are absent.
- Remove or bypass provider fallback behavior during verification.

### Phase C7: International provider model

- Add provider registry metadata: countries, currencies, credential schema, fee owner, merchant-owned billing flag, sandbox/live mode.
- Implement direct-carrier courier adapters only where merchant credentials are enough.
- Keep aggregators optional and disabled unless they meet the no-cost-liability rule.

### Phase C8: CDN media pipeline

- Separate public product media from private proof documents.
- Generate image variants and cache tags.
- Add purge workflow on replacement/deletion.
- Add storage quota and alerting.

### Phase C9: Reconciliation and audit

- Schedule payment reconciliation.
- Add subscription invoice reconciliation for manual payments.
- Add provider health history.
- Add operator audit reports.

---

## 11. Edge Cases To Test

| Area | Edge case |
|---|---|
| Pricing | 0 months, negative months, huge month count, 12-month threshold, 24-month discount |
| FX | Rate source unavailable, stale rate, invoice paid after expiry, rounding discrepancy |
| Manual payment | Underpayment, overpayment, duplicate transaction ID, wrong screenshot, old screenshot, partial payment |
| Entitlement | Approval after current Pro expiry, approval before current Pro expiry, rejected payment after proof upload |
| Email | Mail delivery failure must not incorrectly activate or deactivate Pro |
| Provider credentials | Missing key, invalid key, sandbox/live mismatch, expired credential, region mismatch |
| Courier | Bad credential, unsupported city/zone, unknown courier status, provider timeout |
| CDN | Oversized image, unsupported MIME, malicious upload, private proof leaked to CDN, purge failure |
| International | Unsupported country, unsupported currency, provider disabled by merchant, aggregator cost liability |

---

## 12. References

- Google Play price conversion pattern: <https://support.google.com/googleplay/android-developer/answer/6334373>
- Google Play regional price conversion API: <https://developers.google.com/android-publisher/api-ref/rest/v3/monetization/convertRegionPrices>
- Bangladesh Bank exchange rates: <https://www.bb.org.bd/en/index.php/econdata/exchangerate>
- Bangladesh Bank FX reference rate: <https://www.bb.org.bd/en/index.php/econdata/rfrexrate>
- Stripe adaptive/local currency pricing concepts: <https://docs.stripe.com/payments/currencies/localize-prices>
