//! Space Control Plane integration and inbound webhook reception.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use std::sync::Arc;
use stheno_connector_space::SpaceClient;

#[derive(Clone)]
pub struct SpaceIntegrationState {
    pub client: Arc<SpaceClient>,
}

pub fn space_integration_routes(state: SpaceIntegrationState) -> Router {
    Router::new()
        .route(
            "/api/integrations/space/webhooks",
            post(receive_space_webhook),
        )
        .with_state(state)
}

async fn receive_space_webhook(
    State(state): State<SpaceIntegrationState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let signature_header = headers
        .get("x-space-signature")
        .or_else(|| headers.get("X-Space-Signature"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    let timestamp_header = headers
        .get("x-space-timestamp")
        .or_else(|| headers.get("X-Space-Timestamp"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| {
            signature_header
                .split(',')
                .find(|p| p.trim().starts_with("t="))
                .and_then(|p| p.trim().strip_prefix("t="))
                .and_then(|t| t.parse::<u64>().ok())
                .unwrap_or(0)
        });

    let current_time = chrono::Utc::now().timestamp() as u64;

    let payload = state
        .client
        .verify_webhook(timestamp_header, signature_header, &body, current_time)
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "unauthorized",
                    "message": e.to_string()
                })),
            )
        })?;

    match payload.event.as_str() {
        "instance.suspend" => {
            tracing::warn!(
                store_id = ?payload.store_id,
                instance_ref = ?payload.instance_ref,
                "Received Space instance.suspend webhook - Store entering maintenance"
            );
            Ok(Json(serde_json::json!({
                "status": "acknowledged",
                "action": "instance.suspend",
                "operational_state": "maintenance"
            })))
        }
        "instance.reactivate" => {
            tracing::info!(
                store_id = ?payload.store_id,
                instance_ref = ?payload.instance_ref,
                "Received Space instance.reactivate webhook - Store restored to active"
            );
            Ok(Json(serde_json::json!({
                "status": "acknowledged",
                "action": "instance.reactivate",
                "operational_state": "active"
            })))
        }
        "entitlements.snapshot.published" => {
            tracing::info!(
                store_id = ?payload.store_id,
                instance_ref = ?payload.instance_ref,
                "Received Space entitlements.snapshot.published webhook"
            );
            Ok(Json(serde_json::json!({
                "status": "acknowledged",
                "action": "entitlements.snapshot.published"
            })))
        }
        other => {
            tracing::info!(event = other, "Received Space webhook event");
            Ok(Json(serde_json::json!({
                "status": "acknowledged",
                "event": other
            })))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use stheno_connector_space::{sign_space_webhook, SpaceConnectorConfig};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_space_webhook_lifecycle_handling() {
        let secret = "test_webhook_secret_key";
        let config = SpaceConnectorConfig {
            webhook_secret: secret.to_string(),
            tolerance_seconds: 300,
            ..Default::default()
        };
        let state = SpaceIntegrationState {
            client: Arc::new(SpaceClient::new(config)),
        };
        let app = space_integration_routes(state);

        let now = chrono::Utc::now().timestamp() as u64;
        let payload = serde_json::json!({
            "event": "instance.suspend",
            "store_id": "ten_default",
            "instance_ref": "prod_inst_vendryx_default"
        });
        let body_bytes = serde_json::to_vec(&payload).unwrap();
        let sig_header = sign_space_webhook(secret.as_bytes(), now, &body_bytes);

        // Valid suspend request
        let req = Request::builder()
            .method("POST")
            .uri("/api/integrations/space/webhooks")
            .header("Content-Type", "application/json")
            .header("X-Space-Signature", sig_header.clone())
            .header("X-Space-Timestamp", now.to_string())
            .body(Body::from(body_bytes.clone()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Invalid signature request
        let bad_req = Request::builder()
            .method("POST")
            .uri("/api/integrations/space/webhooks")
            .header("Content-Type", "application/json")
            .header("X-Space-Signature", "t=123,v1=badbadbad")
            .header("X-Space-Timestamp", now.to_string())
            .body(Body::from(body_bytes.clone()))
            .unwrap();

        let bad_resp = app.clone().oneshot(bad_req).await.unwrap();
        assert_eq!(bad_resp.status(), StatusCode::UNAUTHORIZED);
    }
}
