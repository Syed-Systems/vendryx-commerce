use stheno_cart::{CartService, CreateCartInput};

#[tokio::test]
async fn test_cart_secret_cookie_binding_and_claim_on_login() {
    let service = CartService::new();

    // 1. Create anonymous cart
    let cart = service
        .create_cart(CreateCartInput {
            currency_code: "usd".to_string(),
            ..Default::default()
        })
        .await
        .expect("anonymous cart created");

    assert!(cart.customer_id.is_none());
    assert!(cart.cart_secret.starts_with("csec_"));

    // 2. Fetch with correct secret
    let fetched = service
        .get_cart_with_secret(&cart.id, &cart.cart_secret)
        .await
        .expect("fetch with valid secret succeeds");
    assert_eq!(fetched.id, cart.id);

    // 3. Fetch with incorrect secret -> Unauthorized (F6 / F32)
    let err = service
        .get_cart_with_secret(&cart.id, "csec_wrong_secret")
        .await
        .unwrap_err();
    match err {
        stheno_types::Error::Unauthorized(_) => {}
        _ => panic!("expected Unauthorized error on bad secret"),
    }

    // 4. Claim cart on customer login
    let claimed = service
        .claim_cart_on_login(&cart.id, "cus_12345", &cart.cart_secret)
        .await
        .expect("claim cart on login succeeds");

    assert_eq!(claimed.customer_id.as_deref(), Some("cus_12345"));

    // 5. Attempting to claim with wrong secret fails
    let claim_err = service
        .claim_cart_on_login(&cart.id, "cus_attacker", "csec_fake")
        .await
        .unwrap_err();
    match claim_err {
        stheno_types::Error::Unauthorized(_) => {}
        _ => panic!("expected Unauthorized error on fake secret"),
    }

    // 6. Attempting to claim already owned cart by different customer fails with Conflict
    let steal_err = service
        .claim_cart_on_login(&cart.id, "cus_different", &cart.cart_secret)
        .await
        .unwrap_err();
    match steal_err {
        stheno_types::Error::Conflict { .. } => {}
        _ => panic!("expected Conflict error when stealing claimed cart"),
    }
}
