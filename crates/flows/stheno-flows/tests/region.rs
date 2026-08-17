use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::*;
use stheno_region::RegionService;

#[tokio::test]
async fn test_all_3_region_workflows() {
    let db = TestDb::migrated().await;
    let service = Arc::new(RegionService::new(create_pool(&db.url, 5).expect("pool")));

    // 1. Workflow: create-regions
    let saga_create = build_create_region_saga(service.clone());
    let res = saga_create
        .execute(
            "tx_reg_1",
            "create-region",
            json!({
                "name": "North America",
                "currency_code": "usd",
                "automatic_taxes": true,
                "countries": ["us", "ca"]
            }),
        )
        .await
        .expect("must execute create-region workflow");

    let region_id = res["id"].as_str().unwrap().to_string();
    assert_eq!(res["name"], "North America");
    assert_eq!(res["currency_code"], "usd");
    assert_eq!(res["countries"].as_array().unwrap().len(), 2);

    // 2. Workflow: update-regions
    let saga_update = build_update_region_saga(service.clone());
    let upd_res = saga_update
        .execute(
            "tx_reg_2",
            "update-region",
            json!({
                "id": region_id,
                "data": {
                    "name": "North America (Updated)",
                    "automatic_taxes": false
                }
            }),
        )
        .await
        .expect("must execute update-region workflow");

    assert_eq!(upd_res["name"], "North America (Updated)");
    assert_eq!(upd_res["automatic_taxes"], false);

    // 3. Workflow: delete-regions
    let saga_delete = build_delete_region_saga(service.clone());
    let del_res = saga_delete
        .execute("tx_reg_3", "delete-region", json!({ "id": region_id }))
        .await
        .expect("must execute delete-region workflow");

    assert_eq!(del_res["deleted"], true);

    db.drop().await;
}
