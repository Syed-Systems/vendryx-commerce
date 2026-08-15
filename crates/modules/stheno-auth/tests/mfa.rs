//! Test P7-09: MFA TOTP factor enrollment, recovery codes, and verification.

use stheno_auth::{AuthService, CreateIdentityInput};

#[tokio::test]
async fn test_mfa_totp_enrollment_and_recovery_codes() {
    let service = AuthService::new(None);

    let identity = service
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "mfa_user@stheno.store".to_string(),
            password: Some("SecureMfaPass123!".to_string()),
            actor_id: Some("user_mfa_1".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .unwrap();

    // 1. Enroll TOTP factor
    let enrollment = service
        .enroll_totp(&identity.id, "Stheno", "mfa_user@stheno.store")
        .await
        .unwrap();

    assert!(!enrollment.factor_id.is_empty());
    assert!(!enrollment.secret.is_empty());
    assert!(enrollment.uri.starts_with("otpauth://totp/"));
    assert_eq!(enrollment.recovery_codes.len(), 8);

    // 2. Verify with valid recovery code
    let first_recovery_code = &enrollment.recovery_codes[0];
    let is_verified = service
        .verify_mfa(&enrollment.factor_id, first_recovery_code)
        .await
        .unwrap();
    assert!(is_verified);

    // 3. Re-using the same single-use recovery code fails!
    let reuse_attempt = service
        .verify_mfa(&enrollment.factor_id, first_recovery_code)
        .await;
    assert!(reuse_attempt.is_err());

    // 4. Using the second recovery code succeeds!
    let second_recovery_code = &enrollment.recovery_codes[1];
    let second_verified = service
        .verify_mfa(&enrollment.factor_id, second_recovery_code)
        .await
        .unwrap();
    assert!(second_verified);
}
