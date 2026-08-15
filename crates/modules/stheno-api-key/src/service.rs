//! API Key Service managing publishable and secret keys, scoping and validation (P7-16).

use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    ApiKeyDTO, CreateApiKeyInput, CreatedApiKeyResponse, UpdateApiKeyInput, ValidateKeyResult,
};
use crate::models::ApiKey;

#[derive(Clone, Default)]
pub struct ApiKeyService {
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl ApiKeyService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_api_key(&self, input: CreateApiKeyInput) -> Result<CreatedApiKeyResponse> {
        let id = Id::new();
        let now = Utc::now();

        let prefix = if input.key_type == "secret" {
            "sk_"
        } else {
            "pk_"
        };
        let random_bytes: [u8; 24] = rand::thread_rng().gen();
        let random_hex = hex::encode(random_bytes);
        let plain_token = format!("{prefix}{random_hex}");

        let token_hash = self.hash_token(&plain_token);
        let redacted_key = format!("{prefix}...{}", &random_hex[random_hex.len() - 4..]);

        let sales_channel_ids = input.sales_channel_ids.unwrap_or_default();
        let permissions = input.permissions.unwrap_or_default();

        let api_key = ApiKey {
            id,
            key_type: input.key_type,
            title: input.title,
            redacted_key,
            token_hash,
            sales_channel_ids,
            permissions,
            created_by: input.created_by,
            revoked_by: None,
            revoked_at: None,
            last_used_at: None,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let dto = self.to_dto(&api_key);
        let mut map = self.keys.write().unwrap();
        map.insert(id.to_string(), api_key);

        Ok(CreatedApiKeyResponse {
            api_key: dto,
            token: plain_token,
        })
    }

    pub async fn validate_key(&self, plain_token: &str) -> Result<ValidateKeyResult> {
        let token_hash = self.hash_token(plain_token);

        let mut map = self.keys.write().unwrap();
        let found = map
            .values_mut()
            .find(|k| k.token_hash == token_hash && k.deleted_at.is_none());

        if let Some(key) = found {
            if key.revoked_at.is_some() {
                return Ok(ValidateKeyResult {
                    is_valid: false,
                    key: Some(self.to_dto(key)),
                    sales_channel_ids: Vec::new(),
                    permissions: Vec::new(),
                });
            }

            key.last_used_at = Some(Utc::now());
            let dto = self.to_dto(key);

            Ok(ValidateKeyResult {
                is_valid: true,
                key: Some(dto),
                sales_channel_ids: key.sales_channel_ids.clone(),
                permissions: key.permissions.clone(),
            })
        } else {
            Ok(ValidateKeyResult {
                is_valid: false,
                key: None,
                sales_channel_ids: Vec::new(),
                permissions: Vec::new(),
            })
        }
    }

    pub async fn revoke_key(&self, id: &str, revoked_by: Option<&str>) -> Result<ApiKeyDTO> {
        let mut map = self.keys.write().unwrap();
        let key = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: "api_key",
            id: id.to_string(),
        })?;

        key.revoked_at = Some(Utc::now());
        key.revoked_by = revoked_by.map(|s| s.to_string());
        key.updated_at = Utc::now();

        Ok(self.to_dto(key))
    }

    pub async fn update_key(&self, id: &str, input: UpdateApiKeyInput) -> Result<ApiKeyDTO> {
        let mut map = self.keys.write().unwrap();
        let key = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: "api_key",
            id: id.to_string(),
        })?;

        if let Some(title) = input.title {
            key.title = title;
        }
        if let Some(sc_ids) = input.sales_channel_ids {
            key.sales_channel_ids = sc_ids;
        }
        if let Some(perms) = input.permissions {
            key.permissions = perms;
        }
        if let Some(meta) = input.metadata {
            key.metadata = Some(meta);
        }
        key.updated_at = Utc::now();

        Ok(self.to_dto(key))
    }

    pub async fn get_api_key(&self, id: &str) -> Result<ApiKeyDTO> {
        let map = self.keys.read().unwrap();
        let key = map
            .get(id)
            .filter(|k| k.deleted_at.is_none())
            .ok_or_else(|| Error::NotFound {
                entity: "api_key",
                id: id.to_string(),
            })?;
        Ok(self.to_dto(key))
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyDTO>> {
        let map = self.keys.read().unwrap();
        Ok(map
            .values()
            .filter(|k| k.deleted_at.is_none())
            .map(|k| self.to_dto(k))
            .collect())
    }

    pub async fn delete_key(&self, id: &str) -> Result<ApiKeyDTO> {
        let mut map = self.keys.write().unwrap();
        let key = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: "api_key",
            id: id.to_string(),
        })?;

        key.deleted_at = Some(Utc::now());
        Ok(self.to_dto(key))
    }

    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn to_dto(&self, k: &ApiKey) -> ApiKeyDTO {
        let is_revoked = k.revoked_at.is_some();
        ApiKeyDTO {
            id: k.id.to_string(),
            key_type: k.key_type.clone(),
            title: k.title.clone(),
            redacted_key: k.redacted_key.clone(),
            sales_channel_ids: k.sales_channel_ids.clone(),
            permissions: k.permissions.clone(),
            is_revoked,
            revoked_at: k.revoked_at,
            last_used_at: k.last_used_at,
            metadata: k.metadata.clone(),
            created_at: k.created_at,
            updated_at: k.updated_at,
        }
    }
}
