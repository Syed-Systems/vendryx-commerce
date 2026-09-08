# Runbook: Gateway Outage & Degradation

**Incident Domain:** Payment Processing, Gateway Availability & Failover  
**Affected Systems:** `stheno-payment`, `stheno-payment-stripe`, `stheno-payment-bkash`, `stheno-payment-nagad`, `stheno-payment-sslcommerz`, `stheno-payment-offline`  
**Relevant Tasks & Specs:** P21-08, P21-11, P22-04, P28-03, [PAYMENTS.md](../PAYMENTS.md)

---

## 1. Incident Definition & Severity

A gateway outage occurs when a payment provider experiences:
1. HTTP 5xx responses or connection timeouts (> 30s) exceeding 5% of requests over a 5-minute window.
2. Webhook / IPN delivery omissions where callback events fail to arrive.
3. Upstream payment switch degradation (e.g. bKash core network maintenance, SSLCommerz bank gateway drops).

| Severity | Threshold | Target Response | Target Resolution |
|---|---|---|---|
| **P1** | Sole active gateway down in a region; checkout blocked | 5 mins | 30 mins |
| **P2** | Elevated errors on one of multiple payment options (> 15% failure) | 15 mins | 1 hour |
| **P3** | Delayed webhooks with functioning fallback polling | 1 hour | 4 hours |

---

## 2. Detection & Alert Signals

- **Alert:** `ALERT gateway_error_rate_high` with `provider_id` and `error_rate > 10%`.
- **Alert:** `ALERT gateway_timeout_cluster` with `provider_id` and `timeout_count >= 5`.
- **Checkout Metric:** Sudden drop in conversion rate on payment step.

---

## 3. Investigation & Diagnostic Workflow

### Step 1: Check Status Pages & Provider Portals
- Stripe: `https://status.stripe.com`
- bKash Developer Portal: Check notice board for emergency maintenance windows.
- SSLCommerz: Check merchant operations dashboard for bank-specific gateway downtime.

### Step 2: Query Recent Provider Requests
```sql
SELECT provider_id, operation, state, COUNT(*), 
       AVG(EXTRACT(EPOCH FROM (COALESCE(settled_at, NOW()) - first_sent_at))) AS avg_duration_sec
FROM payment.payment_provider_request
WHERE first_sent_at > NOW() - INTERVAL '15 minutes'
GROUP BY provider_id, operation, state;
```

### Step 3: Inspect Error Payloads
```sql
SELECT provider_id, error, COUNT(*)
FROM payment.payment_provider_request
WHERE state = 'unknown' AND first_sent_at > NOW() - INTERVAL '30 minutes'
GROUP BY provider_id, error;
```

---

## 4. Mitigation & Emergency Failover Procedures

### Action A: Activate Gateway Kill Switch
When an outage is confirmed, immediately disable the failing provider in affected regions to prevent customer checkout drops:
```http
POST /admin/payment-providers/{provider_id}/disable
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "region_id": "reg_bd_main",
  "reason": "Emergency failover: upstream gateway outage ticket #9941"
}
```
*Effect: The disabled provider is instantly removed from `/store/payment-providers` list for shoppers. Active checkouts present healthy alternative gateways (e.g. COD or secondary gateway).*

### Action B: Route Traffic to Fallback Gateway
- Bangladesh: If bKash is down, route traffic to Nagad or SSLCommerz or COD.
- International: If Stripe card network degrades, prompt customers to use PayPal / Apple Pay alternative if mounted.

### Action C: Reconcile Stranded In-Flight Payments
For transactions initiated immediately before or during the outage:
1. Run the background reconciliation sweep manually:
   ```bash
   stheno admin sweep-unsettled-payments --min-age-seconds 60 --limit 1000
   ```
2. The sweep queries status endpoints for each stranded request and converges them without customer double-charging.

### Action D: Re-enabling Gateway After Recovery
1. Verify provider status is green on official channels.
2. Execute synthetic health check in sandbox mode:
   ```bash
   stheno test gateway-conformance --provider {provider_id} --sandbox
   ```
3. Once synthetic tests pass, re-enable the provider in admin:
   ```http
   POST /admin/payment-providers/{provider_id}/enable
   Authorization: Bearer <admin_token>
   ```

---

## 5. Post-Incident Verification
1. Verify error rate on payment endpoint drops below 0.1%.
2. Confirm all stranded requests from outage window are transitioned to `succeeded` or `failed`.
3. Check that zero customers were double-charged during the failover and recovery cycles.
