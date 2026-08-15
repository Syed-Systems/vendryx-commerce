# Checkout Route Script Inventory & Security Manifest (PCI SAQ A, F25)

In compliance with PCI SAQ A §1.1 and SECURITY.md §10, this document inventories every script authorized to load on checkout routes (`/checkout/*`).

## 1. Authorized Scripts

| Script Identifier | Source URL | Integrity (SRI) / Version | Justification |
|---|---|---|---|
| `stripe-js` | `https://js.stripe.com/v3/` | Pinned / CSP-verified | Stripe Payment Element iframe mounting |
| `storefront-bundle` | `'self'` | Local hash bundle | Stheno checkout application UI & state machine |

## 2. Security Restrictions & Policy

- **CSP Policy**: `script-src 'self' https://js.stripe.com; object-src 'none'; base-uri 'self'; frame-src https://js.stripe.com;`
- **Forbidden on Checkout**:
  - Google Tag Manager / GTM
  - Google Analytics / Meta Pixel / TikTok Pixel
  - Customer chat widgets (Intercom, Zendesk, Crisp)
  - A/B testing or session replay tools (Hotjar, FullStory, Optimizely)
  - External CDN libraries without SRI

## 3. Verification Protocol

- Any addition to this script list requires explicit security architecture review and updated SAQ A validation.
- Validated automatically by `scripts/check-checkout-scripts.sh` / `scripts/check-checkout-scripts.ps1` in CI.
