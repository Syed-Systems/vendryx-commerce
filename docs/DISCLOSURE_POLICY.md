# Responsible Vulnerability Disclosure Policy

Stheno values the security and privacy of its users and the open-source community. If you have discovered a potential security vulnerability in any Stheno repository or deployed service, we encourage you to report it to us immediately and responsibly.

---

## 1. Reporting Channels

- **Primary Contact**: Send an email to **`security@stheno.io`**.
- **PGP Encryption**: For highly sensitive vulnerability details, please encrypt your communication using our PGP public key:
  - **Key ID**: `0x94B2E5C81F037A91`
  - **Fingerprint**: `A72B 8891 43D0 1F9C E608 5519 94B2 E5C8 1F03 7A91`
  - **Key URL**: `https://stheno.io/.well-known/security-pgp-key.asc`
- **RFC 9116 Discovery**: Machine-readable security contact information is published at `/.well-known/security.txt`.

---

## 2. Submission Guidelines

To help us triage and resolve your report quickly, please include:
1. **Summary**: Description of the vulnerability and its potential impact.
2. **Product / Component**: Affected repository, branch, API endpoint, or module name.
3. **Step-by-Step Reproduction**: Detailed instructions, HTTP request/response transcripts, or proof-of-concept (PoC) code.
4. **Environment**: Version or commit hash tested against.
5. **Mitigation Suggestion**: (Optional) Recommended code fix or configuration change.

---

## 3. Response & Remediation Service Level Agreements (SLAs)

We commit to the following response timeline for all valid security submissions:

| Phase | Target Timeline | Details |
|---|---|---|
| **Acknowledgment** | Within 24 hours | Initial receipt confirmation by a member of the security team. |
| **Triage & Reproduction** | Within 72 hours | Validation of the vulnerability and determination of CVSS score. |
| **Critical Fix SLA** | Within 7 calendar days | Emergency patch release for vulnerabilities with CVSS ≥ 9.0. |
| **High Fix SLA** | Within 14 calendar days | Patch release for vulnerabilities with CVSS 7.0 – 8.9. |
| **Medium / Low Fix SLA** | Within 30 calendar days | Standard release cycle patch. |
| **Public Disclosure** | Coordinated | Disclosure coordinated mutually after patch deployment (standard 90-day embargo). |

---

## 4. Safe Harbor & Legal Protection

We consider security research conducted under this policy to be authorized and will not initiate legal action against researchers who:
- Act in good faith to avoid privacy violations, data destruction, and service interruption.
- Do not exploit a security vulnerability beyond the minimum necessary to prove its existence.
- Do not access, modify, or retain customer data or personal identifiable information (PII).
- Give us reasonable time to remediate the vulnerability before public disclosure.
- Comply with all applicable local, national, and international laws.

---

## 5. Scope

### In-Scope
- Stheno Server core HTTP APIs (`/store/*`, `/admin/*`, `/auth/*`, `/hooks/*`).
- Authentication, session management, and RBAC authorization engines.
- Saga workflow execution engine and state persistence.
- Payment integrations, webhook signature verifications, and idempotency handling.
- PostgreSQL database access and schema isolation controls.

### Out-of-Scope
- Denial of Service (DoS/DDoS) attacks against testing or production infrastructure.
- Social engineering (phishing, vishing) targeting Stheno employees or users.
- Physical attacks against hosting data centers or office infrastructure.
- Subdomain takeover without active hosting / exploitability.
- Vulnerabilities in third-party services that Stheno does not operate or control.
