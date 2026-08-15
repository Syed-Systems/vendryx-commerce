//! Service layer for `stheno-currency` with ISO 4217 precision lookup table (TASKS.md P3-01).

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Result};

use crate::dto::CurrencyDTO;
use crate::models::Currency;

#[derive(Clone)]
pub struct CurrencyService {
    currencies: Arc<RwLock<HashMap<String, Currency>>>,
}

impl Default for CurrencyService {
    fn default() -> Self {
        let mut map = HashMap::new();
        let now = Utc::now();

        // Standard ISO 4217 currencies and their precision
        let standard_currencies = vec![
            ("usd", "$", "$", "US Dollar", 2),
            ("eur", "€", "€", "Euro", 2),
            ("gbp", "£", "£", "British Pound", 2),
            ("jpy", "¥", "￥", "Japanese Yen", 0),
            ("cad", "CA$", "$", "Canadian Dollar", 2),
            ("aud", "AU$", "$", "Australian Dollar", 2),
            ("chf", "CHF", "CHF", "Swiss Franc", 2),
            ("cny", "CN¥", "¥", "Chinese Yuan", 2),
            ("inr", "₹", "₹", "Indian Rupee", 2),
            ("krw", "₩", "₩", "South Korean Won", 0),
            ("bhd", "BD", "د.ب", "Bahraini Dinar", 3),
            ("kwd", "KD", "د.ك", "Kuwaiti Dinar", 3),
            ("omr", "OMR", "ر.ع.", "Omani Rial", 3),
            ("sek", "kr", "kr", "Swedish Krona", 2),
            ("nok", "kr", "kr", "Norwegian Krone", 2),
            ("dkk", "kr", "kr", "Danish Krone", 2),
            ("sgd", "S$", "$", "Singapore Dollar", 2),
            ("hkd", "HK$", "$", "Hong Kong Dollar", 2),
            ("nzd", "NZ$", "$", "New Zealand Dollar", 2),
            ("aed", "AED", "د.إ", "United Arab Emirates Dirham", 2),
        ];

        for (code, symbol, symbol_native, name, digits) in standard_currencies {
            map.insert(
                code.to_string(),
                Currency {
                    code: code.to_string(),
                    symbol: symbol.to_string(),
                    symbol_native: symbol_native.to_string(),
                    name: name.to_string(),
                    decimal_digits: digits,
                    rounding: 0,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                },
            );
        }

        Self {
            currencies: Arc::new(RwLock::new(map)),
        }
    }
}

impl CurrencyService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_currency(&self, code: &str) -> Result<CurrencyDTO> {
        let code_lower = code.to_lowercase();
        let map = self.currencies.read().unwrap();
        let c = map
            .get(&code_lower)
            .filter(|c| c.deleted_at.is_none())
            .ok_or_else(|| Error::NotFound {
                entity: "currency",
                id: code.to_string(),
            })?;

        Ok(CurrencyDTO {
            code: c.code.clone(),
            symbol: c.symbol.clone(),
            symbol_native: c.symbol_native.clone(),
            name: c.name.clone(),
            decimal_digits: c.decimal_digits,
            rounding: c.rounding,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
    }

    pub async fn list_currencies(&self) -> Result<Vec<CurrencyDTO>> {
        let map = self.currencies.read().unwrap();
        let list = map
            .values()
            .filter(|c| c.deleted_at.is_none())
            .map(|c| CurrencyDTO {
                code: c.code.clone(),
                symbol: c.symbol.clone(),
                symbol_native: c.symbol_native.clone(),
                name: c.name.clone(),
                decimal_digits: c.decimal_digits,
                rounding: c.rounding,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect();
        Ok(list)
    }

    /// Resolves the decimal digits (precision) for a currency code according to ISO 4217.
    pub fn decimal_digits(&self, code: &str) -> i16 {
        let code_lower = code.to_lowercase();
        let map = self.currencies.read().unwrap();
        map.get(&code_lower).map(|c| c.decimal_digits).unwrap_or(2)
    }
}
