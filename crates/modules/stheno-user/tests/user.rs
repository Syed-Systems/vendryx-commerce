//! Test P7-15: User management and team invite acceptance.

use stheno_user::{
    AcceptInviteInput, CreateInviteInput, CreateUserInput, UpdateUserInput, UserService,
};

#[tokio::test]
async fn test_user_and_invite_lifecycle() {
    let service = UserService::new();

    // 1. Create User
    let admin = service
        .create_user(CreateUserInput {
            email: "owner@stheno.store".to_string(),
            first_name: Some("Store".to_string()),
            last_name: Some("Owner".to_string()),
            avatar_url: None,
            role: Some("admin".to_string()),
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(admin.email, "owner@stheno.store");
    assert_eq!(admin.role, "admin");

    // 2. Create Invite for new team member
    let invite = service
        .create_invite(CreateInviteInput {
            email: "new_manager@stheno.store".to_string(),
            role: Some("manager".to_string()),
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(invite.email, "new_manager@stheno.store");
    assert!(!invite.token.is_empty());
    assert!(!invite.is_expired);

    // 3. Accept invite using the token
    let new_user = service
        .accept_invite(AcceptInviteInput {
            token: invite.token,
            first_name: Some("New".to_string()),
            last_name: Some("Manager".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(new_user.email, "new_manager@stheno.store");
    assert_eq!(new_user.role, "manager");

    // 4. Update user
    let updated = service
        .update_user(
            &new_user.id,
            UpdateUserInput {
                email: None,
                first_name: Some("Senior".to_string()),
                last_name: None,
                avatar_url: Some("https://avatar.url/img.png".to_string()),
                role: None,
                metadata: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.first_name, Some("Senior".to_string()));
    assert_eq!(
        updated.avatar_url,
        Some("https://avatar.url/img.png".to_string())
    );

    // 5. List users
    let users = service.list_users().await.unwrap();
    assert_eq!(users.len(), 2);
}
