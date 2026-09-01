//! User & Invite Service managing administrative users, team invites and token acceptance (P7-15, P14-04).

use chrono::{Duration, Utc};
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    AcceptInviteInput, CreateInviteInput, CreateUserInput, InviteDTO, UpdateUserInput, UserDTO,
};
use crate::models::{Invite, User};

#[derive(Clone)]
pub struct UserService {
    users: PgCrudRepository<User>,
    invites: PgCrudRepository<Invite>,
}

impl UserService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            users: PgCrudRepository::new(pool.clone()),
            invites: PgCrudRepository::new(pool),
        }
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

    pub async fn delete_user(&self, id: &str) -> Result<UserDTO> {
        let user = self.users.retrieve(None, id).await?;
        let dto = self.to_user_dto(&user);
        self.users.soft_delete(None, &user.id.to_string()).await?;
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
        let filter = FilterSpec::new()
            .with_predicate("token", FilterOp::Eq(input.token.clone()))
            .with_predicate("accepted_at", FilterOp::IsNull)
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.invites.list(None, &filter).await?;
        let mut invite = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "invite_token",
            id: input.token.clone(),
        })?;

        if invite.expires_at < Utc::now() {
            return Err(Error::Unauthorized("Invite token has expired".to_string()));
        }

        // Create user
        let user = self
            .create_user(CreateUserInput {
                email: invite.email.clone(),
                first_name: input.first_name,
                last_name: input.last_name,
                avatar_url: None,
                role: Some(invite.role.clone()),
                metadata: invite.metadata.clone(),
            })
            .await?;

        // Mark invite accepted
        invite.accepted_at = Some(Utc::now());
        invite.updated_at = Utc::now();
        self.invites
            .update(None, &invite.id.to_string(), invite)
            .await?;

        Ok(user)
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
