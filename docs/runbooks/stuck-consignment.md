# Runbook: Stuck Consignment

**Incident Domain:** Courier Delivery, Fulfillment State Machine & Consignment Tracking  
**Affected Systems:** `stheno-fulfillment`, `stheno-fulfillment-pathao`, `stheno-fulfillment-steadfast`, `stheno-fulfillment-redx`  
**Relevant Tasks & Specs:** P23-04, P23-05, P23-10, P23-11, [WORKFLOWS.md](../WORKFLOWS.md)

---

## 1. Incident Definition & Severity

A consignment is considered "stuck" when:
1. Creation call returned an ambiguous error or timed out, leaving an unsettled outbound request in `fulfillment.outbound_requests` (`state IN ('in_flight', 'unknown')`).
2. Subsequent attempts to re-dispatch produce `409 Conflict: provider_request_unsettled`.
3. Tracking updates have ceased for > 48 hours without reaching a terminal status (`Delivered`, `Returned`, `Canceled`).
4. An unmapped status code was reported by courier webhooks, triggering an `unknown_status_alarm` (P23-11).

| Severity | Threshold | Target Response | Target Resolution |
|---|---|---|---|
| **P1** | Systemic failure across all consignments for a courier | 15 mins | 1 hour |
| **P2** | High-value order (> 20,000 BDT) stuck in transit or unconfirmed dispatch | 30 mins | 4 hours |
| **P3** | Individual parcel tracking stalled in hub | 2 hours | 24 hours |

---

## 2. Detection & Alert Signals

- **Alert:** `ALERT unknown_status_alarm` with `provider_id`, `raw_status`, and `tracking_id`.
- **Conflict Response:** `409 Conflict` on `POST /admin/fulfillments` with code `provider_request_unsettled`.
- **Dashboard Flag:** Order detail page displaying `Consignment Dispatch Unconfirmed`.

---

## 3. Investigation & Diagnostic Workflow

### Step 1: Check Outbound Request Ledger
```sql
SELECT id, idempotency_key, provider_id, operation, state, error, first_sent_at, settled_at
FROM fulfillment.outbound_requests
WHERE resource_id = :fulfillment_id OR idempotency_key LIKE '%' || :reference || '%'
ORDER BY first_sent_at DESC;
```

### Step 2: Query Courier Live API Directly
Execute live status query using provider client:
```bash
stheno courier track --provider pathao --tracking-id PT-CNS-789012
```
Or check courier merchant portal using the order reference.

### Step 3: Determine State:
- **Case A: Consignment exists at courier:** Courier confirmed receipt and assigned tracking ID.
- **Case B: Consignment does NOT exist at courier:** Upstream call was never received by courier due to early network timeout.
- **Case C: Parcel is lost or damaged at courier sorting facility.**

---

## 4. Mitigation & Resolution Procedures

### Resolution for Case A: Parcel Exists at Courier (Reconciliation)
If the parcel was minted at the courier but the engine was unaware:
1. Manually resolve the unsettled request:
   ```http
   POST /admin/fulfillments/{fulfillment_id}/reconcile-consignment
   Authorization: Bearer <admin_token>
   Content-Type: application/json

   {
     "tracking_number": "PT-CNS-789012",
     "action": "attach_existing_consignment"
   }
   ```
2. Engine attaches tracking label and marks fulfillment as `shipped`.

### Resolution for Case B: Parcel Does Not Exist (Safe Re-dispatch)
If courier confirmed no parcel was created:
1. Mark the stranded outbound request as `canceled_clean`:
   ```sql
   UPDATE fulfillment.outbound_requests
   SET state = 'canceled', settled_at = NOW()
   WHERE id = :request_id AND state IN ('unknown', 'in_flight');
   ```
2. Re-trigger fulfillment dispatch from Admin UI. A fresh consignment ID is minted without duplicate risk.

### Resolution for Case C: Unmapped Courier Status Alert
If courier webhook sent an unknown status:
1. Check error log for `raw_status`:
   ```
   WARN unmapped courier status received: provider=steadfast, status="in_review_fraud_check"
   ```
2. Per P23-11, map the status in `crates/providers/stheno-fulfillment-<courier>/src/lib.rs` under `map_webhook_event`.
3. Submit fast-track hotfix to prevent repeated alerts.

---

## 5. Post-Incident Verification
1. Verify `fulfillment.outbound_requests.state == 'succeeded'` or `'canceled'`.
2. Ensure tracking URL opens the legitimate courier tracking page.
3. Verify zero duplicate consignments were created for the merchant order.
