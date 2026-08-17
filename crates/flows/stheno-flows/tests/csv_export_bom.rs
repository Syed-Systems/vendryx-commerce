use stheno_flows::platform::CsvExportWorkflow;

#[test]
fn test_csv_export_bom() {
    let workflow = CsvExportWorkflow::new();

    let headers = &["ID", "Name", "Price", "Description"];
    let rows = vec![
        vec![
            "1".to_string(),
            "Coffee Mug".to_string(),
            "£12.99".to_string(), // international character
            "A nice mug".to_string(),
        ],
        vec![
            "2".to_string(),
            "\"Quote\" Test".to_string(), // quote in field
            "€9.99".to_string(),
            "Description with, comma".to_string(), // comma in field
        ],
    ];

    let exported_bytes = workflow.export_to_csv(headers, &rows);

    // Verify UTF-8 BOM is present
    assert_eq!(&exported_bytes[0..3], &[0xEF, 0xBB, 0xBF]);

    // Convert the rest to a string and verify contents
    let csv_content = std::str::from_utf8(&exported_bytes[3..]).unwrap();
    let lines: Vec<&str> = csv_content.trim().split('\n').collect();

    assert_eq!(lines[0], "ID,Name,Price,Description");
    assert_eq!(lines[1], "1,Coffee Mug,£12.99,A nice mug");
    // Verify quotes and commas are escaped correctly
    assert_eq!(
        lines[2],
        "2,\"\"\"Quote\"\" Test\",€9.99,\"Description with, comma\""
    );
}
