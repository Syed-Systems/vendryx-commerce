//! User & Invite Service managing administrative users, team invites and token acceptance (P7-15, P14-04).

use chrono::{Duration, Utc};
use stheno_db::diesel::{
    result::OptionalExtension,
    sql_types::{BigInt, Text},
    QueryableByName,
};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{
    tenant, transaction, CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection,
    PgCrudRepository,
};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    AcceptInviteInput, CreateInviteInput, CreateUserInput, InviteDTO, UpdateUserInput, UserDTO,
};
use crate::models::{Invite, User};

#[derive(QueryableByName)]
struct InviteIdRow {
    #[diesel(sql_type = Text)]
    id: String,
}

#[derive(QueryableByName)]
struct UserCountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(Clone)]
pub struct UserService {
    pool: DbPool,
    users: PgCrudRepository<User>,
    invites: PgCrudRepository<Invite>,
    event_bus: Option<std::sync::Arc<dyn stheno_events::EventBus>>,
}

impl UserService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool: pool.clone(),
            users: PgCrudRepository::new(pool.clone()),
            invites: PgCrudRepository::new(pool),
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, bus: std::sync::Arc<dyn stheno_events::EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    // ==========================================
    // 1. User Management
    // ==========================================

    pub async fn create_user(&self, input: CreateUserInput) -> Result<UserDTO> {
        let filter = FilterSpec::new()
            .with_predicate("email", FilterOp::Eq(input.email.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.users.list(None, &filter).await?;
        if !existing.is_empty() {
            return Err(Error::Conflict {
                code: "user_email_exists",
                message: format!("User with email {} already exists", input.email),
            });
        }

        let id = Id::new();
        let now = Utc::now();

        let user = User {
            id,
            email: input.email,
            first_name: input.first_name,
            last_name: input.last_name,
            avatar_url: input.avatar_url,
            role: input.role.unwrap_or_else(|| "member".to_string()),
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.users.create(None, user).await?;
        Ok(self.to_user_dto(&created))
    }

    pub async fn update_user(&self, id: &str, input: UpdateUserInput) -> Result<UserDTO> {
        let mut user = self.users.retrieve(None, id).await?;

        if let Some(email) = input.email {
            user.email = email;
        }
        if let Some(fname) = input.first_name {
            user.first_name = Some(fname);
        }
        if let Some(lname) = input.last_name {
            user.last_name = Some(lname);
        }
        if let Some(avatar) = input.avatar_url {
            user.avatar_url = Some(avatar);
        }
        if let Some(role) = input.role {
            user.role = role;
        }
        if let Some(meta) = input.metadata {
            user.metadata = Some(meta);
        }
        user.updated_at = Utc::now();

        let updated = self.users.update(None, &user.id.to_string(), user).await?;
        Ok(self.to_user_dto(&updated))
    }

    pub async fn get_user(&self, id: &str) -> Result<UserDTO> {
        let user = self.users.retrieve(None, id).await?;
        Ok(self.to_user_dto(&user))
    }

    pub async fn list_users(&self) -> Result<Vec<UserDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.users.list(None, &filter).await?;
        Ok(list.iter().map(|u| self.to_user_dto(u)).collect())
    }

    /// Counts all live staff seats without applying the admin list page ceiling.
    pub async fn count_active_users(&self) -> Result<i64> {
        let pool = self.pool.clone();
        tenant::with_tenant_conn(&pool, |conn| {
            Box::pin(async move {
                stheno_db::diesel::sql_query(
                    "SELECT COUNT(*)::BIGINT AS count FROM \"user\".\"user\" WHERE deleted_at IS NULL",
                )
                .get_result::<UserCountRow>(conn)
                .await
                .map(|row| row.count)
                .map_err(|error| Error::Database(error.to_string()))
            })
        })
        .await
    }

    pub async fn delete_user(&self, id: &str) -> Result<UserDTO> {
        let user = self.users.retrieve(None, id).await?;
        let dto = self.to_user_dto(&user);
        let pool = self.pool.clone();
        let user_id = user.id.to_string();
        let delete_id = user_id.clone();
        let affected = tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                stheno_db::diesel::sql_query(
                    "UPDATE \"user\".\"user\" SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind::<Text, _>(&delete_id)
                .execute(conn)
                .await
                .map_err(|error| Error::Database(error.to_string()))
            })
        })
        .await?;
        if affected == 0 {
            return Err(Error::NotFound {
                entity: "user",
                id: user_id,
            });
        }
        Ok(dto)
    }

    /// Soft-deletes a user and releases its tenant quota in one transaction.
    pub async fn delete_user_with_quota(
        &self,
        id: &str,
        counter: &stheno_db::QuotaCounter,
    ) -> Result<UserDTO> {
        let user = self.users.retrieve(None, id).await?;
        let dto = self.to_user_dto(&user);
        let user_id = user.id.to_string();
        let pool = self.pool.clone();
        let counter = counter.clone();
        tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                let affected = stheno_db::diesel::sql_query(
                    "UPDATE \"user\".\"user\" SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind::<Text, _>(&user_id)
                .execute(conn)
                .await
                .map_err(|error| Error::Database(error.to_string()))?;
                if affected == 0 {
                    return Err(Error::NotFound {
                        entity: "user",
                        id: user_id,
                    });
                }
                counter.decrement(conn, "users").await?;
                Ok(())
            })
        })
        .await?;
        Ok(dto)
    }

    // ==========================================
    // 2. Invite Management
    // ==========================================

    /// Every outstanding invite.
    ///
    /// `/admin/invites` answered with a hardcoded empty array, so an invite created here
    /// was never listed and could not be resent or revoked from the screen that owns it.
    pub async fn list_invites(&self) -> Result<Vec<InviteDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        Ok(self
            .invites
            .list(None, &filter)
            .await?
            .iter()
            .map(|invite| self.to_invite_dto(invite))
            .collect())
    }

    pub async fn create_invite(&self, input: CreateInviteInput) -> Result<InviteDTO> {
        let filter = FilterSpec::new()
            .with_predicate("email", FilterOp::Eq(input.email.clone()))
            .with_predicate("accepted_at", FilterOp::IsNull)
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 10);

        let existing = self.invites.list(None, &filter).await?;
        if existing.iter().any(|i| i.expires_at > Utc::now()) {
            return Err(Error::Conflict {
                code: "invite_exists",
                message: format!("Active invite for email {} already exists", input.email),
            });
        }

        let id = Id::new();
        let now = Utc::now();
        let raw_token = Id::<Invite>::new().to_string();

        let invite = Invite {
            id,
            email: input.email,
            role: input.role.unwrap_or_else(|| "member".to_string()),
            token: raw_token,
            expires_at: now + Duration::days(7),
            accepted_at: None,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.invites.create(None, invite).await?;

        if let Some(bus) = &self.event_bus {
            let event = stheno_events::Event::new(
                "user.invite_created",
                serde_json::json!({
                    "invite_id": created.id.to_string(),
                    "email": created.email,
                    "role": created.role,
                    "token": created.token,
                    "expires_at": created.expires_at,
                }),
            );
            let _ = bus.emit(vec![event]).await;
        }

        Ok(self.to_invite_dto(&created))
    }

    pub async fn resend_invite(&self, invite_id: &str) -> Result<InviteDTO> {
        let mut invite = self.invites.retrieve(None, invite_id).await?;

        let now = Utc::now();
        invite.token = Id::<Invite>::new().to_string();
        invite.expires_at = now + Duration::days(7);
        invite.updated_at = now;

        let updated = self
            .invites
            .update(None, &invite.id.to_string(), invite)
            .await?;

        if let Some(bus) = &self.event_bus {
            let event = stheno_events::Event::new(
                "user.invite_created",
                serde_json::json!({
                    "invite_id": updated.id.to_string(),
                    "email": updated.email,
                    "role": updated.role,
                    "token": updated.token,
                    "expires_at": updated.expires_at,
                }),
            );
            let _ = bus.emit(vec![event]).await;
        }

        Ok(self.to_invite_dto(&updated))
    }

    pub async fn revoke_invite(&self, invite_id: &str) -> Result<InviteDTO> {
        let invite = self.invites.retrieve(None, invite_id).await?;
        let dto = self.to_invite_dto(&invite);
        self.invites
            .soft_delete(None, &invite.id.to_string())
            .await?;
        Ok(dto)
    }

    pub async fn accept_invite(&self, input: AcceptInviteInput) -> Result<UserDTO> {
        let pool = self.pool.clone();
        let users = self.users.clone();
        let invites = self.invites.clone();
        let token = input.token.clone();
        let user = tenant::with_tenant_conn(&pool, move |conn| {
            let users = users.clone();
            let invites = invites.clone();
            let input = input.clone();
            let token = token.clone();
            Box::pin(async move {
                transaction::with_transaction(conn, move |mut conn| {
                    let users = users.clone();
                    let invites = invites.clone();
                    let input = input.clone();
                    let token = token.clone();
                    Box::pin(async move {
                        let invite_id = stheno_db::diesel::sql_query(
                            "SELECT id FROM \"user\".invite \
                             WHERE token = $1 AND accepted_at IS NULL AND deleted_at IS NULL \
                             FOR UPDATE",
                        )
                        .bind::<Text, _>(&token)
                        .get_result::<InviteIdRow>(&mut conn)
                        .await
                        .optional()
                        .map_err(|error| Error::Database(error.to_string()))?
                        .ok_or_else(|| Error::NotFound {
                            entity: "invite_token",
                            id: token.clone(),
                        })?
                        .id;
                        let mut invite = invites.retrieve(Some(&mut conn), &invite_id).await?;
                        if invite.expires_at < Utc::now() {
                            return Err(Error::Unauthorized(
                                "Invite token has expired".to_string(),
                            ));
                        }
                        let existing = users
                            .list(
                                Some(&mut conn),
                                &FilterSpec::new()
                                    .with_predicate("email", FilterOp::Eq(invite.email.clone()))
                                    .with_predicate("deleted_at", FilterOp::IsNull)
                                    .with_pagination(0, 1),
                            )
                            .await?;
                        if !existing.is_empty() {
                            return Err(Error::Conflict {
                                code: "user_email_exists",
                                message: format!("User with email {} already exists", invite.email),
                            });
                        }

                        let now = Utc::now();
                        let user = users
                            .create(
                                Some(&mut conn),
                                User {
                                    id: Id::new(),
                                    email: invite.email.clone(),
                                    first_name: input.first_name,
                                    last_name: input.last_name,
                                    avatar_url: None,
                                    role: invite.role.clone(),
                                    metadata: invite.metadata.clone(),
                                    created_at: now,
                                    updated_at: now,
                                    deleted_at: None,
                                },
                            )
                            .await?;
                        invite.accepted_at = Some(now);
                        invite.updated_at = now;
                        invites
                            .update(Some(&mut conn), &invite.id.to_string(), invite)
                            .await?;
                        Ok(user)
                    })
                })
                .await
            })
        })
        .await?;
        Ok(self.to_user_dto(&user))
    }

    fn to_user_dto(&self, u: &User) -> UserDTO {
        UserDTO {
            id: u.id.to_string(),
            email: u.email.clone(),
            first_name: u.first_name.clone(),
            last_name: u.last_name.clone(),
            avatar_url: u.avatar_url.clone(),
            role: u.role.clone(),
            metadata: u.metadata.clone(),
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }

    fn to_invite_dto(&self, i: &Invite) -> InviteDTO {
        let is_expired = i.expires_at < Utc::now();
        InviteDTO {
            id: i.id.to_string(),
            email: i.email.clone(),
            role: i.role.clone(),
            token: i.token.clone(),
            expires_at: i.expires_at,
            accepted_at: i.accepted_at,
            is_expired,
            metadata: i.metadata.clone(),
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}
