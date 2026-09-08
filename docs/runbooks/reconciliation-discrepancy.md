# Runbook: Reconciliation Discrepancy

**Incident Domain:** COD Settlement, Courier Accounting & Payment Reconciliation  
**Affected Systems:** `stheno-payment`, `stheno-order`, `stheno-fulfillment`, Admin Reconciliation UI  
**Relevant Tasks & Specs:** P24-05, P24-06, P24-07, P24-08, [PAYMENTS.md](../PAYMENTS.md)

---

## 1. Incident Definition & Severity

A reconciliation discrepancy occurs when:
1. Automated settlement import raises a discrepancy row (`payment.payment_reconciliation_run` with mismatched totals).
2. Courier fee accounting fails the invariant: `sum(remitted) + sum(fees) == sum(collected)`.
3. An unmatched settlement line exceeds the cycle length window (> 14 days), triggering a `settlement_alerting` event.
4. A merchant reports a cash mismatch between courier payout and delivered COD orders.

| Severity | Threshold | Target Response | Target Resolution |
|---|---|---|---|
| **P1** | Total variance > 50,000 BDT or systemic mismatch across all shipments | 15 mins | 2 hours |
| **P2** | Variance on single courier cycle > 5,000 BDT | 1 hour | 8 hours |
| **P3** | Isolated parcel rounding variance (< 100 BDT) | 1 business day | Next cycle |

---

## 2. Detection & Alert Signals

- **Log Alert:** `WARN stheno_payment::service: settlement_mismatch_detected` with `cycle_id` and `variance_amount`.
- **System Metric:** `payment_reconciliation_discrepancies_total > 0`.
- **Admin Alert:** Highlighting on `/admin/reconciliation` with status `DISCREPANCY_FLAGGED`.

---

## 3. Investigation & Diagnostic Workflow

### Step 1: Query the Reconciliation Run Record
Execute via psql or platform read-only tooling:
```sql
SELECT id, courier_id, cycle_start, cycle_end, 
       total_collected, total_remitted, total_fees, discrepancy_amount, state
FROM payment.payment_reconciliation_run
WHERE state = 'discrepancy' AND tenant_id = :tenant_id
ORDER BY created_at DESC LIMIT 5;
```

### Step 2: Inspect Discrepant Lines
```sql
SELECT id, consignment_reference, order_id, 
       amount_collected, amount_remitted, fee_deducted, discrepancy_reason
FROM payment.payment_reconciliation_line
WHERE run_id = :run_id AND status != 'matched';
```

### Step 3: Categorize the Root Cause
- **Courier Under-Remittance:** Courier collected full amount from customer but remitted less than agreed contract rate.
- **Courier Fee Drift:** Courier applied unannounced surcharge (e.g. holiday fuel surcharge, COD fee hike).
- **Return / Undelivered Mismatch:** Order was returned by customer, but courier statement mistakenly charged collection fees.
- **Decimal Rounding Anomaly:** Currency minor unit conversion mismatch across provider boundary.

---

## 4. Mitigation & Resolution Procedures

### Action A: Re-importing Corrected Settlement File (Idempotent)
Per P24-05, settlement imports are strictly idempotent:
1. Obtain corrected settlement CSV/JSON from the courier partner.
2. Re-upload through Admin UI or CLI:
   ```bash
   stheno admin settlement-import --courier pathao --file corrected-settlement.csv
   ```
3. Verify that the previous discrepancy rows converge without creating duplicate line entries.

### Action B: Courier Dispute Notice
If the courier under-remitted:
1. Export the discrepancy report from `/admin/reconciliation/cycles/:cycle_id/export`.
2. Transmit dispute packet to courier merchant ops account manager with order references.
3. Mark line items as `DISPUTE_PENDING` in the reconciliation ledger.

### Action C: Manual Settlement Adjustment
For approved fee variances or one-off waiver credits:
```http
POST /admin/reconciliation/cycles/{cycle_id}/adjustments
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "amount": "150.00",
  "reason": "Approved courier fuel surcharge variance per ticket #8821",
  "approved_by": "finance@merchant.com"
}
```

---

## 5. Post-Incident Verification
1. Verify `payment.payment_reconciliation_run.state == 'settled'`.
2. Confirm mathematical balance: `total_remitted + total_fees == total_collected + adjustments`.
3. Ensure no dangling lines remain in `payment_reconciliation_line`.
