use stheno_cli::import::run_import;

#[test]
fn test_import_medusa_dry_run_fidelity_report() {
    let source_db = "postgres://postgres:postgres@localhost:5432/medusa_source";
    let report = run_import(Some(source_db), true);

    // Verify dry run parameters
    assert!(report.dry_run, "Dry run mode must be enabled");
    assert_eq!(report.source_db.as_deref(), Some(source_db));

    // Verify fidelity assertions
    assert!(
        report.is_lossless(),
        "Migration dry run must be 100% lossless"
    );
    assert_eq!(
        report.fidelity_percentage, 100.0,
        "Entity fidelity must be 100.0%"
    );
    assert_eq!(
        report.money_precision_loss_count, 0,
        "There must be zero precision loss in financial fields"
    );
    assert_eq!(
        report.total_entities, report.total_ids_preserved,
        "All entities must have IDs preserved identically"
    );

    // Verify table mapping coverage
    assert!(!report.tables.is_empty(), "Must map standard domain tables");
    assert!(report
        .tables
        .iter()
        .any(|t| t.medusa_table == "product" && t.stheno_table == "product.product"));
    assert!(report
        .tables
        .iter()
        .any(|t| t.medusa_table == "order" && t.stheno_table == "order.order"));
    assert!(report
        .tables
        .iter()
        .any(|t| t.medusa_table == "price_set" && t.stheno_table == "pricing.price_set"));

    // Verify formatted report output
    let output = report.format_report();
    assert!(output.contains("MEDUSA -> STHENO IMPORT FIDELITY REPORT"));
    assert!(output.contains("DRY RUN (zero writes)"));
    assert!(output.contains("100.0%"));
    assert!(output.contains("STATUS: PASSED"));
}
