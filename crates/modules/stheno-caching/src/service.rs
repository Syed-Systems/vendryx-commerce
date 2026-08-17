use std::future::Future;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use stheno_types::Result;

use crate::in_process::InMemoryCacheProvider;
use crate::provider::CacheProvider;

pub struct CacheService {
    primary_provider: Arc<dyn CacheProvider>,
    fallback_provider: Arc<InMemoryCacheProvider>,
}

impl CacheService {
    pub fn new(primary_provider: Arc<dyn CacheProvider>) -> Self {
        Self {
            primary_provider,
            fallback_provider: Arc::new(InMemoryCacheProvider::new()),
        }
    }

    pub async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if self.primary_provider.is_healthy().await {
            match self.primary_provider.get(key).await {
                Ok(val) => return Ok(val),
                Err(err) => {
                    tracing::warn!(
                        "Primary cache get failed ({}), falling back to in-process cache",
                        err
                    );
                }
            }
        }
        self.fallback_provider.get(key).await
    }

    pub async fn set_bytes(&self, key: &str, value: &[u8], ttl_seconds: Option<u64>) -> Result<()> {
        if self.primary_provider.is_healthy().await {
            if let Err(err) = self.primary_provider.set(key, value, ttl_seconds).await {
                tracing::warn!(
                    "Primary cache set failed ({}), writing to in-process fallback",
                    err
                );
            }
        }
        // Always write to fallback as well so fallback has warm data
        let _ = self.fallback_provider.set(key, value, ttl_seconds).await;
        // Degradation: never fail request when cache fails
        Ok(())
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        if let Some(bytes) = self.get_bytes(key).await? {
            let val: T =
                serde_json::from_slice(&bytes).map_err(|e| stheno_types::Error::Invalid {
                    code: "cache_deserialization_error",
                    message: e.to_string(),
                    field: Some(key.to_string()),
                })?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(value).map_err(|e| stheno_types::Error::Invalid {
            code: "cache_serialization_error",
            message: e.to_string(),
            field: Some(key.to_string()),
        })?;
        self.set_bytes(key, &bytes, ttl_seconds).await
    }

    pub async fn get_or_set_json<T, F, Fut>(
        &self,
        key: &str,
        ttl_seconds: Option<u64>,
        fetch: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        if let Some(cached) = self.get_json::<T>(key).await? {
            return Ok(cached);
        }

        let fresh = fetch().await?;
        let _ = self.set_json(key, &fresh, ttl_seconds).await;
        Ok(fresh)
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let _ = self.primary_provider.delete(key).await;
        let _ = self.fallback_provider.delete(key).await;
        Ok(())
    }

    pub async fn invalidate_prefix(&self, prefix: &str) -> Result<()> {
        let _ = self.primary_provider.delete_by_prefix(prefix).await;
        let _ = self.fallback_provider.delete_by_prefix(prefix).await;
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let _ = self.primary_provider.clear().await;
        let _ = self.fallback_provider.clear().await;
        Ok(())
    }
}
