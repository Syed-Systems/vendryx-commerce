use serde_json::json;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::PaymentService;
use stheno_types::Result;

#[tokio::test]
async fn test_idempotency_durability_across_restarts() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let key = "idem_durable_restart_key_456";
    let payload = json!({
        "amount": 10000,
        "currency": "usd"
    });

    // 1. Process initial idempotent call with first service instance
    let mut initial_executed = false;
    {
        let service1 = PaymentService::new(pool.clone());
        let res1: Result<serde_json::Value> = service1
            .idempotent_call(
                key,
                "pp_stripe_stripe",
                "capture",
                "payment",
                "pay_durable_1",
                &payload,
                |_k| {
                    initial_executed = true;
                    async move {
                        Ok(json!({
                            "status": "succeeded",
                            "charge_id": "ch_durable_123"
                        }))
                    }
                },
            )
            .await;

        assert!(res1.is_ok());
        assert!(initial_executed);
        assert_eq!(res1.unwrap()["status"], "succeeded");
    }

    // 2. Simulate complete restart: instantiate a brand-new PaymentService from pool
    {
        let service2 = PaymentService::new(pool.clone());
        let mut second_executed = false;

        let res2: Result<serde_json::Value> = service2
            .idempotent_call(
                key,
                "pp_stripe_stripe",
                "capture",
                "payment",
                "pay_durable_1",
                &payload,
                |_k| {
                    second_executed = true;
                    async move {
                        panic!("Should not execute closure for existing durable idempotency key!");
                    }
                },
            )
            .await;

        assert!(res2.is_ok());
        assert!(
            !second_executed,
            "Durable key must replay cached response without re-executing"
        );
        let body = res2.unwrap();
        assert_eq!(body["status"], "succeeded");
        assert_eq!(body["charge_id"], "ch_durable_123");
    }

    db.drop().await;
}
