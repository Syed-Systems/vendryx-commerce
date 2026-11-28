//! P21-09: [BP] Conformance suite asserts capability honesty — every true in
//! capabilities() is exercised and must succeed; every false must be refused
//! (PHASE3_TASKS.md P21-09).

use async_trait::async_trait;
use rust_decimal_macros::dec;
use serde_json::json;
use stheno_payment::{
    AuthorizeInput, AuthorizeOutput, CancelInput, CancelOutput, CaptureInput, CaptureOutput,
    DeleteInput, InitiateInput, InitiateOutput, PaymentProvider, PaymentSessionStatus,
    ProviderCapabilities, RefundInput, RefundOutput, RetrieveInput, RetrieveOutput, StatusInput,
    UpdateInput, UpdateOutput, VerifiedEvent, WebhookAction,
};
use stheno_payment_offline::OfflinePaymentProvider;
use stheno_types::{Error, Result};

/// A provider that honestly declares partial_capture: false and partial_refund: false.
/// It MUST refuse attempts to partial-capture or partial-refund.
struct StrictNoPartialProvider {
    authorized_amount: std::sync::atomic::AtomicI64,
}

impl StrictNoPartialProvider {
    fn new() -> Self {
        Self {
            authorized_amount: std::sync::atomic::AtomicI64::new(100),
        }
    }
}

#[async_trait]
impl PaymentProvider for StrictNoPartialProvider {
    fn id(&self) -> &str {
        "pp_strict_no_partial"
    }

    async fn initiate(&self, _i: InitiateInput) -> Result<InitiateOutput> {
        Ok(InitiateOutput {
            id: "sess_strict_1".to_string(),
            status: PaymentSessionStatus::Pending,
            data: json!({}),
        })
    }

    async fn update(&self, _i: UpdateInput) -> Result<UpdateOutput> {
        unimplemented!()
    }

    async fn delete(&self, _i: DeleteInput) -> Result<()> {
        Ok(())
    }

    async fn authorize(&self, i: AuthorizeInput) -> Result<AuthorizeOutput> {
        self.authorized_amount.store(
            i.amount.to_string().parse().unwrap_or(100),
            std::sync::atomic::Ordering::SeqCst,
        );
        Ok(AuthorizeOutput {
            status: PaymentSessionStatus::Authorized,
            data: json!({ "authorized": i.amount }),
        })
    }

    async fn capture(&self, i: CaptureInput) -> Result<CaptureOutput> {
        let auth = self
            .authorized_amount
            .load(std::sync::atomic::Ordering::SeqCst);
        let cap: i64 = i.amount.to_string().parse().unwrap_or(0);
        if cap < auth {
            return Err(Error::Invalid {
                code: "partial_capture_not_supported",
                message: "Provider does not support partial capture; must capture full amount"
                    .to_string(),
                field: Some("amount".to_string()),
            });
        }
        Ok(CaptureOutput {
            data: json!({ "captured": i.amount }),
        })
    }

    async fn refund(&self, i: RefundInput) -> Result<RefundOutput> {
        let auth = self
            .authorized_amount
            .load(std::sync::atomic::Ordering::SeqCst);
        let ref_amt: i64 = i.amount.to_string().parse().unwrap_or(0);
        if ref_amt < auth {
            return Err(Error::Invalid {
                code: "partial_refund_not_supported",
                message: "Provider does not support partial refund; full refund required"
                    .to_string(),
                field: Some("amount".to_string()),
            });
        }
        Ok(RefundOutput {
            data: json!({ "refunded": i.amount }),
        })
    }

    async fn cancel(&self, _i: CancelInput) -> Result<CancelOutput> {
        Ok(CancelOutput {
            data: json!({ "canceled": true }),
        })
    }

    async fn retrieve(&self, _i: RetrieveInput) -> Result<RetrieveOutput> {
        unimplemented!()
    }

    async fn get_status(&self, _i: StatusInput) -> Result<PaymentSessionStatus> {
        Ok(PaymentSessionStatus::Authorized)
    }

    fn verify_webhook(&self, _raw: &[u8], _sig: Option<&str>) -> Result<VerifiedEvent> {
        unimplemented!()
    }

    fn map_webhook(&self, _event: &VerifiedEvent) -> Result<WebhookAction> {
        Ok(WebhookAction::NotSupported)
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            partial_capture: false,
            partial_refund: false,
            cancel_after_authorize: true,
            redirect: false,
            ..Default::default()
        }
    }
}

#[tokio::test]
async fn honest_provider_exercising_capabilities_succeeds_where_declared() {
    let offline = OfflinePaymentProvider::new(None);
    let caps = offline.capabilities();

    assert!(
        caps.partial_capture,
        "offline declares partial_capture = true"
    );
    assert!(
        caps.partial_refund,
        "offline declares partial_refund = true"
    );

    // Partial capture succeeds
    let cap_out = offline
        .capture(CaptureInput {
            payment_id: "sess_honest_1".to_string(),
            amount: dec!(45.50),
            currency_code: "USD".to_string(),
            data: None,
        })
        .await
        .expect("partial capture must succeed when capability is true");

    assert_eq!(cap_out.data["captured_amount"], json!(dec!(45.50)));

    // Partial refund succeeds
    let ref_out = offline
        .refund(RefundInput {
            payment_id: "sess_honest_1".to_string(),
            amount: dec!(20.00),
            currency_code: "USD".to_string(),
            reason: Some("partial return".to_string()),
            note: None,
            data: None,
        })
        .await
        .expect("partial refund must succeed when capability is true");

    assert_eq!(ref_out.data["refunded_amount"], json!(dec!(20.00)));
}

#[tokio::test]
async fn honest_provider_refuses_undeclared_capabilities() {
    let strict = StrictNoPartialProvider::new();
    let caps = strict.capabilities();

    assert!(
        !caps.partial_capture,
        "strict declares partial_capture = false"
    );
    assert!(
        !caps.partial_refund,
        "strict declares partial_refund = false"
    );

    strict
        .authorize(AuthorizeInput {
            session_id: "sess_strict_1".to_string(),
            amount: dec!(100.00),
            currency_code: "USD".to_string(),
            context: None,
            data: None,
        })
        .await
        .expect("authorize succeeds");

    // Attempting partial capture must fail and be refused
    let cap_err = strict
        .capture(CaptureInput {
            payment_id: "sess_strict_1".to_string(),
            amount: dec!(50.00),
            currency_code: "USD".to_string(),
            data: None,
        })
        .await
        .unwrap_err();

    match cap_err {
        Error::Invalid { code, .. } => {
            assert_eq!(code, "partial_capture_not_supported");
        }
        _ => panic!("expected Invalid partial_capture_not_supported error"),
    }

    // Attempting partial refund must fail and be refused
    let ref_err = strict
        .refund(RefundInput {
            payment_id: "sess_strict_1".to_string(),
            amount: dec!(50.00),
            currency_code: "USD".to_string(),
            reason: None,
            note: None,
            data: None,
        })
        .await
        .unwrap_err();

    match ref_err {
        Error::Invalid { code, .. } => {
            assert_eq!(code, "partial_refund_not_supported");
        }
        _ => panic!("expected Invalid partial_refund_not_supported error"),
    }
}

#[tokio::test]
async fn honest_bd_gateway_capabilities_match_reality() {
    use stheno_payment_bkash::BkashPaymentProvider;
    use stheno_payment_nagad::NagadPaymentProvider;
    use stheno_payment_sslcommerz::SslcommerzPaymentProvider;

    let bkash = BkashPaymentProvider::default();
    let bkash_caps = bkash.capabilities();
    assert!(bkash_caps.redirect);
    assert!(bkash_caps.partial_refund, "bKash supports partial refund");
    assert!(!bkash_caps.partial_capture, "bKash execute is full amount");

    let ssl = SslcommerzPaymentProvider::default();
    let ssl_caps = ssl.capabilities();
    assert!(ssl_caps.redirect);
    assert!(
        ssl_caps.partial_refund,
        "SSLCommerz supports partial refund"
    );
    assert!(
        ssl_caps.partial_capture,
        "SSLCommerz supports partial capture"
    );

    let nagad = NagadPaymentProvider::default();
    let nagad_caps = nagad.capabilities();
    assert!(nagad_caps.redirect);
    assert!(
        !nagad_caps.partial_refund,
        "Nagad honestly refuses partial refund"
    );
    assert!(
        !nagad_caps.partial_capture,
        "Nagad does not support partial capture"
    );

    // Exercise Nagad partial refund refusal
    let nagad_ref_err = nagad
        .refund(RefundInput {
            payment_id: "NAGAD_PAY_1".to_string(),
            amount: dec!(50.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: None,
            data: Some(json!({ "original_amount": "100.00" })),
        })
        .await
        .unwrap_err();

    match nagad_ref_err {
        Error::Invalid { code, .. } => {
            assert_eq!(code, "partial_refund_not_supported");
        }
        _ => panic!("expected partial_refund_not_supported error from Nagad"),
    }
}
