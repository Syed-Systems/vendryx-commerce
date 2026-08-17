//! `PgCrudRepository<T>` -- the Diesel-backed implementation of `CrudRepository`
//! (P11-07, P11-08).
//!
//! One generic implementation, not 25 hand-written ones. Entities are moved
//! between Rust and Postgres as JSON:
//!
//! * reading uses `row_to_json(t)`, so any `Deserialize` entity can be built
//!   from any table without a Diesel `Queryable` derive per entity;
//! * writing uses `json_populate_record(NULL::<table>, $1)`, which coerces each
//!   JSON field to the column's own type using the table's row type.
//!
//! The alternative -- Diesel's typed DSL -- cannot be made generic over 127
//! entities without a `Queryable`/`Insertable`/`Table` triple per entity, which
//! is precisely the 25 hand-written repositories ADR-004 set out to avoid.

use async_trait::async_trait;
use diesel::sql_types::Text;
use diesel::QueryableByName;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde_json::Value;
use std::marker::PhantomData;
use stheno_types::{Error, Result};

use crate::filter::FilterSpec;
use crate::pool::DbPool;
use crate::service::{CrudEntity, CrudRepository};
use crate::sql::{ident, quote_table};
use crate::transaction::Conn;

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = Text)]
    value: String,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    value: i64,
}

pub struct PgCrudRepository<T: CrudEntity> {
    pool: DbPool,
    _marker: PhantomData<fn() -> T>,
}

impl<T: CrudEntity> Clone for PgCrudRepository<T> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: CrudEntity> PgCrudRepository<T> {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            _marker: PhantomData,
        }
    }

    fn table() -> Result<String> {
        quote_table(T::TABLE)
    }

    /// The primary-key column, identifier-checked.
    fn id_column() -> Result<&'static str> {
        ident(T::ID_COLUMN)
    }

    fn to_json(item: &T) -> Result<String> {
        serde_json::to_string(item)
            .map_err(|e| Error::Database(format!("could not serialise {}: {e}", T::TABLE)))
    }

    fn from_json(raw: &str) -> Result<T> {
        serde_json::from_str(raw)
            .map_err(|e| Error::Database(format!("could not deserialise {}: {e}", T::TABLE)))
    }

    fn not_found(id: &str) -> Error {
        Error::NotFound {
            entity: T::ENTITY,
            id: id.to_string(),
        }
    }

    /// The column list of a serialized entity, identifier-checked.
    ///
    /// Taken from the entity's own JSON rather than from the database, so an
    /// entity that has drifted from its table fails here rather than writing a
    /// partial row.
    fn columns(json: &str) -> Result<Vec<String>> {
        let value: Value = serde_json::from_str(json)
            .map_err(|e| Error::Database(format!("could not read entity fields: {e}")))?;
        let obj = value.as_object().ok_or_else(|| {
            Error::Database(format!("{} does not serialise to a JSON object", T::TABLE))
        })?;

        obj.keys()
            .map(|k| ident(k).map(str::to_string))
            .collect::<Result<Vec<_>>>()
    }

    /// Runs `f` against the caller's transaction if there is one, otherwise
    /// against a pooled connection.
    ///
    /// The `Option<&mut Conn>` shape is what makes a workflow step and a plain
    /// read share one repository: inside a SAGA the caller owns the transaction
    /// and every write must join it, while a simple GET has no transaction to
    /// join and should not open one.
    async fn with_conn<'a, R, F>(&self, conn: Option<&mut Conn<'a>>, f: F) -> Result<R>
    where
        F: for<'c> FnOnce(
            &'c mut AsyncPgConnection,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<R>> + Send + 'c>,
        >,
    {
        match conn {
            Some(c) => f(c).await,
            None => {
                let mut pooled = self
                    .pool
                    .get()
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;
                f(&mut pooled).await
            }
        }
    }
}

#[async_trait]
impl<T: CrudEntity> CrudRepository<T> for PgCrudRepository<T> {
    async fn retrieve(&self, conn: Option<&mut Conn<'_>>, id: &str) -> Result<T> {
        let table = Self::table()?;
        let key = Self::id_column()?;
        let id = id.to_string();
        let sql = format!(
            "SELECT row_to_json(t)::text AS value FROM {table} t \
             WHERE t.\"{key}\" = $1 AND t.deleted_at IS NULL"
        );

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let row: Option<JsonRow> = diesel::sql_query(sql)
                    .bind::<Text, _>(&id)
                    .get_result(c)
                    .await
                    .optional_row()?;
                match row {
                    Some(r) => Self::from_json(&r.value),
                    None => Err(Self::not_found(&id)),
                }
            })
        })
        .await
    }

    async fn list(&self, conn: Option<&mut Conn<'_>>, filter: &FilterSpec) -> Result<Vec<T>> {
        let table = Self::table()?;
        let rendered = filter.to_sql(T::TABLE)?;
        let params = rendered.params_json();
        let sql = format!(
            "SELECT row_to_json(t)::text AS value FROM {table} t {}",
            rendered.clause
        );

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let rows: Vec<JsonRow> = diesel::sql_query(sql)
                    .bind::<Text, _>(&params)
                    .load(c)
                    .await
                    .map_err(to_db_error)?;
                rows.iter().map(|r| Self::from_json(&r.value)).collect()
            })
        })
        .await
    }

    async fn list_and_count(
        &self,
        conn: Option<&mut Conn<'_>>,
        filter: &FilterSpec,
    ) -> Result<(Vec<T>, i64)> {
        let table = Self::table()?;
        let page = filter.to_sql(T::TABLE)?;
        let total = filter.to_count_sql(T::TABLE)?;
        let page_params = page.params_json();
        let count_params = total.params_json();

        let list_sql = format!(
            "SELECT row_to_json(t)::text AS value FROM {table} t {}",
            page.clause
        );
        let count_sql = format!("SELECT count(*) AS value FROM {table} t {}", total.clause);

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let rows: Vec<JsonRow> = diesel::sql_query(list_sql)
                    .bind::<Text, _>(&page_params)
                    .load(c)
                    .await
                    .map_err(to_db_error)?;
                let items: Vec<T> = rows
                    .iter()
                    .map(|r| Self::from_json(&r.value))
                    .collect::<Result<_>>()?;

                let count: CountRow = diesel::sql_query(count_sql)
                    .bind::<Text, _>(&count_params)
                    .get_result(c)
                    .await
                    .map_err(to_db_error)?;

                Ok((items, count.value))
            })
        })
        .await
    }

    async fn create(&self, conn: Option<&mut Conn<'_>>, item: T) -> Result<T> {
        let table = Self::table()?;
        let json = Self::to_json(&item)?;
        let sql = format!(
            "INSERT INTO {table} AS t \
             SELECT * FROM json_populate_record(NULL::{table}, $1::json) \
             RETURNING row_to_json(t)::text AS value"
        );

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let row: JsonRow = diesel::sql_query(sql)
                    .bind::<Text, _>(&json)
                    .get_result(c)
                    .await
                    .map_err(|e| conflict_or_db_error::<T>(e, "entity_already_exists"))?;
                Self::from_json(&row.value)
            })
        })
        .await
    }

    async fn create_many(&self, conn: Option<&mut Conn<'_>>, items: Vec<T>) -> Result<Vec<T>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }
        let table = Self::table()?;
        let json = serde_json::to_string(&items)
            .map_err(|e| Error::Database(format!("could not serialise batch: {e}")))?;
        // One statement, so the batch is atomic even without a caller transaction.
        let sql = format!(
            "INSERT INTO {table} AS t \
             SELECT * FROM json_populate_recordset(NULL::{table}, $1::json) \
             RETURNING row_to_json(t)::text AS value"
        );

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let rows: Vec<JsonRow> = diesel::sql_query(sql)
                    .bind::<Text, _>(&json)
                    .load(c)
                    .await
                    .map_err(|e| conflict_or_db_error::<T>(e, "entity_already_exists"))?;
                rows.iter().map(|r| Self::from_json(&r.value)).collect()
            })
        })
        .await
    }

    async fn update(&self, conn: Option<&mut Conn<'_>>, id: &str, item: T) -> Result<T> {
        let table = Self::table()?;
        let json = Self::to_json(&item)?;
        let cols = Self::columns(&json)?
            .into_iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let key = Self::id_column()?;
        let id = id.to_string();
        let sql = format!(
            "UPDATE {table} AS t SET ({cols}) = \
             (SELECT {cols} FROM json_populate_record(NULL::{table}, $1::json)) \
             WHERE t.\"{key}\" = $2 RETURNING row_to_json(t)::text AS value"
        );

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let row: Option<JsonRow> = diesel::sql_query(sql)
                    .bind::<Text, _>(&json)
                    .bind::<Text, _>(&id)
                    .get_result(c)
                    .await
                    .optional_row()?;
                match row {
                    Some(r) => Self::from_json(&r.value),
                    None => Err(Self::not_found(&id)),
                }
            })
        })
        .await
    }

    async fn upsert(&self, conn: Option<&mut Conn<'_>>, item: T) -> Result<T> {
        let table = Self::table()?;
        let json = Self::to_json(&item)?;
        let cols = Self::columns(&json)?;
        let key = Self::id_column()?;
        // The key is the conflict target, so assigning it would be a no-op at best.
        let assigned: Vec<String> = cols
            .iter()
            .filter(|c| c.as_str() != key)
            .map(|c| format!("\"{c}\""))
            .collect();
        let assign_list = assigned.join(", ");
        let excluded: Vec<String> = cols
            .iter()
            .filter(|c| c.as_str() != key)
            .map(|c| format!("EXCLUDED.\"{c}\""))
            .collect();

        let sql = format!(
            "INSERT INTO {table} AS t \
             SELECT * FROM json_populate_record(NULL::{table}, $1::json) \
             ON CONFLICT (\"{key}\") DO UPDATE SET ({assign_list}) = ROW({}) \
             RETURNING row_to_json(t)::text AS value",
            excluded.join(", ")
        );

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let row: JsonRow = diesel::sql_query(sql)
                    .bind::<Text, _>(&json)
                    .get_result(c)
                    .await
                    .map_err(to_db_error)?;
                Self::from_json(&row.value)
            })
        })
        .await
    }

    async fn delete(&self, conn: Option<&mut Conn<'_>>, id: &str) -> Result<()> {
        let table = Self::table()?;
        let id = id.to_string();
        let key = Self::id_column()?;
        let sql = format!("DELETE FROM {table} AS t WHERE t.\"{key}\" = $1");

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let affected = diesel::sql_query(sql)
                    .bind::<Text, _>(&id)
                    .execute(c)
                    .await
                    .map_err(to_db_error)?;
                if affected == 0 {
                    return Err(Self::not_found(&id));
                }
                Ok(())
            })
        })
        .await
    }

    async fn soft_delete(&self, conn: Option<&mut Conn<'_>>, id: &str) -> Result<()> {
        self.set_deleted_at(conn, id, true).await
    }

    async fn restore(&self, conn: Option<&mut Conn<'_>>, id: &str) -> Result<()> {
        self.set_deleted_at(conn, id, false).await
    }
}

impl<T: CrudEntity> PgCrudRepository<T> {
    async fn set_deleted_at(
        &self,
        conn: Option<&mut Conn<'_>>,
        id: &str,
        deleted: bool,
    ) -> Result<()> {
        let table = Self::table()?;
        let id = id.to_string();
        let value = if deleted { "now()" } else { "NULL" };
        // Soft delete is idempotent, but restoring a row that was never deleted
        // must still find it -- hence no `deleted_at IS NULL` guard here.
        let key = Self::id_column()?;
        let sql = format!("UPDATE {table} AS t SET deleted_at = {value} WHERE t.\"{key}\" = $1");

        self.with_conn(conn, move |c| {
            Box::pin(async move {
                let affected = diesel::sql_query(sql)
                    .bind::<Text, _>(&id)
                    .execute(c)
                    .await
                    .map_err(to_db_error)?;
                if affected == 0 {
                    return Err(Self::not_found(&id));
                }
                Ok(())
            })
        })
        .await
    }
}

fn to_db_error(e: diesel::result::Error) -> Error {
    Error::Database(e.to_string())
}

/// Maps a unique-violation to a domain conflict, everything else to a database error.
///
/// A duplicate id is a caller mistake and must surface as 409, not 500.
fn conflict_or_db_error<T: CrudEntity>(e: diesel::result::Error, code: &'static str) -> Error {
    use diesel::result::{DatabaseErrorKind, Error as DieselError};
    match e {
        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, ref info) => {
            Error::Conflict {
                code,
                message: format!("{} already exists: {}", T::ENTITY, info.message()),
            }
        }
        other => Error::Database(other.to_string()),
    }
}

/// `get_result` returns `NotFound` for an empty result; a missing row is a
/// domain outcome here, not a database failure.
trait OptionalRow<R> {
    fn optional_row(self) -> Result<Option<R>>;
}

impl<R> OptionalRow<R> for std::result::Result<R, diesel::result::Error> {
    fn optional_row(self) -> Result<Option<R>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(to_db_error(e)),
        }
    }
}
