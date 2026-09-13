use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use stheno_types::currency::CurrencyCode;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FxMetadata {
    pub source: String,
    pub rate_kind: String,
    pub rate: Option<Decimal>,
    pub rate_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionInvoice {
    pub months: i32,
    pub subtotal_usd: Decimal,
    pub discount_usd: Decimal,
    pub total_usd: Decimal,
    pub presentment_currency: CurrencyCode,
    pub fx_metadata: FxMetadata,
    pub payable_amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalculatorError {
    InvalidMonths,
    MissingFxRate,
}

pub struct SubscriptionInvoiceCalculator;

impl SubscriptionInvoiceCalculator {
    pub fn calculate(
        base_price_usd: Decimal,
        discount_percent: Option<Decimal>,
        discount_starts_at_months: Option<i32>,
        months: i32,
        billing_country: &str,
        provided_fx_metadata: Option<FxMetadata>,
    ) -> Result<SubscriptionInvoice, CalculatorError> {
        if months <= 0 {
            return Err(CalculatorError::InvalidMonths);
        }

        let subtotal_usd = base_price_usd * Decimal::from(months);

        let mut discount_usd = Decimal::from(0);

        if let (Some(percent), Some(starts_at)) = (discount_percent, discount_starts_at_months) {
            if months >= starts_at {
                discount_usd = subtotal_usd * (percent / Decimal::from(100));
            }
        }

        let total_usd = subtotal_usd - discount_usd;

        let is_bangladesh = billing_country.eq_ignore_ascii_case("BD") || billing_country.eq_ignore_ascii_case("BGD");

        let presentment_currency = if is_bangladesh {
            CurrencyCode::Bdt
        } else {
            CurrencyCode::Usd
        };

        let (payable_amount, fx_metadata) = if is_bangladesh {
            let meta = provided_fx_metadata.ok_or(CalculatorError::MissingFxRate)?;
            let rate = meta.rate.unwrap_or_else(|| Decimal::from(120)); // Fallback if missing
            
            // Bangladesh payable is rounded to whole taka
            let amount = (total_usd * rate).round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven);
            (amount, meta)
        } else {
            (
                total_usd, // USD remains exact
                FxMetadata {
                    source: "none".to_string(),
                    rate_kind: "none".to_string(),
                    rate: None,
                    rate_date: None,
                },
            )
        };

        Ok(SubscriptionInvoice {
            months,
            subtotal_usd,
            discount_usd,
            total_usd,
            presentment_currency,
            fx_metadata,
            payable_amount,
        })
    }
}
