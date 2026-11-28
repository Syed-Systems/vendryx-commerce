//! Admin User and Invite API handlers (P7-15, P14-04).

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use stheno_http::HttpError;
use stheno_user::dto::{CreateInviteInput, CreateUserInput, UpdateUserInput};
use stheno_user::UserService;

#[derive(Clone)]
pub struct AdminUserContext {
    pub user_service: Arc<UserService>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// GET /admin/users/me
pub async fn get_me(
    State(ctx): State<AdminUserContext>,
    quota: Option<Extension<crate::AdminQuotaContext>>,
) -> Result<impl IntoResponse, HttpError> {
    let users = ctx.user_service.list_users().await?;
    let me = if let Some(first) = users.into_iter().next() {
        first
    } else if let Some(Extension(quota)) = quota {
        let saga = stheno_flows::build_create_seat_quota_saga(
            ctx.user_service.clone(),
            quota.counter.clone(),
            quota.limit(stheno_api_core::QuotaKey::MaxStaffSeats),
        );
        let output = saga
            .execute(
                &format!("quota_user_bootstrap_{}", ulid::Ulid::new()),
                "create-staff-seat",
                json!({
                    "email": "admin@stheno.store",
                    "first_name": "System",
                    "last_name": "Admin",
                    "role": "admin",
                }),
            )
            .await?;
        let id = output["user_id"].as_str().ok_or_else(|| {
            HttpError(stheno_types::Error::Database(
                "Seat workflow returned no user ID".to_string(),
            ))
        })?;
        ctx.user_service.get_user(id).await?
    } else {
        ctx.user_service
            .create_user(CreateUserInput {
                email: "admin@stheno.store".to_string(),
                first_name: Some("System".to_string()),
                last_name: Some("Admin".to_string()),
                avatar_url: None,
                role: Some("admin".to_string()),
                metadata: None,
            })
            .await?
    };

    Ok((StatusCode::OK, Json(json!({ "user": me }))))
}

/// GET /admin/users
pub async fn list_users(
    State(ctx): State<AdminUserContext>,
) -> Result<impl IntoResponse, HttpError> {
    let users = ctx.user_service.list_users().await?;
    let count = users.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "users": users,
            "count": count,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

/// GET /admin/users/{id}
pub async fn get_user(
    State(ctx): State<AdminUserContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let user = ctx.user_service.get_user(&id).await?;
    Ok((StatusCode::OK, Json(json!({ "user": user }))))
}

/// POST /admin/users
pub async fn create_user(
    State(ctx): State<AdminUserContext>,
    quota: Option<Extension<crate::AdminQuotaContext>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let user = if let Some(Extension(quota)) = quota {
        let saga = stheno_flows::build_create_seat_quota_saga(
            ctx.user_service.clone(),
            quota.counter.clone(),
            quota.limit(stheno_api_core::QuotaKey::MaxStaffSeats),
        );
        let output = saga
            .execute(
                &format!("quota_user_{}", ulid::Ulid::new()),
                "create-staff-seat",
                json!({
                    "email": payload.email,
                    "first_name": payload.first_name,
                    "last_name": payload.last_name,
                    "avatar_url": payload.avatar_url,
                    "role": payload.role,
                    "metadata": payload.metadata,
                }),
            )
            .await?;
        let id = output["user_id"].as_str().ok_or_else(|| {
            HttpError(stheno_types::Error::Database(
                "Seat workflow returned no user ID".to_string(),
            ))
        })?;
        ctx.user_service.get_user(id).await?
    } else {
        ctx.user_service
            .create_user(CreateUserInput {
                email: payload.email,
                first_name: payload.first_name,
                last_name: payload.last_name,
                avatar_url: payload.avatar_url,
                role: payload.role,
                metadata: payload.metadata,
            })
            .await?
    };

    Ok((StatusCode::OK, Json(json!({ "user": user }))))
}

/// POST /admin/users/{id}
pub async fn update_user(
    State(ctx): State<AdminUserContext>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let user = ctx
        .user_service
        .update_user(
            &id,
            UpdateUserInput {
                email: payload.email,
                first_name: payload.first_name,
                last_name: payload.last_name,
                avatar_url: payload.avatar_url,
                role: payload.role,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok((StatusCode::OK, Json(json!({ "user": user }))))
}

/// DELETE /admin/users/{id}
pub async fn delete_user(
    State(ctx): State<AdminUserContext>,
    quota: Option<Extension<crate::AdminQuotaContext>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    if let Some(Extension(quota)) = quota {
        ctx.user_service
            .delete_user_with_quota(&id, &quota.counter)
            .await?;
    } else {
        let _ = ctx.user_service.delete_user(&id).await?;
    }
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "user",
            "deleted": true
        })),
    ))
}

/// GET /admin/invites
pub async fn list_invites(
    State(ctx): State<AdminUserContext>,
) -> Result<impl IntoResponse, HttpError> {
    let invites = ctx.user_service.list_invites().await?;
    let count = invites.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "invites": invites,
            "data": invites,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

/// POST /admin/invites
pub async fn create_invite(
    State(ctx): State<AdminUserContext>,
    Json(payload): Json<CreateInviteRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let invite = ctx
        .user_service
        .create_invite(CreateInviteInput {
            email: payload.email,
            role: payload.role,
            metadata: payload.metadata,
        })
        .await?;

    Ok((StatusCode::OK, Json(json!({ "invite": invite }))))
}

/// POST /admin/invites/{id}/resend
pub async fn resend_invite(
    State(ctx): State<AdminUserContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let invite = ctx.user_service.resend_invite(&id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "invite": invite, "data": invite })),
    ))
}

/// DELETE /admin/invites/{id}
pub async fn revoke_invite(
    State(ctx): State<AdminUserContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let invite = ctx.user_service.revoke_invite(&id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "invite", "deleted": true, "invite": invite })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct AcceptInviteRequest {
    pub token: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// POST /admin/invites/accept
pub async fn accept_invite(
    State(ctx): State<AdminUserContext>,
    quota: Option<Extension<crate::AdminQuotaContext>>,
    Json(payload): Json<AcceptInviteRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let user = if let Some(Extension(quota)) = quota {
        let saga = stheno_flows::build_accept_invite_quota_saga(
            ctx.user_service.clone(),
            quota.counter.clone(),
            quota.limit(stheno_api_core::QuotaKey::MaxStaffSeats),
        );
        let output = saga
            .execute(
                &format!("quota_invite_{}", ulid::Ulid::new()),
                "accept-staff-invite",
                json!({ "token": payload.token, "first_name": payload.first_name, "last_name": payload.last_name }),
            )
            .await?;
        let id = output["user_id"].as_str().ok_or_else(|| {
            HttpError(stheno_types::Error::Database(
                "Invite workflow returned no user ID".to_string(),
            ))
        })?;
        ctx.user_service.get_user(id).await?
    } else {
        ctx.user_service
            .accept_invite(stheno_user::dto::AcceptInviteInput {
                token: payload.token,
                first_name: payload.first_name,
                last_name: payload.last_name,
            })
            .await?
    };
    Ok((StatusCode::OK, Json(json!({ "user": user, "data": user }))))
}

/// GET /admin/invites/{id}
pub async fn get_invite(
    State(ctx): State<AdminUserContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let invites = ctx.user_service.list_invites().await?;
    let found = invites.into_iter().find(|i| i.id == id);
    match found {
        Some(invite) => Ok((
            StatusCode::OK,
            Json(json!({ "invite": invite, "data": invite })),
        )),
        None => Err(stheno_types::Error::NotFound {
            entity: "invite",
            id,
        }
        .into()),
    }
}
