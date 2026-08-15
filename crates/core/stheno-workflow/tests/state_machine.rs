use stheno_workflow::{StepState, TransactionState};

#[test]
fn test_transaction_state_machine_valid_transitions() {
    let s = TransactionState::NotStarted;
    assert!(s.can_transition_to(TransactionState::Invoking));
    assert!(!s.can_transition_to(TransactionState::Done));

    let invoking = TransactionState::Invoking;
    assert!(invoking.can_transition_to(TransactionState::Done));
    assert!(invoking.can_transition_to(TransactionState::Compensating));
    assert!(!invoking.can_transition_to(TransactionState::NotStarted));
}

#[test]
fn test_step_state_machine_valid_transitions() {
    let pending = StepState::Pending;
    assert!(pending.can_transition_to(StepState::Running));
    assert!(!pending.can_transition_to(StepState::Completed));

    let running = StepState::Running;
    assert!(running.can_transition_to(StepState::Completed));
    assert!(running.can_transition_to(StepState::Failed));
    assert!(running.can_transition_to(StepState::Skipped));
}
