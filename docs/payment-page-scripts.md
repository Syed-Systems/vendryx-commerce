# Checkout Route Script Inventory & Security Manifest (PCI SAQ A, F25)

In compliance with PCI SAQ A §1.1 and SECURITY.md §10, this document inventories every script, iframe, and redirect origin authorized to load on checkout routes (`/checkout/*`).

## 1. Authorized Scripts

| Script Identifier | Source URL | Integrity (SRI) / Version | Justification |
|---|---|---|---|
| `stripe-js` | `https://js.stripe.com/v3/` | Pinned / CSP-verified | Stripe Payment Element iframe mounting |
| `storefront-bundle` | `'self'` | Local hash bundle | Stheno checkout application UI & state machine |

## 2. Authorized Payment Gateway Redirect & Frame Origins

Every registered payment gateway is evaluated against SAQ-A eligibility (ADR-013 decision 7):

| Provider | Integration Type | Authorized Origins (Sandbox & Production) | CSP Directives |
|---|---|---|---|
| **Stripe** | Iframe / Elements | `https://js.stripe.com`, `https://api.stripe.com` | `script-src`, `frame-src`, `connect-src` |
| **bKash** | Hosted Checkout Redirect | `https://tokenized.sandbox.bka.sh`, `https://tokenized.pay.bka.sh` | `form-action` |
| **Nagad** | Hosted Checkout Redirect | `http://sandbox.mynagad.com`, `https://api.mynagad.com` | `form-action` |
| **SSLCommerz** | Hosted Checkout / Popup | `https://sandbox.sslcommerz.com`, `https://securepay.sslcommerz.com` | `form-action`, `frame-src` |
| **COD / Offline** | Internal State | *None* (Internal handler only) | *None* |

## 3. Security Restrictions & Content Security Policy (CSP)

- **Comprehensive Checkout CSP Policy**:
  ```http
  Content-Security-Policy: default-src 'self'; script-src 'self' https://js.stripe.com; frame-src 'self' https://js.stripe.com https://sandbox.sslcommerz.com https://securepay.sslcommerz.com; connect-src 'self' https://api.stripe.com; form-action 'self' https://tokenized.sandbox.bka.sh https://tokenized.pay.bka.sh http://sandbox.mynagad.com https://api.mynagad.com https://sandbox.sslcommerz.com https://securepay.sslcommerz.com; object-src 'none'; base-uri 'self';
  ```

- **Forbidden on Checkout (Strictly Prohibited under PCI SAQ A)**:
  - Google Tag Manager / GTM
  - Google Analytics / Meta Pixel / TikTok Pixel
  - Customer chat widgets (Intercom, Zendesk, Crisp)
  - A/B testing or session replay tools (Hotjar, FullStory, Optimizely)
  - External CDN libraries without SRI

## 4. Verification Protocol

- Any addition to this script or origin list requires explicit security architecture review and updated SAQ A validation.
- Validated automatically by `scripts/check-checkout-scripts.sh` / `scripts/check-checkout-scripts.ps1` in CI.
