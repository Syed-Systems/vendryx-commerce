use serde::{Deserialize, Serialize};
use stheno_api_core::{DeleteResponse, ErrorResponse, GetResponse, ListResponse};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct DummyProduct {
    id: String,
    title: String,
}

#[test]
fn test_medusa_list_envelopes_serialization() {
    let products = vec![
        DummyProduct {
            id: "prod_1".to_string(),
            title: "Classic Tee".to_string(),
        },
    ];

    let envelope = ListResponse {
        data: products,
        count: 1,
        offset: 0,
        limit: 10,
    };

    let json_value = serde_json::to_value(&envelope).unwrap();

    assert_eq!(json_value["count"], 1);
    assert_eq!(json_value["offset"], 0);
    assert_eq!(json_value["limit"], 10);
    assert_eq!(json_value["data"][0]["id"], "prod_1");
    assert_eq!(json_value["data"][0]["title"], "Classic Tee");
}

#[test]
fn test_medusa_get_envelopes_serialization() {
    let product = DummyProduct {
        id: "prod_2".to_string(),
        title: "Hoodie".to_string(),
    };

    let envelope = GetResponse { data: product };
    let json_value = serde_json::to_value(&envelope).unwrap();

    assert_eq!(json_value["data"]["id"], "prod_2");
    assert_eq!(json_value["data"]["title"], "Hoodie");
}

#[test]
fn test_medusa_delete_envelopes_serialization() {
    let envelope = DeleteResponse {
        id: "prod_delete_1".to_string(),
        object: "product".to_string(),
        deleted: true,
    };

    let json_value = serde_json::to_value(&envelope).unwrap();

    assert_eq!(json_value["id"], "prod_delete_1");
    assert_eq!(json_value["object"], "product");
    assert_eq!(json_value["deleted"], true);
}

#[test]
fn test_medusa_flat_error_envelopes_serialization() {
    let envelope = ErrorResponse::new("not_found", "Product not found");
    let json_value = serde_json::to_value(&envelope).unwrap();

    assert_eq!(json_value["type"], "not_found");
    assert_eq!(json_value["message"], "Product not found");
}
