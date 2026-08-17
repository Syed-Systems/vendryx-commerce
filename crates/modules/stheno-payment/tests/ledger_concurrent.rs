use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::{
    AuthorizePaymentSessionInput, CapturePaymentInput, CreatePaymentCollectionInput,
    CreatePaymentSessionInput, PaymentService,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_ledger_concurrency_100_parallel_partial_captures() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 20).expect("pool");
    let service = Arc::new(PaymentService::new(pool));
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    service.register_provider(offline);

    // Payment authorized for $50.00
    let col = service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "usd".to_string(),
            amount: dec!(50.00),
            metadata: None,
        })
        .await
        .expect("col created");

    let session = service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: col.id.clone(),
                provider_id: "pp_system_default".to_string(),
                amount: dec!(50.00),
                currency_code: "usd".to_string(),
                data: None,
                context: None,
            },
            None,
        )
        .await
        .expect("session created");

    let payment = service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id: session.id.clone(),
            context: None,
        })
        .await
        .expect("authorized");

    let payment_id = Arc::new(payment.id);
    let mut handles = Vec::with_capacity(100);

    // 100 concurrent tasks attempting to capture $1.00 each against $50.00 limit
    for _ in 0..100 {
        let svc = Arc::clone(&service);
        let pid = Arc::clone(&payment_id);
        handles.push(tokio::spawn(async move {
            svc.capture_payment(CapturePaymentInput {
                payment_id: pid.to_string(),
                amount: Some(dec!(1.00)),
                created_by: None,
                metadata: None,
            })
            .await
        }));
    }

    let mut successful_captures = 0;
    let mut rejected_captures = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => successful_captures += 1,
            Err(stheno_types::Error::Conflict {
                code: "over_capture_prevented",
                ..
            }) => {
                rejected_captures += 1;
            }
            Err(e) => panic!("unexpected error during concurrency test: {:?}", e),
        }
    }

    // Invariant: Exactly 50 succeeded and 50 were rejected
    assert_eq!(
        successful_captures, 50,
        "exactly 50 $1.00 captures must succeed"
    );
    assert_eq!(
        rejected_captures, 50,
        "exactly 50 $1.00 captures must be rejected"
    );

    let reloaded = service
        .get_payment(&payment_id)
        .await
        .expect("reloaded payment");
    assert_eq!(reloaded.captured_amount, dec!(50.00));
}
