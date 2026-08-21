# Stheno Platform Control Plane Runbook

This document details operational procedures for Stheno platform operators managing multi-tenant infrastructure, incidents, credential compromise, and recovery.

---

## 1. Incident Response & Merchant Read-Only Lockdown

When investigating a tenant data anomaly, database migration, or security incident:

1. **Lock Merchant Writes (Non-Destructive)**:
   - Navigate to **Support & Incidents** tab in the Platform Control Plane.
   - Or issue API command:
     ```http
     POST /platform/tenants/{tenant_id}/read-only
     Authorization: Bearer <operator_token>
     Content-Type: application/json

     {
       "read_only": true,
       "reason": "Incident #1042: Isolating cart mutation race during database partition"
     }
     ```
   - **Behavior**: Reads continue serving with 200 OK. All mutations (orders, inventory updates, settings) immediately return `423 Locked` or `503 Service Unavailable`.

2. **Unlock After Resolution**:
   ```http
   POST /platform/tenants/{tenant_id}/read-only
   Authorization: Bearer <operator_token>
   Content-Type: application/json

   {
     "read_only": false,
     "reason": "Incident #1042 resolved; migrations applied cleanly"
   }
   ```

---

## 2. Operator Credential Compromise & Rotation

If an operator token is suspected of leaking:

1. **Immediate Revocation**:
   ```http
   DELETE /platform/operators/{operator_id}
   Authorization: Bearer <active_operator_or_bootstrap_token>
   ```
   *Revocation takes effect in Postgres within the current transaction. All subsequent bearer calls return 404.*

2. **Issue Replacement Credential**:
   ```http
   POST /platform/operators
   Authorization: Bearer <active_operator_token>
   Content-Type: application/json

   {
     "name": "Ops Engineer (Rotated)"
   }
   ```

3. **Audit Verification**:
   Inspect the immutable audit trail to identify all actions taken by the compromised credential:
   ```http
   GET /platform/audit?entity_id={operator_id}
   ```

---

## 3. Tenant Purge & Data Retention Reversal

Under GDPR/privacy requests, tenants may be scheduled for purge:

1. **Pre-Purge Archive**:
   Ensure `GET /platform/tenants/{id}/export` creates a signed bundle before initiating purge.
2. **Purge Delay Window**:
   Purges are staged with a 14-day grace period where `status = "closed"`.
3. **Reversal Prior to Final Purge**:
   ```http
   POST /platform/tenants/{id}/reactivate
   Authorization: Bearer <operator_token>
   Content-Type: application/json

   {
     "reason": "Merchant requested retention cancel within 14-day window"
   }
   ```

---

## 4. Payment Gateway Kill Switch

In the event of upstream PSP downtime (e.g. SSLCommerz, bKash, Stripe):

1. **Engage Kill Switch**:
   ```http
   POST /platform/gateways/{provider_id}/kill-switch
   Authorization: Bearer <operator_token>
   Content-Type: application/json

   {
     "is_killed": true,
     "reason": "Upstream bKash gateway 502 outage"
   }
   ```
2. **Behavior**:
   Prevents double charging and immediately fails closed with unambiguous error messaging rather than unhandled timeout.
