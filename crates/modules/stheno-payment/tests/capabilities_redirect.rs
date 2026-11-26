//! P21-06: ProviderCapabilities.redirect: bool, defaulting false; Stripe and offline declare theirs.

use stheno_payment::{PaymentProvider, ProviderCapabilities};
use stheno_payment_offline::OfflinePaymentProvider;
use stheno_payment_stripe::StripePaymentProvider;

#[test]
fn default_capabilities_has_redirect_false() {
    let caps = ProviderCapabilities::default();
    assert!(
        !caps.redirect,
        "default capabilities redirect must be false"
    );
}

#[test]
fn offline_declares_redirect_false() {
    let provider = OfflinePaymentProvider::new(None);
    let caps = provider.capabilities();
    assert!(!caps.redirect, "offline provider redirect must be false");
}

#[test]
fn stripe_declares_redirect_false() {
    let provider = StripePaymentProvider::new("sk_test_mock", "whsec_mock", None, false);
    let caps = provider.capabilities();
    assert!(
        !caps.redirect,
        "stripe direct card provider redirect must be false"
    );
}
