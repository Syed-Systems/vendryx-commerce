//! Order Edit SAGA & Workflows (P6-01 to P6-06, F6).
use rust_decimal::Decimal;
use std::sync::Arc;
use stheno_order::{
    CreateOrderChangeActionInput, CreateOrderChangeInput, CreateTransactionInput, OrderChangeDTO,
    OrderChangePreview, OrderDTO, OrderService,
};
use stheno_payment::PaymentService;
use stheno_types::Result;

#[derive(Clone)]
pub struct OrderEditWorkflowEngine {
    order_service: Arc<OrderService>,
    _payment_service: Option<Arc<PaymentService>>,
}

impl OrderEditWorkflowEngine {
    pub fn new(
        order_service: Arc<OrderService>,
        payment_service: Option<Arc<PaymentService>>,
    ) -> Self {
        Self {
            order_service,
            _payment_service: payment_service,
        }
    }

    /// Begin an order edit session (P6-01).
    pub async fn begin_order_edit(
        &self,
        order_id: &str,
        requested_by: Option<&str>,
    ) -> Result<OrderChangeDTO> {
        self.order_service
            .create_order_change(CreateOrderChangeInput {
                order_id: order_id.to_string(),
                change_type: "edit".to_string(),
                requested_by: requested_by.map(|s| s.to_string()),
                metadata: None,
            })
            .await
    }

    /// Add an item to the order edit (P6-01).
    pub async fn add_item(
        &self,
        edit_id: &str,
        title: &str,
        quantity: i32,
        unit_price: Decimal,
        variant_id: Option<&str>,
    ) -> Result<()> {
        let amount = unit_price * Decimal::from(quantity);
        self.order_service
            .add_order_change_action(
                edit_id,
                CreateOrderChangeActionInput {
                    action: "ITEM_ADD".to_string(),
                    reference: Some("order_item".to_string()),
                    reference_id: None,
                    amount: Some(amount),
                    details: Some(serde_json::json!({
                        "title": title,
                        "quantity": quantity,
                        "unit_price": unit_price.to_string(),
                        "variant_id": variant_id,
                    })),
                },
            )
            .await?;
        Ok(())
    }

    /// Update an existing item in the order edit (P6-01).
    pub async fn update_item(
        &self,
        edit_id: &str,
        line_item_id: &str,
        new_quantity: i32,
        new_unit_price: Option<Decimal>,
    ) -> Result<()> {
        let mut details = serde_json::json!({
            "quantity": new_quantity,
        });
        if let Some(price) = new_unit_price {
            details["unit_price"] = serde_json::json!(price.to_string());
        }

        self.order_service
            .add_order_change_action(
                edit_id,
                CreateOrderChangeActionInput {
                    action: "ITEM_UPDATE".to_string(),
                    reference: Some("order_item".to_string()),
                    reference_id: Some(line_item_id.to_string()),
                    amount: None,
                    details: Some(details),
                },
            )
            .await?;
        Ok(())
    }

    /// Remove an item in the order edit (P6-01).
    pub async fn remove_item(&self, edit_id: &str, line_item_id: &str) -> Result<()> {
        self.order_service
            .add_order_change_action(
                edit_id,
                CreateOrderChangeActionInput {
                    action: "ITEM_REMOVE".to_string(),
                    reference: Some("order_item".to_string()),
                    reference_id: Some(line_item_id.to_string()),
                    amount: None,
                    details: None,
                },
            )
            .await?;
        Ok(())
    }

    /// Add a shipping method to the order edit (P6-02).
    pub async fn add_shipping_method(
        &self,
        edit_id: &str,
        name: &str,
        amount: Decimal,
        shipping_option_id: Option<&str>,
    ) -> Result<()> {
        self.order_service
            .add_order_change_action(
                edit_id,
                CreateOrderChangeActionInput {
                    action: "SHIPPING_ADD".to_string(),
                    reference: Some("shipping_method".to_string()),
                    reference_id: None,
                    amount: Some(amount),
                    details: Some(serde_json::json!({
                        "name": name,
                        "amount": amount.to_string(),
                        "shipping_option_id": shipping_option_id,
                    })),
                },
            )
            .await?;
        Ok(())
    }

    /// Update a shipping method in the order edit (P6-02).
    pub async fn update_shipping_method(
        &self,
        edit_id: &str,
        shipping_method_id: &str,
        amount: Decimal,
    ) -> Result<()> {
        self.order_service
            .add_order_change_action(
                edit_id,
                CreateOrderChangeActionInput {
                    action: "SHIPPING_UPDATE".to_string(),
                    reference: Some("shipping_method".to_string()),
                    reference_id: Some(shipping_method_id.to_string()),
                    amount: Some(amount),
                    details: Some(serde_json::json!({
                        "amount": amount.to_string(),
                    })),
                },
            )
            .await?;
        Ok(())
    }

    /// Remove a shipping method in the order edit (P6-02).
    pub async fn remove_shipping_method(
        &self,
        edit_id: &str,
        shipping_method_id: &str,
    ) -> Result<()> {
        self.order_service
            .add_order_change_action(
                edit_id,
                CreateOrderChangeActionInput {
                    action: "SHIPPING_REMOVE".to_string(),
                    reference: Some("shipping_method".to_string()),
                    reference_id: Some(shipping_method_id.to_string()),
                    amount: None,
                    details: None,
                },
            )
            .await?;
        Ok(())
    }

    /// Edit preview — computed in-memory, ZERO writes (P6-03).
    pub async fn preview_edit(&self, edit_id: &str) -> Result<OrderChangePreview> {
        self.order_service.preview_order_change(edit_id).await
    }

    /// Edit confirm: bumps version to N+1, recomputes summary, handles payment delta (P6-04, P6-05).
    pub async fn confirm_edit(
        &self,
        edit_id: &str,
        confirmed_by: Option<&str>,
    ) -> Result<OrderDTO> {
        // 1. Calculate preview to get the payment delta
        let preview = self.order_service.preview_order_change(edit_id).await?;
        let delta_amt = preview.difference_total.amount;

        // 2. Confirm change in order service (version bump to N+1)
        let order = self
            .order_service
            .confirm_order_change(edit_id, confirmed_by)
            .await?;

        // 3. Handle delta: collect on increase, refund on decrease (P6-05)
        if delta_amt > Decimal::ZERO {
            // Collect additional amount
            self.order_service
                .add_transaction(
                    &order.id,
                    CreateTransactionInput {
                        amount: delta_amt,
                        currency_code: order.currency_code.clone(),
                        reference: "payment".to_string(),
                        reference_id: format!("edit_collect_{}", edit_id),
                        metadata: None,
                    },
                )
                .await?;
        } else if delta_amt < Decimal::ZERO {
            // Refund difference
            let refund_amt = delta_amt.abs();
            self.order_service
                .add_transaction(
                    &order.id,
                    CreateTransactionInput {
                        amount: refund_amt,
                        currency_code: order.currency_code.clone(),
                        reference: "refund".to_string(),
                        reference_id: format!("edit_refund_{}", edit_id),
                        metadata: None,
                    },
                )
                .await?;
        }

        self.order_service.get_order(&order.id).await
    }

    /// Edit cancel/discard leaves version and items untouched (P6-06).
    pub async fn cancel_edit(&self, edit_id: &str) -> Result<OrderChangeDTO> {
        self.order_service.cancel_order_change(edit_id).await
    }
}
