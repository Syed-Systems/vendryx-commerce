//! `stheno-db::quota`: Entitlement quota atomic counters (ENTITLEMENTS.md §6.2).

use crate::diesel_async::RunQueryDsl;
use crate::{DbConnection, DbPool};
use diesel::sql_types::{BigInt, Nullable, Text};
use diesel::QueryableByName;
use stheno_types::{Error, Result};

#[derive(Debug, Clone, QueryableByName)]
pub struct UsageRow {
    #[diesel(sql_type = BigInt)]
    pub used: i64,
}

#[derive(Clone)]
pub struct QuotaCounter {
    pool: DbPool,
}

impl QuotaCounter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    /// Atomically increment usage if under limit in the given transaction/connection.
    /// Returns the new `used` count or `Error::Conflict` if limit is exceeded (0 rows updated).
    pub async fn increment(
        &self,
        conn: &mut DbConnection,
        resource: &str,
        limit: Option<i64>,
    ) -> Result<i64> {
        let query = diesel::sql_query(
            "INSERT INTO entitlement_usage (resource, used) \
             VALUES ($1, 1) \
             ON CONFLICT (resource) DO UPDATE \
                SET used = entitlement_usage.used + 1 \
              WHERE ($2::bigint IS NULL OR entitlement_usage.used + 1 <= $2) \
             RETURNING used;",
        )
        .bind::<Text, _>(resource)
        .bind::<Nullable<BigInt>, _>(limit);

        let rows: Vec<UsageRow> = query
            .load(conn)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        if let Some(row) = rows.as_slice().first() {
            Ok(row.used)
        } else {
            Err(Error::Conflict {
                code: "quota_exceeded",
                message: format!("Quota limit reached for resource: {}", resource),
            })
        }
    }

    /// Decrement usage upon deletion (bounded at zero).
    pub async fn decrement(&self, conn: &mut DbConnection, resource: &str) -> Result<i64> {
        let query = diesel::sql_query(
            "UPDATE entitlement_usage \
                SET used = GREATEST(0, used - 1) \
              WHERE resource = $1 \
          RETURNING used;",
        )
        .bind::<Text, _>(resource);

        let rows: Vec<UsageRow> = query
            .load(conn)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(rows.as_slice().first().map(|r| r.used).unwrap_or(0))
    }

    /// Get current usage count
    pub async fn get_usage(&self, conn: &mut DbConnection, resource: &str) -> Result<i64> {
        let query = diesel::sql_query("SELECT used FROM entitlement_usage WHERE resource = $1;")
            .bind::<Text, _>(resource);

        let rows: Vec<UsageRow> = query
            .load(conn)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(rows.as_slice().first().map(|r| r.used).unwrap_or(0))
    }

    /// Set usage explicitly (used for reconciliation or testing)
    pub async fn set_usage(
        &self,
        conn: &mut DbConnection,
        resource: &str,
        count: i64,
    ) -> Result<i64> {
        let query = diesel::sql_query(
            "INSERT INTO entitlement_usage (resource, used) VALUES ($1, $2) \
             ON CONFLICT (resource) DO UPDATE SET used = EXCLUDED.used \
             RETURNING used;",
        )
        .bind::<Text, _>(resource)
        .bind::<BigInt, _>(count);

        let rows: Vec<UsageRow> = query
            .load(conn)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(rows.as_slice().first().map(|r| r.used).unwrap_or(count))
    }
}
