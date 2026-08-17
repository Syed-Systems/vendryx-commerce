//! Test P7-02: Argon2id hashing + scrypt verification for imports + transparent re-hash.

use stheno_auth::{AuthService, AuthenticateInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_argon2id_hashing_and_verification() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = AuthService::new(pool, None);

    let password = "SuperSecretPassword123!";
    let hash = service.hash_password(password).unwrap();
    assert!(hash.starts_with("$argon2id$"));

    let (valid, needs_rehash) = service.verify_password_internal(&hash, password).unwrap();
    assert!(valid);
    assert!(!needs_rehash);

    let (invalid, _) = service
        .verify_password_internal(&hash, "WrongPassword")
        .unwrap();
    assert!(!invalid);

    db.drop().await;
}

#[tokio::test]
async fn test_imported_scrypt_verification_and_transparent_rehash() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = AuthService::new(pool, None);

    // Legacy imported scrypt password hash format
    let salt = "saltysalt";
    let params = scrypt::Params::new(14, 8, 1, 32).unwrap();
    let mut out = [0u8; 32];
    scrypt::scrypt("password123".as_bytes(), salt.as_bytes(), &params, &mut out).unwrap();
    let scrypt_hash_str = format!("scrypt$ln=14,r=8,p=1${}${}", salt, hex::encode(out));

    let imported = service
        .import_scrypt_identity(
            "legacy_user@stheno.store",
            &scrypt_hash_str,
            Some("cust_123"),
            Some("customer"),
        )
        .await
        .unwrap();

    assert_eq!(imported.entity_id, "legacy_user@stheno.store");

    // Authenticate with the imported scrypt identity
    let (auth_id, tokens) = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "legacy_user@stheno.store".to_string(),
            password: Some("password123".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(auth_id.entity_id, "legacy_user@stheno.store");
    assert!(!tokens.access_token.is_empty());

    // Subsequent authentication works cleanly with the upgraded Argon2id hash
    let (reauth_id, _) = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "legacy_user@stheno.store".to_string(),
            password: Some("password123".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(reauth_id.id, auth_id.id);

    db.drop().await;
}
