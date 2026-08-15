use crate::currency::CurrencyCode;
use crate::error::Error;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: CurrencyCode,
}

impl Money {
    pub fn new(amount: Decimal, currency: CurrencyCode) -> Self {
        Self { amount, currency }
    }

    pub fn zero(currency: CurrencyCode) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive() && !self.amount.is_zero()
    }

    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative()
    }

    /// Converts the money amount into minor units (integer cents/points),
    /// respecting zero-decimal and three-decimal currencies.
    pub fn to_minor_units(&self) -> Result<i64, Error> {
        let exp = self.currency.minor_unit_exponent();
        let multiplier = match exp {
            0 => Decimal::from(1),
            2 => Decimal::from(100),
            3 => Decimal::from(1000),
            _ => Decimal::from(10u64.pow(exp)),
        };

        let scaled = (self.amount * multiplier).round();
        let mut v = scaled
            .to_string()
            .parse::<i64>()
            .map_err(|_| Error::AmountOverflow)?;

        // Stripe rule: 3-decimal currencies must have a zero as the last digit (round up to nearest 10)
        if exp == 3 {
            v = ((v + 9) / 10) * 10;
        }

        Ok(v)
    }

    /// Reconstructs Money from minor units.
    pub fn from_minor_units(v: i64, currency: CurrencyCode) -> Self {
        let exp = currency.minor_unit_exponent();
        let divisor = match exp {
            0 => Decimal::from(1),
            2 => Decimal::from(100),
            3 => Decimal::from(1000),
            _ => Decimal::from(10u64.pow(exp)),
        };
        let amount = Decimal::from(v) / divisor;
        Self { amount, currency }
    }

    /// Allocates money proportionally across a list of ratios using the Largest Remainder Method (Hare-Niemeyer).
    /// Guarantees that the sum of the allocated amounts equals the original amount EXACTLY at the currency's precision.
    pub fn allocate_proportionally(&self, ratios: &[Decimal]) -> Result<Vec<Money>, Error> {
        if ratios.is_empty() {
            return Ok(Vec::new());
        }

        let total_ratio: Decimal = ratios.iter().sum();
        if total_ratio.is_zero() {
            return Err(Error::Invalid {
                code: "invalid_ratio",
                message: "Sum of ratios must not be zero".to_string(),
                field: None,
            });
        }

        let exp = self.currency.minor_unit_exponent();
        let multiplier = match exp {
            0 => Decimal::from(1),
            2 => Decimal::from(100),
            3 => Decimal::from(1000),
            _ => Decimal::from(10u64.pow(exp)),
        };

        let total_units = (self.amount * multiplier).round();
        let mut allocated_units = Vec::with_capacity(ratios.len());
        let mut remainders = Vec::with_capacity(ratios.len());
        let mut total_allocated_units = Decimal::ZERO;

        for (idx, &ratio) in ratios.iter().enumerate() {
            let exact = (total_units * ratio) / total_ratio;
            let floor_units = exact.floor();
            let remainder = exact - floor_units;
            allocated_units.push(floor_units);
            remainders.push((idx, remainder));
            total_allocated_units += floor_units;
        }

        // Calculate missing units due to floor
        let mut missing_units = (total_units - total_allocated_units)
            .to_string()
            .parse::<i64>()
            .unwrap_or(0);

        // Sort remainders in descending order
        remainders.sort_by_key(|b| std::cmp::Reverse(b.1));

        // Distribute missing units to highest remainders
        for (idx, _) in remainders {
            if missing_units <= 0 {
                break;
            }
            allocated_units[idx] += Decimal::ONE;
            missing_units -= 1;
        }

        let result = allocated_units
            .into_iter()
            .map(|units| Money::new(units / multiplier, self.currency))
            .collect();

        Ok(result)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

impl Add for Money {
    type Output = Result<Money, Error>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(Error::CurrencyMismatch {
                expected: self.currency,
                found: rhs.currency,
            });
        }
        Ok(Money::new(self.amount + rhs.amount, self.currency))
    }
}

impl Sub for Money {
    type Output = Result<Money, Error>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(Error::CurrencyMismatch {
                expected: self.currency,
                found: rhs.currency,
            });
        }
        Ok(Money::new(self.amount - rhs.amount, self.currency))
    }
}

impl Mul<Decimal> for Money {
    type Output = Money;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Money::new(self.amount * rhs, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_money_ops() {
        let m1 = Money::new(dec!(10.50), CurrencyCode::Usd);
        let m2 = Money::new(dec!(4.50), CurrencyCode::Usd);
        let sum = (m1 + m2).unwrap();
        assert_eq!(sum.amount, dec!(15.00));
        assert_eq!(sum.currency, CurrencyCode::Usd);
    }

    #[test]
    fn test_minor_units_zero_decimal() {
        let m = Money::new(dec!(1500), CurrencyCode::Jpy);
        assert_eq!(m.to_minor_units().unwrap(), 1500);
        let rec = Money::from_minor_units(1500, CurrencyCode::Jpy);
        assert_eq!(rec.amount, dec!(1500));
    }

    #[test]
    fn test_minor_units_three_decimal_stripe_rule() {
        let m = Money::new(dec!(1.234), CurrencyCode::Kwd);
        // 1.234 * 1000 = 1234 -> rounded up to nearest 10 = 1240
        assert_eq!(m.to_minor_units().unwrap(), 1240);
    }

    #[test]
    fn test_allocation_exact_sum() {
        let total = Money::new(dec!(10.00), CurrencyCode::Usd);
        let ratios = vec![dec!(1), dec!(1), dec!(1)];
        let parts = total.allocate_proportionally(&ratios).unwrap();
        assert_eq!(parts.len(), 3);
        let sum: Decimal = parts.iter().map(|p| p.amount).sum();
        assert_eq!(sum, dec!(10.00));
        assert_eq!(parts[0].amount, dec!(3.34));
        assert_eq!(parts[1].amount, dec!(3.33));
        assert_eq!(parts[2].amount, dec!(3.33));
    }
}
