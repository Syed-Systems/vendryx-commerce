use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::*;
use stheno_store::StoreService;

#[tokio::test]
async fn test_all_3_store_workflows() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(StoreService::new(pool));

    // 1. Workflow: create-stores
    let saga_create = build_create_store_saga(service.clone());
    let res = saga_create
        .execute(
            "tx_store_1",
            "create-store",
            json!({
                "name": "Acme Commerce Flagship",
                "default_currency_code": "usd",
                "supported_currencies": ["usd", "eur", "gbp"],
                "default_locale": "en",
                "supported_locales": ["en", "fr", "de"]
            }),
        )
        .await
        .expect("must execute create-store workflow");

    let store_id = res["id"].as_str().unwrap().to_string();
    assert_eq!(res["name"], "Acme Commerce Flagship");
    assert_eq!(res["default_currency_code"], "usd");
    assert_eq!(res["supported_currencies"].as_array().unwrap().len(), 3);

    // 2. Workflow: update-stores
    let saga_update = build_update_store_saga(service.clone());
    let upd_res = saga_update
        .execute(
            "tx_store_2",
            "update-store",
            json!({
                "id": store_id,
                "data": {
                    "name": "Acme Global Store",
                    "default_currency_code": "eur"
                }
            }),
        )
        .await
        .expect("must execute update-store workflow");

    assert_eq!(upd_res["name"], "Acme Global Store");

    // 3. Workflow: delete-stores
    let saga_delete = build_delete_store_saga(service.clone());
    let del_res = saga_delete
        .execute("tx_store_3", "delete-store", json!({ "id": store_id }))
        .await
        .expect("must execute delete-store workflow");

    assert_eq!(del_res["deleted"], true);
}
