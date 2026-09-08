//! Admin Onboarding Engine (ROUTE-02, ADM-041).
//!
//! Provides genuine setup progression, step completion checks against real domain tables,
//! and resumable merchant checklist state for the Vendryx dashboard.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_fulfillment::FulfillmentService;
use stheno_product::ProductService;
use stheno_settings::dto::SetStoreSettingInput;
use stheno_settings::SettingsService;
use stheno_store::StoreService;

#[derive(Clone)]
pub struct AdminOnboardingContext {
    pub store_service: Arc<StoreService>,
    pub product_service: Arc<ProductService>,
    pub fulfillment_service: Arc<FulfillmentService>,
    pub settings_service: Arc<SettingsService>,
}

fn failed(status: StatusCode, code: &'static str, message: String) -> Response {
    (status, Json(ErrorResponse::new(code, &message))).into_response()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnboardingStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnboardingStatusResponse {
    pub is_completed: bool,
    pub dismissed: bool,
    pub completion_percentage: u32,
    pub completed_steps_count: usize,
    pub total_steps_count: usize,
    pub steps: Vec<OnboardingStep>,
}

/// `GET /admin/onboarding`
pub async fn get_onboarding_status(
    State(ctx): State<AdminOnboardingContext>,
) -> Result<impl IntoResponse, Response> {
    // 1. Check Store Profile: store name is not empty and not default
    let store_configured = if let Ok(store) = ctx.store_service.get_default_store().await {
        !store.name.trim().is_empty() && store.name != "My Store"
    } else {
        false
    };

    // 2. Check Products: at least 1 product created
    let products_configured = if let Ok(products) = ctx.product_service.list_products().await {
        !products.is_empty()
    } else {
        false
    };

    // 3. Check Couriers & Shipping: shipping options configured OR courier settings exist
    let shipping_options_exist = if let Ok((options, _)) = ctx
        .fulfillment_service
        .list_shipping_options(Default::default())
        .await
    {
        !options.is_empty()
    } else {
        false
    };
    let courier_settings_exist = ctx
        .settings_service
        .list_settings_by_category("couriers")
        .await
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let shipping_configured = shipping_options_exist || courier_settings_exist;

    // 4. Check Payments: settings exist in "payment" category (e.g. COD or providers)
    let payment_configured = ctx
        .settings_service
        .list_settings_by_category("payment")
        .await
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    // 5. Check Branding: store branding setting exists
    let branding_configured = ctx
        .settings_service
        .get_setting("store_branding")
        .await
        .is_ok();

    // 6. Check if merchant previously dismissed onboarding
    let dismissed = if let Ok(setting) = ctx
        .settings_service
        .get_setting("onboarding_dismissed")
        .await
    {
        setting.value.as_bool().unwrap_or(false)
    } else {
        false
    };

    let steps = vec![
        OnboardingStep {
            id: "store".to_string(),
            title: "Store Profile".to_string(),
            description: "Set your legal store name and default currency".to_string(),
            completed: store_configured,
            href: "/settings/store".to_string(),
        },
        OnboardingStep {
            id: "products".to_string(),
            title: "Add Products".to_string(),
            description: "Create catalog items with pricing, inventory variants, and images"
                .to_string(),
            completed: products_configured,
            href: "/products/create".to_string(),
        },
        OnboardingStep {
            id: "shipping".to_string(),
            title: "Couriers & Delivery Zones".to_string(),
            description: "Connect Pathao, Steadfast, RedX or setup manual shipping rates"
                .to_string(),
            completed: shipping_configured,
            href: "/settings/couriers".to_string(),
        },
        OnboardingStep {
            id: "payment".to_string(),
            title: "Payment Methods".to_string(),
            description: "Enable Cash on Delivery (COD), bKash, or card gateway".to_string(),
            completed: payment_configured,
            href: "/settings/payment-providers".to_string(),
        },
        OnboardingStep {
            id: "branding".to_string(),
            title: "Storefront Branding".to_string(),
            description: "Upload your store logo and customize brand colors".to_string(),
            completed: branding_configured,
            href: "/settings/branding".to_string(),
        },
    ];

    let total_steps_count = steps.len();
    let completed_steps_count = steps.iter().filter(|s| s.completed).count();
    let completion_percentage = if total_steps_count > 0 {
        ((completed_steps_count as f32 / total_steps_count as f32) * 100.0).round() as u32
    } else {
        0
    };
    let is_completed = (completed_steps_count == total_steps_count) || dismissed;

    let res = OnboardingStatusResponse {
        is_completed,
        dismissed,
        completion_percentage,
        completed_steps_count,
        total_steps_count,
        steps,
    };

    Ok((
        StatusCode::OK,
        Json(json!({ "onboarding": res, "data": res })),
    ))
}

/// `POST /admin/onboarding/dismiss`
pub async fn dismiss_onboarding(
    State(ctx): State<AdminOnboardingContext>,
) -> Result<impl IntoResponse, Response> {
    ctx.settings_service
        .set_setting(SetStoreSettingInput {
            key: "onboarding_dismissed".to_string(),
            value: json!(true),
            category: "onboarding".to_string(),
            is_secret: Some(false),
        })
        .await
        .map_err(|e| {
            failed(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(json!({ "dismissed": true }))))
}

/// `POST /admin/onboarding/reset`
pub async fn reset_onboarding(
    State(ctx): State<AdminOnboardingContext>,
) -> Result<impl IntoResponse, Response> {
    ctx.settings_service
        .set_setting(SetStoreSettingInput {
            key: "onboarding_dismissed".to_string(),
            value: json!(false),
            category: "onboarding".to_string(),
            is_secret: Some(false),
        })
        .await
        .map_err(|e| {
            failed(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(json!({ "dismissed": false }))))
}

/// Mounts admin onboarding routes.
pub fn admin_onboarding_routes(
    store_service: Arc<StoreService>,
    product_service: Arc<ProductService>,
    fulfillment_service: Arc<FulfillmentService>,
    settings_service: Arc<SettingsService>,
) -> Router {
    let ctx = AdminOnboardingContext {
        store_service,
        product_service,
        fulfillment_service,
        settings_service,
    };
    Router::new()
        .route("/admin/onboarding", get(get_onboarding_status))
        .route("/admin/onboarding/dismiss", post(dismiss_onboarding))
        .route("/admin/onboarding/reset", post(reset_onboarding))
        .with_state(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onboarding_status_response_serialization() {
        let step = OnboardingStep {
            id: "store".to_string(),
            title: "Store Profile".to_string(),
            description: "Configure name".to_string(),
            completed: true,
            href: "/settings/store".to_string(),
        };
        let res = OnboardingStatusResponse {
            is_completed: false,
            dismissed: false,
            completion_percentage: 20,
            completed_steps_count: 1,
            total_steps_count: 5,
            steps: vec![step],
        };

        let serialized = serde_json::to_string(&res).expect("serialization succeeds");
        let deserialized: OnboardingStatusResponse =
            serde_json::from_str(&serialized).expect("deserialization succeeds");
        assert_eq!(res, deserialized);
    }
}
