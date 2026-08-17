//! API Key Service managing publishable and secret keys, scoping and validation (P7-16, P14-05).

use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Id, Result};

use crate::dto::{
    ApiKeyDTO, CreateApiKeyInput, CreatedApiKeyResponse, UpdateApiKeyInput, ValidateKeyResult,
};
use crate::models::ApiKey;

#[derive(Clone)]
pub struct ApiKeyService {
    keys: PgCrudRepository<ApiKey>,
}

impl ApiKeyService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            keys: PgCrudRepository::new(pool),
        }
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

        let created = self.keys.create(None, api_key).await?;
        let dto = self.to_dto(&created);

        Ok(CreatedApiKeyResponse {
            api_key: dto,
            token: plain_token,
        })
    }

    pub async fn validate_key(&self, plain_token: &str) -> Result<ValidateKeyResult> {
        let token_hash = self.hash_token(plain_token);

        let filter = FilterSpec::new()
            .with_predicate("token_hash", FilterOp::Eq(token_hash))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.keys.list(None, &filter).await?;

        if let Some(mut key) = list.into_iter().next() {
            if key.revoked_at.is_some() {
                return Ok(ValidateKeyResult {
                    is_valid: false,
                    key: Some(self.to_dto(&key)),
                    sales_channel_ids: Vec::new(),
                    permissions: Vec::new(),
                });
            }

            key.last_used_at = Some(Utc::now());
            key.updated_at = Utc::now();
            let updated = self.keys.update(None, &key.id.to_string(), key).await?;
            let dto = self.to_dto(&updated);

            Ok(ValidateKeyResult {
                is_valid: true,
                key: Some(dto),
                sales_channel_ids: updated.sales_channel_ids.clone(),
                permissions: updated.permissions.clone(),
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
        let mut key = self.keys.retrieve(None, id).await?;

        key.revoked_at = Some(Utc::now());
        key.revoked_by = revoked_by.map(|s| s.to_string());
        key.updated_at = Utc::now();

        let updated = self.keys.update(None, &key.id.to_string(), key).await?;
        Ok(self.to_dto(&updated))
    }

    pub async fn update_key(&self, id: &str, input: UpdateApiKeyInput) -> Result<ApiKeyDTO> {
        let mut key = self.keys.retrieve(None, id).await?;

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

        let updated = self.keys.update(None, &key.id.to_string(), key).await?;
        Ok(self.to_dto(&updated))
    }

    pub async fn get_api_key(&self, id: &str) -> Result<ApiKeyDTO> {
        let key = self.keys.retrieve(None, id).await?;
        Ok(self.to_dto(&key))
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.keys.list(None, &filter).await?;
        Ok(list.iter().map(|k| self.to_dto(k)).collect())
    }

    pub async fn delete_key(&self, id: &str) -> Result<ApiKeyDTO> {
        let key = self.keys.retrieve(None, id).await?;
        let dto = self.to_dto(&key);
        self.keys.soft_delete(None, &key.id.to_string()).await?;
        Ok(dto)
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
