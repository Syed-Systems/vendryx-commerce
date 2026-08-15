use async_trait::async_trait;
use stheno_types::Result;

#[async_trait]
pub trait CacheProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl_seconds: Option<u64>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn delete_by_prefix(&self, prefix: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
    async fn is_healthy(&self) -> bool {
        true
    }
}
