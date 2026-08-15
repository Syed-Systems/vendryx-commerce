use stheno_payment::PaymentSessionStatus;

#[test]
fn test_monotonic_state_transitions_and_convergence_reducer() {
    // 1. Valid forward transitions (PAYMENTS §3.1)
    assert!(
        PaymentSessionStatus::Pending.can_transition_to(PaymentSessionStatus::PendingAuthorization)
    );
    assert!(PaymentSessionStatus::Pending.can_transition_to(PaymentSessionStatus::Authorized));
    assert!(PaymentSessionStatus::Pending.can_transition_to(PaymentSessionStatus::Canceled));
    assert!(PaymentSessionStatus::Pending.can_transition_to(PaymentSessionStatus::Error));

    assert!(PaymentSessionStatus::PendingAuthorization
        .can_transition_to(PaymentSessionStatus::RequiresMore));
    assert!(PaymentSessionStatus::PendingAuthorization
        .can_transition_to(PaymentSessionStatus::Authorized));

    assert!(PaymentSessionStatus::Authorized.can_transition_to(PaymentSessionStatus::Captured));
    assert!(PaymentSessionStatus::Authorized.can_transition_to(PaymentSessionStatus::Canceled));

    // 2. Terminal states & Backwards transitions prohibited (E8)
    assert!(!PaymentSessionStatus::Captured.can_transition_to(PaymentSessionStatus::Authorized));
    assert!(!PaymentSessionStatus::Captured.can_transition_to(PaymentSessionStatus::Pending));
    assert!(!PaymentSessionStatus::Canceled.can_transition_to(PaymentSessionStatus::Authorized));
    assert!(!PaymentSessionStatus::Error.can_transition_to(PaymentSessionStatus::Authorized));
}
