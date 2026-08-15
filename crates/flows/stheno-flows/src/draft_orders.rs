//! Draft order creation, editing, and live conversion workflows (P6-15, P6-16).
use std::sync::Arc;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateShippingMethodInput, OrderDTO,
    OrderService,
};
use stheno_types::Result;

#[derive(Clone)]
pub struct DraftOrderWorkflowEngine {
    order_service: Arc<OrderService>,
}

impl DraftOrderWorkflowEngine {
    pub fn new(order_service: Arc<OrderService>) -> Self {
        Self { order_service }
    }

    /// Create a draft order (P6-15).
    #[allow(clippy::too_many_arguments)]
    pub async fn create_draft_order(
        &self,
        customer_id: Option<&str>,
        email: Option<&str>,
        region_id: Option<&str>,
        currency_code: &str,
        items: Vec<CreateLineItemInput>,
        shipping_methods: Vec<CreateShippingMethodInput>,
        shipping_address: Option<CreateAddressInput>,
        billing_address: Option<CreateAddressInput>,
    ) -> Result<OrderDTO> {
        self.order_service
            .create_order(CreateOrderInput {
                customer_id: customer_id.map(|s| s.to_string()),
                sales_channel_id: None,
                region_id: region_id.map(|s| s.to_string()),
                currency_code: currency_code.to_string(),
                shipping_address,
                billing_address,
                email: email.map(|s| s.to_string()),
                items: Some(items),
                shipping_methods: Some(shipping_methods),
                is_draft_order: Some(true),
                metadata: None,
            })
            .await
    }

    /// Convert draft order to live order (P6-16).
    pub async fn convert_draft_order(&self, order_id: &str) -> Result<OrderDTO> {
        self.order_service.convert_draft_order(order_id).await
    }
}
