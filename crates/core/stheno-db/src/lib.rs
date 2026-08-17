//! `stheno-db`: Database pool, connection management, migrations, and filtering.

pub mod filter;
pub mod migrations;
pub mod pg_repository;
pub mod pool;
pub mod quota;
pub mod service;
pub mod sql;
#[cfg(feature = "testing")]
pub mod testing;
pub mod transaction;

pub use diesel;
pub use diesel_async;
pub use filter::{FilterOp, FilterSpec, OrderBy, OrderDirection};
pub use pg_repository::PgCrudRepository;
pub use pool::{create_pool, DbConnection, DbPool};
pub use quota::{QuotaCounter, UsageRow};
pub use service::{CrudEntity, CrudRepository, InMemoryCrudStore};
pub use sql::SqlFilter;
pub use stheno_types::{Error, Result};
pub use transaction::{
    with_savepoint, with_serializable_retry, with_transaction, Conn, DEFAULT_MAX_ATTEMPTS,
};
