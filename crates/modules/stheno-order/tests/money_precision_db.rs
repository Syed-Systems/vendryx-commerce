//! Property tests for Money precision through Postgres `NUMERIC` at ISO 4217 precisions (P13-23).

use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_order::*;

#[tokio::test]
async fn test_money_precision_zero_decimals_jpy() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = OrderService::new(pool);

    // JPY (0 decimal places) e.g. 12500 JPY
    let order = service
        .create_order(CreateOrderInput {
            region_id: Some("reg_jp".to_string()),
            customer_id: Some("cus_jp".to_string()),
            currency_code: "jpy".to_string(),
            items: Some(vec![
                CreateLineItemInput {
                    title: "Item 1".to_string(),
                    quantity: 3,
                    unit_price: dec!(4000), // 3 * 4000 = 12000
                    ..Default::default()
                },
                CreateLineItemInput {
                    title: "Item 2".to_string(),
                    quantity: 1,
                    unit_price: dec!(500), // 1 * 500 = 500
                    ..Default::default()
                },
            ]),
            shipping_methods: Some(vec![CreateShippingMethodInput {
                name: "Standard Express".to_string(),
                amount: dec!(800), // 800 JPY
                ..Default::default()
            }]),
            ..Default::default()
        })
        .await
        .expect("order created");

    // Total = 12000 + 500 + 800 = 13300 JPY
    assert_eq!(
        order.summary.as_ref().unwrap().current_order_total,
        dec!(13300)
    );

    // Add transaction of exact amount
    let tx = service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(13300),
                currency_code: "jpy".to_string(),
                reference: "capture".to_string(),
                reference_id: "pay_jpy_1".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("transaction added");

    assert_eq!(tx.amount, dec!(13300));
    let reloaded = service.get_order(&order.id).await.expect("reloaded");
    assert_eq!(reloaded.payment_status, "captured");
    assert_eq!(reloaded.summary.as_ref().unwrap().paid_total, dec!(13300));

    db.drop().await;
}

#[tokio::test]
async fn test_money_precision_two_decimals_usd_eur() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = OrderService::new(pool);

    // USD (2 decimal places) e.g. 19.99, 0.01 cents precision
    let order = service
        .create_order(CreateOrderInput {
            currency_code: "usd".to_string(),
            items: Some(vec![
                CreateLineItemInput {
                    title: "Product A".to_string(),
                    quantity: 7,
                    unit_price: dec!(19.99), // 7 * 19.99 = 139.93
                    ..Default::default()
                },
                CreateLineItemInput {
                    title: "Product B".to_string(),
                    quantity: 3,
                    unit_price: dec!(0.05), // 3 * 0.05 = 0.15
                    ..Default::default()
                },
            ]),
            shipping_methods: Some(vec![CreateShippingMethodInput {
                name: "Ground".to_string(),
                amount: dec!(5.49),
                ..Default::default()
            }]),
            ..Default::default()
        })
        .await
        .expect("order created");

    // Total = 139.93 + 0.15 + 5.49 = 145.57
    assert_eq!(
        order.summary.as_ref().unwrap().current_order_total,
        dec!(145.57)
    );

    // Partial transaction of 100.00
    service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(100.00),
                currency_code: "usd".to_string(),
                reference: "capture".to_string(),
                reference_id: "pay_usd_1".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("tx 1");

    // Remaining transaction of 45.57
    service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(45.57),
                currency_code: "usd".to_string(),
                reference: "capture".to_string(),
                reference_id: "pay_usd_2".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("tx 2");

    let reloaded = service.get_order(&order.id).await.expect("reloaded");
    assert_eq!(reloaded.payment_status, "captured");
    assert_eq!(reloaded.summary.as_ref().unwrap().paid_total, dec!(145.57));
    assert_eq!(
        reloaded.summary.as_ref().unwrap().pending_difference,
        dec!(0.00)
    );

    db.drop().await;
}

#[tokio::test]
async fn test_money_precision_three_decimals_kwd_bhd() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = OrderService::new(pool);

    // KWD / BHD (3 decimal places) e.g. 1.250 KWD, 0.005 KWD fils
    let order = service
        .create_order(CreateOrderInput {
            currency_code: "kwd".to_string(),
            items: Some(vec![
                CreateLineItemInput {
                    title: "Perfume".to_string(),
                    quantity: 4,
                    unit_price: dec!(12.375), // 4 * 12.375 = 49.500
                    ..Default::default()
                },
                CreateLineItemInput {
                    title: "Sample".to_string(),
                    quantity: 2,
                    unit_price: dec!(0.125), // 2 * 0.125 = 0.250
                    ..Default::default()
                },
            ]),
            shipping_methods: Some(vec![CreateShippingMethodInput {
                name: "Kuwait Post".to_string(),
                amount: dec!(1.875),
                ..Default::default()
            }]),
            ..Default::default()
        })
        .await
        .expect("order created");

    // Total = 49.500 + 0.250 + 1.875 = 51.625 KWD
    assert_eq!(
        order.summary.as_ref().unwrap().current_order_total,
        dec!(51.625)
    );

    service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(51.625),
                currency_code: "kwd".to_string(),
                reference: "capture".to_string(),
                reference_id: "pay_kwd_1".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("tx kwd");

    let reloaded = service.get_order(&order.id).await.expect("reloaded");
    assert_eq!(reloaded.payment_status, "captured");
    assert_eq!(reloaded.summary.as_ref().unwrap().paid_total, dec!(51.625));

    db.drop().await;
}

#[tokio::test]
async fn test_money_precision_four_decimals_clf() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = OrderService::new(pool);

    // CLF (4 decimal places) e.g. 0.0001 precision
    let order = service
        .create_order(CreateOrderInput {
            currency_code: "clf".to_string(),
            items: Some(vec![CreateLineItemInput {
                title: "Asset Unit".to_string(),
                quantity: 10,
                unit_price: dec!(0.0123), // 10 * 0.0123 = 0.1230
                ..Default::default()
            }]),
            shipping_methods: Some(vec![CreateShippingMethodInput {
                name: "Secure Courier".to_string(),
                amount: dec!(0.0005),
                ..Default::default()
            }]),
            ..Default::default()
        })
        .await
        .expect("order created");

    // Total = 0.1230 + 0.0005 = 0.1235
    assert_eq!(
        order.summary.as_ref().unwrap().current_order_total,
        dec!(0.1235)
    );

    db.drop().await;
}
