//! Return workflows: request, receive with partial/damaged, dismiss/write-off (P6-07 to P6-10).
use rust_decimal::Decimal;
use std::sync::Arc;
use stheno_inventory::InventoryService;
use stheno_order::{
    AddReturnItemInput, CreateReturnInput, OrderService, ReceiveReturnItemsInput, ReturnDTO,
    ReturnItemDTO,
};
use stheno_types::Result;

#[derive(Clone)]
pub struct ReturnWorkflowEngine {
    order_service: Arc<OrderService>,
    _inventory_service: Option<Arc<InventoryService>>,
}

impl ReturnWorkflowEngine {
    pub fn new(
        order_service: Arc<OrderService>,
        inventory_service: Option<Arc<InventoryService>>,
    ) -> Self {
        Self {
            order_service,
            _inventory_service: inventory_service,
        }
    }

    /// Request a return (P6-07).
    pub async fn request_return(
        &self,
        order_id: &str,
        location_id: Option<&str>,
        refund_amount: Option<Decimal>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ReturnDTO> {
        self.order_service
            .create_return(CreateReturnInput {
                order_id: order_id.to_string(),
                location_id: location_id.map(|s| s.to_string()),
                refund_amount,
                metadata,
            })
            .await
    }

    /// Add an item to return (P6-07).
    pub async fn add_return_item(
        &self,
        return_id: &str,
        item_id: &str,
        quantity: i32,
        reason_id: Option<&str>,
        note: Option<&str>,
    ) -> Result<ReturnItemDTO> {
        self.order_service
            .add_return_item(AddReturnItemInput {
                return_id: return_id.to_string(),
                item_id: item_id.to_string(),
                quantity,
                reason_id: reason_id.map(|s| s.to_string()),
                note: note.map(|s| s.to_string()),
                metadata: None,
            })
            .await
    }

    /// Receive return items with partial & damaged tracking and inventory restock (P6-08, P6-09).
    pub async fn receive_return_items(
        &self,
        return_id: &str,
        items: Vec<(String, i32, Option<i32>)>, // (item_id, received_qty, damaged_qty)
    ) -> Result<ReturnDTO> {
        let receive_inputs = items
            .into_iter()
            .map(
                |(item_id, received_quantity, damaged_quantity)| ReceiveReturnItemsInput {
                    item_id,
                    received_quantity,
                    damaged_quantity,
                },
            )
            .collect();

        self.order_service
            .receive_return_items(return_id, receive_inputs)
            .await
    }

    /// Confirm return and issue refund (P6-08).
    pub async fn confirm_return(
        &self,
        return_id: &str,
        refund_amount: Option<Decimal>,
    ) -> Result<ReturnDTO> {
        self.order_service
            .confirm_return(return_id, refund_amount)
            .await
    }

    /// Cancel return (P6-08).
    pub async fn cancel_return(&self, return_id: &str) -> Result<ReturnDTO> {
        self.order_service.cancel_return(return_id).await
    }

    /// Return dismiss / write-off (P6-10).
    pub async fn dismiss_return(&self, return_id: &str, reason: Option<&str>) -> Result<ReturnDTO> {
        self.order_service.dismiss_return(return_id, reason).await
    }
}
