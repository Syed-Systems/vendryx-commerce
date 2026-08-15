//! Claims workflows: begin, add items with images, confirm replace/refund, cancel (P6-11, P6-12).
use rust_decimal::Decimal;
use std::sync::Arc;
use stheno_order::{AddClaimItemInput, ClaimDTO, ClaimItemDTO, CreateClaimInput, OrderService};
use stheno_types::Result;

#[derive(Clone)]
pub struct ClaimWorkflowEngine {
    order_service: Arc<OrderService>,
}

impl ClaimWorkflowEngine {
    pub fn new(order_service: Arc<OrderService>) -> Self {
        Self { order_service }
    }

    /// Begin a claim (P6-11).
    pub async fn begin_claim(
        &self,
        order_id: &str,
        claim_type: &str, // "refund" or "replace"
        refund_amount: Option<Decimal>,
        created_by: Option<&str>,
    ) -> Result<ClaimDTO> {
        self.order_service
            .create_claim(CreateClaimInput {
                order_id: order_id.to_string(),
                r#type: claim_type.to_string(),
                refund_amount,
                created_by: created_by.map(|s| s.to_string()),
                metadata: None,
            })
            .await
    }

    /// Add an item with reason and evidence images (P6-11).
    pub async fn add_claim_item(
        &self,
        claim_id: &str,
        item_id: &str,
        quantity: i32,
        reason: &str,
        images: Option<Vec<String>>,
    ) -> Result<ClaimItemDTO> {
        self.order_service
            .add_claim_item(AddClaimItemInput {
                claim_id: claim_id.to_string(),
                item_id: item_id.to_string(),
                quantity,
                reason: reason.to_string(),
                images,
                metadata: None,
            })
            .await
    }

    /// Confirm claim: handles replacement and refund kinds (P6-12).
    pub async fn confirm_claim(&self, claim_id: &str) -> Result<ClaimDTO> {
        self.order_service.confirm_claim(claim_id).await
    }

    /// Cancel claim (P6-11).
    pub async fn cancel_claim(&self, claim_id: &str) -> Result<ClaimDTO> {
        self.order_service.cancel_claim(claim_id).await
    }
}
