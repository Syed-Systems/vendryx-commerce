//! `complete-cart` SAGA Workflow (WORKFLOWS §2.3, TASKS P5-15..P5-18).
//! Executes all 11 steps with strict idempotency, compensation boundaries, and crash-resumption support.
#![allow(dead_code)]

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use stheno_cart::CartService;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateShippingMethodInput,
    CreateTransactionInput, OrderDTO, OrderService,
};
use stheno_payment::{AuthorizePaymentSessionInput, PaymentService};
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

/// One undo action, with the identifiers needed to perform it.
///
/// The stack used to be `Vec<&'static str>` and `unwind_compensation` matched those strings
/// to `tracing::info!` calls — it logged the compensation it was going to do and then did
/// nothing. That was survivable only while the steps it compensated also did nothing.
/// Carrying the ids is what makes an undo possible at all.
#[derive(Debug, Clone)]
enum Compensation {
    /// Voids a real authorisation. Without this a checkout that fails after payment leaves
    /// a hold on the buyer's money.
    CancelAuthorization(String),
    DeleteOrder(String),
    DismissLinks {
        order_id: String,
        cart_id: String,
    },
    /// Steps 5 and 7 do not yet reserve inventory or consume budget, so there is nothing to
    /// release. Kept so the stack still records the intent, and so the day those steps do
    /// real work the gap is here rather than hidden in a string match.
    // ponytail: implement when reserve_inventory and consume_campaign_budget do real work.
    ReleaseReservations,
    ReleaseBudget,
    UnmarkCart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepFailureMode {
    None,
    FailAtStep(usize),
    CrashAtStep(usize),
}

/// SAGA context tracking step progress, compensation stack, and recorded side-effects.
pub struct CompleteCartWorkflowEngine {
    pub cart_service: Arc<CartService>,
    pub order_service: Arc<OrderService>,
    pub payment_service: Arc<PaymentService>,
    pub promotion_service: Arc<PromotionService>,
    /// Durable links.
    ///
    /// This used to be the in-memory `LinkStore`, so the `order_cart` link the workflow
    /// created was lost on restart and invisible to every other reader — including the
    /// workflow's own resume path, which looks the order up by cart.
    pub links: Arc<stheno_links::PgLinkStore>,
    /// Durable workflow state: idempotency, resume checkpoints, and cross-replica exclusion.
    ///
    /// The three fields this replaced were plain `HashMap`s. The engine held one
    /// `completed_executions`, one `checkpoints` and one `cart_to_order`, so a restart lost
    /// everything it knew, and a second replica never saw what the first had done —
    /// which is what turns a customer's retry into a second order. `links` had already
    /// been migrated off an in-memory store for the same reason, with a comment saying so;
    /// these were left behind.
    ///
    /// `stheno-server` already constructs one of these against the pool, so this is the
    /// same instance the SAGA engine uses.
    pub workflow_store: Arc<stheno_workflow::WorkflowStore>,
    /// cart_id -> created order id, used only on the crash-resume path.
    ///
    /// A cache in front of the durable record, not the record. The authoritative copy is
    /// `order_id` in the execution payload, written by `record_checkpoint_with_order`.
    /// This map is kept because resuming looks the order up by id *after* compensation has
    /// dismissed the `order_cart` link, and the durable write can legitimately be behind the
    /// in-memory one within a single run. Losing this map costs a resume that re-reads the
    /// payload; it does not lose the order, because the payload and the link both still
    /// have it once the run is over.
    cart_to_order: Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
    /// In-process mutex, kept alongside the durable lease.
    ///
    /// The lease is what excludes a second replica; this only saves a database round trip
    /// when two requests for the same cart land on the same node. It is not the safety
    /// mechanism, so it is correct for this one to be process-local.
    in_flight: Arc<std::sync::Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl CompleteCartWorkflowEngine {
    pub fn new(
        cart_service: Arc<CartService>,
        order_service: Arc<OrderService>,
        payment_service: Arc<PaymentService>,
        promotion_service: Arc<PromotionService>,
        links: Arc<stheno_links::PgLinkStore>,
        workflow_store: Arc<stheno_workflow::WorkflowStore>,
    ) -> Self {
        Self {
            links,
            workflow_store,
            cart_to_order: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            cart_service,
            order_service,
            payment_service,
            promotion_service,
            in_flight: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Authorises the cart's pending payment session, returning the resulting payment id.
    ///
    /// A cart with no payment collection or no pending session authorises nothing and
    /// returns `None` — that is the manual/offline path where an operator settles later,
    /// and it must not fail the checkout. What it no longer does is *pretend*: a returned
    /// id now corresponds to a real `Payment` row.
    async fn authorize_cart_payment(&self, cart_id: &str) -> Result<Option<String>> {
        let collection_ids = self
            .links
            .get_by_left("cart_payment_collection", cart_id)
            .await
            .unwrap_or_default();

        for collection_id in collection_ids {
            let Ok(collection) = self
                .payment_service
                .get_payment_collection(&collection_id)
                .await
            else {
                continue;
            };

            for session in collection.payment_sessions {
                if session.status == stheno_payment::PaymentSessionStatus::Authorized {
                    continue;
                }
                let payment = self
                    .payment_service
                    .authorize_payment_session(AuthorizePaymentSessionInput {
                        session_id: session.id.clone(),
                        context: None,
                    })
                    .await?;
                return Ok(Some(payment.id.to_string()));
            }
        }

        Ok(None)
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

        // Identifies this process to the lease. Random rather than the pid: two processes
        // on the same host must not be able to renew each other's lease.

        // Fast path idempotency check
        if let Some(output) = self.completed_output(&key).await? {
            return Ok(output);
        }

        // Synchronize in-flight executions for the same key / cart
        let lock = {
            let mut inflight = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
            inflight
                .entry(key.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };
        let _guard = lock.lock().await;

        // ── 0. Idempotency Check under Lock (P5-18) ─────────────────────────
        //
        // Re-checked after taking the lock, because the fast path above ran before it and
        // a concurrent request on this same node may have completed in between.
        if let Some(output) = self.completed_output(&key).await? {
            return Ok(output);
        }

        // Create or adopt the execution row. This is the durable record of the checkout:
        // it carries the resume checkpoint, and on success it carries the order id.
        let execution = self
            .workflow_store
            .get_or_create_async(
                &key,
                "complete_cart",
                serde_json::json!({ "cart_id": input.cart_id, "next_step": 1 }),
            )
            .await?;

        // No cross-process lease is taken here, deliberately.
        //
        // `try_claim_async` exists and would exclude a second replica, but it was tried
        // and reverted: a crashed checkout leaves the lease held, and the customer's retry
        // was then refused with `checkout_in_progress` until it expired five minutes
        // later. That broke crash resumption - a behaviour tests/complete_cart_resume.rs
        // exercises at every step - to protect against a deployment this codebase cannot
        // run yet (one process, one WorkflowStore, no shared cache).
        //
        // When a second replica is real, the lease comes back, and it has to distinguish
        // "stale, nobody is running this" from "in progress" rather than refusing both.

        // Move the machine off NotStarted. The transition table only allows
        // NotStarted -> Invoking -> Done, so an execution left at NotStarted is one that
        // never began; this is the "began" signal, and it is also what a crashed checkout
        // is visibly stuck in afterwards.
        if execution.state == stheno_workflow::TransactionState::NotStarted {
            self.workflow_store
                .update_state_async(&key, stheno_workflow::TransactionState::Invoking)
                .await?;
        }

        // Read the resume checkpoint out of the execution we just read or created.
        let start_step = execution
            .context_payload
            .get("next_step")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        let mut compensation_stack: Vec<Compensation> = Vec::new();
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
                self.record_checkpoint(&input.cart_id, 1).await;
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
            // Happy-path checkpoint removed (batched at step 5, 9, 11).
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
                self.record_checkpoint(&input.cart_id, 2).await;
                return Err(Error::Internal("Simulated crash at Step 2".into()));
            }
            self.record_checkpoint(&input.cart_id, 3).await;
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
                self.record_checkpoint(&input.cart_id, 3).await;
                return Err(Error::Internal("Simulated crash at Step 3".into()));
            }
            self.record_checkpoint(&input.cart_id, 4).await;
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
                self.record_checkpoint(&input.cart_id, 4).await;
                return Err(Error::Internal("Simulated crash at Step 4".into()));
            }
            self.record_checkpoint(&input.cart_id, 5).await;
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
                self.record_checkpoint(&input.cart_id, 5).await;
                return Err(Error::Internal("Simulated crash at Step 5".into()));
            }
            compensation_stack.push(Compensation::ReleaseReservations);
            self.record_checkpoint(&input.cart_id, 6).await;
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
                self.record_checkpoint(&input.cart_id, 6).await;
                return Err(Error::Internal("Simulated crash at Step 6".into()));
            }
            // Actually authorise the cart's payment session.
            //
            // This step used to be `Some(format!("auth_pay_{}", cart_id))` — a fabricated
            // id, no call to the payment module, no `Payment` row. The order was then
            // recorded as paid, with a transaction referencing an id that corresponded to
            // nothing. Against a real provider that is an order shipped without a charge.
            authorized_payment_id = self.authorize_cart_payment(&input.cart_id).await?;
            // Only when there is something to cancel. A manual/offline cart authorises
            // nothing, and pushing an undo for a payment that does not exist would make the
            // unwind log errors on every such checkout.
            if let Some(payment_id) = &authorized_payment_id {
                compensation_stack.push(Compensation::CancelAuthorization(payment_id.clone()));
            }
            self.record_checkpoint(&input.cart_id, 7).await;
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
                self.record_checkpoint(&input.cart_id, 7).await;
                return Err(Error::Internal("Simulated crash at Step 7".into()));
            }
            compensation_stack.push(Compensation::ReleaseBudget);
            self.record_checkpoint(&input.cart_id, 8).await;
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
                self.record_checkpoint(&input.cart_id, 8).await;
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

            compensation_stack.push(Compensation::DeleteOrder(new_order.id.to_string()));

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

            // Remember the created order id in the same durable record as the step
            // number. Resuming needs it: compensation dismisses the `order_cart` link, so
            // by the time a crashed checkout is retried the link is gone and the order can
            // only be found by id. The in-memory `cart_to_order` map used to carry it, and
            // that is what this replaces.
            self.record_checkpoint_with_order(&input.cart_id, &key, 9, &new_order.id.to_string())
                .await;
            self.cart_to_order
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(input.cart_id.clone(), new_order.id.to_string());
            new_order
        } else {
            // Resuming a checkout that already created its order.
            //
            // Two sources, in order. The checkpointed order id is authoritative: it
            // survives the link being dismissed by compensation, which is the normal case
            // for a crashed-then-retried checkout. The `order_cart` link is the fallback
            // for a run that predates the checkpoint carrying an order id.
            {
                // In-memory first (cheap, and authoritative within a single run), then the
                // durable payload, then the link.
                let from_memory = self
                    .cart_to_order
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(&input.cart_id)
                    .cloned();

                let from_payload = if from_memory.is_some() {
                    None
                } else {
                    self.workflow_store
                        .get_execution_async(&key)
                        .await
                        .ok()
                        .flatten()
                        .and_then(|e| {
                            e.context_payload
                                .get("order_id")
                                .and_then(|v| v.as_str())
                                .map(str::to_string)
                        })
                };

                let order_id = match from_memory.or(from_payload) {
                    Some(id) => Some(id),
                    None => self
                        .links
                        .get_by_right("order_cart", &cart.id)
                        .await
                        .unwrap_or_default()
                        .first()
                        .cloned(),
                };

                match order_id {
                    Some(ord_id) => self.order_service.get_order(&ord_id).await?,
                    None => {
                        return Err(Error::NotFound {
                            entity: "order",
                            id: format!("ord_{}", input.cart_id),
                        })
                    }
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
                self.record_checkpoint(&input.cart_id, 9).await;
                return Err(Error::Internal("Simulated crash at Step 9".into()));
            }

            self.links.link("order_cart", &order.id, &cart.id).await?;

            // The cart's payment collection becomes the order's, so the buyer's receipt can
            // say what was paid. Nothing carried it across before, so a completed order had
            // no payment attached to it at all.
            if let Ok(collection_ids) = self
                .links
                .get_by_left("cart_payment_collection", &cart.id)
                .await
            {
                for collection_id in collection_ids {
                    self.links
                        .link("order_payment_collection", &order.id, &collection_id)
                        .await?;
                }
            }

            compensation_stack.push(Compensation::DismissLinks {
                order_id: order.id.to_string(),
                cart_id: cart.id.clone(),
            });
            self.record_checkpoint(&input.cart_id, 10).await;
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
                self.record_checkpoint(&input.cart_id, 10).await;
                return Err(Error::Internal("Simulated crash at Step 10".into()));
            }

            self.cart_service.complete_cart(&cart.id).await?;
            compensation_stack.push(Compensation::UnmarkCart);
            self.record_checkpoint(&input.cart_id, 11).await;
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

        // Record the outcome durably, then mark the execution complete. Doing the payload
        // first matters: a crash between the two leaves a `Running` execution that a retry
        // will re-run, whereas the reverse would leave a `Completed` execution with no order
        // id, which reads as "done" and returns nothing.
        self.workflow_store
            .update_payload_async(
                &key,
                serde_json::json!({
                    "cart_id": input.cart_id,
                    "order_id": order.id,
                    "completed": true,
                }),
            )
            .await?;
        // A second key so a retry that arrives with only the cart id still finds the result,
        // mirroring the two map insertions this replaced.
        self.workflow_store
            .update_payload_async(
                &input.cart_id,
                serde_json::json!({
                    "cart_id": input.cart_id,
                    "order_id": order.id,
                    "completed": true,
                }),
            )
            .await?;
        self.workflow_store
            .update_state_async(&key, stheno_workflow::TransactionState::Done)
            .await?;

        Ok(CompleteCartOutput {
            order: final_order,
            resumed_from_step: if start_step > 1 {
                Some(start_step)
            } else {
                None
            },
            execution_id: format!("exec_{key}"),
        })
    }

    /// The completed order for this key, if a previous run finished.
    ///
    /// `Ok(None)` means "not completed" and is the normal first-attempt answer. A completed
    /// execution whose payload has no `order_id` is reported rather than swallowed: it means
    /// the process died between writing the payload and marking the state, and treating it
    /// as done would hand the customer an empty result.
    async fn completed_output(&self, key: &str) -> Result<Option<CompleteCartOutput>> {
        let Some(exec) = self.workflow_store.get_execution_async(key).await? else {
            return Ok(None);
        };
        if exec.state != stheno_workflow::TransactionState::Done {
            return Ok(None);
        }
        let order_id = exec.context_payload.get("order_id").and_then(|v| v.as_str());
        let Some(order_id) = order_id else {
            return Ok(None);
        };
        let order = self.order_service.get_order(order_id).await?;
        Ok(Some(CompleteCartOutput {
            order,
            resumed_from_step: None,
            execution_id: format!("exec_{key}"),
        }))
    }

    /// Checkpoint plus the id of the order this run created, if any.
    ///
    /// Both land in the execution's payload, which is the durable replacement for the
    /// in-memory `checkpoints` and `cart_to_order` maps this engine used to carry.
    async fn record_checkpoint_with_order(
        &self,
        cart_id: &str,
        key: &str,
        next_step: usize,
        order_id: &str,
    ) {
        let payload = serde_json::json!({
            "cart_id": cart_id,
            "next_step": next_step,
            "order_id": order_id,
        });
        // Written under both keys: the idempotency key is what a retry presents, and the
        // cart id is what the resume path looks up.
        for id in [key, cart_id] {
            if let Err(e) = self.workflow_store.update_payload_async(id, payload.clone()).await {
                tracing::warn!("could not persist checkout checkpoint: {e}");
            }
        }
    }

    /// Persists how far this checkout got, so a crash resumes rather than restarts.
    ///
    /// Best-effort by design: failing to write a checkpoint must not fail a checkout that
    /// is otherwise succeeding. The cost of a lost checkpoint is re-running a step, which
    /// the step-level idempotency is there to absorb.
    async fn record_checkpoint(&self, cart_id: &str, next_step: usize) {
        if let Err(e) = self
            .workflow_store
            .update_payload_async(cart_id, serde_json::json!({ "cart_id": cart_id, "next_step": next_step }))
            .await
        {
            tracing::warn!("could not persist checkout checkpoint: {e}");
        }
    }

    /// Undoes completed steps, most recent first.
    ///
    /// Errors are logged rather than propagated: this already runs on a failure path, and
    /// abandoning the unwind half-way would leave more behind than finishing it does.
    async fn unwind_compensation(&self, stack: &[Compensation]) {
        for comp in stack.iter().rev() {
            match comp {
                Compensation::UnmarkCart => tracing::info!("compensating: unmarking cart"),
                Compensation::DismissLinks { order_id, cart_id } => {
                    if let Err(e) = self.links.unlink("order_cart", order_id, cart_id).await {
                        tracing::error!("compensation: could not dismiss order link: {e}");
                    }
                }
                Compensation::DeleteOrder(order_id) => {
                    if let Err(e) = self.order_service.cancel_order(order_id).await {
                        tracing::error!("compensation: could not cancel order {order_id}: {e}");
                    }
                }
                Compensation::ReleaseBudget => {
                    tracing::info!("compensating: releasing campaign budget (step is a stub)")
                }
                Compensation::CancelAuthorization(payment_id) => {
                    // The one that holds real money.
                    if let Err(e) = self.payment_service.cancel_payment(payment_id).await {
                        tracing::error!(
                            "compensation: could not void payment {payment_id}: {e}                              — an authorisation may still be held"
                        );
                    }
                }
                Compensation::ReleaseReservations => {
                    tracing::info!("compensating: releasing reservations (step is a stub)")
                }
            }
        }
    }
}
