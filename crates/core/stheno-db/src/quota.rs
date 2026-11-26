//! `stheno-db::quota`: Entitlement quota atomic counters (ENTITLEMENTS.md §6.2).
//!
//! Counters are per tenant. The table was keyed by `resource` alone, which was correct
//! while ADR-009 held (one deployment per merchant) and became a shared-counter bug the
//! moment one database served many tenants: every tenant would have contended for the
//! same three rows. The key is now `(tenant_id, resource)`; `tenant_id` is supplied by
//! the column DEFAULT from the transaction's tenant scope, so the SQL below never names
//! it and RLS keeps reads and decrements scoped without a predicate here.

use crate::diesel_async::RunQueryDsl;
use crate::DbPool;
use diesel::sql_types::{BigInt, Nullable, Text};
use diesel::QueryableByName;
use diesel_async::AsyncPgConnection;
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
        conn: &mut AsyncPgConnection,
        resource: &str,
        limit: Option<i64>,
    ) -> Result<i64> {
        let query = diesel::sql_query(
            "INSERT INTO entitlement_usage (resource, used) \
             VALUES ($1, 1) \
             ON CONFLICT (tenant_id, resource) DO UPDATE \
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
                message: format!("Quota limit reached for resource: {resource}"),
            })
        }
    }

    /// Decrement usage upon deletion (bounded at zero).
    pub async fn decrement(&self, conn: &mut AsyncPgConnection, resource: &str) -> Result<i64> {
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
    pub async fn get_usage(&self, conn: &mut AsyncPgConnection, resource: &str) -> Result<i64> {
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
        conn: &mut AsyncPgConnection,
        resource: &str,
        count: i64,
    ) -> Result<i64> {
        let query = diesel::sql_query(
            "INSERT INTO entitlement_usage (resource, used) VALUES ($1, $2) \
             ON CONFLICT (tenant_id, resource) DO UPDATE SET used = EXCLUDED.used \
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

    /// Reads usage on a connection this method scopes itself.
    ///
    /// The methods above take a caller-supplied connection so they can join a SAGA's
    /// transaction. Callers that have no transaction of their own were each repeating
    /// the same checkout-and-scope dance — and getting it wrong: a raw `pool.get()` has
    /// no tenant, so under RLS it reads zero rather than the caller's real count
    /// (ADR-012).
    pub async fn usage_scoped(&self, resource: &str) -> Result<i64> {
        let resource = resource.to_string();
        let counter = self.clone();
        crate::tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move { counter.get_usage(conn, &resource).await })
        })
        .await
    }

    /// Sets usage on a connection this method scopes itself. Reconciliation and tests.
    pub async fn set_usage_scoped(&self, resource: &str, count: i64) -> Result<i64> {
        let resource = resource.to_string();
        let counter = self.clone();
        crate::tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move { counter.set_usage(conn, &resource, count).await })
        })
        .await
    }
}
