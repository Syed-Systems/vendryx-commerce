//! User & Invite Service managing administrative users, team invites and token acceptance (P7-15).

use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    AcceptInviteInput, CreateInviteInput, CreateUserInput, InviteDTO, UpdateUserInput, UserDTO,
};
use crate::models::{Invite, User};

#[derive(Clone, Default)]
pub struct UserService {
    users: Arc<RwLock<HashMap<String, User>>>,
    invites: Arc<RwLock<HashMap<String, Invite>>>,
}

impl UserService {
    pub fn new() -> Self {
        Self::default()
    }

    // ==========================================
    // 1. User Management
    // ==========================================

    pub async fn create_user(&self, input: CreateUserInput) -> Result<UserDTO> {
        let mut map = self.users.write().unwrap();

        if map
            .values()
            .any(|u| u.email == input.email && u.deleted_at.is_none())
        {
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

        let dto = self.to_user_dto(&user);
        map.insert(id.to_string(), user);
        Ok(dto)
    }

    pub async fn update_user(&self, id: &str, input: UpdateUserInput) -> Result<UserDTO> {
        let mut map = self.users.write().unwrap();
        let user = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: "user",
            id: id.to_string(),
        })?;

        if user.deleted_at.is_some() {
            return Err(Error::NotFound {
                entity: "user",
                id: id.to_string(),
            });
        }

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

        Ok(self.to_user_dto(user))
    }

    pub async fn get_user(&self, id: &str) -> Result<UserDTO> {
        let map = self.users.read().unwrap();
        let user = map
            .get(id)
            .filter(|u| u.deleted_at.is_none())
            .ok_or_else(|| Error::NotFound {
                entity: "user",
                id: id.to_string(),
            })?;
        Ok(self.to_user_dto(user))
    }

    pub async fn list_users(&self) -> Result<Vec<UserDTO>> {
        let map = self.users.read().unwrap();
        Ok(map
            .values()
            .filter(|u| u.deleted_at.is_none())
            .map(|u| self.to_user_dto(u))
            .collect())
    }

    pub async fn delete_user(&self, id: &str) -> Result<UserDTO> {
        let mut map = self.users.write().unwrap();
        let user = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: "user",
            id: id.to_string(),
        })?;

        user.deleted_at = Some(Utc::now());
        Ok(self.to_user_dto(user))
    }

    // ==========================================
    // 2. Invite Management
    // ==========================================

    pub async fn create_invite(&self, input: CreateInviteInput) -> Result<InviteDTO> {
        let mut map = self.invites.write().unwrap();

        // Check active invite
        if map.values().any(|i| {
            i.email == input.email
                && i.accepted_at.is_none()
                && i.deleted_at.is_none()
                && i.expires_at > Utc::now()
        }) {
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

        let dto = self.to_invite_dto(&invite);
        map.insert(id.to_string(), invite);
        Ok(dto)
    }

    pub async fn resend_invite(&self, invite_id: &str) -> Result<InviteDTO> {
        let mut map = self.invites.write().unwrap();
        let invite = map.get_mut(invite_id).ok_or_else(|| Error::NotFound {
            entity: "invite",
            id: invite_id.to_string(),
        })?;

        let now = Utc::now();
        invite.token = Id::<Invite>::new().to_string();
        invite.expires_at = now + Duration::days(7);
        invite.updated_at = now;

        Ok(self.to_invite_dto(invite))
    }

    pub async fn revoke_invite(&self, invite_id: &str) -> Result<InviteDTO> {
        let mut map = self.invites.write().unwrap();
        let invite = map.get_mut(invite_id).ok_or_else(|| Error::NotFound {
            entity: "invite",
            id: invite_id.to_string(),
        })?;

        invite.deleted_at = Some(Utc::now());
        Ok(self.to_invite_dto(invite))
    }

    pub async fn accept_invite(&self, input: AcceptInviteInput) -> Result<UserDTO> {
        let (mut invite, invite_id) = {
            let map = self.invites.read().unwrap();
            let found = map
                .values()
                .find(|i| {
                    i.token == input.token && i.accepted_at.is_none() && i.deleted_at.is_none()
                })
                .cloned();

            let inv = found.ok_or_else(|| Error::NotFound {
                entity: "invite_token",
                id: input.token.clone(),
            })?;

            if inv.expires_at < Utc::now() {
                return Err(Error::Unauthorized("Invite token has expired".to_string()));
            }

            (inv.clone(), inv.id.to_string())
        };

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
        {
            let mut map = self.invites.write().unwrap();
            map.insert(invite_id, invite);
        }

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
