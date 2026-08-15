use diesel::sql_query;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::future::Future;
use stheno_types::Error;

/// Explicit database connection reference passed to services.
pub type Conn<'a> = &'a mut AsyncPgConnection;

/// Runs an operation inside a PostgreSQL SAVEPOINT for nested transaction semantics.
/// If the operation returns Ok, the savepoint is released.
/// If the operation returns Err, the savepoint is rolled back.
pub async fn with_savepoint<T, F, Fut>(
    conn: &mut AsyncPgConnection,
    savepoint_name: &str,
    f: F,
) -> Result<T, Error>
where
    F: FnOnce(&mut AsyncPgConnection) -> Fut,
    Fut: Future<Output = Result<T, Error>>,
{
    sql_query(format!("SAVEPOINT {}", savepoint_name))
        .execute(conn)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    match f(conn).await {
        Ok(val) => {
            sql_query(format!("RELEASE SAVEPOINT {}", savepoint_name))
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;
            Ok(val)
        }
        Err(err) => {
            let _ = sql_query(format!("ROLLBACK TO SAVEPOINT {}", savepoint_name))
                .execute(conn)
                .await;
            Err(err)
        }
    }
}
