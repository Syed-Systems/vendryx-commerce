//! `SthenoService` CRUD operations generation and traits (TRD §4.2).
//!
//! Provides standard CRUD operations:
//! `list`, `list_and_count`, `retrieve`, `create`, `create_many`, `update`, `upsert`,
//! `delete`, `soft_delete`, and `restore`.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Result};

use crate::filter::FilterSpec;
use crate::transaction::Conn;

pub trait CrudEntity: Clone + Send + Sync + 'static {
    fn id(&self) -> &str;
    fn is_deleted(&self) -> bool;
    fn set_deleted(&mut self, deleted: bool);
}

#[async_trait]
pub trait CrudRepository<T: CrudEntity>: Send + Sync {
    async fn retrieve(&self, conn: Option<&mut Conn<'_>>, id: &str) -> Result<T>;
    async fn list(&self, conn: Option<&mut Conn<'_>>, filter: &FilterSpec) -> Result<Vec<T>>;
    async fn list_and_count(
        &self,
        conn: Option<&mut Conn<'_>>,
        filter: &FilterSpec,
    ) -> Result<(Vec<T>, i64)>;
    async fn create(&self, conn: Option<&mut Conn<'_>>, item: T) -> Result<T>;
    async fn create_many(&self, conn: Option<&mut Conn<'_>>, items: Vec<T>) -> Result<Vec<T>>;
    async fn update(&self, conn: Option<&mut Conn<'_>>, id: &str, item: T) -> Result<T>;
    async fn upsert(&self, conn: Option<&mut Conn<'_>>, item: T) -> Result<T>;
    async fn delete(&self, conn: Option<&mut Conn<'_>>, id: &str) -> Result<()>;
    async fn soft_delete(&self, conn: Option<&mut Conn<'_>>, id: &str) -> Result<()>;
    async fn restore(&self, conn: Option<&mut Conn<'_>>, id: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct InMemoryCrudStore<T: CrudEntity> {
    entity_name: &'static str,
    items: Arc<RwLock<HashMap<String, T>>>,
}

impl<T: CrudEntity> InMemoryCrudStore<T> {
    pub fn new(entity_name: &'static str) -> Self {
        Self {
            entity_name,
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<T: CrudEntity> CrudRepository<T> for InMemoryCrudStore<T> {
    async fn retrieve(&self, _conn: Option<&mut Conn<'_>>, id: &str) -> Result<T> {
        let map = self
            .items
            .read()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        let item = map
            .get(id)
            .filter(|i| !i.is_deleted())
            .ok_or_else(|| Error::NotFound {
                entity: self.entity_name,
                id: id.to_string(),
            })?;
        Ok(item.clone())
    }

    async fn list(&self, _conn: Option<&mut Conn<'_>>, _filter: &FilterSpec) -> Result<Vec<T>> {
        let map = self
            .items
            .read()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        let results = map.values().filter(|i| !i.is_deleted()).cloned().collect();
        Ok(results)
    }

    async fn list_and_count(
        &self,
        _conn: Option<&mut Conn<'_>>,
        _filter: &FilterSpec,
    ) -> Result<(Vec<T>, i64)> {
        let map = self
            .items
            .read()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        let results: Vec<T> = map.values().filter(|i| !i.is_deleted()).cloned().collect();
        let total = results.len() as i64;
        Ok((results, total))
    }

    async fn create(&self, _conn: Option<&mut Conn<'_>>, item: T) -> Result<T> {
        let mut map = self
            .items
            .write()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        if map.contains_key(item.id()) {
            return Err(Error::Conflict {
                code: "entity_already_exists",
                message: format!("{} with id {} already exists", self.entity_name, item.id()),
            });
        }
        map.insert(item.id().to_string(), item.clone());
        Ok(item)
    }

    async fn create_many(&self, _conn: Option<&mut Conn<'_>>, items: Vec<T>) -> Result<Vec<T>> {
        let mut map = self
            .items
            .write()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        for item in &items {
            if map.contains_key(item.id()) {
                return Err(Error::Conflict {
                    code: "entity_already_exists",
                    message: format!("{} with id {} already exists", self.entity_name, item.id()),
                });
            }
        }
        for item in &items {
            map.insert(item.id().to_string(), item.clone());
        }
        Ok(items)
    }

    async fn update(&self, _conn: Option<&mut Conn<'_>>, id: &str, item: T) -> Result<T> {
        let mut map = self
            .items
            .write()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        if !map.contains_key(id) {
            return Err(Error::NotFound {
                entity: self.entity_name,
                id: id.to_string(),
            });
        }
        map.insert(id.to_string(), item.clone());
        Ok(item)
    }

    async fn upsert(&self, _conn: Option<&mut Conn<'_>>, item: T) -> Result<T> {
        let mut map = self
            .items
            .write()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        map.insert(item.id().to_string(), item.clone());
        Ok(item)
    }

    async fn delete(&self, _conn: Option<&mut Conn<'_>>, id: &str) -> Result<()> {
        let mut map = self
            .items
            .write()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        if map.remove(id).is_none() {
            return Err(Error::NotFound {
                entity: self.entity_name,
                id: id.to_string(),
            });
        }
        Ok(())
    }

    async fn soft_delete(&self, _conn: Option<&mut Conn<'_>>, id: &str) -> Result<()> {
        let mut map = self
            .items
            .write()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        let item = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: self.entity_name,
            id: id.to_string(),
        })?;
        item.set_deleted(true);
        Ok(())
    }

    async fn restore(&self, _conn: Option<&mut Conn<'_>>, id: &str) -> Result<()> {
        let mut map = self
            .items
            .write()
            .map_err(|_| Error::Database("Lock poisoned".into()))?;
        let item = map.get_mut(id).ok_or_else(|| Error::NotFound {
            entity: self.entity_name,
            id: id.to_string(),
        })?;
        item.set_deleted(false);
        Ok(())
    }
}

#[macro_export]
macro_rules! stheno_service_crud {
    ($service_name:ident, $entity:ident, $store_field:ident) => {
        impl $service_name {
            pub async fn retrieve(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                id: &str,
            ) -> $crate::Result<$entity> {
                $crate::service::CrudRepository::retrieve(&self.$store_field, conn, id).await
            }

            pub async fn list(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                filter: &$crate::filter::FilterSpec,
            ) -> $crate::Result<Vec<$entity>> {
                $crate::service::CrudRepository::list(&self.$store_field, conn, filter).await
            }

            pub async fn list_and_count(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                filter: &$crate::filter::FilterSpec,
            ) -> $crate::Result<(Vec<$entity>, i64)> {
                $crate::service::CrudRepository::list_and_count(&self.$store_field, conn, filter)
                    .await
            }

            pub async fn create(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                item: $entity,
            ) -> $crate::Result<$entity> {
                $crate::service::CrudRepository::create(&self.$store_field, conn, item).await
            }

            pub async fn create_many(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                items: Vec<$entity>,
            ) -> $crate::Result<Vec<$entity>> {
                $crate::service::CrudRepository::create_many(&self.$store_field, conn, items).await
            }

            pub async fn update(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                id: &str,
                item: $entity,
            ) -> $crate::Result<$entity> {
                $crate::service::CrudRepository::update(&self.$store_field, conn, id, item).await
            }

            pub async fn upsert(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                item: $entity,
            ) -> $crate::Result<$entity> {
                $crate::service::CrudRepository::upsert(&self.$store_field, conn, item).await
            }

            pub async fn delete(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                id: &str,
            ) -> $crate::Result<()> {
                $crate::service::CrudRepository::delete(&self.$store_field, conn, id).await
            }

            pub async fn soft_delete(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                id: &str,
            ) -> $crate::Result<()> {
                $crate::service::CrudRepository::soft_delete(&self.$store_field, conn, id).await
            }

            pub async fn restore(
                &self,
                conn: Option<&mut $crate::transaction::Conn<'_>>,
                id: &str,
            ) -> $crate::Result<()> {
                $crate::service::CrudRepository::restore(&self.$store_field, conn, id).await
            }
        }
    };
}
