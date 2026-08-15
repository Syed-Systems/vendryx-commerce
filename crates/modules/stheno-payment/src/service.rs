#![allow(dead_code)]

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};
use tokio::sync::Mutex as AsyncMutex;

use crate::dto::*;
use crate::models::{
    AccountHolder, Capture, Payment, PaymentCollection, PaymentProviderRequest, PaymentSession,
    PaymentWebhookEvent, Refund, RefundReason,
};

#[derive(Clone, Default)]
pub struct PaymentService {
    collections: Arc<RwLock<HashMap<String, PaymentCollection>>>,
    sessions: Arc<RwLock<HashMap<String, PaymentSession>>>,
    payments: Arc<RwLock<HashMap<String, Payment>>>,
    payment_locks: Arc<RwLock<HashMap<String, Arc<AsyncMutex<()>>>>>, // Row-lock per payment for atomic ledger invariant (P4-21, P4-22)
    captures: Arc<RwLock<HashMap<String, Vec<Capture>>>>,
    refunds: Arc<RwLock<HashMap<String, Vec<Refund>>>>,
    refund_reasons: Arc<RwLock<HashMap<String, RefundReason>>>,
    account_holders: Arc<RwLock<HashMap<String, AccountHolder>>>,
    outbound_requests: Arc<RwLock<HashMap<String, PaymentProviderRequest>>>,
    webhook_events: Arc<RwLock<HashMap<String, PaymentWebhookEvent>>>,
    providers: Arc<RwLock<HashMap<String, Arc<dyn PaymentProvider>>>>,
    ip_session_attempts: Arc<RwLock<HashMap<String, Vec<chrono::DateTime<Utc>>>>>,
}

impl PaymentService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_provider(&self, provider: Arc<dyn PaymentProvider>) {
        let mut map = self.providers.write().unwrap();
        map.insert(provider.id().to_string(), provider);
    }

    pub fn get_provider(&self, id: &str) -> Result<Arc<dyn PaymentProvider>> {
        let map = self.providers.read().unwrap();
        map.get(id).cloned().ok_or_else(|| Error::NotFound {
            entity: "payment_provider",
            id: id.to_string(),
        })
    }

    // ── Payment Collection CRUD ──────────────────────────────────────────────

    pub async fn create_payment_collection(
        &self,
        input: CreatePaymentCollectionInput,
    ) -> Result<PaymentCollectionDTO> {
        let id: Id<PaymentCollection> = Id::new();
        let now = Utc::now();
        let col = PaymentCollection {
            id,
            currency_code: input.currency_code.to_lowercase(),
            amount: input.amount,
            authorized_amount: dec!(0),
            captured_amount: dec!(0),
            refunded_amount: dec!(0),
            status: "not_paid".to_string(),
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let col_id_str = col.id.to_string();
        {
            let mut map = self.collections.write().unwrap();
            map.insert(col_id_str.clone(), col);
        }

        self.get_payment_collection(&col_id_str).await
    }

    pub async fn get_payment_collection(&self, id: &str) -> Result<PaymentCollectionDTO> {
        let col = {
            let map = self.collections.read().unwrap();
            map.values()
                .find(|c| {
                    (c.id.to_string() == id || c.id.to_string().ends_with(id))
                        && c.deleted_at.is_none()
                })
                .cloned()
                .or_else(|| map.get(id).filter(|c| c.deleted_at.is_none()).cloned())
                .ok_or_else(|| Error::NotFound {
                    entity: "payment_collection",
                    id: id.to_string(),
                })?
        };

        let col_id_str = col.id.to_string();
        let sessions = self.get_sessions_for_collection(&col_id_str).await;
        let payments = self.get_payments_for_collection(&col_id_str).await;

        Ok(PaymentCollectionDTO {
            id: col.id.to_string(),
            currency_code: col.currency_code,
            amount: col.amount,
            authorized_amount: col.authorized_amount,
            captured_amount: col.captured_amount,
            refunded_amount: col.refunded_amount,
            status: col.status,
            payment_sessions: sessions,
            payments,
            metadata: col.metadata,
            created_at: col.created_at,
            updated_at: col.updated_at,
        })
    }

    // ── Payment Session Lifecycle & Card-Testing Limits (P4-27) ──────────────

    pub async fn create_payment_session(
        &self,
        input: CreatePaymentSessionInput,
        client_ip: Option<&str>,
    ) -> Result<PaymentSessionDTO> {
        // Card testing rate limit (P4-27, F10)
        if let Some(ip) = client_ip {
            if !self.check_ip_session_rate_limit(ip, 10) {
                return Err(Error::Invalid {
                    code: "card_testing_limit_exceeded",
                    message: "Too many payment sessions created from this IP".to_string(),
                    field: None,
                });
            }
        }

        let col = {
            let map = self.collections.read().unwrap();
            map.get(&input.payment_collection_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "payment_collection",
                    id: input.payment_collection_id.clone(),
                })?
        };

        // Cancel previous active sessions on this collection (PAYMENTS §11, E30)
        {
            let mut s_map = self.sessions.write().unwrap();
            for s in s_map.values_mut() {
                if s.payment_collection_id == col.id.to_string()
                    && s.status != "canceled"
                    && s.status != "error"
                    && s.status != "captured"
                {
                    s.status = "canceled".to_string();
                    s.updated_at = Utc::now();
                }
            }
        }

        let provider = self.get_provider(&input.provider_id)?;
        let init_out = provider
            .initiate(InitiateInput {
                amount: input.amount,
                currency_code: input.currency_code.clone(),
                context: input.context.clone(),
                metadata: None,
            })
            .await?;

        let id: Id<PaymentSession> = Id::new();
        let now = Utc::now();
        let session = PaymentSession {
            id,
            payment_collection_id: input.payment_collection_id,
            provider_id: input.provider_id,
            amount: input.amount,
            currency_code: input.currency_code.to_lowercase(),
            status: init_out.status.as_str().to_string(),
            data: Some(init_out.data),
            context: input.context,
            authorized_at: None,
            status_observed_at: now,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let session_id_str = session.id.to_string();
        {
            let mut map = self.sessions.write().unwrap();
            map.insert(session_id_str.clone(), session);
        }

        self.get_payment_session(&session_id_str).await
    }

    pub async fn get_payment_session(&self, id: &str) -> Result<PaymentSessionDTO> {
        let s = {
            let map = self.sessions.read().unwrap();
            map.values()
                .find(|s| {
                    (s.id.to_string() == id || s.id.to_string().ends_with(id))
                        && s.deleted_at.is_none()
                })
                .cloned()
                .or_else(|| map.get(id).filter(|s| s.deleted_at.is_none()).cloned())
                .ok_or_else(|| Error::NotFound {
                    entity: "payment_session",
                    id: id.to_string(),
                })?
        };

        let status =
            PaymentSessionStatus::from_str_name(&s.status).unwrap_or(PaymentSessionStatus::Pending);

        Ok(PaymentSessionDTO {
            id: s.id.to_string(),
            payment_collection_id: s.payment_collection_id,
            provider_id: s.provider_id,
            amount: s.amount,
            currency_code: s.currency_code,
            status,
            data: s.data,
            context: s.context,
            authorized_at: s.authorized_at,
            status_observed_at: s.status_observed_at,
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
    }

    // ── Outbound Idempotency & Ambiguity Resolution (P4-17, P4-18, PAYMENTS §6) ──

    #[allow(clippy::too_many_arguments)]
    pub async fn idempotent_call<F, Fut, T>(
        &self,
        idempotency_key: &str,
        provider_id: &str,
        operation: &str,
        resource_type: &str,
        resource_id: &str,
        payload: &serde_json::Value,
        call_fn: F,
    ) -> Result<T>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<T>>,
        T: Serialize + for<'de> Deserialize<'de> + Clone,
    {
        let request_hash = compute_sha256(payload);

        // Check if request already recorded
        enum ExistingState {
            Succeeded(serde_json::Value),
            Failed,
            Ambiguous,
        }

        let existing = {
            let map = self.outbound_requests.read().unwrap();
            if let Some(req) = map.get(idempotency_key) {
                // Key reuse with changed payload (PAYMENTS §6, E4)
                if req.request_hash != request_hash {
                    return Err(Error::Invalid {
                        code: "idempotency_key_payload_mismatch",
                        message: "Idempotency key reused with different request payload"
                            .to_string(),
                        field: None,
                    });
                }

                if req.state == "succeeded" {
                    req.response.clone().map(ExistingState::Succeeded)
                } else if req.state == "failed" {
                    Some(ExistingState::Failed)
                } else if req.state == "in_flight" || req.state == "unknown" {
                    Some(ExistingState::Ambiguous)
                } else {
                    None
                }
            } else {
                None
            }
        };

        match existing {
            Some(ExistingState::Succeeded(resp_val)) => {
                let parsed: T = serde_json::from_value(resp_val).map_err(|e| {
                    Error::Internal(format!("Failed to deserialize cached response: {}", e).into())
                })?;
                return Ok(parsed);
            }
            Some(ExistingState::Failed) => {
                return Err(Error::Invalid {
                    code: "previous_request_failed",
                    message: format!(
                        "Previous provider request with key {} failed",
                        idempotency_key
                    ),
                    field: None,
                });
            }
            Some(ExistingState::Ambiguous) => {
                // Ambiguous timeout recovery path (P4-18, PAYMENTS §6 step 5)
                if let Ok(provider) = self.get_provider(provider_id) {
                    let ret_res = provider
                        .retrieve(RetrieveInput {
                            reference_id: resource_id.to_string(),
                            data: None,
                        })
                        .await;

                    if let Ok(ret) = ret_res {
                        if ret.status == PaymentSessionStatus::Authorized
                            || ret.status == PaymentSessionStatus::Captured
                        {
                            // Succeeded on remote, backfill and return
                            let resp_json = serde_json::to_value(&ret.data).unwrap_or_default();
                            {
                                let mut w_map = self.outbound_requests.write().unwrap();
                                if let Some(entry) = w_map.get_mut(idempotency_key) {
                                    entry.state = "succeeded".to_string();
                                    entry.response = Some(resp_json.clone());
                                    entry.settled_at = Some(Utc::now());
                                }
                            }
                            if let Ok(parsed) = serde_json::from_value::<T>(resp_json) {
                                return Ok(parsed);
                            }
                        }
                    }
                }
            }
            None => {}
        }

        // Record in_flight request
        let req_id: Id<PaymentProviderRequest> = Id::new();
        let now = Utc::now();
        {
            let mut map = self.outbound_requests.write().unwrap();
            map.insert(
                idempotency_key.to_string(),
                PaymentProviderRequest {
                    id: req_id,
                    idempotency_key: idempotency_key.to_string(),
                    provider_id: provider_id.to_string(),
                    operation: operation.to_string(),
                    resource_type: resource_type.to_string(),
                    resource_id: resource_id.to_string(),
                    request_hash,
                    state: "in_flight".to_string(),
                    response: None,
                    error: None,
                    attempts: 1,
                    first_sent_at: now,
                    settled_at: None,
                },
            );
        }

        // Execute provider call
        let result = call_fn(idempotency_key.to_string()).await;

        let mut map = self.outbound_requests.write().unwrap();
        let entry = map.get_mut(idempotency_key).unwrap();

        match &result {
            Ok(output) => {
                entry.state = "succeeded".to_string();
                entry.response = serde_json::to_value(output).ok();
                entry.settled_at = Some(Utc::now());
            }
            Err(e) => {
                // Ambiguous timeout/failure -> set to 'unknown' to trigger retrieve() reconciliation on retry (PAYMENTS §6)
                entry.state = "unknown".to_string();
                entry.error = Some(serde_json::json!({ "error": format!("{:?}", e) }));
                entry.settled_at = Some(Utc::now());
            }
        }

        result
    }

    // ── Authorization & Payment Creation ─────────────────────────────────────

    pub async fn authorize_payment_session(
        &self,
        input: AuthorizePaymentSessionInput,
    ) -> Result<PaymentDTO> {
        let session = self.get_payment_session(&input.session_id).await?;
        let provider = self.get_provider(&session.provider_id)?;

        let auth_out = provider
            .authorize(AuthorizeInput {
                session_id: session.id.clone(),
                amount: session.amount,
                currency_code: session.currency_code.clone(),
                context: input.context.clone(),
                data: session.data.clone(),
            })
            .await?;

        if auth_out.status != PaymentSessionStatus::Authorized {
            // Update session status and fail
            {
                let mut s_map = self.sessions.write().unwrap();
                if let Some(s) = s_map.get_mut(&session.id) {
                    s.status = auth_out.status.as_str().to_string();
                    s.data = Some(auth_out.data);
                    s.updated_at = Utc::now();
                }
            }
            return Err(Error::Invalid {
                code: "payment_authorization_failed",
                message: format!(
                    "Session authorization resulted in {}",
                    auth_out.status.as_str()
                ),
                field: None,
            });
        }

        // Update session to authorized
        let now = Utc::now();
        {
            let mut s_map = self.sessions.write().unwrap();
            if let Some(s) = s_map.get_mut(&session.id) {
                s.status = "authorized".to_string();
                s.authorized_at = Some(now);
                s.data = Some(auth_out.data.clone());
                s.updated_at = now;
            }
        }

        // Create Payment entity ONLY on successful authorization (DATA_MODEL §2.7, PAYMENTS §2)
        let payment_id: Id<Payment> = Id::new();
        let payment = Payment {
            id: payment_id,
            payment_collection_id: session.payment_collection_id.clone(),
            payment_session_id: Some(session.id.clone()),
            provider_id: session.provider_id.clone(),
            currency_code: session.currency_code.clone(),
            amount: session.amount,
            captured_at: None,
            canceled_at: None,
            data: Some(auth_out.data),
            metadata: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let payment_id_str = payment.id.to_string();
        {
            let mut p_map = self.payments.write().unwrap();
            p_map.insert(payment_id_str.clone(), payment);
        }

        // Update collection authorized_amount & status
        {
            let mut c_map = self.collections.write().unwrap();
            if let Some(col) = c_map.get_mut(&session.payment_collection_id) {
                col.authorized_amount += session.amount;
                col.status = "authorized".to_string();
                col.updated_at = now;
            }
        }

        self.get_payment(&payment_id_str).await
    }

    pub async fn get_payment(&self, id: &str) -> Result<PaymentDTO> {
        let p = {
            let map = self.payments.read().unwrap();
            map.values()
                .find(|p| {
                    (p.id.to_string() == id || p.id.to_string().ends_with(id))
                        && p.deleted_at.is_none()
                })
                .cloned()
                .or_else(|| map.get(id).filter(|p| p.deleted_at.is_none()).cloned())
                .ok_or_else(|| Error::NotFound {
                    entity: "payment",
                    id: id.to_string(),
                })?
        };

        let payment_id_str = p.id.to_string();
        let captures = self.get_captures_for_payment(&payment_id_str);
        let refunds = self.get_refunds_for_payment(&payment_id_str);

        let captured_amount: Decimal = captures.iter().map(|c| c.amount).sum();
        let refunded_amount: Decimal = refunds.iter().map(|r| r.amount).sum();

        Ok(PaymentDTO {
            id: p.id.to_string(),
            payment_collection_id: p.payment_collection_id,
            payment_session_id: p.payment_session_id,
            provider_id: p.provider_id,
            currency_code: p.currency_code,
            amount: p.amount,
            captured_at: p.captured_at,
            canceled_at: p.canceled_at,
            captures,
            refunds,
            captured_amount,
            refunded_amount,
            data: p.data,
            metadata: p.metadata,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
    }

    // ── Ledger Invariants: Capture & Refund under Row Lock (P4-21, P4-22, PAYMENTS §3.3) ──

    fn get_or_create_payment_lock(&self, payment_id: &str) -> Arc<AsyncMutex<()>> {
        let mut locks = self.payment_locks.write().unwrap();
        locks
            .entry(payment_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    pub async fn capture_payment(&self, input: CapturePaymentInput) -> Result<CaptureDTO> {
        let lock = self.get_or_create_payment_lock(&input.payment_id);
        let _guard = lock.lock().await;

        let payment = self.get_payment(&input.payment_id).await?;
        let current_captured = payment.captured_amount;
        let amount_to_capture = input.amount.unwrap_or(payment.amount - current_captured);

        // Ledger Invariant 1: 0 <= sum(captures) + amount <= payment.amount (PAYMENTS §3.3)
        if amount_to_capture <= dec!(0) {
            return Err(Error::Invalid {
                code: "invalid_capture_amount",
                message: "Capture amount must be greater than zero".to_string(),
                field: None,
            });
        }

        if current_captured + amount_to_capture > payment.amount {
            return Err(Error::Conflict {
                code: "over_capture_prevented",
                message: format!(
                    "Cannot capture {}: existing captured {} + {} exceeds authorized amount {}",
                    amount_to_capture, current_captured, amount_to_capture, payment.amount
                ),
            });
        }

        // Call provider capture
        let provider = self.get_provider(&payment.provider_id)?;
        provider
            .capture(CaptureInput {
                payment_id: payment.id.clone(),
                amount: amount_to_capture,
                currency_code: payment.currency_code.clone(),
                data: payment.data.clone(),
            })
            .await?;

        let capture = Capture {
            id: Id::new(),
            payment_id: payment.id.clone(),
            amount: amount_to_capture,
            created_by: input.created_by,
            metadata: input.metadata,
            created_at: Utc::now(),
        };

        let dto = CaptureDTO {
            id: capture.id.to_string(),
            payment_id: capture.payment_id.clone(),
            amount: capture.amount,
            created_by: capture.created_by.clone(),
            metadata: capture.metadata.clone(),
            created_at: capture.created_at,
        };

        {
            let mut cap_map = self.captures.write().unwrap();
            cap_map.entry(payment.id.clone()).or_default().push(capture);
        }

        // Update payment collection captured_amount
        {
            let mut col_map = self.collections.write().unwrap();
            if let Some(col) = col_map.get_mut(&payment.payment_collection_id) {
                col.captured_amount += amount_to_capture;
                if col.captured_amount >= col.amount {
                    col.status = "captured".to_string();
                }
                col.updated_at = Utc::now();
            }
        }

        // Update payment captured_at
        {
            let mut p_map = self.payments.write().unwrap();
            if let Some(p) = p_map.get_mut(&payment.id) {
                if current_captured + amount_to_capture >= payment.amount {
                    p.captured_at = Some(Utc::now());
                }
                p.updated_at = Utc::now();
            }
        }

        Ok(dto)
    }

    pub async fn refund_payment(&self, input: RefundPaymentInput) -> Result<RefundDTO> {
        let lock = self.get_or_create_payment_lock(&input.payment_id);
        let _guard = lock.lock().await;

        let payment = self.get_payment(&input.payment_id).await?;
        let current_captured = payment.captured_amount;
        let current_refunded = payment.refunded_amount;

        // Ledger Invariant 2: 0 <= sum(refunds) + amount <= sum(captures) (PAYMENTS §3.3)
        if input.amount <= dec!(0) {
            return Err(Error::Invalid {
                code: "invalid_refund_amount",
                message: "Refund amount must be greater than zero".to_string(),
                field: None,
            });
        }

        if current_refunded + input.amount > current_captured {
            return Err(Error::Conflict {
                code: "over_refund_prevented",
                message: format!(
                    "Cannot refund {}: total refunded {} exceeds total captured {}",
                    input.amount,
                    current_refunded + input.amount,
                    current_captured
                ),
            });
        }

        // Call provider refund
        let provider = self.get_provider(&payment.provider_id)?;
        provider
            .refund(RefundInput {
                payment_id: payment.id.clone(),
                amount: input.amount,
                currency_code: payment.currency_code.clone(),
                reason: input.refund_reason_id.clone(),
                note: input.note.clone(),
                data: payment.data.clone(),
            })
            .await?;

        let refund = Refund {
            id: Id::new(),
            payment_id: payment.id.clone(),
            amount: input.amount,
            refund_reason_id: input.refund_reason_id,
            note: input.note,
            created_by: input.created_by,
            metadata: input.metadata,
            created_at: Utc::now(),
        };

        let dto = RefundDTO {
            id: refund.id.to_string(),
            payment_id: refund.payment_id.clone(),
            amount: refund.amount,
            refund_reason_id: refund.refund_reason_id.clone(),
            note: refund.note.clone(),
            created_by: refund.created_by.clone(),
            metadata: refund.metadata.clone(),
            created_at: refund.created_at,
        };

        {
            let mut ref_map = self.refunds.write().unwrap();
            ref_map.entry(payment.id.clone()).or_default().push(refund);
        }

        // Update payment collection refunded_amount
        {
            let mut col_map = self.collections.write().unwrap();
            if let Some(col) = col_map.get_mut(&payment.payment_collection_id) {
                col.refunded_amount += input.amount;
                col.updated_at = Utc::now();
            }
        }

        Ok(dto)
    }

    // ── Webhook Store & Convergence Reducer (P4-19, P4-20, PAYMENTS §7) ──────

    pub async fn receive_webhook(
        &self,
        provider_id: &str,
        raw_body: &[u8],
        signature_header: Option<&str>,
    ) -> Result<PaymentWebhookEventDTO> {
        let provider = self.get_provider(provider_id)?;
        let verified = provider.verify_webhook(raw_body, signature_header)?;

        let event_key = format!("{}:{}", provider_id, verified.provider_event_id);

        // Webhook replay guard (PAYMENTS §7.1, §7.3, E9): Unique constraint
        {
            let map = self.webhook_events.read().unwrap();
            if let Some(existing) = map.get(&event_key) {
                // Replay detected -> return 200 OK with existing event without processing
                return Ok(PaymentWebhookEventDTO {
                    id: existing.id.to_string(),
                    provider_id: existing.provider_id.clone(),
                    provider_event_id: existing.provider_event_id.clone(),
                    event_type: existing.event_type.clone(),
                    payload: existing.payload.clone(),
                    signature_verified: existing.signature_verified,
                    provider_created_at: existing.provider_created_at,
                    state: "ignored".to_string(),
                    processed_at: existing.processed_at,
                    error: existing.error.clone(),
                    created_at: existing.created_at,
                });
            }
        }

        let event = PaymentWebhookEvent {
            id: Id::new(),
            provider_id: provider_id.to_string(),
            provider_event_id: verified.provider_event_id.clone(),
            event_type: verified.event_type.clone(),
            payload: verified.payload.clone(),
            signature_verified: true,
            provider_created_at: verified.provider_created_at,
            state: "received".to_string(),
            processed_at: None,
            error: None,
            created_at: Utc::now(),
        };

        let dto = PaymentWebhookEventDTO {
            id: event.id.to_string(),
            provider_id: event.provider_id.clone(),
            provider_event_id: event.provider_event_id.clone(),
            event_type: event.event_type.clone(),
            payload: event.payload.clone(),
            signature_verified: true,
            provider_created_at: event.provider_created_at,
            state: "received".to_string(),
            processed_at: None,
            error: None,
            created_at: event.created_at,
        };

        {
            let mut map = self.webhook_events.write().unwrap();
            map.insert(event_key, event);
        }

        // Apply convergence reducer
        let action = provider.map_webhook(&verified)?;
        self.apply_webhook_action(provider_id, &verified, action)
            .await;

        Ok(dto)
    }

    /// Convergence Reducer (PAYMENTS §7.2): Monotonic transitions + provider timestamp ordering
    async fn apply_webhook_action(
        &self,
        _provider_id: &str,
        event: &VerifiedEvent,
        action: WebhookAction,
    ) {
        if action == WebhookAction::NotSupported {
            return;
        }

        let target_status = match action {
            WebhookAction::Authorized => PaymentSessionStatus::Authorized,
            WebhookAction::Captured => PaymentSessionStatus::Captured,
            WebhookAction::Failed => PaymentSessionStatus::Error,
            WebhookAction::Pending => PaymentSessionStatus::Pending,
            WebhookAction::RequiresMore => PaymentSessionStatus::RequiresMore,
            WebhookAction::Canceled => PaymentSessionStatus::Canceled,
            WebhookAction::PendingAuthorization => PaymentSessionStatus::PendingAuthorization,
            WebhookAction::NotSupported => return,
        };

        // If event payload has a session_id, apply monotonic convergence
        if let Some(session_id) = event.payload.get("session_id").and_then(|v| v.as_str()) {
            let mut map = self.sessions.write().unwrap();
            if let Some(session) = map.get_mut(session_id) {
                let current_status = PaymentSessionStatus::from_str_name(&session.status)
                    .unwrap_or(PaymentSessionStatus::Pending);

                // Convergence Rule 1: Monotonic transition check
                if !current_status.can_transition_to(target_status) {
                    // Backwards transition dropped (PAYMENTS §7.2, E8)
                    return;
                }

                // Convergence Rule 2: Provider timestamp ordering
                if event.provider_created_at < session.status_observed_at {
                    // Stale observation dropped
                    return;
                }

                session.status = target_status.as_str().to_string();
                session.status_observed_at = event.provider_created_at;
                session.updated_at = Utc::now();
            }
        }
    }

    // ── Card Testing Abuse Limits (P4-27, F10) ────────────────────────────────

    pub fn check_ip_session_rate_limit(&self, ip: &str, max_per_minute: usize) -> bool {
        let mut map = self.ip_session_attempts.write().unwrap();
        let one_minute_ago = Utc::now() - Duration::minutes(1);
        let timestamps = map.entry(ip.to_string()).or_default();
        timestamps.retain(|t| *t > one_minute_ago);

        if timestamps.len() >= max_per_minute {
            false
        } else {
            timestamps.push(Utc::now());
            true
        }
    }

    // ── Internal Helpers ─────────────────────────────────────────────────────

    async fn get_sessions_for_collection(&self, collection_id: &str) -> Vec<PaymentSessionDTO> {
        let map = self.sessions.read().unwrap();
        map.values()
            .filter(|s| s.payment_collection_id == collection_id && s.deleted_at.is_none())
            .map(|s| {
                let status = PaymentSessionStatus::from_str_name(&s.status)
                    .unwrap_or(PaymentSessionStatus::Pending);
                PaymentSessionDTO {
                    id: s.id.to_string(),
                    payment_collection_id: s.payment_collection_id.clone(),
                    provider_id: s.provider_id.clone(),
                    amount: s.amount,
                    currency_code: s.currency_code.clone(),
                    status,
                    data: s.data.clone(),
                    context: s.context.clone(),
                    authorized_at: s.authorized_at,
                    status_observed_at: s.status_observed_at,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                }
            })
            .collect()
    }

    async fn get_payments_for_collection(&self, collection_id: &str) -> Vec<PaymentDTO> {
        let map = self.payments.read().unwrap();
        let mut list = Vec::new();
        for p in map
            .values()
            .filter(|p| p.payment_collection_id == collection_id && p.deleted_at.is_none())
        {
            let p_id_str = p.id.to_string();
            let captures = self.get_captures_for_payment(&p_id_str);
            let refunds = self.get_refunds_for_payment(&p_id_str);
            let captured_amount = captures.iter().map(|c| c.amount).sum();
            let refunded_amount = refunds.iter().map(|r| r.amount).sum();

            list.push(PaymentDTO {
                id: p_id_str,
                payment_collection_id: p.payment_collection_id.clone(),
                payment_session_id: p.payment_session_id.clone(),
                provider_id: p.provider_id.clone(),
                currency_code: p.currency_code.clone(),
                amount: p.amount,
                captured_at: p.captured_at,
                canceled_at: p.canceled_at,
                captures,
                refunds,
                captured_amount,
                refunded_amount,
                data: p.data.clone(),
                metadata: p.metadata.clone(),
                created_at: p.created_at,
                updated_at: p.updated_at,
            });
        }
        list
    }

    fn get_captures_for_payment(&self, payment_id: &str) -> Vec<CaptureDTO> {
        let map = self.captures.read().unwrap();
        map.get(payment_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|c| CaptureDTO {
                id: c.id.to_string(),
                payment_id: c.payment_id,
                amount: c.amount,
                created_by: c.created_by,
                metadata: c.metadata,
                created_at: c.created_at,
            })
            .collect()
    }

    fn get_refunds_for_payment(&self, payment_id: &str) -> Vec<RefundDTO> {
        let map = self.refunds.read().unwrap();
        map.get(payment_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|r| RefundDTO {
                id: r.id.to_string(),
                payment_id: r.payment_id,
                amount: r.amount,
                refund_reason_id: r.refund_reason_id,
                note: r.note,
                created_by: r.created_by,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect()
    }

    /// Payment reconciliation job comparing PSP settlement records with Stheno payment ledgers (PAYMENTS §9, Task P8-22).
    /// RULE: Drift is detected and recorded, but NEVER auto-corrects without manual review.
    pub async fn run_reconciliation(
        &self,
        provider_id: &str,
        psp_report: Vec<PspReportItem>,
    ) -> Result<ReconciliationReportDTO> {
        let payments_map = self.payments.read().unwrap();
        let mut matched_count = 0;
        let mut discrepancies = Vec::new();

        for psp_item in &psp_report {
            let internal_payment = if let Some(ref pid) = psp_item.payment_id {
                payments_map.get(pid)
            } else {
                payments_map.values().find(|p| {
                    p.data
                        .as_ref()
                        .and_then(|d| d.get("provider_payment_id"))
                        .and_then(|v| v.as_str())
                        == Some(&psp_item.provider_payment_id)
                })
            };

            match internal_payment {
                Some(p) => {
                    let mut has_discrepancy = false;
                    let payment_id_str = p.id.to_string();
                    let captures = self.get_captures_for_payment(&payment_id_str);
                    let captured_total: Decimal = captures.iter().map(|c| c.amount).sum();
                    let p_effective_amount = if p.captured_at.is_some() || !captures.is_empty() {
                        captured_total
                    } else {
                        p.amount
                    };

                    if p_effective_amount != psp_item.amount {
                        has_discrepancy = true;
                        discrepancies.push(ReconciliationDiscrepancy {
                            payment_id: payment_id_str.clone(),
                            provider_payment_id: psp_item.provider_payment_id.clone(),
                            field: "amount".to_string(),
                            internal_value: p_effective_amount.to_string(),
                            psp_value: psp_item.amount.to_string(),
                            description: format!(
                                "Amount mismatch: internal {} vs PSP settlement {}",
                                p_effective_amount, psp_item.amount
                            ),
                        });
                    }

                    let is_status_match = match (p.captured_at.is_some(), psp_item.status.as_str())
                    {
                        (true, "captured") => true,
                        (false, "authorized") => true,
                        _ => psp_item.status == "captured" && !captures.is_empty(),
                    };

                    if !is_status_match {
                        has_discrepancy = true;
                        discrepancies.push(ReconciliationDiscrepancy {
                            payment_id: payment_id_str.clone(),
                            provider_payment_id: psp_item.provider_payment_id.clone(),
                            field: "status".to_string(),
                            internal_value: if p.captured_at.is_some() || !captures.is_empty() {
                                "captured"
                            } else {
                                "authorized"
                            }
                            .to_string(),
                            psp_value: psp_item.status.clone(),
                            description: format!("Status mismatch for payment {}", payment_id_str),
                        });
                    }

                    if !has_discrepancy {
                        matched_count += 1;
                    }
                }
                None => {
                    discrepancies.push(ReconciliationDiscrepancy {
                        payment_id: "unknown".to_string(),
                        provider_payment_id: psp_item.provider_payment_id.clone(),
                        field: "missing_in_stheno".to_string(),
                        internal_value: "none".to_string(),
                        psp_value: psp_item.amount.to_string(),
                        description: format!(
                            "PSP record {} exists in gateway report but not in Stheno records",
                            psp_item.provider_payment_id
                        ),
                    });
                }
            }
        }

        Ok(ReconciliationReportDTO {
            id: Id::<Payment>::new().to_string(),
            provider_id: provider_id.to_string(),
            total_records_checked: psp_report.len(),
            matched_count,
            discrepancy_count: discrepancies.len(),
            discrepancies,
            generated_at: Utc::now(),
        })
    }
}

pub fn compute_sha256(val: &serde_json::Value) -> String {
    let canonical = serde_json::to_string(val).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}
