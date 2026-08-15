//! Order lifecycle and transfer workflows (TASKS P5-19, P5-21).
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_order::{CreateOrderInput, OrderDTO, OrderService, UpdateOrderInput};
use stheno_types::{Error, Result};

#[derive(Debug, Clone)]
pub struct OrderTransferRequest {
    pub order_id: String,
    pub original_customer_id: String,
    pub target_customer_id: String,
    pub status: String, // "pending", "accepted", "declined", "canceled"
}

#[derive(Clone, Default)]
pub struct OrderWorkflowEngine {
    pub order_service: Arc<OrderService>,
    transfers: Arc<RwLock<HashMap<String, OrderTransferRequest>>>,
}

impl OrderWorkflowEngine {
    pub fn new(order_service: Arc<OrderService>) -> Self {
        Self {
            order_service,
            transfers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ── Order Lifecycle Workflows (P5-19) ───────────────────────────────────

    pub async fn create_order_flow(&self, input: CreateOrderInput) -> Result<OrderDTO> {
        self.order_service.create_order(input).await
    }

    pub async fn update_order_flow(
        &self,
        order_id: &str,
        input: UpdateOrderInput,
    ) -> Result<OrderDTO> {
        self.order_service.update_order(order_id, input).await
    }

    pub async fn cancel_order_flow(&self, order_id: &str) -> Result<OrderDTO> {
        self.order_service.cancel_order(order_id).await
    }

    pub async fn archive_order_flow(&self, order_id: &str) -> Result<OrderDTO> {
        self.order_service.archive_order(order_id).await
    }

    pub async fn complete_order_flow(&self, order_id: &str) -> Result<OrderDTO> {
        self.order_service.complete_order(order_id).await
    }

    // ── Order Transfer Workflows (P5-21) ────────────────────────────────────

    pub async fn request_order_transfer(
        &self,
        order_id: &str,
        target_customer_id: &str,
    ) -> Result<OrderTransferRequest> {
        let order = self.order_service.get_order(order_id).await?;
        let original_customer_id = order.customer_id.unwrap_or_default();

        let req = OrderTransferRequest {
            order_id: order_id.to_string(),
            original_customer_id,
            target_customer_id: target_customer_id.to_string(),
            status: "pending".to_string(),
        };

        {
            let mut map = self.transfers.write().unwrap();
            map.insert(order_id.to_string(), req.clone());
        }

        Ok(req)
    }

    pub async fn accept_order_transfer(&self, order_id: &str) -> Result<OrderDTO> {
        let transfer = {
            let map = self.transfers.read().unwrap();
            map.get(order_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "order_transfer",
                id: order_id.to_string(),
            })?
        };

        if transfer.status != "pending" {
            return Err(Error::Conflict {
                code: "transfer_not_pending",
                message: format!("Transfer is already {}", transfer.status),
            });
        }

        let updated_order = self
            .order_service
            .update_order(
                order_id,
                UpdateOrderInput {
                    customer_id: Some(transfer.target_customer_id),
                    ..Default::default()
                },
            )
            .await?;

        {
            let mut map = self.transfers.write().unwrap();
            if let Some(t) = map.get_mut(order_id) {
                t.status = "accepted".to_string();
            }
        }

        Ok(updated_order)
    }

    pub async fn decline_order_transfer(&self, order_id: &str) -> Result<OrderTransferRequest> {
        let mut map = self.transfers.write().unwrap();
        let transfer = map.get_mut(order_id).ok_or_else(|| Error::NotFound {
            entity: "order_transfer",
            id: order_id.to_string(),
        })?;

        if transfer.status != "pending" {
            return Err(Error::Conflict {
                code: "transfer_not_pending",
                message: format!("Transfer is already {}", transfer.status),
            });
        }

        transfer.status = "declined".to_string();
        Ok(transfer.clone())
    }

    pub async fn cancel_order_transfer(&self, order_id: &str) -> Result<OrderTransferRequest> {
        let mut map = self.transfers.write().unwrap();
        let transfer = map.get_mut(order_id).ok_or_else(|| Error::NotFound {
            entity: "order_transfer",
            id: order_id.to_string(),
        })?;

        transfer.status = "canceled".to_string();
        Ok(transfer.clone())
    }
}
