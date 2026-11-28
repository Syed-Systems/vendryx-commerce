use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use stheno_api_admin::admin_order_routes_with_social_flows_and_settings;
use stheno_auth::JwtClaims;
use stheno_db::testing::TestDb;
use stheno_fulfillment::{
    CalculatePriceContext, CreateFulfillmentItemInput, CreateFulfillmentLabelInput,
    CreateFulfillmentResponse, CreateFulfillmentSetInput, CreateServiceZoneInput,
    CreateShippingOptionInput, CreateShippingProfileInput, FulfillmentOptionDTO,
    FulfillmentProvider, FulfillmentService,
};
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateShippingMethodInput,
    OrderService,
};
use stheno_payment::PaymentService;
use stheno_settings::{dto::SetStoreSettingInput, SettingsService};
use stheno_types::{Error, Result};
use tower::ServiceExt;

struct RouteTestCourier {
    calls: AtomicUsize,
    fail_ambiguously: AtomicBool,
}

#[async_trait]
impl FulfillmentProvider for RouteTestCourier {
    fn identifier(&self) -> &'static str {
        "pathao"
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(Vec::new())
    }

    async fn validate_option(&self, _option_data: Option<&Value>) -> Result<bool> {
        Ok(true)
    }

    async fn calculate_price(
        &self,
        _option_data: Option<&Value>,
        _data: Option<&Value>,
        _context: Option<&CalculatePriceContext>,
    ) -> Result<Decimal> {
        Ok(Decimal::ZERO)
    }

    async fn create_fulfillment(
        &self,
        data: Option<&Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&Value>,
    ) -> Result<CreateFulfillmentResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_ambiguously.load(Ordering::SeqCst) {
            return Err(Error::Internal("courier response was lost".into()));
        }
        let mut response_data = data.cloned().unwrap_or_else(|| json!({}));
        if let Some(object) = response_data.as_object_mut() {
            object.insert(
                "consignment_id".to_string(),
                json!("pathao-route-consignment"),
            );
        }
        Ok(CreateFulfillmentResponse {
            data: Some(response_data),
            labels: vec![CreateFulfillmentLabelInput {
                tracking_number: "PATHAO-ROUTE-TRACK".to_string(),
                tracking_url: None,
                label_url: None,
                metadata: None,
            }],
        })
    }

    async fn cancel_fulfillment(&self, _data: Option<&Value>) -> Result<()> {
        Ok(())
    }

    async fn create_return_fulfillment(
        &self,
        _data: Option<&Value>,
    ) -> Result<CreateFulfillmentResponse> {
        Err(Error::Internal(
            "test courier does not create returns".into(),
        ))
    }
}

#[tokio::test]
async fn staff_conversion_dispatch_preflight_replay_and_ambiguous_retry_contract() {
    let previous_key = std::env::var("STHENO_CREDENTIAL_KEY").ok();
    std::env::set_var("STHENO_CREDENTIAL_KEY", "social-route-test-key-32-bytes!!");

    let db = TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("test pool");
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let fulfillment_service = Arc::new(FulfillmentService::new(pool.clone()));
    let settings_service = Arc::new(SettingsService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    let notification_service = Arc::new(stheno_notification::NotificationService::new(pool));
    let courier = Arc::new(RouteTestCourier {
        calls: AtomicUsize::new(0),
        fail_ambiguously: AtomicBool::new(false),
    });
    fulfillment_service.register_provider(courier.clone());

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('pathao', 'Pathao route test') ON CONFLICT (id) DO NOTHING",
    )
    .await;
    let fulfillment_set = fulfillment_service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "Route test fulfillment".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("fulfillment set");
    let service_zone = fulfillment_service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: fulfillment_set.id,
            name: "Bangladesh".to_string(),
            metadata: None,
        })
        .await
        .expect("service zone");
    let shipping_profile = fulfillment_service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Route test shipping".to_string(),
            r#type: None,
            metadata: None,
        })
        .await
        .expect("shipping profile");
    let shipping_option = fulfillment_service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Pathao".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: service_zone.id,
            shipping_profile_id: shipping_profile.id,
            provider_id: "pathao".to_string(),
            shipping_option_type_id: None,
            data: None,
            metadata: None,
        })
        .await
        .expect("shipping option");
    settings_service
        .set_setting(SetStoreSettingInput {
            key: "courier_pathao".to_string(),
            value: json!({
                "is_enabled": true,
                "client_id": "route-test-client",
                "client_secret": "route-test-secret",
                "destination_mappings": "{\"bd|dhaka|dhaka\":{\"city_id\":1,\"zone_id\":2}}"
            }),
            category: "couriers".to_string(),
            is_secret: Some(true),
        })
        .await
        .expect("test courier credentials");

    let router = admin_order_routes_with_social_flows_and_settings(
        order_service.clone(),
        fulfillment_service.clone(),
        payment_service,
        Arc::new(stheno_flows_order::SocialDraftWorkflow::new(
            order_service.clone(),
        )),
        settings_service,
        notification_service,
    )
    .layer(axum::Extension(JwtClaims {
        sub: "usr_social_route_staff".to_string(),
        actor_id: Some("usr_social_route_staff".to_string()),
        actor_type: Some("user".to_string()),
        token_version: 0,
        exp: i64::MAX,
        iat: 0,
        jti: "social-courier-dispatch-route".to_string(),
    }));

    let make_draft = |include_address: bool| {
        let shipping_address = include_address.then(|| CreateAddressInput {
            first_name: Some("Route".to_string()),
            last_name: Some("Buyer".to_string()),
            address_1: Some("1 Test Road".to_string()),
            city: Some("Dhaka".to_string()),
            country_code: Some("BD".to_string()),
            province: Some("Dhaka".to_string()),
            phone: Some("+8801700000000".to_string()),
            ..Default::default()
        });
        order_service.create_order(CreateOrderInput {
            sales_channel_id: Some("sc_social_route_test".to_string()),
            currency_code: "bdt".to_string(),
            shipping_address,
            items: Some(vec![CreateLineItemInput {
                title: "Dispatch test item".to_string(),
                quantity: 1,
                unit_price: Decimal::ONE,
                requires_shipping: Some(true),
                ..Default::default()
            }]),
            shipping_methods: Some(vec![CreateShippingMethodInput {
                name: "Pathao".to_string(),
                amount: Decimal::ZERO,
                shipping_option_id: Some(shipping_option.id.clone()),
                ..Default::default()
            }]),
            is_draft_order: Some(true),
            metadata: Some(json!({"source": stheno_flows_order::SOCIAL_SOURCE})),
            ..Default::default()
        })
    };
    let dispatch = |app: &axum::Router, order_id: &str| {
        app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/draft-orders/{order_id}/convert-to-order"))
                .body(Body::empty())
                .expect("conversion request"),
        )
    };

    let successful_draft = make_draft(true).await.expect("successful social draft");
    let success = dispatch(&router, &successful_draft.id)
        .await
        .expect("dispatch route responds");
    let success_status = success.status();
    let success_body = axum::body::to_bytes(success.into_body(), usize::MAX)
        .await
        .expect("read dispatch response");
    let success_response = String::from_utf8_lossy(&success_body);
    assert_eq!(success_status, StatusCode::OK, "{success_response}");
    let converted = order_service
        .get_order(&successful_draft.id)
        .await
        .expect("converted order loads");
    assert!(!converted.is_draft_order);
    let replay = dispatch(&router, &successful_draft.id)
        .await
        .expect("dispatch replay responds");
    assert_eq!(replay.status(), StatusCode::OK);
    assert_eq!(courier.calls.load(Ordering::SeqCst), 1);
    let fulfillment = fulfillment_service
        .lookup_by_reference(&format!("social-order:{}", successful_draft.id))
        .await
        .expect("fulfillment lookup")
        .expect("one courier fulfillment persisted");
    assert_eq!(fulfillment.labels[0].tracking_number, "PATHAO-ROUTE-TRACK");

    let invalid_draft = make_draft(false)
        .await
        .expect("draft missing delivery address");
    let preflight = dispatch(&router, &invalid_draft.id)
        .await
        .expect("preflight route responds");
    assert_eq!(preflight.status(), StatusCode::BAD_REQUEST);
    assert!(
        order_service
            .get_order(&invalid_draft.id)
            .await
            .expect("preflight failure preserves draft")
            .is_draft_order
    );
    assert_eq!(courier.calls.load(Ordering::SeqCst), 1);

    courier.fail_ambiguously.store(true, Ordering::SeqCst);
    let ambiguous_draft = make_draft(true).await.expect("ambiguous social draft");
    let ambiguous = dispatch(&router, &ambiguous_draft.id)
        .await
        .expect("ambiguous route responds");
    assert_eq!(ambiguous.status(), StatusCode::BAD_REQUEST);
    assert!(
        !order_service
            .get_order(&ambiguous_draft.id)
            .await
            .expect("confirmed order survives provider ambiguity")
            .is_draft_order
    );
    let blocked_replay = dispatch(&router, &ambiguous_draft.id)
        .await
        .expect("unsettled replay responds");
    assert_eq!(blocked_replay.status(), StatusCode::BAD_REQUEST);
    assert_eq!(courier.calls.load(Ordering::SeqCst), 2);

    db.drop().await;
    match previous_key {
        Some(key) => std::env::set_var("STHENO_CREDENTIAL_KEY", key),
        None => std::env::remove_var("STHENO_CREDENTIAL_KEY"),
    }
}
