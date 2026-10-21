import { describe, expect, it, vi } from "vitest"

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

import { render, screen, fireEvent, act } from "@testing-library/react"
import { CourierSettlementSummary } from "../components/reconciliation-summary"
import { ReconciliationBucketsView } from "../components/reconciliation-buckets-view"
import { DiscrepancyAlertsBanner } from "../components/discrepancy-alerts-banner"
import { ReconciliationView } from "../reconciliation-view"

describe("P24-11: Admin Reconciliation UI", () => {
  describe("CourierSettlementSummary", () => {
    it("renders collected, fees, and remitted amounts accurately", () => {
      render(
        <CourierSettlementSummary
          cycleId="cset_cyc_01"
          providerId="pathao"
          courierCycleId="pth_inv_w36"
          totalCollected={15000.0}
          totalFees={950.0}
          totalRemitted={14050.0}
          currency="BDT"
          status="reconciled"
        />
      )

      expect(screen.getByText("pathao Settlement Cycle")).not.toBeNull()
      expect(screen.getByText("৳ 15000.00")).not.toBeNull()
      expect(screen.getByText("- ৳ 950.00")).not.toBeNull()
      expect(screen.getByText("৳ 14050.00")).not.toBeNull()
      expect(screen.getByText("BALANCED")).not.toBeNull()
    })

    it("detects fee mismatch when remitted + fees != collected", () => {
      render(
        <CourierSettlementSummary
          cycleId="cset_cyc_02"
          providerId="steadfast"
          courierCycleId="sf_inv_w36"
          totalCollected={10000.0}
          totalFees={600.0}
          totalRemitted={9000.0} // 9000 + 600 = 9600 != 10000
          currency="BDT"
          status="discrepant"
        />
      )

      expect(screen.getByText("FEE MISMATCH")).not.toBeNull()
    })
  })

  describe("ReconciliationBucketsView", () => {
    const mockBuckets = {
      matched: [
        {
          consignmentId: "cons_matched_101",
          trackingCode: "trk_matched_101",
          orderId: "ord_101",
          expectedAmount: 2500,
          collectedAmount: 2500,
          remittedAmount: 2380,
          feeAmount: 120,
        },
      ],
      collectedNotRemitted: [
        {
          consignmentId: "cons_unremitted_202",
          trackingCode: "trk_unremitted_202",
          expectedAmount: 1800,
          collectedAmount: 1800,
          remittedAmount: 0,
          feeAmount: 0,
          reason: "Cash collected at doorstep but no courier remittance received",
        },
      ],
      remittedNotMatched: [
        {
          consignmentId: "cons_ghost_303",
          trackingCode: "trk_ghost_303",
          expectedAmount: 0,
          collectedAmount: 3200,
          remittedAmount: 3050,
          feeAmount: 150,
          reason: "No matching internal order found",
        },
      ],
      discrepant: [
        {
          consignmentId: "cons_disc_404",
          trackingCode: "trk_disc_404",
          orderId: "ord_404",
          expectedAmount: 2000,
          collectedAmount: 1600,
          remittedAmount: 1500,
          feeAmount: 100,
          reason: "Collected amount mismatch: captured 2000 != courier collected 1600",
        },
      ],
    }

    it("displays matched bucket by default and switches to other buckets", () => {
      render(
        <ReconciliationBucketsView
          matched={mockBuckets.matched}
          collectedNotRemitted={mockBuckets.collectedNotRemitted}
          remittedNotMatched={mockBuckets.remittedNotMatched}
          discrepant={mockBuckets.discrepant}
        />
      )

      // Matched bucket visible by default
      expect(screen.getByText("cons_matched_101")).not.toBeNull()
      expect(screen.getByText("ord_101")).not.toBeNull()

      // Switch to Collected Not Remitted
      fireEvent.click(screen.getByText(/Collected Not Remitted/))
      expect(screen.getByText("cons_unremitted_202")).not.toBeNull()
      expect(
        screen.getByText(/Cash collected at doorstep but no courier remittance received/)
      ).not.toBeNull()

      // Switch to Remitted Not Matched
      fireEvent.click(screen.getByText(/Remitted Not Matched/))
      expect(screen.getByText("cons_ghost_303")).not.toBeNull()
      expect(screen.getByText(/No matching internal order found/)).not.toBeNull()

      // Switch to Discrepant
      fireEvent.click(screen.getByText(/Discrepant/))
      expect(screen.getByText("cons_disc_404")).not.toBeNull()
      expect(
        screen.getByText(/Collected amount mismatch: captured 2000 != courier collected 1600/)
      ).not.toBeNull()
    })
  })

  describe("DiscrepancyAlertsBanner", () => {
    it("renders alerts for aged unsettled lines exceeding cycle threshold", () => {
      const alerts = [
        {
          lineId: "ln_01",
          cycleId: "cyc_01",
          consignmentId: "cons_alert_999",
          trackingCode: "trk_alert_999",
          orderId: "ord_alert_999",
          unremittedAmount: 1500,
          ageDays: 12,
          alertLevel: "critical",
          message: "Unsettled consignment has remained unmatched for 12 days",
        },
      ]

      render(<DiscrepancyAlertsBanner alerts={alerts} />)

      expect(screen.getByText("cons_alert_999")).not.toBeNull()
      expect(screen.getByText("12 days overdue")).not.toBeNull()
      expect(screen.getByText("Unremitted: ৳ 1500.00")).not.toBeNull()
    })
  })

  describe("ReconciliationView", () => {
    it("integrates summary, buckets, alerts, and actions", async () => {
      const onImport = vi.fn()
      const onReconcile = vi.fn()

      render(
        <ReconciliationView
          cycle={{
            cycleId: "cyc_01",
            providerId: "redx",
            courierCycleId: "rx_payout_2026_w36",
            totalCollected: 8000,
            totalFees: 500,
            totalRemitted: 7500,
            currency: "BDT",
            status: "reconciled",
          }}
          onImportReport={onImport}
          onRunReconciliation={onReconcile}
        />
      )

      expect(
        screen.getByText("Courier COD Settlement & Reconciliation")
      ).not.toBeNull()
      expect(screen.getByText("redx Settlement Cycle")).not.toBeNull()

      fireEvent.click(screen.getByText("Import Settlement Report"))
      expect(onImport).toHaveBeenCalled()

      await act(async () => {
        fireEvent.click(screen.getByText("Reconcile Cycle"))
      })
      expect(onReconcile).toHaveBeenCalledWith("cyc_01")
    })
  })
})
