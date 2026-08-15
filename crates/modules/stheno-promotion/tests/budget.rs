use rust_decimal_macros::dec;
use stheno_promotion::{CreateCampaignBudgetInput, CreateCampaignInput, PromotionService};

#[tokio::test]
async fn test_campaign_atomic_budget_consumption() {
    let service = PromotionService::new();

    let camp = service
        .create_campaign(CreateCampaignInput {
            name: "Flash Sale".to_string(),
            campaign_identifier: "FLASH2026".to_string(),
            starts_at: None,
            ends_at: None,
            currency: Some("usd".to_string()),
            budget: Some(CreateCampaignBudgetInput {
                budget_type: "spend".to_string(),
                limit: Some(dec!(100.00)), // $100 total budget
                currency_code: Some("usd".to_string()),
            }),
            metadata: None,
        })
        .await
        .expect("campaign created");

    // 1st consumption: $60 (allowed, remaining $40)
    let u1 = service
        .consume_campaign_budget(&camp.id, dec!(60.00), Some("ord_1"), None)
        .await
        .expect("1st consumption succeeds");
    assert_eq!(u1.amount, dec!(60.00));

    // 2nd consumption: $50 (exceeds $40 remaining -> rejected with Conflict)
    let err = service
        .consume_campaign_budget(&camp.id, dec!(50.00), Some("ord_2"), None)
        .await
        .unwrap_err();

    match err {
        stheno_types::Error::Conflict { code, .. } => {
            assert_eq!(code, "budget_exhausted");
        }
        _ => panic!("expected Conflict budget_exhausted error"),
    }

    // 3rd consumption: $40 (exactly exhausts remaining budget)
    let u3 = service
        .consume_campaign_budget(&camp.id, dec!(40.00), Some("ord_3"), None)
        .await
        .expect("3rd consumption succeeds");
    assert_eq!(u3.amount, dec!(40.00));

    // Campaign budget used is exactly $100
    let reloaded = service.get_campaign(&camp.id).await.expect("reloaded camp");
    assert_eq!(reloaded.budget.unwrap().used, dec!(100.00));
}
