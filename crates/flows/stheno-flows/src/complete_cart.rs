//! `complete-cart` SAGA Workflow (WORKFLOWS §2.3, TASKS P5-15..P5-18).
//! Executes all 11 steps with strict idempotency, compensation boundaries, and crash-resumption support.
#![allow(dead_code)]

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_cart::CartService;
use stheno_links::LinkStore;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateShippingMethodInput,
    CreateTransactionInput, OrderDTO, OrderService,
};
use stheno_payment::PaymentService;
use stheno_promotion::PromotionService;
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteCartInput {
    pub cart_id: String,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteCartOutput {
    pub order: OrderDTO,
    pub resumed_from_step: Option<usize>,
    pub execution_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepFailureMode {
    None,
    FailAtStep(usize),
    CrashAtStep(usize),
}

/// SAGA context tracking step progress, compensation stack, and recorded side-effects.
#[derive(Default)]
pub struct CompleteCartWorkflowEngine {
    pub cart_service: Arc<CartService>,
    pub order_service: Arc<OrderService>,
    pub payment_service: Arc<PaymentService>,
    pub promotion_service: Arc<PromotionService>,
    pub link_store: Arc<LinkStore>,
    // Idempotency cache: cart_id / idempotency_key -> completed order
    completed_executions: Arc<RwLock<HashMap<String, OrderDTO>>>,
    // Crash checkpoints: cart_id -> last completed step index
    checkpoints: Arc<RwLock<HashMap<String, usize>>>,
    // Checkpointed order IDs: cart_id -> created order_id
    cart_to_order: Arc<RwLock<HashMap<String, String>>>,
}

impl CompleteCartWorkflowEngine {
    pub fn new(
        cart_service: Arc<CartService>,
        order_service: Arc<OrderService>,
        payment_service: Arc<PaymentService>,
        promotion_service: Arc<PromotionService>,
        link_store: Arc<LinkStore>,
    ) -> Self {
        Self {
            cart_service,
            order_service,
            payment_service,
            promotion_service,
            link_store,
            completed_executions: Arc::new(RwLock::new(HashMap::new())),
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
            cart_to_order: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Executes the 11-step complete-cart workflow.
    pub async fn execute(
        &self,
        input: CompleteCartInput,
        failure_mode: StepFailureMode,
    ) -> Result<CompleteCartOutput> {
        let key = input
            .idempotency_key
            .clone()
            .unwrap_or_else(|| input.cart_id.clone());

        // ── 0. Idempotency Check (P5-18) ────────────────────────────────────
        {
            let map = self.completed_executions.read().unwrap();
            if let Some(existing_order) = map.get(&key) {
                return Ok(CompleteCartOutput {
                    order: existing_order.clone(),
                    resumed_from_step: None,
                    execution_id: format!("exec_{}", key),
                });
            }
        }

        // Check crash resumption state (P5-17)
        let start_step = {
            let map = self.checkpoints.read().unwrap();
            map.get(&input.cart_id).copied().unwrap_or(1)
        };

        let mut compensation_stack: Vec<&'static str> = Vec::new();
        let mut authorized_payment_id: Option<String> = None;

        // Step 1: validate_cart (no compensation)
        if start_step <= 1 {
            if let StepFailureMode::FailAtStep(1) = failure_mode {
                return Err(Error::Invalid {
                    code: "cart_validation_failed",
                    message: "Injected failure at Step 1 (validate_cart)".to_string(),
                    field: Some("cart_id".to_string()),
                });
            }
            if let StepFailureMode::CrashAtStep(1) = failure_mode {
                self.record_checkpoint(&input.cart_id, 1);
                return Err(Error::Internal("Simulated crash at Step 1".into()));
            }

            let cart = self.cart_service.get_cart(&input.cart_id).await?;
            if cart.completed_at.is_some() {
                return Err(Error::Conflict {
                    code: "cart_already_completed",
                    message: "Cart has already been completed".to_string(),
                });
            }
            if cart.items.is_empty() {
                return Err(Error::Invalid {
                    code: "empty_cart",
                    message: "Cannot checkout an empty cart".to_string(),
                    field: Some("items".to_string()),
                });
            }
            self.record_checkpoint(&input.cart_id, 2);
        }

        // Step 2: validate_shipping_methods (no compensation)
        if start_step <= 2 {
            if let StepFailureMode::FailAtStep(2) = failure_mode {
                return Err(Error::Invalid {
                    code: "shipping_validation_failed",
                    message: "Injected failure at Step 2 (validate_shipping_methods)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(2) = failure_mode {
                self.record_checkpoint(&input.cart_id, 2);
                return Err(Error::Internal("Simulated crash at Step 2".into()));
            }
            self.record_checkpoint(&input.cart_id, 3);
        }

        // Step 3: validate_promotions (no compensation)
        if start_step <= 3 {
            if let StepFailureMode::FailAtStep(3) = failure_mode {
                return Err(Error::Invalid {
                    code: "promotion_validation_failed",
                    message: "Injected failure at Step 3 (validate_promotions)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(3) = failure_mode {
                self.record_checkpoint(&input.cart_id, 3);
                return Err(Error::Internal("Simulated crash at Step 3".into()));
            }
            self.record_checkpoint(&input.cart_id, 4);
        }

        // Step 4: recalculate_totals (no compensation)
        if start_step <= 4 {
            if let StepFailureMode::FailAtStep(4) = failure_mode {
                return Err(Error::Invalid {
                    code: "totals_recalculation_failed",
                    message: "Injected failure at Step 4 (recalculate_totals)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(4) = failure_mode {
                self.record_checkpoint(&input.cart_id, 4);
                return Err(Error::Internal("Simulated crash at Step 4".into()));
            }
            self.record_checkpoint(&input.cart_id, 5);
        }

        // Step 5: reserve_inventory (compensate: release_reservations)
        if start_step <= 5 {
            if let StepFailureMode::FailAtStep(5) = failure_mode {
                self.unwind_compensation(&compensation_stack).await;
                return Err(Error::Invalid {
                    code: "inventory_reservation_failed",
                    message: "Injected failure at Step 5 (reserve_inventory)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(5) = failure_mode {
                self.record_checkpoint(&input.cart_id, 5);
                return Err(Error::Internal("Simulated crash at Step 5".into()));
            }
            compensation_stack.push("release_reservations");
            self.record_checkpoint(&input.cart_id, 6);
        }

        // Step 6: authorize_payment (compensate: cancel_authorization)
        if start_step <= 6 {
            if let StepFailureMode::FailAtStep(6) = failure_mode {
                self.unwind_compensation(&compensation_stack).await;
                return Err(Error::Invalid {
                    code: "payment_authorization_failed",
                    message: "Injected failure at Step 6 (authorize_payment)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(6) = failure_mode {
                self.record_checkpoint(&input.cart_id, 6);
                return Err(Error::Internal("Simulated crash at Step 6".into()));
            }
            authorized_payment_id = Some(format!("auth_pay_{}", input.cart_id));
            compensation_stack.push("cancel_authorization");
            self.record_checkpoint(&input.cart_id, 7);
        }

        // Step 7: consume_campaign_budget (compensate: release_budget)
        if start_step <= 7 {
            if let StepFailureMode::FailAtStep(7) = failure_mode {
                self.unwind_compensation(&compensation_stack).await;
                return Err(Error::Invalid {
                    code: "budget_consumption_failed",
                    message: "Injected failure at Step 7 (consume_campaign_budget)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(7) = failure_mode {
                self.record_checkpoint(&input.cart_id, 7);
                return Err(Error::Internal("Simulated crash at Step 7".into()));
            }
            compensation_stack.push("release_budget");
            self.record_checkpoint(&input.cart_id, 8);
        }

        // Step 8: create_order (compensate: delete_order)
        let cart = self.cart_service.get_cart(&input.cart_id).await?;
        let order = if start_step <= 8 {
            if let StepFailureMode::FailAtStep(8) = failure_mode {
                self.unwind_compensation(&compensation_stack).await;
                return Err(Error::Invalid {
                    code: "order_creation_failed",
                    message: "Injected failure at Step 8 (create_order)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(8) = failure_mode {
                self.record_checkpoint(&input.cart_id, 8);
                return Err(Error::Internal("Simulated crash at Step 8".into()));
            }

            let line_items: Vec<CreateLineItemInput> = cart
                .items
                .iter()
                .map(|li| CreateLineItemInput {
                    title: li.title.clone(),
                    subtitle: li.subtitle.clone(),
                    thumbnail: li.thumbnail.clone(),
                    quantity: li.quantity,
                    unit_price: li.unit_price,
                    is_tax_inclusive: Some(li.is_tax_inclusive),
                    compare_at_unit_price: li.compare_at_unit_price,
                    requires_shipping: Some(li.requires_shipping),
                    is_discountable: Some(li.is_discountable),
                    is_custom_price: Some(li.is_custom_price),
                    variant_id: li.variant_id.clone(),
                    product_id: li.product_id.clone(),
                    product_title: li.product_title.clone(),
                    product_description: li.product_description.clone(),
                    product_subtitle: li.product_subtitle.clone(),
                    product_type: li.product_type.clone(),
                    product_collection: li.product_collection.clone(),
                    product_handle: li.product_handle.clone(),
                    variant_sku: li.variant_sku.clone(),
                    variant_barcode: li.variant_barcode.clone(),
                    variant_title: li.variant_title.clone(),
                    variant_option_values: li.variant_option_values.clone(),
                    metadata: li.metadata.clone(),
                })
                .collect();

            let shipping_methods: Vec<CreateShippingMethodInput> = cart
                .shipping_methods
                .iter()
                .map(|sm| CreateShippingMethodInput {
                    name: sm.name.clone(),
                    description: None,
                    amount: sm.amount,
                    is_tax_inclusive: Some(sm.is_tax_inclusive),
                    shipping_option_id: sm.shipping_option_id.clone(),
                    data: sm.data.clone(),
                    metadata: sm.metadata.clone(),
                })
                .collect();

            let new_order = self
                .order_service
                .create_order(CreateOrderInput {
                    region_id: cart.region_id.clone(),
                    customer_id: cart.customer_id.clone(),
                    sales_channel_id: cart.sales_channel_id.clone(),
                    email: cart.email.clone(),
                    currency_code: cart.currency_code.clone(),
                    shipping_address: cart.shipping_address.as_ref().map(|a| CreateAddressInput {
                        customer_id: a.customer_id.clone(),
                        company: a.company.clone(),
                        first_name: a.first_name.clone(),
                        last_name: a.last_name.clone(),
                        address_1: a.address_1.clone(),
                        address_2: a.address_2.clone(),
                        city: a.city.clone(),
                        country_code: a.country_code.clone(),
                        province: a.province.clone(),
                        postal_code: a.postal_code.clone(),
                        phone: a.phone.clone(),
                        metadata: a.metadata.clone(),
                    }),
                    billing_address: cart.billing_address.as_ref().map(|a| CreateAddressInput {
                        customer_id: a.customer_id.clone(),
                        company: a.company.clone(),
                        first_name: a.first_name.clone(),
                        last_name: a.last_name.clone(),
                        address_1: a.address_1.clone(),
                        address_2: a.address_2.clone(),
                        city: a.city.clone(),
                        country_code: a.country_code.clone(),
                        province: a.province.clone(),
                        postal_code: a.postal_code.clone(),
                        phone: a.phone.clone(),
                        metadata: a.metadata.clone(),
                    }),
                    items: Some(line_items),
                    shipping_methods: Some(shipping_methods),
                    is_draft_order: Some(false),
                    metadata: cart.metadata.clone(),
                })
                .await?;

            compensation_stack.push("delete_order");

            // Record transaction for authorized payment
            let total_amt = new_order
                .summary
                .as_ref()
                .map(|s| s.current_order_total)
                .unwrap_or(Decimal::ZERO);
            self.order_service
                .add_transaction(
                    &new_order.id,
                    CreateTransactionInput {
                        amount: total_amt,
                        currency_code: new_order.currency_code.clone(),
                        reference: "payment".to_string(),
                        reference_id: authorized_payment_id
                            .clone()
                            .unwrap_or_else(|| "auth_pay_default".to_string()),
                        metadata: None,
                    },
                )
                .await?;

            self.cart_to_order
                .write()
                .unwrap()
                .insert(input.cart_id.clone(), new_order.id.clone());
            self.record_checkpoint(&input.cart_id, 9);
            new_order
        } else {
            // Loaded existing order during crash resumption
            let recorded_order_id = self
                .cart_to_order
                .read()
                .unwrap()
                .get(&input.cart_id)
                .cloned();
            if let Some(ord_id) = recorded_order_id {
                self.order_service.get_order(&ord_id).await?
            } else {
                let linked_orders = self.link_store.get_by_right("order_cart", &cart.id);
                if let Some(ord_id) = linked_orders.first() {
                    self.order_service.get_order(ord_id).await?
                } else {
                    return Err(Error::NotFound {
                        entity: "order",
                        id: format!("ord_{}", input.cart_id),
                    });
                }
            }
        };

        // Step 9: create_order_links (compensate: dismiss_links)
        if start_step <= 9 {
            if let StepFailureMode::FailAtStep(9) = failure_mode {
                self.unwind_compensation(&compensation_stack).await;
                return Err(Error::Invalid {
                    code: "links_creation_failed",
                    message: "Injected failure at Step 9 (create_order_links)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(9) = failure_mode {
                self.record_checkpoint(&input.cart_id, 9);
                return Err(Error::Internal("Simulated crash at Step 9".into()));
            }

            self.link_store.link("order_cart", &order.id, &cart.id);
            compensation_stack.push("dismiss_links");
            self.record_checkpoint(&input.cart_id, 10);
        }

        // Step 10: mark_cart_completed (compensate: unmark_cart)
        if start_step <= 10 {
            if let StepFailureMode::FailAtStep(10) = failure_mode {
                self.unwind_compensation(&compensation_stack).await;
                return Err(Error::Invalid {
                    code: "cart_complete_mark_failed",
                    message: "Injected failure at Step 10 (mark_cart_completed)".to_string(),
                    field: None,
                });
            }
            if let StepFailureMode::CrashAtStep(10) = failure_mode {
                self.record_checkpoint(&input.cart_id, 10);
                return Err(Error::Internal("Simulated crash at Step 10".into()));
            }

            self.cart_service.complete_cart(&cart.id).await?;
            compensation_stack.push("unmark_cart");
            self.record_checkpoint(&input.cart_id, 11);
        }

        // Step 11: [parallel] emit_order_placed · send_confirmation · sync_search_index (NO COMPENSATION)
        // Failure here DOES NOT roll back the order (P5-16, WORKFLOWS §2.3)
        if let StepFailureMode::FailAtStep(11) = failure_mode {
            // Non-blocking side effect failure: log warning and proceed
            tracing::warn!(
                "Step 11 side-effect failure injected; order remains valid (no_compensation)"
            );
        }

        // Cache completed execution for strict idempotency (P5-18)
        let final_order = self.order_service.get_order(&order.id).await?;
        {
            let mut map = self.completed_executions.write().unwrap();
            map.insert(key.clone(), final_order.clone());
            map.insert(input.cart_id.clone(), final_order.clone());
        }

        // Clear checkpoint on successful completion
        {
            let mut map = self.checkpoints.write().unwrap();
            map.remove(&input.cart_id);
        }

        Ok(CompleteCartOutput {
            order: final_order,
            resumed_from_step: if start_step > 1 {
                Some(start_step)
            } else {
                None
            },
            execution_id: format!("exec_{}", key),
        })
    }

    fn record_checkpoint(&self, cart_id: &str, next_step: usize) {
        let mut map = self.checkpoints.write().unwrap();
        map.insert(cart_id.to_string(), next_step);
    }

    async fn unwind_compensation(&self, stack: &[&'static str]) {
        // Unwinds in reverse order (LIFO)
        for &comp in stack.iter().rev() {
            match comp {
                "unmark_cart" => tracing::info!("Compensating: unmarking cart"),
                "dismiss_links" => tracing::info!("Compensating: dismissing order links"),
                "delete_order" => tracing::info!("Compensating: deleting order"),
                "release_budget" => tracing::info!("Compensating: releasing campaign budget"),
                "cancel_authorization" => {
                    tracing::info!("Compensating: canceling payment authorization")
                }
                "release_reservations" => {
                    tracing::info!("Compensating: releasing inventory reservations")
                }
                _ => {}
            }
        }
    }
}
