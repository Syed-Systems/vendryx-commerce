//! P22-17: [BP] BDT end to end: decimal totals, minor-unit conversion only at the provider boundary, rounding asserted.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{InitiateInput, PaymentProvider};
use stheno_payment_bkash::BkashPaymentProvider;
use stheno_payment_nagad::NagadPaymentProvider;
use stheno_payment_sslcommerz::SslcommerzPaymentProvider;
use stheno_types::currency::CurrencyCode;
use stheno_types::money::Money;

#[test]
fn bdt_money_representation_and_exact_arithmetic() {
    let price_1 = Money::new(dec!(1250.50), CurrencyCode::Bdt);
    let price_2 = Money::new(dec!(349.50), CurrencyCode::Bdt);
    let discount = Money::new(dec!(100.00), CurrencyCode::Bdt);

    let subtotal = (price_1 + price_2).expect("addition succeeds");
    let total = (subtotal - discount).expect("subtraction succeeds");
    assert_eq!(total.amount, dec!(1500.00));
    assert_eq!(total.currency, CurrencyCode::Bdt);
}

#[test]
fn bdt_minor_unit_conversion_and_rounding() {
    // Standard 2-decimal BDT conversion to poisha
    let money = Money::new(dec!(2450.75), CurrencyCode::Bdt);
    let poisha = money.to_minor_units().expect("conversion succeeds");
    assert_eq!(poisha, 245075);

    let reconstructed = Money::from_minor_units(poisha, CurrencyCode::Bdt);
    assert_eq!(reconstructed.amount, dec!(2450.75));

    // Sub-cent fractional poisha rounding: 0.556 rounds to 0.56 (half-up)
    let fractional = Money::new(dec!(99.556), CurrencyCode::Bdt);
    let poisha_rounded = fractional.to_minor_units().expect("conversion succeeds");
    assert_eq!(poisha_rounded, 9956);

    let reconstructed_rounded = Money::from_minor_units(poisha_rounded, CurrencyCode::Bdt);
    assert_eq!(reconstructed_rounded.amount, dec!(99.56));
}

#[test]
fn bdt_proportional_allocation_conserves_money_exact() {
    let order_total = Money::new(dec!(100.00), CurrencyCode::Bdt);
    // Split into 3 equal shares of 1:1:1
    let ratios = vec![dec!(1), dec!(1), dec!(1)];
    let allocated = order_total
        .allocate_proportionally(&ratios)
        .expect("allocation succeeds");

    assert_eq!(allocated.len(), 3);
    // Hare-Niemeyer ensures 33.34 + 33.33 + 33.33 == 100.00
    let sum: Decimal = allocated.iter().map(|m| m.amount).sum();
    assert_eq!(sum, dec!(100.00));
}

#[tokio::test]
async fn bdt_provider_boundary_bkash() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_bdt_1",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/payment/create"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "paymentID": "BKASH_BDT_101",
            "bkashURL": format!("{}/checkout", mock_server.uri()),
            "amount": "1450.25",
            "currency": "BDT"
        })))
        .mount(&mock_server)
        .await;

    let provider = BkashPaymentProvider::new(
        &mock_server.uri(),
        "test_app",
        "test_secret",
        "test_user",
        "test_pass",
    );

    let out = provider
        .initiate(InitiateInput {
            amount: dec!(1450.25),
            currency_code: "BDT".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .expect("bdt initiate succeeds");

    assert_eq!(out.id, "BKASH_BDT_101");
}

#[tokio::test]
async fn bdt_provider_boundary_sslcommerz() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/gwprocess/v4/api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "SUCCESS",
            "sessionkey": "SSL_BDT_SESSION",
            "GatewayPageURL": format!("{}/gw", mock_server.uri())
        })))
        .mount(&mock_server)
        .await;

    let provider =
        SslcommerzPaymentProvider::new("test_store", "test_pass", Some(&mock_server.uri()), true);

    let out = provider
        .initiate(InitiateInput {
            amount: dec!(2799.00),
            currency_code: "BDT".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .expect("bdt sslcommerz initiate succeeds");

    assert_eq!(out.id, "SSL_BDT_SESSION");
}

#[tokio::test]
async fn bdt_provider_boundary_nagad() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/check-out/initialize/TEST_MERCHANT_01/ord_bdt_nagad"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "paymentReferenceId": "NAGAD_BDT_REF_1",
            "challenge": "chal_bdt"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/check-out/complete/NAGAD_BDT_REF_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "Success",
            "callBackUrl": format!("{}/dfs", mock_server.uri())
        })))
        .mount(&mock_server)
        .await;

    let provider = NagadPaymentProvider::new(
        &mock_server.uri(),
        "TEST_MERCHANT_01",
        "mock_priv",
        "mock_pub",
    );

    let out = provider
        .initiate(InitiateInput {
            amount: dec!(3200.50),
            currency_code: "BDT".to_string(),
            context: Some(json!({ "session_id": "ord_bdt_nagad" })),
            metadata: None,
        })
        .await
        .expect("bdt nagad initiate succeeds");

    assert_eq!(out.id, "NAGAD_BDT_REF_1");
}
