use rust_decimal_macros::dec;
use stheno_cart::{CartService, CreateCartInput, CreateCreditLineInput, CreateLineItemInput};

#[tokio::test]
async fn test_credit_lines_deduction_and_balance_due() {
    let service = CartService::new();

    let cart = service
        .create_cart(CreateCartInput {
            currency_code: "usd".to_string(),
            ..Default::default()
        })
        .await
        .expect("cart created");

    // Add item: 1x $100
    let _ = service
        .add_line_item(
            &cart.id,
            CreateLineItemInput {
                title: "Leather Jacket".to_string(),
                quantity: 1,
                unit_price: dec!(100.00),
                ..Default::default()
            },
        )
        .await
        .expect("item added");

    let cart_initial = service.get_cart(&cart.id).await.expect("cart fetched");
    assert_eq!(cart_initial.totals.total, dec!(100.00));
    assert_eq!(cart_initial.totals.balance_due, dec!(100.00));

    // Add Gift Card 1: $30
    let cl1 = service
        .add_credit_line(
            &cart.id,
            CreateCreditLineInput {
                amount: dec!(30.00),
                reference: "gift_card".to_string(),
                reference_id: "gc_promo_30".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("credit line 1 added");

    let cart_after_cl1 = service.get_cart(&cart.id).await.expect("cart fetched");
    assert_eq!(cart_after_cl1.totals.credit_total, dec!(30.00));
    assert_eq!(cart_after_cl1.totals.balance_due, dec!(70.00));

    // Add Store Credit 2: $80 (exceeds balance of $70)
    let _ = service
        .add_credit_line(
            &cart.id,
            CreateCreditLineInput {
                amount: dec!(80.00),
                reference: "store_credit".to_string(),
                reference_id: "sc_user_loyalty".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("credit line 2 added");

    let cart_after_cl2 = service.get_cart(&cart.id).await.expect("cart fetched");
    assert_eq!(cart_after_cl2.totals.credit_total, dec!(110.00));
    // Balance due should clamp to $0.00 (not negative)
    assert_eq!(cart_after_cl2.totals.balance_due, dec!(0.00));

    // Remove Gift Card 1
    service
        .delete_credit_line(&cart.id, &cl1.id)
        .await
        .expect("credit line 1 deleted");

    let cart_final = service.get_cart(&cart.id).await.expect("cart fetched");
    assert_eq!(cart_final.totals.credit_total, dec!(80.00));
    assert_eq!(cart_final.totals.balance_due, dec!(20.00));
}
