//! Service layer for `stheno-currency`, backed by Postgres (P11-18).
//!
//! The ISO 4217 table is seeded by the migration rather than by this service, so
//! the precision a deployment rounds money with is part of the schema and
//! survives a restart.

use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Result};

use crate::dto::CurrencyDTO;
use crate::models::Currency;

/// ISO 4217 defines fewer than 200 active codes, so one page holds all of them.
const MAX_CURRENCIES: usize = 100;

#[derive(Clone)]
pub struct CurrencyService {
    currencies: PgCrudRepository<Currency>,
}

impl CurrencyService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            currencies: PgCrudRepository::new(pool),
        }
    }

    fn dto(c: &Currency) -> CurrencyDTO {
        CurrencyDTO {
            code: c.code.clone(),
            symbol: c.symbol.clone(),
            symbol_native: c.symbol_native.clone(),
            name: c.name.clone(),
            decimal_digits: c.decimal_digits,
            rounding: c.rounding,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }

    async fn load(&self, code: &str) -> Result<Currency> {
        self.currencies
            .retrieve(None, &code.to_lowercase())
            .await
            .map_err(|_| Error::NotFound {
                entity: "currency",
                id: code.to_string(),
            })
    }

    pub async fn get_currency(&self, code: &str) -> Result<CurrencyDTO> {
        self.load(code).await.map(|c| Self::dto(&c))
    }

    pub async fn list_currencies(&self) -> Result<Vec<CurrencyDTO>> {
        let all = self
            .currencies
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_order("code", OrderDirection::Asc)
                    .with_pagination(0, MAX_CURRENCIES),
            )
            .await?;
        Ok(all.iter().map(Self::dto).collect())
    }

    /// Decimal digits for a currency code, per ISO 4217.
    ///
    /// Returns an error for an unknown code rather than assuming 2. Silently
    /// defaulting would round JPY (0 digits) and KWD (3) to cents, which is a
    /// financial error of exactly the kind PRD risk R6 names -- and it would do
    /// so invisibly.
    pub async fn decimal_digits(&self, code: &str) -> Result<i16> {
        self.load(code).await.map(|c| c.decimal_digits)
    }
}
