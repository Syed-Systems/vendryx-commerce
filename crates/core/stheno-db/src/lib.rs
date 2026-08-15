//! `stheno-db`: Database pool, connection management, migrations, and filtering.

pub mod filter;
pub mod pool;
pub mod service;
pub mod transaction;

pub use filter::{FilterOp, FilterSpec, OrderBy, OrderDirection};
pub use pool::{create_pool, DbConnection, DbPool};
pub use service::{CrudEntity, CrudRepository, InMemoryCrudStore};
pub use stheno_types::{Error, Result};
pub use transaction::{with_savepoint, Conn};
