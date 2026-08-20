//! Plan definitions: reading and rewriting what a tier means.
//!
//! `config/tiers.toml` ships the defaults. A `platform.tier_definition` row replaces one
//! tier's definition wholesale, which is how a plan gets retuned on a hosted deployment
//! where the TOML lives inside a container image.
//!
//! Every key is validated **here**, before it is stored. ENTITLEMENTS.md 5b makes an
//! unknown key abort startup; a write API can do better than that, because rejecting the
//! request means the bad key never reaches the table at all. A stored typo would make
//! every subsequent request for the affected merchants fall back to the least-privileged
//! entitlements — indistinguishable from an outage, and not obviously caused by a typo
//! made three days earlier.

use std::collections::BTreeMap;
use std::str::FromStr;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use stheno_api_core::{
    Entitlements, EntitlementsError, Feature, QuotaKey, Tier, TierConfig, WindowKey,
};
use stheno_platform::TierDefinitionDTO;

use crate::{authorised, bad_request, deny, effective_tier_config, PlatformApiContext};

/// One plan as the control plane sees it.
#[derive(Debug, Serialize)]
pub struct TierView {
    pub tier: &'static str,
    /// `default` while the shipped TOML still governs, `stored` once it has been edited.
    /// The operator needs to know which plans they have taken ownership of, because a
    /// stored plan stops tracking the defaults a future release ships.
    pub source: &'static str,
    pub features: Vec<&'static str>,
    /// Every known key, `0` meaning unlimited — not just the keys the plan happens to
    /// set. A quota that is absent from the response cannot be given a value in the UI.
    pub quotas: BTreeMap<&'static str, u64>,
    pub windows: BTreeMap<&'static str, u64>,
}

/// Renders a resolved entitlement the same way a plan is rendered, so the operator can
/// compare a merchant's effective state against the plan it came from without translating
/// between two shapes.
pub(crate) fn entitlements_json(ent: &Entitlements) -> serde_json::Value {
    let quotas: BTreeMap<&str, u64> = QuotaKey::ALL
        .iter()
        .map(|k| (k.as_str(), ent.quota(*k).unwrap_or(0)))
        .collect();
    let windows: BTreeMap<&str, u64> = WindowKey::ALL
        .iter()
        .map(|k| (k.as_str(), ent.window(*k).unwrap_or(0)))
        .collect();

    json!({
        "tier": ent.tier().as_str(),
        "features": ent.features().map(|f| f.as_str()).collect::<Vec<_>>(),
        "quotas": quotas,
        "windows": windows,
    })
}

fn view(config: &TierConfig, tier: Tier, stored: bool) -> Result<TierView, EntitlementsError> {
    // Resolving rather than reading the definition straight off means the view goes
    // through the same code the request path does — including the `0`-is-unlimited
    // normalisation — so the UI can never show a plan the router would not mount.
    let ent = config.resolve(tier, &BTreeMap::new())?;
    Ok(TierView {
        tier: tier.as_str(),
        source: if stored { "stored" } else { "default" },
        features: ent.features().map(|f| f.as_str()).collect(),
        quotas: QuotaKey::ALL
            .iter()
            .map(|k| (k.as_str(), ent.quota(*k).unwrap_or(0)))
            .collect(),
        windows: WindowKey::ALL
            .iter()
            .map(|k| (k.as_str(), ent.window(*k).unwrap_or(0)))
            .collect(),
    })
}

/// `GET /platform/tiers` — every plan, plus the full key universe the UI renders from.
pub async fn get_tiers(State(ctx): State<PlatformApiContext>, headers: HeaderMap) -> Response {
    if !authorised(&ctx, &headers) {
        return deny();
    }

    let stored: Vec<String> = ctx
        .platform
        .list_tier_definitions()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.tier)
        .collect();

    let config = effective_tier_config(&ctx).await;

    let mut tiers = Vec::new();
    for tier in Tier::ALL {
        match view(&config, tier, stored.iter().any(|t| t == tier.as_str())) {
            Ok(v) => tiers.push(v),
            // A tier the config cannot resolve is a broken deployment, not an empty plan.
            // Reporting it as a 500 is right: rendering it as "no features" would invite
            // an operator to "fix" it by saving, quietly making the breakage permanent.
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "type": "unexpected_state",
                        "message": format!("tier {} does not resolve: {e}", tier.as_str())
                    })),
                )
                    .into_response()
            }
        }
    }

    Json(json!({
        "tiers": tiers,
        // The catalog is what makes the editor complete: it is every gate the engine
        // knows how to apply, so a feature added in Rust shows up as a new checkbox
        // without a matching edit in TypeScript.
        "catalog": {
            "features": Feature::ALL.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
            "quotas": QuotaKey::ALL.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
            "windows": WindowKey::ALL.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
            "statuses": ["active", "grace", "cancelled"],
        }
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct TierDefinitionBody {
    pub features: Vec<String>,
    /// `{"max_products": 100}`. `0` means unlimited; a key that is left out is also
    /// unlimited, because the resolver treats absent and `0` identically.
    #[serde(default)]
    pub quotas: BTreeMap<String, u64>,
    #[serde(default)]
    pub windows: BTreeMap<String, u64>,
}

/// `PUT /platform/tiers/{tier}` — redefine a plan.
///
/// Replacement, not merge: the body is the whole plan. The UI sends every checkbox and
/// every limit, so a partial body would mean an unticked box was indistinguishable from
/// a field the client's version did not know about.
pub async fn put_tier(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(tier): Path<String>,
    Json(body): Json<TierDefinitionBody>,
) -> Response {
    if !authorised(&ctx, &headers) {
        return deny();
    }

    let Ok(parsed) = tier.parse::<Tier>() else {
        return bad_request(format!("unknown tier {tier:?}"));
    };

    let features = match body
        .features
        .iter()
        .map(|f| f.parse::<Feature>())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(f) => f,
        Err(e) => return bad_request(e),
    };
    if let Err(e) = parse_keys::<QuotaKey>(&body.quotas) {
        return bad_request(e);
    }
    if let Err(e) = parse_keys::<WindowKey>(&body.windows) {
        return bad_request(e);
    }

    let row = TierDefinitionDTO {
        tier: parsed.as_str().to_string(),
        features: json!(features.iter().map(|f| f.as_str()).collect::<Vec<_>>()),
        quotas: json!(body.quotas),
        windows: json!(body.windows),
        updated_at: Utc::now(),
    };

    if let Err(e) = ctx.platform.upsert_tier_definition(row).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "type": "unexpected_state", "message": e.to_string() })),
        )
            .into_response();
    }

    // Re-read rather than echo the body back: this is the same path the request layer
    // takes, so a definition that stored but does not resolve is visible immediately
    // instead of on the next merchant's request.
    let config = effective_tier_config(&ctx).await;
    match view(&config, parsed, true) {
        Ok(v) => Json(json!({ "tier": v })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "type": "unexpected_state", "message": e.to_string() })),
        )
            .into_response(),
    }
}

/// `DELETE /platform/tiers/{tier}` — drop the override, restoring the shipped default.
pub async fn delete_tier(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(tier): Path<String>,
) -> Response {
    if !authorised(&ctx, &headers) {
        return deny();
    }

    let Ok(parsed) = tier.parse::<Tier>() else {
        return bad_request(format!("unknown tier {tier:?}"));
    };

    // A tier with no stored row is already at its default, so deleting twice succeeds.
    let _ = ctx.platform.reset_tier_definition(parsed.as_str()).await;

    let config = effective_tier_config(&ctx).await;
    match view(&config, parsed, false) {
        Ok(v) => Json(json!({ "tier": v })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "type": "unexpected_state", "message": e.to_string() })),
        )
            .into_response(),
    }
}

fn parse_keys<K: FromStr<Err = EntitlementsError>>(
    limits: &BTreeMap<String, u64>,
) -> Result<(), EntitlementsError> {
    limits.keys().try_for_each(|k| k.parse::<K>().map(|_| ()))
}

/// Validates a per-tenant `{"max_products": 250}` override blob.
pub(crate) fn validate_limits<K: FromStr<Err = EntitlementsError>>(
    value: &serde_json::Value,
) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "overrides must be a JSON object".to_string())?;
    for (key, raw) in object {
        key.parse::<K>().map_err(|e| e.to_string())?;
        if raw.as_u64().is_none() {
            return Err(format!(
                "limit for `{key}` must be a non-negative integer, got {raw}"
            ));
        }
    }
    Ok(())
}

/// Validates a per-tenant `{"payment_gateway": true}` override blob.
pub(crate) fn validate_feature_flags(value: &serde_json::Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "feature overrides must be a JSON object".to_string())?;
    for (key, raw) in object {
        key.parse::<Feature>().map_err(|e| e.to_string())?;
        if !raw.is_boolean() {
            return Err(format!("feature override `{key}` must be true or false"));
        }
    }
    Ok(())
}
