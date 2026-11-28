use stheno_types::currency::CurrencyCode;
use stheno_types::error::Error;
use stheno_types::money::Money;

/// Converts a Medusa minor-unit integer to a Stheno `Money` equivalent,
/// asserting that no precision is lost during the conversion.
pub fn convert_medusa_money_to_stheno(amount: i64, currency: CurrencyCode) -> Result<Money, Error> {
    let money = Money::from_minor_units(amount, currency);

    // Explicit assertion that precision is lossless.
    // If reconstructing minor units fails or doesn't match the original, it's a critical error.
    let reconstructed = money.to_minor_units()?;
    if reconstructed != amount {
        return Err(Error::Invalid {
            code: "precision_loss",
            message: format!(
                "Precision lost converting Medusa amount {amount}. Reconstructed: {reconstructed}"
            ),
            field: Some("amount".to_string()),
        });
    }

    Ok(money)
}

/// Represents the raw financial line items and totals of an order as stored in Medusa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderFinancialSnapshot {
    pub currency: CurrencyCode,
    pub line_item_subtotal_minor: i64,
    pub shipping_total_minor: i64,
    pub discount_total_minor: i64,
    pub tax_total_minor: i64,
    pub expected_order_total_minor: i64,
    pub payment_captures_minor: Vec<i64>,
    pub refunds_minor: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinancialReconciliation {
    pub order_total: Money,
    pub total_captured: Money,
    pub total_refunded: Money,
    pub net_settled: Money,
    pub order_total_matches: bool,
    pub capture_matches: bool,
    pub refund_matches: bool,
    pub is_exact_match: bool,
}

/// Reconciles an imported Medusa order snapshot against Stheno's financial calculations,
/// verifying that order, payment capture, and refund totals match source amounts with exact zero drift.
pub fn reconcile_order_financials(
    snapshot: &OrderFinancialSnapshot,
) -> Result<FinancialReconciliation, Error> {
    let currency = snapshot.currency;

    let subtotal = convert_medusa_money_to_stheno(snapshot.line_item_subtotal_minor, currency)?;
    let shipping = convert_medusa_money_to_stheno(snapshot.shipping_total_minor, currency)?;
    let discount = convert_medusa_money_to_stheno(snapshot.discount_total_minor, currency)?;
    let tax = convert_medusa_money_to_stheno(snapshot.tax_total_minor, currency)?;

    // Stheno calculated order total = (subtotal + shipping + tax) - discount
    let order_total = ((subtotal + shipping)? + tax)? - discount;
    let order_total = order_total?;

    let order_total_matches = order_total.to_minor_units()? == snapshot.expected_order_total_minor;

    // Aggregate captures
    let mut total_captured = Money::zero(currency);
    let mut source_captured_sum: i64 = 0;
    for &cap in &snapshot.payment_captures_minor {
        source_captured_sum += cap;
        let cap_money = convert_medusa_money_to_stheno(cap, currency)?;
        total_captured = (total_captured + cap_money)?;
    }
    let capture_matches = total_captured.to_minor_units()? == source_captured_sum;

    // Aggregate refunds
    let mut total_refunded = Money::zero(currency);
    let mut source_refunded_sum: i64 = 0;
    for &ref_amt in &snapshot.refunds_minor {
        source_refunded_sum += ref_amt;
        let ref_money = convert_medusa_money_to_stheno(ref_amt, currency)?;
        total_refunded = (total_refunded + ref_money)?;
    }
    let refund_matches = total_refunded.to_minor_units()? == source_refunded_sum;

    let net_settled = (total_captured - total_refunded)?;
    let is_exact_match = order_total_matches && capture_matches && refund_matches;

    if !is_exact_match {
        return Err(Error::Invalid {
            code: "reconciliation_mismatch",
            message: format!(
                "Order totals mismatch. Expected order total {}, calculated {}.",
                snapshot.expected_order_total_minor,
                order_total.to_minor_units()?
            ),
            field: Some("total".to_string()),
        });
    }

    Ok(FinancialReconciliation {
        order_total,
        total_captured,
        total_refunded,
        net_settled,
        order_total_matches,
        capture_matches,
        refund_matches,
        is_exact_match,
    })
}
