# Vendryx Platform Control Plane (Transitional)

> Operator control plane console for Vendryx Commerce, built with React 18, Vite, TanStack Query, and Medusa UI. Governed by [ADR-030](../../docs/DECISIONS.md#adr-030), [docs/CONTROL_PLANE.md](../../docs/CONTROL_PLANE.md), and [STATUS.md](../../docs/STATUS.md).

---

## 1. Architectural Role & Space Migration Baseline

### The Space Extraction ([ADR-030](../../docs/DECISIONS.md#adr-030))
Syed Systems operates multiple B2B enterprise SaaS products (Vendryx Commerce, ERP, LMS, and future offerings). To prevent cross-product entanglement and protect modular boundaries:
- **Permanent Enterprise Control Plane:** Lives in the standalone repository **Space** at `D:\Syed Systems\Space`. Space owns global organizations, unified billing, cross-product catalog bundles, and centralized identity.
- **In-Repo Platform Role (`apps/platform`):** This application and its backend companion (`stheno-api-platform`) provide the **transitional commerce operations surface and connector baseline**. It contains verified business logic for tenant provisioning, tier assignment, operator audit trails, and provider oversight for Vendryx Commerce.

---

## 2. Operator Domains & Capabilities

The platform console manages 10 core operator domains (D1–D10) defined in [docs/CONTROL_PLANE.md](../../docs/CONTROL_PLANE.md):

1. **Operator Identity & Audit (Domain D9 - P29 Complete):**
   - Session-based operator authentication and role-based access.
   - Comprehensive audit logging recording every administrative change with actor IDs and IP context.
2. **Tenant Provisioning & Lifecycle (Domain D1):**
   - Tenant creation, subdomain reservation, database initialization, and status tracking (Active, Suspended, Maintenance).
3. **Plan Tiers & Entitlements (Domain D2):**
   - Dynamic tier assignments (Free vs Pro) resolved per request.
4. **Provider Oversight & Gateways (Domain D4 & D5):**
   - Real-time status oversight of payment gateways (Stripe, bKash, Nagad, SSLCommerz) and couriers (Pathao, Steadfast, RedX).
   - Circuit-breaking and outage triage controls.
5. **Support & Time-Boxed Impersonation (Domain D6 - [ADR-018](../../docs/DECISIONS.md#adr-018)):**
   - Cryptographically signed, time-limited operator tokens for customer support without credential exposure.

---

## 3. Directory Structure

```
apps/platform/
├── index.html                            # Console entry point
├── package.json                          # Package scripts and dependencies
├── vite.config.ts                        # Vite configuration and backend proxying
├── tsconfig.json                         # TypeScript configuration
└── src/
    ├── App.tsx                           # Main operator dashboard view
    ├── components/                       # Operator tables, metric widgets, modal forms
    ├── hooks/                            # TanStack Query hooks for platform endpoints
    └── lib/                              # Platform API client and auth session handling
```

---

## 4. Environment Variables

Create `.env.local` in `apps/platform/`:

```bash
# Platform API Base URL (proxied to Rust engine port 9000)
VITE_PLATFORM_API_URL="http://127.0.0.1:9000"

# Operator session authentication cookie name
VITE_OPERATOR_AUTH_COOKIE="stheno_op_sess"
```

---

## 5. Development & Build Commands

All commands are run from the `apps/platform/` directory:

| Command | Action |
|---|---|
| `npm run dev` | Starts local Vite dev server on `http://127.0.0.1:5174` |
| `npm run build` | Builds the production bundle to `dist/` |
| `npm run preview` | Previews the production build locally |
| `npm run typecheck` | Validates TypeScript types (`tsc --noEmit`) |
