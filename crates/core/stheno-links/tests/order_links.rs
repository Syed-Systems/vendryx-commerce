use stheno_links::*;

#[test]
fn test_order_cross_module_links_crud_and_cascade() {
    let store = LinkStore::new();

    let order_id = "order_123";
    let cart_id = "cart_456";
    let pcol_id = "pcol_789";
    let promo_id = "promo_101";
    let ful_id = "ful_202";

    // 1. Link order -> cart
    store.link("order_cart", order_id, cart_id);

    // 2. Link order -> payment_collection
    store.link("order_payment_collection", order_id, pcol_id);

    // 3. Link order -> promotion
    store.link("order_promotion", order_id, promo_id);

    // 4. Link order -> fulfillment
    store.link("order_fulfillment", order_id, ful_id);

    // 5. Query bidirectional resolutions
    let carts = store.get_by_left("order_cart", order_id);
    assert_eq!(carts, vec![cart_id]);

    let orders_for_cart = store.get_by_right("order_cart", cart_id);
    assert_eq!(orders_for_cart, vec![order_id]);

    let pcols = store.get_by_left("order_payment_collection", order_id);
    assert_eq!(pcols, vec![pcol_id]);

    let promos = store.get_by_left("order_promotion", order_id);
    assert_eq!(promos, vec![promo_id]);

    let fulfillments = store.get_by_left("order_fulfillment", order_id);
    assert_eq!(fulfillments, vec![ful_id]);

    // 6. Delete link
    store.unlink("order_fulfillment", order_id, ful_id);

    let empty_fuls = store.get_by_left("order_fulfillment", order_id);
    assert!(empty_fuls.is_empty());
}
