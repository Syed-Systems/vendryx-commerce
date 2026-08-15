use rust_decimal_macros::dec;
use stheno_order_change::{OrderAction, OrderActionType, OrderChange, OrderChangeStatus};
use stheno_types::{CurrencyCode, Id, Money};

#[test]
fn test_order_change_corpus_item_add_and_refund() {
    let mut change = OrderChange::new("ord_test_001");
    let unit_price = Money::new(dec!(25.00), CurrencyCode::Usd);

    // 1. Add 2 items of $25.00 (+ $50.00)
    change.add_action(OrderAction {
        id: Id::new(),
        action: OrderActionType::ItemAdd,
        line_item_id: Some("item_new_1".to_string()),
        quantity: Some(2),
        unit_price: Some(unit_price),
        amount: None,
    });

    // 2. Refund $10.00 (- $10.00)
    change.add_action(OrderAction {
        id: Id::new(),
        action: OrderActionType::RefundAdd,
        line_item_id: None,
        quantity: None,
        unit_price: None,
        amount: Some(Money::new(dec!(10.00), CurrencyCode::Usd)),
    });

    let current_total = Money::new(dec!(100.00), CurrencyCode::Usd);
    let preview = change.preview(current_total).expect("must preview");

    assert_eq!(preview.current_total, current_total);
    assert_eq!(
        preview.difference_total,
        Money::new(dec!(40.00), CurrencyCode::Usd)
    );
    assert_eq!(
        preview.new_total,
        Money::new(dec!(140.00), CurrencyCode::Usd)
    );
    assert_eq!(preview.actions_count, 2);

    change.confirm();
    assert_eq!(change.status, OrderChangeStatus::Confirmed);
    assert!(change.confirmed_at.is_some());
}
