//! P24-02: Order carries three distinct amounts: order total, collected, remitted. Never conflated.
//! Verified against live Postgres with tenant isolation under RLS.

use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::tenant;
use stheno_db::testing::TestDb;
use stheno_order::dto::{CodAmounts, CreateOrderInput};
use stheno_order::service::OrderService;

#[test]
fn cod_amounts_accounting_identity_and_distinctness() {
    let order_total = dec!(2500.00);
    let collected = dec!(2500.00);
    let courier_fee = dec!(120.00); // 100 delivery + 20 COD percentage fee
    let remitted = dec!(2380.00); // 2500 - 120

    let cod =
        CodAmounts::new(order_total, collected, remitted, courier_fee).expect("valid COD amounts");

    // Assert all three figures are distinct numbers (BP8: never conflated)
    assert_eq!(cod.order_total(), dec!(2500.00));
    assert_eq!(cod.collected_amount(), dec!(2500.00));
    assert_eq!(cod.remitted_amount(), dec!(2380.00));
    assert_eq!(cod.courier_fee(), dec!(120.00));

    assert_ne!(
        cod.order_total(),
        cod.remitted_amount(),
        "order total must never be conflated with courier net remittance"
    );
    assert_ne!(
        cod.collected_amount(),
        cod.remitted_amount(),
        "cash collected must never be conflated with courier net remittance"
    );

    // Accounting identity: sum(remitted) + sum(fees) == sum(collected)
    assert_eq!(
        cod.remitted_amount() + cod.courier_fee(),
        cod.collected_amount(),
        "remitted + fees must exactly equal collected in decimal arithmetic"
    );

    assert!(cod.assert_never_conflated().is_ok());
}

#[test]
fn invalid_accounting_identity_is_rejected() {
    let order_total = dec!(2500.00);
    let collected = dec!(2500.00);
    let courier_fee = dec!(120.00);
    let invalid_remitted = dec!(2400.00); // 2400 + 120 = 2520 != 2500

    let err = CodAmounts::new(order_total, collected, invalid_remitted, courier_fee);
    assert!(
        err.is_err(),
        "mismatched courier fee accounting must be rejected"
    );
}

#[test]
fn partial_collection_preserves_three_way_distinctness() {
    // Customer partially paid 2000 on a 2500 order
    let order_total = dec!(2500.00);
    let collected = dec!(2000.00);
    let courier_fee = dec!(100.00);
    let remitted = dec!(1900.00); // 2000 - 100

    let cod = CodAmounts::new(order_total, collected, remitted, courier_fee)
        .expect("valid partial COD amounts");

    assert_eq!(cod.order_total(), dec!(2500.00));
    assert_eq!(cod.collected_amount(), dec!(2000.00));
    assert_eq!(cod.remitted_amount(), dec!(1900.00));

    assert_ne!(cod.order_total(), cod.collected_amount());
    assert_ne!(cod.collected_amount(), cod.remitted_amount());
    assert_ne!(cod.order_total(), cod.remitted_amount());
}

#[tokio::test]
async fn order_persists_three_distinct_amounts_in_database() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = OrderService::new(pool);

    tenant::scope("ten_order_amounts", async {
        // 1. Create a base order
        let order = service
            .create_order(CreateOrderInput {
                currency_code: "bdt".to_string(),
                email: Some("customer@example.com".to_string()),
                ..Default::default()
            })
            .await
            .expect("order creation");

        // 2. Attach COD distinct amounts
        let order_total = dec!(3500.00);
        let collected = dec!(3500.00);
        let courier_fee = dec!(150.00);
        let remitted = dec!(3350.00);

        let saved_cod = service
            .update_cod_amounts(&order.id, order_total, collected, remitted, courier_fee)
            .await
            .expect("save COD amounts");

        assert_eq!(saved_cod.order_total(), dec!(3500.00));
        assert_eq!(saved_cod.collected_amount(), dec!(3500.00));
        assert_eq!(saved_cod.remitted_amount(), dec!(3350.00));

        // 3. Re-read through full DTO assembly and verify amounts are populated and distinct
        let reloaded = service.get_order(&order.id).await.expect("reloaded order");
        let cod = reloaded
            .cod_amounts
            .expect("cod_amounts present on reloaded order");

        assert_eq!(cod.order_total, dec!(3500.00));
        assert_eq!(cod.collected_amount, dec!(3500.00));
        assert_eq!(cod.remitted_amount, dec!(3350.00));
        assert_eq!(cod.courier_fee, dec!(150.00));

        assert_ne!(cod.order_total, cod.remitted_amount);
        assert_ne!(cod.collected_amount, cod.remitted_amount);
        assert_eq!(cod.remitted_amount + cod.courier_fee, cod.collected_amount);
    })
    .await;

    db.drop().await;
}
