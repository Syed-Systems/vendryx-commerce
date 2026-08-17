use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::*;
use stheno_tax::TaxService;

#[tokio::test]
async fn test_all_9_tax_workflows() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(TaxService::new(pool));

    // 1. Workflow: create-tax-regions
    let saga_create_region = build_create_tax_region_saga(service.clone());
    let region_res = saga_create_region
        .execute(
            "tx_tax_reg_1",
            "create-tax-region",
            json!({
                "country_code": "us",
                "province_code": "ca",
                "parent_id": null,
                "provider_id": null
            }),
        )
        .await
        .expect("must execute create-tax-region workflow");

    let tax_region_id = region_res["id"].as_str().unwrap().to_string();
    assert_eq!(region_res["country_code"], "us");
    assert_eq!(region_res["province_code"], "ca");

    // 2. Workflow: update-tax-regions
    let saga_update_region = build_update_tax_region_saga(service.clone());
    let reg_upd_res = saga_update_region
        .execute(
            "tx_tax_reg_2",
            "update-tax-region",
            json!({
                "id": tax_region_id,
                "data": {
                    "provider_id": "system"
                }
            }),
        )
        .await
        .expect("must execute update-tax-region workflow");

    assert_eq!(reg_upd_res["provider_id"], "system");

    // 3. Workflow: create-tax-rates
    let saga_create_rate = build_create_tax_rate_saga(service.clone());
    let rate_res = saga_create_rate
        .execute(
            "tx_tax_rate_1",
            "create-tax-rate",
            json!({
                "tax_region_id": tax_region_id,
                "rate": "0.0725",
                "code": "CA_STATE",
                "name": "California Sales Tax",
                "is_default": true,
                "is_combinable": false,
                "rules": null
            }),
        )
        .await
        .expect("must execute create-tax-rate workflow");

    let tax_rate_id = rate_res["id"].as_str().unwrap().to_string();
    assert_eq!(rate_res["code"], "CA_STATE");

    // 4. Workflow: update-tax-rates
    let saga_update_rate = build_update_tax_rate_saga(service.clone());
    let rate_upd_res = saga_update_rate
        .execute(
            "tx_tax_rate_2",
            "update-tax-rate",
            json!({
                "id": tax_rate_id,
                "data": {
                    "name": "California General Sales Tax"
                }
            }),
        )
        .await
        .expect("must execute update-tax-rate workflow");

    assert_eq!(rate_upd_res["name"], "California General Sales Tax");

    // 5. Workflow: create-tax-rate-rules
    let saga_create_rule = build_create_tax_rate_rule_saga(service.clone());
    let rule_res = saga_create_rule
        .execute(
            "tx_tax_rule_1",
            "create-tax-rate-rule",
            json!({
                "tax_rate_id": tax_rate_id,
                "rule": {
                    "reference": "product_type_id",
                    "reference_id": "ptype_apparel"
                }
            }),
        )
        .await
        .expect("must execute create-tax-rate-rule workflow");

    assert_eq!(rule_res["reference"], "product_type_id");
    let rule_id = rule_res["id"].as_str().unwrap().to_string();

    // 6. Workflow: resolve-tax-lines
    let saga_resolve = build_resolve_tax_lines_saga(service.clone());
    let lines_res = saga_resolve
        .execute(
            "tx_tax_resolve_1",
            "resolve-tax-lines",
            json!({
                "country_code": "us",
                "province_code": "ca",
                "product_id": null,
                "product_type_id": "ptype_apparel",
                "shipping_option_id": null,
                "is_tax_inclusive": false
            }),
        )
        .await
        .expect("must execute resolve-tax-lines workflow");

    let lines_arr = lines_res.as_array().unwrap();
    assert_eq!(lines_arr.len(), 1);
    assert_eq!(lines_arr[0]["code"], "CA_STATE");

    // 7. Workflow: delete-tax-rate-rules
    let saga_delete_rule = build_delete_tax_rate_rule_saga(service.clone());
    let del_rule_res = saga_delete_rule
        .execute(
            "tx_tax_rule_del_1",
            "delete-tax-rate-rule",
            json!({
                "tax_rate_id": tax_rate_id,
                "rule_id": rule_id
            }),
        )
        .await
        .expect("must execute delete-tax-rate-rule workflow");

    assert_eq!(del_rule_res["deleted"], true);

    // 8. Workflow: delete-tax-rates
    let saga_delete_rate = build_delete_tax_rate_saga(service.clone());
    let del_rate_res = saga_delete_rate
        .execute(
            "tx_tax_rate_del_1",
            "delete-tax-rate",
            json!({ "id": tax_rate_id }),
        )
        .await
        .expect("must execute delete-tax-rate workflow");

    assert_eq!(del_rate_res["deleted"], true);

    // 9. Workflow: delete-tax-regions
    let saga_delete_region = build_delete_tax_region_saga(service.clone());
    let del_reg_res = saga_delete_region
        .execute(
            "tx_tax_reg_del_1",
            "delete-tax-region",
            json!({ "id": tax_region_id }),
        )
        .await
        .expect("must execute delete-tax-region workflow");

    assert_eq!(del_reg_res["deleted"], true);
}
