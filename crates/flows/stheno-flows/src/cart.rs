//! Cart workflows and line item mutations (WORKFLOWS §2, TASKS P4-28).

use std::sync::Arc;
use stheno_cart::*;
use stheno_types::Result;

pub struct CartWorkflows {
    cart_service: Arc<CartService>,
}

impl CartWorkflows {
    pub fn new(cart_service: Arc<CartService>) -> Self {
        Self { cart_service }
    }

    // 1. create_cart_workflow
    pub async fn create_cart(&self, input: CreateCartInput) -> Result<CartDTO> {
        self.cart_service.create_cart(input).await
    }

    // 2. get_cart_workflow
    pub async fn get_cart(&self, cart_id: &str) -> Result<CartDTO> {
        self.cart_service.get_cart(cart_id).await
    }

    // 3. update_cart_workflow
    pub async fn update_cart(&self, cart_id: &str, input: UpdateCartInput) -> Result<CartDTO> {
        self.cart_service.update_cart(cart_id, input).await
    }

    // 4. delete_cart_workflow
    pub async fn delete_cart(&self, cart_id: &str) -> Result<()> {
        self.cart_service.delete_cart(cart_id).await
    }

    // 5. add_line_item_workflow (denormalizes product fields)
    pub async fn add_line_item(
        &self,
        cart_id: &str,
        input: CreateLineItemInput,
    ) -> Result<LineItemDTO> {
        self.cart_service.add_line_item(cart_id, input).await
    }

    // 6. get_line_item_workflow
    pub async fn get_line_item(&self, item_id: &str) -> Result<LineItemDTO> {
        self.cart_service.get_line_item(item_id).await
    }

    // 7. update_line_item_workflow
    pub async fn update_line_item(
        &self,
        cart_id: &str,
        item_id: &str,
        input: UpdateLineItemInput,
    ) -> Result<LineItemDTO> {
        self.cart_service
            .update_line_item(cart_id, item_id, input)
            .await
    }

    // 8. delete_line_item_workflow
    pub async fn delete_line_item(&self, cart_id: &str, item_id: &str) -> Result<()> {
        self.cart_service.delete_line_item(cart_id, item_id).await
    }

    // 9. add_shipping_method_workflow
    pub async fn add_shipping_method(
        &self,
        cart_id: &str,
        input: CreateShippingMethodInput,
    ) -> Result<ShippingMethodDTO> {
        self.cart_service.add_shipping_method(cart_id, input).await
    }

    // 10. add_credit_line_workflow
    pub async fn add_credit_line(
        &self,
        cart_id: &str,
        input: CreateCreditLineInput,
    ) -> Result<CreditLineDTO> {
        self.cart_service.add_credit_line(cart_id, input).await
    }

    // 11. delete_credit_line_workflow
    pub async fn delete_credit_line(&self, cart_id: &str, credit_line_id: &str) -> Result<()> {
        self.cart_service
            .delete_credit_line(cart_id, credit_line_id)
            .await
    }

    // 12. claim_cart_on_login_workflow (F6, F32)
    pub async fn claim_cart(
        &self,
        cart_id: &str,
        secret: &str,
        customer_id: &str,
    ) -> Result<CartDTO> {
        self.cart_service
            .claim_cart_on_login(cart_id, secret, customer_id)
            .await
    }

    // 13. set_shipping_address_workflow
    pub async fn set_shipping_address(
        &self,
        cart_id: &str,
        address: CreateAddressInput,
    ) -> Result<CartDTO> {
        self.cart_service
            .update_cart(
                cart_id,
                UpdateCartInput {
                    shipping_address: Some(address),
                    ..Default::default()
                },
            )
            .await
    }

    // 14. set_billing_address_workflow
    pub async fn set_billing_address(
        &self,
        cart_id: &str,
        address: CreateAddressInput,
    ) -> Result<CartDTO> {
        self.cart_service
            .update_cart(
                cart_id,
                UpdateCartInput {
                    billing_address: Some(address),
                    ..Default::default()
                },
            )
            .await
    }

    // 15. get_cart_with_secret_workflow
    pub async fn get_cart_with_secret(&self, cart_id: &str, secret: &str) -> Result<CartDTO> {
        self.cart_service
            .get_cart_with_secret(cart_id, secret)
            .await
    }

    // 16. cleanup_expired_carts_workflow
    pub async fn cleanup_expired_carts(&self, ttl_days: i64) -> Result<usize> {
        self.cart_service.cleanup_expired_carts(ttl_days).await
    }
}
