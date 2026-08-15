//! Order change cancel/decline control workflows for all change subtypes (P6-17).
use std::sync::Arc;
use stheno_order::{OrderChangeDTO, OrderService};
use stheno_types::Result;

#[derive(Clone)]
pub struct OrderChangeControlWorkflowEngine {
    order_service: Arc<OrderService>,
}

impl OrderChangeControlWorkflowEngine {
    pub fn new(order_service: Arc<OrderService>) -> Self {
        Self { order_service }
    }

    /// Cancel order change for any subtype (edit, return, claim, exchange, transfer) (P6-17).
    pub async fn cancel_change(&self, change_id: &str) -> Result<OrderChangeDTO> {
        self.order_service.cancel_order_change(change_id).await
    }

    /// Decline order change for any subtype (P6-17).
    pub async fn decline_change(
        &self,
        change_id: &str,
        declined_by: Option<&str>,
        reason: Option<&str>,
    ) -> Result<OrderChangeDTO> {
        self.order_service
            .decline_order_change(change_id, declined_by, reason)
            .await
    }
}
