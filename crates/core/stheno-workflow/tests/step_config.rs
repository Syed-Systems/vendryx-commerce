use stheno_workflow::StepConfig;

#[test]
fn test_step_config_defaults_and_options() {
    let mut config = StepConfig::default();
    assert_eq!(config.retries, 3);
    assert_eq!(config.max_retries, 5);
    assert!(config.store);
    assert!(config.idempotent);
    assert!(!config.no_compensation);
    assert!(!config.async_step);
    assert_eq!(config.retention_days, 30);

    config.retries = 5;
    config.no_compensation = true;
    config.async_step = true;

    assert_eq!(config.retries, 5);
    assert!(config.no_compensation);
    assert!(config.async_step);
}
