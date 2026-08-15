//! User & Invite workflows (WORKFLOWS §2, TASKS P7-21).

use std::sync::Arc;
use stheno_types::Result;
use stheno_user::*;

pub struct UserWorkflows {
    user_service: Arc<UserService>,
}

impl UserWorkflows {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }

    // 1. create_user_workflow
    pub async fn create_user(&self, input: CreateUserInput) -> Result<UserDTO> {
        self.user_service.create_user(input).await
    }

    // 2. update_user_workflow
    pub async fn update_user(&self, id: &str, input: UpdateUserInput) -> Result<UserDTO> {
        self.user_service.update_user(id, input).await
    }

    // 3. delete_user_workflow
    pub async fn delete_user(&self, id: &str) -> Result<UserDTO> {
        self.user_service.delete_user(id).await
    }

    // 4. get_user_workflow
    pub async fn get_user(&self, id: &str) -> Result<UserDTO> {
        self.user_service.get_user(id).await
    }

    // 5. list_users_workflow
    pub async fn list_users(&self) -> Result<Vec<UserDTO>> {
        self.user_service.list_users().await
    }

    // 6. create_invite_workflow
    pub async fn create_invite(&self, input: CreateInviteInput) -> Result<InviteDTO> {
        self.user_service.create_invite(input).await
    }

    // 7. accept_invite_workflow
    pub async fn accept_invite(&self, input: AcceptInviteInput) -> Result<UserDTO> {
        self.user_service.accept_invite(input).await
    }

    // 8. resend_invite_workflow
    pub async fn resend_invite(&self, invite_id: &str) -> Result<InviteDTO> {
        self.user_service.resend_invite(invite_id).await
    }

    // 9. revoke_invite_workflow
    pub async fn revoke_invite(&self, invite_id: &str) -> Result<InviteDTO> {
        self.user_service.revoke_invite(invite_id).await
    }
}
