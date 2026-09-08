//! Medusa v2 wire-compatible Auth API handlers and route definitions (P7-22).

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use stheno_auth::{
    AuthenticateInput, CreateIdentityInput, PasswordResetConfirmInput, PasswordResetRequestInput,
    UpdatePasswordInput,
};
use stheno_flows::AuthWorkflows;
use stheno_http::HttpError;

#[derive(Clone)]
pub struct AuthApiContext {
    pub auth_flows: Arc<AuthWorkflows>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthPayload {
    pub email: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub old_password: Option<String>,
    pub new_password: Option<String>,
    pub actor_id: Option<String>,
    pub user_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponseWrapper {
    pub token: String,
}

/// POST /auth/{actor_type}/{provider_id}
pub async fn authenticate(
    State(ctx): State<AuthApiContext>,
    Path((actor_type, provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let email = payload.email.unwrap_or_default();
    let (_, tokens) = ctx
        .auth_flows
        .authenticate(AuthenticateInput {
            provider_id,
            entity_id: email,
            password: payload.password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "actor_type": actor_type,
        })),
    ))
}

/// POST /auth/{actor_type}/{provider_id}/register
pub async fn register(
    State(ctx): State<AuthApiContext>,
    Path((actor_type, provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let email = payload.email.unwrap_or_default();
    let identity = ctx
        .auth_flows
        .create_identity(CreateIdentityInput {
            provider_id: provider_id.clone(),
            entity_id: email.clone(),
            password: payload.password.clone(),
            actor_id: payload.actor_id,
            actor_type: Some(actor_type.clone()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: payload.user_metadata,
        })
        .await?;

    let (_, tokens) = ctx
        .auth_flows
        .authenticate(AuthenticateInput {
            provider_id,
            entity_id: email,
            password: payload.password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "identity": identity,
            "token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "actor_type": actor_type,
        })),
    ))
}

/// POST /auth/token/refresh
pub async fn refresh_token(
    State(ctx): State<AuthApiContext>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let raw_refresh = payload
        .refresh_token
        .ok_or_else(|| stheno_types::Error::Invalid {
            code: "missing_refresh_token",
            message: "Missing 'refresh_token' parameter".to_string(),
            field: Some("refresh_token".to_string()),
        })?;

    let tokens = ctx.auth_flows.refresh_tokens(&raw_refresh).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
        })),
    ))
}

/// POST /auth/{actor_type}/{provider_id}/reset-password
pub async fn reset_password(
    State(ctx): State<AuthApiContext>,
    Path((_actor_type, _provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let email = payload.email.unwrap_or_default();
    let token_opt = ctx
        .auth_flows
        .request_password_reset(PasswordResetRequestInput { entity_id: email })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "reset_token": token_opt,
        })),
    ))
}

/// POST /auth/{actor_type}/{provider_id}/confirm-reset-password
pub async fn confirm_reset_password(
    State(ctx): State<AuthApiContext>,
    Path((_actor_type, _provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let token = payload.token.ok_or_else(|| stheno_types::Error::Invalid {
        code: "missing_reset_token",
        message: "Missing 'token' in request".to_string(),
        field: Some("token".to_string()),
    })?;

    let new_password = payload
        .new_password
        .ok_or_else(|| stheno_types::Error::Invalid {
            code: "missing_new_password",
            message: "Missing 'new_password' in request".to_string(),
            field: Some("new_password".to_string()),
        })?;

    let updated_identity = ctx
        .auth_flows
        .confirm_password_reset(PasswordResetConfirmInput {
            token,
            new_password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "identity": updated_identity,
        })),
    ))
}

/// GET /auth/{actor_type}/providers
/// GET /auth/{actor_type}/providers
///
/// Reads `auth.auth_provider` rather than naming emailpass in the source. The dashboard
/// renders its login form from this list, so a hardcoded answer meant a merchant who
/// enabled Google or GitHub was shown a screen that never offered it.
/// Mounted twice: at `/auth/{actor_type}/providers` and at the static
/// `/auth/user/providers`. The static one carries no path parameter, so a bare
/// `Path<String>` extractor fails there with "Wrong number of path arguments" — a 500 on
/// the endpoint the login screen calls before anything else, which is a blank sign-in
/// form. `Option<Path<_>>` covers both, defaulting to the staff surface the static route
/// names.
pub async fn list_providers(
    State(ctx): State<AuthApiContext>,
    actor_type: Option<Path<String>>,
) -> Result<impl IntoResponse, HttpError> {
    let actor_type = actor_type
        .map(|Path(a)| a)
        .unwrap_or_else(|| "user".to_string());
    let providers = ctx.auth_flows.list_providers().await?;
    let rendered: Vec<serde_json::Value> = providers
        .into_iter()
        .filter(|p| p.is_enabled)
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "actor_type": actor_type,
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(json!({ "providers": rendered }))))
}

/// POST /auth/{actor_type}/{provider_id}/callback
pub async fn auth_callback(
    State(ctx): State<AuthApiContext>,
    Path((actor_type, provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let email = payload.email.unwrap_or_default();
    let (_, tokens) = ctx
        .auth_flows
        .authenticate(AuthenticateInput {
            provider_id,
            entity_id: email,
            password: payload.password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "actor_type": actor_type,
        })),
    ))
}












/// POST /auth/user/{auth_provider}/update
/// POST /auth/user/{auth_provider}/update
///
/// Changes the caller's password. It previously discarded the body and answered
/// `{"success": true}`, so a staff user who rotated a password they believed compromised
/// kept the old one — and was told otherwise.
///
/// The identity comes from the bearer token, never the body: taking `identity_id` from an
/// attacker-controlled payload would let any authenticated caller set anyone's password.
/// `update_password` verifies the old password before it writes.
pub async fn update_user_auth(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
    Path(_auth_provider): Path<String>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let identity_id = identity_from_token(&ctx, &headers).await?;

    let (Some(old_password), Some(new_password)) = (payload.old_password, payload.new_password)
    else {
        return Err(HttpError(stheno_types::Error::Invalid {
            code: "missing_password",
            message: "old_password and new_password are both required".to_string(),
            field: Some("new_password".to_string()),
        }));
    };

    ctx.auth_flows
        .update_password(UpdatePasswordInput {
            identity_id,
            old_password,
            new_password,
        })
        .await?;

    Ok((StatusCode::OK, Json(json!({ "success": true }))))
}

/// POST /auth/{auth_provider}/user
/// POST /auth/{auth_provider}/user
///
/// The actor-implied form of `authenticate`: the provider is in the path and the actor is
/// staff. It answered `{"success": true}` for any body — credentials discarded, no token
/// issued, and a caller with the wrong password told they had signed in.
pub async fn auth_provider_user(
    State(ctx): State<AuthApiContext>,
    Path(auth_provider): Path<String>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let (_, tokens) = ctx
        .auth_flows
        .authenticate(AuthenticateInput {
            provider_id: auth_provider,
            entity_id: payload.email.unwrap_or_default(),
            password: payload.password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "actor_type": "user",
        })),
    ))
}

// ── Staff shorthand routes ───────────────────────────────────────────────────
//
// `/auth/user/{auth_provider}/…` is the one-parameter shorthand the dashboard uses for the
// staff surface; `/auth/{actor_type}/{provider_id}/…` is Medusa's general two-parameter
// form. Both were bound to the same two-parameter handlers, and axum rejects an arity
// mismatch with a 500 *before* the handler runs — so staff login, registration, OAuth
// callback and password reset all failed on the shorthand while the general form worked.
//
// These supply the implied `actor_type` and delegate. Extractors are ordinary tuple
// structs, so there is nothing to refactor: the real handler is called directly.

pub async fn authenticate_user(
    State(ctx): State<AuthApiContext>,
    Path(provider_id): Path<String>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    authenticate(
        State(ctx),
        Path(("user".to_string(), provider_id)),
        Json(payload),
    )
    .await
}

pub async fn register_user(
    State(ctx): State<AuthApiContext>,
    Path(provider_id): Path<String>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    register(
        State(ctx),
        Path(("user".to_string(), provider_id)),
        Json(payload),
    )
    .await
}

pub async fn auth_callback_user(
    State(ctx): State<AuthApiContext>,
    Path(provider_id): Path<String>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    auth_callback(
        State(ctx),
        Path(("user".to_string(), provider_id)),
        Json(payload),
    )
    .await
}

pub async fn reset_password_user(
    State(ctx): State<AuthApiContext>,
    Path(provider_id): Path<String>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    reset_password(
        State(ctx),
        Path(("user".to_string(), provider_id)),
        Json(payload),
    )
    .await
}

/// Mounts all Medusa v2 compatible Auth API routes.
pub fn auth_routes(ctx: AuthApiContext) -> Router {
    Router::new()
        .route("/auth/{actor_type}/providers", get(list_providers))
        .route("/auth/{actor_type}/{provider_id}", post(authenticate))
        .route("/auth/{actor_type}/{provider_id}/register", post(register))
        .route("/auth/{actor_type}/{provider_id}/callback", post(auth_callback))
        .route("/auth/token/refresh", post(refresh_token))
        .route(
            "/auth/session",
            get(get_session).post(create_session).delete(end_session),
        )
        .route("/auth/mfa/generate", post(generate_mfa))
        .route("/auth/mfa/verify", post(verify_mfa))
        .route("/auth/mfa/factors", get(list_mfa_factors).post(create_mfa_factor))
        .route(
            "/auth/mfa/factors/{id}",
            axum::routing::delete(delete_mfa_factor),
        )
        .route("/auth/mfa/factors/{id}/verify", post(verify_mfa_factor))
        .route("/auth/mfa/challenges/{id}/verify", post(verify_mfa_challenge))
        .route("/auth/mfa/recovery-codes", post(generate_recovery_codes))
        .route("/auth/user/providers", get(list_providers))
        .route("/auth/user/{auth_provider}", post(authenticate_user))
        .route("/auth/user/{auth_provider}/callback", post(auth_callback_user))
        .route("/auth/user/{auth_provider}/register", post(register_user))
        .route("/auth/user/{auth_provider}/reset-password", post(reset_password_user))
        .route("/auth/user/{auth_provider}/update", post(update_user_auth))
        .route("/auth/{auth_provider}/user", post(auth_provider_user))
        .route(
            "/auth/{actor_type}/{provider_id}/reset-password",
            post(reset_password),
        )
        .route(
            "/auth/{actor_type}/{provider_id}/confirm-reset-password",
            post(confirm_reset_password),
        )
        .with_state(ctx)
}

// ── Multi-factor authentication ──────────────────────────────────────────────
//
// Every handler below resolves the caller from the bearer token and never takes an
// identity from the request. The previous versions took nothing at all: enrolment returned
// the RFC test-vector secret `JBSWY3DPEHPK3PXP` — the same one for every user, and public
// knowledge — recovery codes were three fixed strings, and the three verify endpoints
// discarded the submitted code and answered `verified: true`.
//
// `AuthService` has had a correct TOTP implementation the whole time: single-use recovery
// codes checked first, then a time-window TOTP check, `Unauthorized` on failure. These
// call it.

/// The verified claims behind the bearer token, or the 401 the client expects.
async fn claims_from_token(
    ctx: &AuthApiContext,
    headers: &HeaderMap,
) -> Result<stheno_auth::JwtClaims, HttpError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            HttpError(stheno_types::Error::Unauthorized(
                "Missing Authorization bearer token".to_string(),
            ))
        })?;
    Ok(ctx.auth_flows.verify_jwt(token).await?)
}

/// The identity behind the bearer token, or the 401 the client expects.
async fn identity_from_token(
    ctx: &AuthApiContext,
    headers: &HeaderMap,
) -> Result<String, HttpError> {
    Ok(claims_from_token(ctx, headers).await?.sub)
}

#[derive(Debug, Deserialize)]
pub struct MfaEnrollRequest {
    pub issuer: Option<String>,
    pub account_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    /// Present on `/auth/mfa/verify`, where the factor is not in the path.
    pub factor_id: Option<String>,
    pub code: String,
}

/// POST /auth/mfa/generate and POST /auth/mfa/factors
///
/// Returns the secret and the recovery codes exactly once. Only a hash of each recovery
/// code is stored, and the secret is not readable back through any route.
pub async fn generate_mfa(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
    body: Option<Json<MfaEnrollRequest>>,
) -> Result<impl IntoResponse, HttpError> {
    let identity_id = identity_from_token(&ctx, &headers).await?;
    let Json(body) = body.unwrap_or(Json(MfaEnrollRequest {
        issuer: None,
        account_name: None,
    }));

    let enrolment = ctx
        .auth_flows
        .enroll_totp(
            &identity_id,
            body.issuer.as_deref().unwrap_or("Stheno"),
            body.account_name.as_deref().unwrap_or(&identity_id),
        )
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "factor": {
                "id": enrolment.factor_id,
                "type": "totp",
                "secret": enrolment.secret,
                "uri": enrolment.uri,
            },
            "factor_id": enrolment.factor_id,
            "secret": enrolment.secret,
            "qr_code_uri": enrolment.uri,
            "uri": enrolment.uri,
            "recovery_codes": enrolment.recovery_codes,
        })),
    ))
}

/// POST /auth/mfa/factors
pub async fn create_mfa_factor(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
    body: Option<Json<MfaEnrollRequest>>,
) -> Result<impl IntoResponse, HttpError> {
    let response = generate_mfa(State(ctx), headers, body).await?;
    Ok((StatusCode::CREATED, response.into_response()))
}

/// GET /auth/mfa/factors
pub async fn list_mfa_factors(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let identity_id = identity_from_token(&ctx, &headers).await?;
    let factors = ctx.auth_flows.list_mfa_factors(&identity_id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "factors": factors, "data": factors, "count": factors.len() })),
    ))
}

/// DELETE /auth/mfa/factors/{id}
pub async fn delete_mfa_factor(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let identity_id = identity_from_token(&ctx, &headers).await?;
    ctx.auth_flows.delete_mfa_factor(&identity_id, &id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "mfa_factor", "deleted": true })),
    ))
}

/// POST /auth/mfa/verify
///
/// A wrong code is a 401 from the service, not `verified: false` with a 200 — the caller
/// must not be able to read failure as anything other than refusal.
pub async fn verify_mfa(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
    Json(body): Json<MfaVerifyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let identity_id = identity_from_token(&ctx, &headers).await?;

    let factor_id = match body.factor_id {
        Some(id) => id,
        None => ctx
            .auth_flows
            .list_mfa_factors(&identity_id)
            .await?
            .into_iter()
            .next()
            .map(|f| f.id)
            .ok_or_else(|| {
                HttpError(stheno_types::Error::NotFound {
                    entity: "mfa_factor",
                    id: identity_id.clone(),
                })
            })?,
    };

    // Ownership, before the code is checked: verifying against another identity's factor
    // must not be possible even with a correct code.
    let owned = ctx
        .auth_flows
        .list_mfa_factors(&identity_id)
        .await?
        .into_iter()
        .any(|f| f.id == factor_id);
    if !owned {
        return Err(HttpError(stheno_types::Error::NotFound {
            entity: "mfa_factor",
            id: factor_id,
        }));
    }

    let verified = ctx.auth_flows.verify_mfa(&factor_id, &body.code).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "verified": verified, "factor_id": factor_id })),
    ))
}

/// POST /auth/mfa/factors/{id}/verify
pub async fn verify_mfa_factor(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<MfaVerifyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let identity_id = identity_from_token(&ctx, &headers).await?;
    let owned = ctx
        .auth_flows
        .list_mfa_factors(&identity_id)
        .await?
        .into_iter()
        .any(|f| f.id == id);
    if !owned {
        return Err(HttpError(stheno_types::Error::NotFound {
            entity: "mfa_factor",
            id,
        }));
    }

    let verified = ctx.auth_flows.verify_mfa(&id, &body.code).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "factor": { "id": id, "verified": verified } })),
    ))
}

/// POST /auth/mfa/challenges/{id}/verify
///
/// This engine issues no separate challenge record — the factor is the thing verified — so
/// the path id is the factor id and this is the same check as above under Medusa's name.
pub async fn verify_mfa_challenge(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<MfaVerifyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let identity_id = identity_from_token(&ctx, &headers).await?;
    let owned = ctx
        .auth_flows
        .list_mfa_factors(&identity_id)
        .await?
        .into_iter()
        .any(|f| f.id == id);
    if !owned {
        return Err(HttpError(stheno_types::Error::NotFound {
            entity: "mfa_factor",
            id,
        }));
    }

    let verified = ctx.auth_flows.verify_mfa(&id, &body.code).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "challenge": { "id": id, "verified": verified } })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct RecoveryCodesRequest {
    pub factor_id: Option<String>,
}

/// POST /auth/mfa/recovery-codes
///
/// Replaces the whole set rather than appending: regeneration exists because the previous
/// codes may have leaked, and leaving those valid would defeat the point.
pub async fn generate_recovery_codes(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
    body: Option<Json<RecoveryCodesRequest>>,
) -> Result<impl IntoResponse, HttpError> {
    let identity_id = identity_from_token(&ctx, &headers).await?;
    let requested = body.and_then(|Json(b)| b.factor_id);

    let factor_id = match requested {
        Some(id) => id,
        None => ctx
            .auth_flows
            .list_mfa_factors(&identity_id)
            .await?
            .into_iter()
            .next()
            .map(|f| f.id)
            .ok_or_else(|| {
                HttpError(stheno_types::Error::NotFound {
                    entity: "mfa_factor",
                    id: identity_id.clone(),
                })
            })?,
    };

    let codes = ctx
        .auth_flows
        .regenerate_recovery_codes(&identity_id, &factor_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({ "recovery_codes": codes, "factor_id": factor_id })),
    ))
}

// ── Session ──────────────────────────────────────────────────────────────────
//
// This engine authenticates with JWTs and keeps no server-side session store, so these
// three report on the token the caller presented rather than inventing a session id. The
// previous versions answered `{"authenticated": true}` to anyone, including callers with
// no token at all — a status endpoint that always says "yes" is worse than none, because
// a client is entitled to trust it.

/// GET /auth/session
///
/// 401 when the token is missing or invalid. The dashboard distinguishes "signed out"
/// from "signed in", and a 200 with `authenticated: false` would be read as the latter by
/// anything checking the status code.
pub async fn get_session(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let claims = claims_from_token(&ctx, &headers).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "authenticated": true,
            "user": {
                "id": claims.actor_id,
                "identity_id": claims.sub,
                "actor_type": claims.actor_type,
            },
        })),
    ))
}

/// POST /auth/session
///
/// Medusa exchanges a bearer token for a cookie session here. There is no session store to
/// write to, so this confirms the token and returns the identity it belongs to — the same
/// answer as GET, which is the honest one for a stateless scheme.
pub async fn create_session(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let claims = claims_from_token(&ctx, &headers).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "authenticated": true,
            "user": {
                "id": claims.actor_id,
                "identity_id": claims.sub,
                "actor_type": claims.actor_type,
            },
        })),
    ))
}

/// DELETE /auth/session
///
/// A real sign-out: bumps the identity's `token_version`, which `verify_jwt` checks, so
/// every token already issued stops working — on every device, not just this browser.
pub async fn end_session(
    State(ctx): State<AuthApiContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let claims = claims_from_token(&ctx, &headers).await?;
    ctx.auth_flows.invalidate_tokens(&claims.sub).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true }))))
}
