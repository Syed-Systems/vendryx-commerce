use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use stheno_types::Error;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection = Object<AsyncPgConnection>;

pub fn create_pool(database_url: &str, max_size: usize) -> Result<DbPool, Error> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(config)
        .max_size(max_size)
        .build()
        .map_err(|e| Error::Database(e.to_string()))
}
