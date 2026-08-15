//! Exchange workflows: begin, inbound/outbound items, confirm with computed difference, cancel (P6-13, P6-14).
use rust_decimal::Decimal;
use std::sync::Arc;
use stheno_order::{
    AddExchangeItemInput, CreateExchangeInput, ExchangeDTO, ExchangeItemDTO, OrderService,
};
use stheno_types::Result;

#[derive(Clone)]
pub struct ExchangeWorkflowEngine {
    order_service: Arc<OrderService>,
}

impl ExchangeWorkflowEngine {
    pub fn new(order_service: Arc<OrderService>) -> Self {
        Self { order_service }
    }

    /// Begin an exchange (P6-13).
    pub async fn begin_exchange(
        &self,
        order_id: &str,
        allow_backorder: bool,
        created_by: Option<&str>,
    ) -> Result<ExchangeDTO> {
        self.order_service
            .create_exchange(CreateExchangeInput {
                order_id: order_id.to_string(),
                allow_backorder: Some(allow_backorder),
                created_by: created_by.map(|s| s.to_string()),
                metadata: None,
            })
            .await
    }

    /// Add an inbound/outbound item to exchange (P6-13).
    pub async fn add_exchange_item(
        &self,
        exchange_id: &str,
        item_id: &str,
        quantity: i32,
    ) -> Result<ExchangeItemDTO> {
        self.order_service
            .add_exchange_item(AddExchangeItemInput {
                exchange_id: exchange_id.to_string(),
                item_id: item_id.to_string(),
                quantity,
                metadata: None,
            })
            .await
    }

    /// Confirm exchange with computed difference (P6-14).
    pub async fn confirm_exchange(
        &self,
        exchange_id: &str,
        difference_due: Option<Decimal>,
    ) -> Result<ExchangeDTO> {
        self.order_service
            .confirm_exchange(exchange_id, difference_due)
            .await
    }

    /// Cancel exchange (P6-13).
    pub async fn cancel_exchange(&self, exchange_id: &str) -> Result<ExchangeDTO> {
        self.order_service.cancel_exchange(exchange_id).await
    }
}
