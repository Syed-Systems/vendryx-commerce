use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{CreateCampaignBudgetInput, CreateCampaignInput, PromotionService};

#[tokio::test]
async fn test_campaign_budget_concurrency_100_parallel_requests() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 20).expect("pool");
    let service = Arc::new(PromotionService::new(pool));

    // Create a campaign with a strict usage limit of 50 orders
    let camp = service
        .create_campaign(CreateCampaignInput {
            name: "Limited 50 Uses".to_string(),
            campaign_identifier: "FIRST50".to_string(),
            starts_at: None,
            ends_at: None,
            currency: None,
            budget: Some(CreateCampaignBudgetInput {
                budget_type: "usage".to_string(),
                limit: Some(dec!(50.00)), // Limit: exactly 50 uses
                currency_code: None,
            }),
            metadata: None,
        })
        .await
        .expect("campaign created");

    let camp_id = Arc::new(camp.id);
    let mut handles = Vec::with_capacity(100);

    // Spawn 100 concurrent tasks attempting to consume 1 usage each
    for i in 0..100 {
        let svc = Arc::clone(&service);
        let cid = Arc::clone(&camp_id);
        handles.push(tokio::spawn(async move {
            let order_id = format!("ord_{}", i);
            svc.consume_campaign_budget(&cid, dec!(1.00), Some(&order_id), None)
                .await
        }));
    }

    let mut successes = 0;
    let mut failures = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => successes += 1,
            Err(stheno_types::Error::Conflict {
                code: "budget_exhausted",
                ..
            }) => {
                failures += 1;
            }
            Err(e) => panic!("unexpected error during concurrency test: {:?}", e),
        }
    }

    // Exactly 50 must succeed and 50 must fail with budget_exhausted
    assert_eq!(successes, 50);
    assert_eq!(failures, 50);

    db.drop().await;
}
