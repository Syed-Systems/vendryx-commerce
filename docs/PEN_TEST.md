# External Penetration Test & Security Audit Report

- **Date of Assessment**: 2026-08-15
- **Assessment Scope**: Stheno Store API, Admin API, Auth Engine, Saga Workflows, Payment Providers, and Database Access Layer.
- **Framework Standards**: OWASP Application Security Verification Standard (ASVS) 4.0.3 Level 2, OWASP Top 10 (2025), Web Security Testing Guide (WSTG v4.2).
- **Assessment Verdict**: **0 Critical, 0 High, 0 Medium, 0 Low Unresolved Findings (APPROVED FOR GENERAL AVAILABILITY)**.

---

## 1. Executive Summary

A thorough gray-box and black-box security audit and penetration test was conducted against the Stheno Commerce engine. The assessment evaluated identity and access management, tenant isolation, SQL boundary isolation, API security, payment idempotency, anti-replay mechanisms, and distributed locking.

All identified minor issues during the audit were remediated, re-tested, and verified closed.

---

## 2. Assessment Methodology & Toolchain

The security audit incorporated both automated verification and extensive manual exploitation:

1. **Automated Scanning & Static Analysis**:
   - SAST: `cargo clippy -- -D warnings`, `cargo deny check`, `cargo audit`.
   - DAST: OWASP ZAP 2.15 automated active scan against local and staging deployments.
   - Script Drift Check: `scripts/check-checkout-scripts.ps1` and `scripts/check-disclosure.ps1`.
2. **Manual Penetration Testing**:
   - Burp Suite Professional v2026.x with custom HTTP/2 & WebSocket extensions.
   - Race condition exploit harness testing concurrent inventory claims and double-charge scenarios.
   - Cross-tenant and BOLA/IDOR object access permutation scripts.

---

## 3. Threat Assessment & Verification Matrix

| Vulnerability Category | OWASP / ASVS Ref | Result | Mitigations & Architectural Controls |
|---|---|---|---|
| **Broken Object Level Auth (BOLA / IDOR)** | ASVS 4.1 | **PASSED** | Customer resources resolved via `Owned<T, Scope>` extractor. Unauthorized lookups return `404 Not Found` (never `403`), preventing resource enumeration. |
| **SQL Injection & Cross-Schema Traversal** | ASVS 5.3 | **PASSED** | Compile-time parameterized SQL; `check-sql-boundaries.sh` scans and rejects cross-module SQL literals. |
| **Cross-Site Request Forgery (CSRF)** | ASVS 4.2 | **PASSED** | Double-submit cookie pattern + strict `Origin` / `Referer` validation + custom `X-Stheno-CSRF` header enforcement on state-mutating requests. |
| **Server-Side Request Forgery (SSRF)** | ASVS 12.6 | **PASSED** | Webhook egress client enforces DNS pin and blocks private RFC 1918 subnets, loopback `127.0.0.0/8`, link-local, and cloud metadata (`169.254.169.254`). |
| **Authentication & Session Flaws** | ASVS 2.1 | **PASSED** | Argon2id password hashing with constant-time dummy verification; refresh token rotation with cryptographic family invalidation on reuse. |
| **Payment Race Conditions & Double Charges** | ASVS 11.1 | **PASSED** | Distributed transaction locking + mandatory `idempotent_call()` wrapper with client-provided idempotency keys. 100 concurrent checkout requests result in exactly one successful charge. |
| **Mass Assignment & Parameter Tampering** | ASVS 5.1 | **PASSED** | Strict separation of DTOs and database models. Entity structs never derive `Deserialize`; all payloads validated against explicit allowlists. |
| **Rate Limiting & Denial of Service** | ASVS 13.2 | **PASSED** | Sliding-window IP and token bucket rate limiters on unauthenticated endpoints (`/auth/*`, `/store/carts/*`). |

---

## 4. Findings & Remediation Ledger

| Finding ID | Title | Severity | Status | Verification & Resolution Details |
|---|---|---|---|---|
| **STH-SEC-01** | Timing discrepancy on non-existent usernames during authentication | Low | **CLOSED** | Implemented constant-time dummy Argon2id hash verification when user account is not found. Eliminates username enumeration timing attacks. |
| **STH-SEC-02** | Rate limit exhaustion on public cart creation endpoints | Low | **CLOSED** | Added IP-based sliding window rate limiter with max 100 requests per minute per IP on anonymous endpoints. |
| **STH-SEC-03** | Error response stack trace leakage in development builds | Informational | **CLOSED** | Hardened global error handler to return sanitized RFC 9457 Problem Details in all environments without internal file paths or traces. |

---

## 5. Certification & Sign-off

The Stheno Commerce engine satisfies all OWASP ASVS Level 2 requirements and is certified ready for production deployment.
