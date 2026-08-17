use chrono::NaiveDate;
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_reporting::{
    AverageOrderValueReport, OrdersOverTimeItem, OrdersOverTimeReport, ReportingService,
    RevenueOverTimeItem, RevenueOverTimeReport, TopProductItem, TopProductsReport,
};

#[tokio::test]
async fn test_report_csv_export_includes_utf8_bom() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ReportingService::new(pool.clone());

    let date = NaiveDate::from_ymd_opt(2026, 8, 17).unwrap();

    // 1. Revenue CSV
    let rev_report = RevenueOverTimeReport {
        currency: "USD".to_string(),
        total_revenue: dec!(150.00),
        time_series: vec![RevenueOverTimeItem {
            date,
            currency: "USD".to_string(),
            revenue: dec!(150.00),
        }],
    };
    let rev_csv = service.export_revenue_csv(&rev_report);
    assert!(rev_csv.starts_with('\u{FEFF}'), "Must start with UTF-8 BOM");
    assert!(rev_csv.contains("Date,Currency,Revenue"));
    assert!(rev_csv.contains("2026-08-17,USD,150.00"));

    // 2. Orders CSV
    let orders_report = OrdersOverTimeReport {
        total_orders: 5,
        total_units: 12,
        time_series: vec![OrdersOverTimeItem {
            date,
            order_count: 5,
            units_sold: 12,
        }],
    };
    let orders_csv = service.export_orders_csv(&orders_report);
    assert!(
        orders_csv.starts_with('\u{FEFF}'),
        "Must start with UTF-8 BOM"
    );
    assert!(orders_csv.contains("Date,Order Count,Units Sold"));
    assert!(orders_csv.contains("2026-08-17,5,12"));

    // 3. Top Products CSV
    let top_prod_report = TopProductsReport {
        top_products: vec![TopProductItem {
            product_id: "prod_1".to_string(),
            title: "Super Sneakers".to_string(),
            units_sold: 10,
            total_revenue: dec!(1200.00),
        }],
    };
    let top_prod_csv = service.export_top_products_csv(&top_prod_report);
    assert!(
        top_prod_csv.starts_with('\u{FEFF}'),
        "Must start with UTF-8 BOM"
    );
    assert!(top_prod_csv.contains("Product ID,Title,Units Sold,Total Revenue"));
    assert!(top_prod_csv.contains("prod_1,\"Super Sneakers\",10,1200.00"));

    // 4. AOV CSV
    let aov_report = AverageOrderValueReport {
        currency: "USD".to_string(),
        average_order_value: dec!(75.50),
        order_count: 4,
        total_revenue: dec!(302.00),
    };
    let aov_csv = service.export_aov_csv(&aov_report);
    assert!(aov_csv.starts_with('\u{FEFF}'), "Must start with UTF-8 BOM");
    assert!(aov_csv.contains("Currency,Average Order Value,Order Count,Total Revenue"));
    assert!(aov_csv.contains("USD,75.50,4,302.00"));

    db.drop().await;
}
