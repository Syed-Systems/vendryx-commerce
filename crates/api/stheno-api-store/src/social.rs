//! Store-side social checkout (PRODUCT.md §5.2–5.4).
//!
//! `SocialDraftWorkflow` has existed in `stheno-flows-order` with tests since the order
//! phase, and nothing exposed it. The storefront's social-checkout component was
//! correspondingly orphaned — no route rendered it, and had one, it would have had
//! nothing to call.
//!
//! This is the free tier's habit-forming path: a buyer messages the seller, a draft
//! lands in the Social queue, staff talk to them and convert it. Deliberately **no
//! notification is emitted** — the seller is already receiving the chat message, and a
//! second ping for the same order is the double-notification trap.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use stheno_flows_order::{
    SocialDraftInput, SocialDraftWorkflow, SocialMessageLine, generate_ref_code,
    render_social_message,
};
use stheno_order::CreateLineItemInput;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct SocialContext {
    pub social_flows: Arc<SocialDraftWorkflow>,
    /// Shown at the top of the buyer's message so they know who they are ordering from.
    pub store_name: String,
    /// Resolves the tenant's default sales channel.
    ///
    /// The draft flow rejects a missing `sales_channel_id` at creation on purpose —
    /// conversion reads it to resolve a reservation location, so a draft without one
    /// fails when staff accept it, which is unrecoverable. But a storefront has no reason
    /// to know channel ids, so the server fills it in rather than pushing the
    /// requirement onto the client.
    pub store_service: Arc<stheno_store::StoreService>,
}

#[derive(Debug, Deserialize)]
pub struct SocialDraftLine {
    pub variant_id: String,
    pub title: String,
    pub sku: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    pub quantity: i32,
    pub unit_price: String,
    pub line_total: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSocialDraftRequest {
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    pub currency_code: String,
    pub items: Vec<SocialDraftLine>,
    pub total: String,
    pub buyer_name: Option<String>,
    pub buyer_phone: Option<String>,
    /// Carried from `?ref=` on the storefront, so an influencer's traffic is attributable
    /// all the way to the order.
    pub source_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSocialDraftResponse {
    pub order_id: String,
    /// The code the buyer sends and staff match against. The only link between the chat
    /// and the queued draft.
    pub ref_code: String,
    /// Pre-rendered so every channel sends an identical message.
    pub message: String,
}

/// `POST /store/social-drafts` — queue a draft and return the message to send.
pub async fn create_social_draft(
    State(ctx): State<SocialContext>,
    Json(body): Json<CreateSocialDraftRequest>,
) -> Result<impl IntoResponse, Response> {
    if body.items.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                r#type: "invalid_data".to_string(),
                message: "A social draft needs at least one line item.".to_string(),
            }),
        )
            .into_response());
    }

    // Generated before the draft so the same code goes into the order metadata and the
    // buyer's message. Deriving it afterwards from the order id would produce something
    // staff cannot read out.
    let ref_code = generate_ref_code();

    let message = render_social_message(
        &ctx.store_name,
        &body
            .items
            .iter()
            .map(|line| SocialMessageLine {
                title: line.title.clone(),
                sku: line.sku.clone(),
                options: line.options.clone(),
                quantity: line.quantity,
                unit_price: line.unit_price.clone(),
                line_total: line.line_total.clone(),
            })
            .collect::<Vec<_>>(),
        &body.total,
        &ref_code,
    );

    let sales_channel_id = match body.sales_channel_id {
        Some(id) if !id.trim().is_empty() => Some(id),
        _ => ctx
            .store_service
            .get_default_store()
            .await
            .ok()
            .and_then(|store| store.default_sales_channel_id),
    };

    let draft = ctx
        .social_flows
        .create_social_draft(SocialDraftInput {
            sales_channel_id,
            region_id: body.region_id,
            currency_code: body.currency_code,
            items: body
                .items
                .iter()
                .map(|line| CreateLineItemInput {
                    // `Default` covers the pricing and tax flags: a social draft is
                    // priced when staff convert it, not at handoff, so inventing values
                    // here would be worse than leaving them at their defaults.
                    title: line.title.clone(),
                    variant_id: Some(line.variant_id.clone()),
                    quantity: line.quantity,
                    ..Default::default()
                })
                .collect(),
            buyer_name: body.buyer_name,
            buyer_phone: body.buyer_phone,
            delivery_address: None,
            shipping_method: None,
            payment_preference: None,
            source_event_id: None,
            // The ref travels in metadata, which is how the admin queue finds it.
            source_ref: Some(
                body.source_ref
                    .map(|r| format!("{ref_code}|{r}"))
                    .unwrap_or_else(|| ref_code.clone()),
            ),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    r#type: "invalid_data".to_string(),
                    message: e.to_string(),
                }),
            )
                .into_response()
        })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "social_draft": CreateSocialDraftResponse {
                order_id: draft.id.to_string(),
                ref_code,
                message,
            }
        })),
    ))
}

pub fn store_social_routes(ctx: SocialContext) -> axum::Router {
    axum::Router::new()
        .route(
            "/store/social-drafts",
            axum::routing::post(create_social_draft),
        )
        .with_state(ctx)
}
