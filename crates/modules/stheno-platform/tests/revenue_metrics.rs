//! P30-16: revenue metrics come from paid invoices, not tier assignment.

use chrono::{TimeZone, Utc};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{GenerateInvoiceInput, NewTenant, PlatformService, RevenueMetricsInput};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;

async fn service() -> (TestDb, PlatformService) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    (db, PlatformService::new(pool))
}

async fn invoice(
    service: &PlatformService,
    tenant_id: &str,
    y: i32,
    m: u32,
    amount_minor: i64,
) -> String {
    let period_start = Utc.with_ymd_and_hms(y, m, 1, 0, 0, 0).unwrap();
    let period_end = if m == 12 {
        Utc.with_ymd_and_hms(y + 1, 1, 1, 0, 0, 0).unwrap()
    } else {
        Utc.with_ymd_and_hms(y, m + 1, 1, 0, 0, 0).unwrap()
    };
    service
        .generate_invoice(GenerateInvoiceInput {
            tenant_id: tenant_id.to_string(),
            tier: "pro".to_string(),
            period_start,
            period_end,
            amount: Money::from_minor_units(amount_minor, CurrencyCode::Bdt),
            payable_amount: Some(Money::from_minor_units(amount_minor, CurrencyCode::Bdt)),
            fx_metadata: None,
        })
        .await
        .expect("invoice")
        .id
}

#[tokio::test]
async fn unpaid_plan_assignments_do_not_count_as_revenue() {
    let (db, service) = service().await;

    let paid_previous = invoice(&service, TestDb::TEST_TENANT, 2026, 7, 100_000).await;
    let paid_current = invoice(&service, TestDb::TEST_TENANT, 2026, 8, 150_000).await;
    service
        .mark_invoice_collected(
            &paid_previous,
            Utc.with_ymd_and_hms(2026, 7, 2, 0, 0, 0).unwrap(),
        )
        .await
        .expect("previous paid");
    service
        .mark_invoice_collected(
            &paid_current,
            Utc.with_ymd_and_hms(2026, 8, 2, 0, 0, 0).unwrap(),
        )
        .await
        .expect("current paid");

    service
        .create_tenant(NewTenant {
            id: "ten_unpaid_pro".to_string(),
            name: "Unpaid Pro".to_string(),
            tier: "pro".to_string(),
            domain: None,
        })
        .await
        .expect("unpaid pro tenant");
    invoice(&service, "ten_unpaid_pro", 2026, 8, 150_000).await;

    service
        .create_tenant(NewTenant {
            id: "ten_churned".to_string(),
            name: "Churned".to_string(),
            tier: "pro".to_string(),
            domain: None,
        })
        .await
        .expect("churned tenant");
    let churned_previous = invoice(&service, "ten_churned", 2026, 7, 150_000).await;
    service
        .mark_invoice_collected(
            &churned_previous,
            Utc.with_ymd_and_hms(2026, 7, 2, 0, 0, 0).unwrap(),
        )
        .await
        .expect("churned previous paid");

    let metrics = service
        .revenue_metrics(RevenueMetricsInput {
            period_start: Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(),
            period_end: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        })
        .await
        .expect("metrics");

    assert_eq!(
        metrics.mrr,
        Money::from_minor_units(150_000, CurrencyCode::Bdt)
    );
    assert_eq!(
        metrics.arr,
        Money::from_minor_units(1_800_000, CurrencyCode::Bdt)
    );
    assert_eq!(
        metrics.expansion,
        Money::from_minor_units(50_000, CurrencyCode::Bdt)
    );
    assert_eq!(metrics.churned_tenants, 1);

    db.drop().await;
}
