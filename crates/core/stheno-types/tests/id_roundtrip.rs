//! P11-04: the prefix survives every boundary an id crosses (ADR-010).

use serde::{Deserialize, Serialize};
use stheno_types::{prefixed_id, Id};

struct Product;
struct Cart;

prefixed_id! {
    Product => "prod",
    Cart => "cart",
}

type ProductId = Id<Product>;
type CartId = Id<Cart>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Envelope {
    id: ProductId,
}

#[test]
fn display_renders_the_medusa_prefixed_form() {
    let id = ProductId::new();
    let rendered = id.to_string();
    assert!(
        rendered.starts_with("prod_"),
        "expected a prod_ prefix, got {rendered}"
    );
    // 26-char ULID plus "prod_"
    assert_eq!(rendered.len(), 5 + 26, "unexpected id length: {rendered}");
}

#[test]
fn from_string_roundtrips_without_losing_the_prefix() {
    let original = ProductId::new();
    let parsed = ProductId::from_string(original.to_string()).expect("must parse its own output");
    assert_eq!(original, parsed);
    assert_eq!(original.to_string(), parsed.to_string());
}

#[test]
fn serde_roundtrips_the_prefixed_form() {
    let envelope = Envelope {
        id: ProductId::new(),
    };
    let json = serde_json::to_string(&envelope).expect("serialize");
    assert!(
        json.contains("prod_"),
        "serialized form dropped the prefix: {json}"
    );

    let back: Envelope = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(envelope, back);
}

#[test]
fn a_parsed_id_equals_the_stored_one_without_suffix_matching() {
    // The regression this whole change exists to prevent: a caller's prefixed id
    // must compare equal to the stored id directly, so no service needs
    // `stored.ends_with(caller)` -- which is a full table scan on Postgres.
    let stored = ProductId::new();
    let from_http_path = ProductId::from_string(stored.to_string()).expect("parse");
    assert_eq!(stored, from_http_path);
}

#[test]
fn a_foreign_prefix_is_rejected_not_silently_accepted() {
    let cart = CartId::new();
    let err = ProductId::from_string(cart.to_string())
        .expect_err("a cart id must not parse as a product id");
    assert!(
        err.to_string().contains("prod_"),
        "error should name the expected prefix, got: {err}"
    );
}

#[test]
fn a_bare_ulid_is_rejected() {
    let bare = ProductId::new().ulid().to_string();
    assert!(
        ProductId::from_string(&bare).is_err(),
        "an unprefixed ULID must not parse; that was the old lossy behaviour"
    );
}

#[test]
fn a_malformed_ulid_is_rejected() {
    assert!(ProductId::from_string("prod_not-a-ulid").is_err());
    assert!(ProductId::from_string("prod_").is_err());
    assert!(ProductId::from_string("").is_err());
}

#[test]
fn ids_are_unique_and_copy() {
    let a = ProductId::new();
    let b = ProductId::new();
    assert_ne!(a, b);
    let copied = a;
    assert_eq!(a, copied);
}
