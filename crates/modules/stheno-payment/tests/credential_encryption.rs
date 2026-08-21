use serde_json::json;
use stheno_payment::dto::{
    decrypt_credential_map, encrypt_credential_map, CredentialMap, EncryptedCredential,
};

const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

fn install_key() {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
}

fn credential_map() -> CredentialMap {
    CredentialMap::from([
        ("api_key".to_string(), json!("sk_live_secret")),
        ("webhook_secret".to_string(), json!("whsec_secret")),
    ])
}

#[test]
fn encrypts_and_decrypts_credential_maps_from_env_key() {
    install_key();
    let encrypted = encrypt_credential_map("ten_a", "stripe", &credential_map()).expect("encrypt");

    assert_ne!(
        encrypted.ciphertext,
        serde_json::to_vec(&credential_map()).unwrap()
    );
    assert_eq!(encrypted.nonce.len(), 12);

    let decrypted =
        decrypt_credential_map("ten_a", "stripe", &encrypted).expect("decrypt with same AAD");
    assert_eq!(decrypted, credential_map());
}

#[test]
fn encryption_uses_a_fresh_nonce_per_write() {
    install_key();
    let first = encrypt_credential_map("ten_a", "stripe", &credential_map()).expect("first");
    let second = encrypt_credential_map("ten_a", "stripe", &credential_map()).expect("second");

    assert_ne!(first.nonce, second.nonce);
    assert_ne!(first.ciphertext, second.ciphertext);
}

#[test]
fn aad_binds_ciphertext_to_tenant_and_provider() {
    install_key();
    let encrypted = encrypt_credential_map("ten_a", "stripe", &credential_map()).expect("encrypt");

    assert!(decrypt_credential_map("ten_b", "stripe", &encrypted).is_err());
    assert!(decrypt_credential_map("ten_a", "offline", &encrypted).is_err());
}

#[test]
fn decrypt_errors_do_not_include_plaintext_secrets() {
    install_key();
    let mut encrypted =
        encrypt_credential_map("ten_a", "stripe", &credential_map()).expect("encrypt");
    encrypted.ciphertext[0] ^= 0x01;

    let err = decrypt_credential_map("ten_a", "stripe", &encrypted).expect_err("tampered");
    let rendered = format!("{err:?}");
    assert!(!rendered.contains("sk_live_secret"));
    assert!(!rendered.contains("whsec_secret"));
}

#[test]
fn invalid_nonce_is_rejected_without_plaintext_context() {
    install_key();
    let encrypted = EncryptedCredential {
        ciphertext: b"sk_live_secret".to_vec(),
        nonce: vec![0; 8],
    };

    let err = decrypt_credential_map("ten_a", "stripe", &encrypted).expect_err("bad nonce");
    let rendered = format!("{err:?}");
    assert!(!rendered.contains("sk_live_secret"));
}
