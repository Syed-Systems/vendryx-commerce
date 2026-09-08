# PCI DSS v4.0 Self-Assessment Questionnaire A (SAQ A) Attestation

- **Entity**: Stheno Commerce Engine Platform
- **Scope**: E-Commerce Checkout Architecture, Payment Workflows & Script Security
- **Version**: PCI DSS v4.0
- **Assessment Date**: 2026-08-15
- **Compliance Status**: **FULLY COMPLIANT (SAQ A Criteria Satisfied)**

---

## 1. Scope Eligibility & Payment Flow Architecture

Stheno qualifies for PCI DSS Self-Assessment Questionnaire A (SAQ A) because all payment card processing operations are completely outsourced to validated PCI-DSS Level 1 Payment Service Providers or hosted redirect gateways (ADR-013 decision 7):

1. **Cardholder Data Outsourcing**:
   - Primary Account Numbers (PAN), expiration dates, and Sensitive Authentication Data (CAV2/CVC2/CVV2/CID) are entered directly into PSP-hosted elements (`Stripe Elements` via secure iframe) or hosted gateway redirect pages.
   - Cardholder data never touches, traverses, resides in memory on, or is stored in Stheno application servers or databases.

2. **Registered Provider Compliance Architecture**:

| Provider | Mechanism | SAQ-A Justification | Level 1 Certification |
|---|---|---|---|
| **Stripe** | Iframe (`js.stripe.com`) | Cardholder inputs isolated in Stripe Elements iframe | Level 1 Service Provider AoC on file |
| **bKash** | Hosted Redirect | Customer enters wallet credentials on bKash-hosted URL | Bangladesh Bank licensed Payment System Operator |
| **Nagad** | Hosted Redirect | Customer enters wallet credentials on Nagad-hosted URL | Bangladesh Bank licensed Payment Service Provider |
| **SSLCommerz** | Hosted Gateway Redirect | Customer enters card/MFS on SSLCommerz-hosted payment gateway | PCI DSS Level 1 Certified Service Provider |
| **COD / Offline** | Physical Cash Collection | Physical cash collected on delivery by courier rider | Out of PCI scope (zero electronic card data) |

3. **No Electronic Storage of Cardholder Data**:
   - Zero PAN, zero track data, zero PIN/CVV in data models, logs, telemetry, or cache entries.
   - Enforced mechanically by compile-time types (`stheno-payment`), database constraints, and CI audits (`scripts/check-pci-saq-a.sh`).

---

## 2. Detailed Requirement-by-Requirement Matrix (PCI DSS v4.0)

| PCI Requirement | Requirement Description | Stheno Implementation & Mitigations | Status |
|---|---|---|---|
| **Req 2.2** | System components configured securely and hardened. | Distroless container image without shell or package managers. Run as non-root UID 65532 with dropped Linux capabilities (`ALL`). Read-only root filesystem. | **COMPLIANT** |
| **Req 3.1 - 3.4** | Cardholder data storage minimization and protection. | Zero PAN or sensitive auth data stored, processed, or transmitted. Only truncated 4-digit last4 and brand strings stored for display. | **COMPLIANT** |
| **Req 6.4.3** | Management of all scripts loaded on payment pages. | Strict checkout script manifest in `docs/payment-page-scripts.md`. Enforced via strict Content Security Policy (`CSP`) headers. CI validation via `scripts/check-checkout-scripts.sh`. Third-party marketing/analytics scripts (GTM, Meta, chat widgets) strictly forbidden on checkout routes. | **COMPLIANT** |
| **Req 8.2 - 8.3** | Multi-Factor Authentication (MFA) & strong authentication. | Argon2id password hashing (RFC 9106 recommended parameters), TOTP-based MFA with recovery codes, refresh token rotation with family reuse revocation. | **COMPLIANT** |
| **Req 10.2** | Audit logging of all access to system components. | Append-only structured audit logs (`stheno-audit`) capturing actor, action, timestamp, and entity ID with automatic write-time PII redaction. | **COMPLIANT** |
| **Req 11.3** | Vulnerability scanning and external penetration testing. | Clean external gray-box penetration test (ASVS Level 2, 0 Critical / High / Medium findings). Automated dependency vulnerability scans in CI (`cargo deny`, `cargo audit`). | **COMPLIANT** |
| **Req 11.6.1** | Change- and tamper-detection mechanism for payment pages. | Subresource Integrity (SRI) on all external assets, strict CSP with hash verification, and automated checkout script drift detection. | **COMPLIANT** |
| **Req 12.1 - 12.8** | Information security policy & third-party service providers. | Third-Party Service Provider Level 1 PCI Attestations of Compliance (AoC) maintained on file and reviewed annually. Responsible disclosure policy in `docs/DISCLOSURE_POLICY.md`. | **COMPLIANT** |

---

## 3. Checkout Script Security & Anti-Skimming Controls (Req 6.4.3 & 11.6.1)

To prevent client-side card skimming (Magecart-style attacks), Stheno enforces strict controls on all checkout URLs (`/checkout/*`):

1. **Explicit Script Allowlist**:
   - Only scripts registered in [`docs/payment-page-scripts.md`](payment-page-scripts.md) are permitted.
   - Current authorized scripts: `https://js.stripe.com/v3/` and `'self'` storefront application bundle.
2. **Automated CI Enforcement**:
   - `scripts/check-checkout-scripts.sh` runs on every pull request and build pipeline.
   - Any unreviewed or unauthorized script tag in checkout templates fails the build immediately.
3. **Content Security Policy (CSP)**:
   ```http
   Content-Security-Policy: default-src 'self'; script-src 'self' https://js.stripe.com; frame-src 'self' https://js.stripe.com https://sandbox.sslcommerz.com https://securepay.sslcommerz.com; connect-src 'self' https://api.stripe.com; form-action 'self' https://tokenized.sandbox.bka.sh https://tokenized.pay.bka.sh http://sandbox.mynagad.com https://api.mynagad.com https://sandbox.sslcommerz.com https://securepay.sslcommerz.com; object-src 'none'; base-uri 'self';
   ```

---

## 4. Attestation & Sign-off

- **Assessor**: Stheno Security Compliance Team
- **PSP Confirmations**: Stripe Inc. and SSLCommerz Level 1 Service Provider AoCs verified and on file; bKash and Nagad licensed operator integrations verified.
- **Verification Script**: `scripts/check-pci-saq-a.sh` (Exit code 0).
