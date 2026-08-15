use serde::{Deserialize, Serialize};
use stheno_types::Id;

struct ProductMarker;
type ProductId = Id<ProductMarker>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TestEntity {
    id: ProductId,
    name: String,
}

#[test]
fn test_id_uniqueness_and_equality() {
    let id1 = ProductId::new();
    let id2 = ProductId::new();
    assert_ne!(id1, id2);

    let id1_clone = id1;
    assert_eq!(id1, id1_clone);
}

#[test]
fn test_id_serde_roundtrip() {
    let entity = TestEntity {
        id: ProductId::new(),
        name: "Test Product".to_string(),
    };

    let json = serde_json::to_string(&entity).expect("Serialization failed");
    let deserialized: TestEntity = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(entity, deserialized);
}
