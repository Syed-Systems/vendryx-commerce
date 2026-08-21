//! A provider request stranded by a crash is converged without anyone retrying it.
//!
//! `ambiguous_timeout.rs` already proves the recovery that happens when the *same
//! idempotency key is retried*. This proves the case where nothing ever retries —
//! the process died mid-call, the customer has gone, and a gateway that only
//! notifies on success will never tell us anything. That is the case where money
//! goes missing, and the only thing that closes it is a sweep.

use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::PaymentService;
use stheno_payment_offline::OfflinePaymentProvider;
use stheno_types::Result;

const PROVIDER: &str = "pp_stranded_test";

async fn service_with_a_stranded_request(key: &str) -> (TestDb, PaymentService) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    service.register_provider(Arc::new(OfflinePaymentProvider::new(Some(PROVIDER))));

    // The call goes out and the answer never comes back. The ledger row is written
    // before the call, so it survives; its state is now ambiguous.
    let res: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            PROVIDER,
            "capture",
            "payment",
            "pay_stranded_1",
            &json!({ "amount": 2500, "currency": "bdt" }),
            |_k| async move {
                Err(stheno_types::Error::Internal(
                    "connection reset while calling the gateway".to_string().into(),
                ))
            },
        )
        .await;

    assert!(res.is_err(), "the caller must see the failure");

    (db, service)
}

#[tokio::test]
async fn a_stranded_request_is_converged_without_any_retry() {
    let (db, service) = service_with_a_stranded_request("idem_stranded_no_retry").await;

    // Nobody retries. The sweep asks the provider what actually happened.
    let report = service
        .reconcile_unsettled_requests(0, 100)
        .await
        .expect("sweep runs");

    assert_eq!(
        report.scanned, 1,
        "the stranded request must be visible to the sweep"
    );
    assert_eq!(
        report.converged, 1,
        "the provider reports the money moved, so the request must settle as succeeded \
         without anyone retrying it — this is the crash path, and leaving it ambiguous \
         is how a real payment goes missing"
    );
    assert_eq!(report.still_unsettled, 0);

    // Idempotent: a second sweep finds nothing left to do.
    let second = service
        .reconcile_unsettled_requests(0, 100)
        .await
        .expect("sweep runs again");
    assert_eq!(
        second.scanned, 0,
        "a settled request must not be swept again"
    );

    db.drop().await;
}

#[tokio::test]
async fn a_request_younger_than_the_threshold_is_left_alone() {
    let (db, service) = service_with_a_stranded_request("idem_stranded_young").await;

    // A request that has only just been sent may simply still be running. Settling it
    // early would decide the outcome of a call that has not finished.
    let report = service
        .reconcile_unsettled_requests(3600, 100)
        .await
        .expect("sweep runs");

    assert_eq!(
        report.scanned, 0,
        "requests younger than the age threshold must not be touched"
    );
    assert_eq!(report.converged, 0);

    db.drop().await;
}
