use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::*;
use stheno_pricing::PricingService;

#[tokio::test]
async fn test_all_10_pricing_workflows() {
    let db = TestDb::migrated().await;
    let service = Arc::new(PricingService::new(create_pool(&db.url, 5).expect("pool")));

    // 1. Workflow: create-price-sets
    let saga_create_set = build_create_price_set_saga(service.clone());
    let set_res = saga_create_set
        .execute(
            "tx_pset_1",
            "create-price-set",
            json!({
                "prices": [
                    {
                        "currency_code": "usd",
                        "amount": "100.00",
                        "min_quantity": null,
                        "max_quantity": null,
                        "rules": null
                    },
                    {
                        "currency_code": "usd",
                        "amount": "80.00",
                        "min_quantity": 10,
                        "max_quantity": null,
                        "rules": null
                    }
                ]
            }),
        )
        .await
        .expect("must execute create-price-set workflow");

    let price_set_id = set_res["id"].as_str().unwrap().to_string();
    assert!(!price_set_id.is_empty());

    // 2. Workflow: update-price-sets
    let saga_update_set = build_update_price_set_saga(service.clone());
    let upd_res = saga_update_set
        .execute(
            "tx_pset_2",
            "update-price-set",
            json!({
                "id": price_set_id,
                "data": {
                    "prices": [
                        {
                            "currency_code": "usd",
                            "amount": "110.00",
                            "min_quantity": null,
                            "max_quantity": null,
                            "rules": null
                        }
                    ]
                }
            }),
        )
        .await
        .expect("must execute update-price-set workflow");

    assert_eq!(upd_res["id"], price_set_id);

    // 3. Workflow: create-price-lists
    let saga_create_list = build_create_price_list_saga(service.clone());
    let list_res = saga_create_list
        .execute(
            "tx_plist_1",
            "create-price-list",
            json!({
                "title": "Summer Flash Sale",
                "description": "20% off selected items",
                "type": "sale",
                "starts_at": null,
                "ends_at": null,
                "rules": {
                    "region_id": "reg_us"
                }
            }),
        )
        .await
        .expect("must execute create-price-list workflow");

    let price_list_id = list_res["id"].as_str().unwrap().to_string();
    assert_eq!(list_res["title"], "Summer Flash Sale");
    assert_eq!(list_res["type"], "sale");

    // 4. Workflow: update-price-lists
    let saga_update_list = build_update_price_list_saga(service.clone());
    let list_upd_res = saga_update_list
        .execute(
            "tx_plist_2",
            "update-price-list",
            json!({
                "id": price_list_id,
                "data": {
                    "title": "Summer Mega Sale",
                    "status": "active"
                }
            }),
        )
        .await
        .expect("must execute update-price-list workflow");

    assert_eq!(list_upd_res["title"], "Summer Mega Sale");

    // 5. Workflow: batch-price-list-prices
    let saga_batch_prices = build_batch_price_list_prices_saga(service.clone());
    let batch_res = saga_batch_prices
        .execute(
            "tx_plist_3",
            "batch-price-list-prices",
            json!({
                "price_list_id": price_list_id,
                "data": {
                    "create": [
                        {
                            "currency_code": "usd",
                            "amount": "75.00",
                            "min_quantity": null,
                            "max_quantity": null,
                            "rules": null
                        }
                    ]
                }
            }),
        )
        .await
        .expect("must execute batch-price-list-prices workflow");

    assert_eq!(batch_res["success"], true);

    // 6. Workflow: create-price-preferences
    let saga_create_pref = build_create_price_preference_saga(service.clone());
    let pref_res = saga_create_pref
        .execute(
            "tx_ppref_1",
            "create-price-preference",
            json!({
                "attribute": "currency_code",
                "value": "eur",
                "is_tax_inclusive": true
            }),
        )
        .await
        .expect("must execute create-price-preference workflow");

    assert_eq!(pref_res["is_tax_inclusive"], true);

    // 7. Workflow: update-price-preferences
    let saga_update_pref = build_update_price_preference_saga(service.clone());
    let pref_upd_res = saga_update_pref
        .execute(
            "tx_ppref_2",
            "update-price-preference",
            json!({
                "attribute": "currency_code",
                "value": "eur",
                "is_tax_inclusive": false
            }),
        )
        .await
        .expect("must execute update-price-preference workflow");

    assert_eq!(pref_upd_res["is_tax_inclusive"], false);

    // 8. Workflow: calculate-prices
    let saga_calc = build_calculate_prices_saga(service.clone());
    let calc_res = saga_calc
        .execute(
            "tx_calc_1",
            "calculate-prices",
            json!({
                "price_set_ids": [price_set_id],
                "context": {
                    "currency_code": "usd",
                    "region_id": null,
                    "customer_group_id": null,
                    "sales_channel_id": null,
                    "quantity": 1,
                    "as_of": null
                }
            }),
        )
        .await
        .expect("must execute calculate-prices workflow");

    assert!(calc_res.get(&price_set_id).is_some());
    // "110" and not "110.00": the amount now round-trips through a
    // numeric(20,6) column, and the DTO normalizes away the scale that column
    // adds. The value is unchanged -- only its trailing zeros are (P12-03).
    assert_eq!(calc_res[&price_set_id]["calculated_amount"], "110");

    // 9. Workflow: delete-price-lists
    let saga_delete_list = build_delete_price_list_saga(service.clone());
    let del_list_res = saga_delete_list
        .execute(
            "tx_plist_del_1",
            "delete-price-list",
            json!({ "id": price_list_id }),
        )
        .await
        .expect("must execute delete-price-list workflow");

    assert_eq!(del_list_res["deleted"], true);

    // 10. Workflow: delete-price-sets
    let saga_delete_set = build_delete_price_set_saga(service.clone());
    let del_set_res = saga_delete_set
        .execute(
            "tx_pset_del_1",
            "delete-price-set",
            json!({ "id": price_set_id }),
        )
        .await
        .expect("must execute delete-price-set workflow");

    assert_eq!(del_set_res["deleted"], true);

    db.drop().await;
}
