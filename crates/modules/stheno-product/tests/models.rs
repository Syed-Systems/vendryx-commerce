use chrono::Utc;
use rust_decimal_macros::dec;
use serde_json::json;
use stheno_product::models::{
    compute_variant_option_values_hash, Product, ProductCategory, ProductCollection, ProductImage,
    ProductOption, ProductOptionValue, ProductProductOption, ProductProductOptionValue,
    ProductStatus, ProductTag, ProductType, ProductVariant, ProductVariantProductImage,
};

#[test]
fn test_all_12_product_entity_models_instantiation_and_serialization() {
    let now = Utc::now();

    // 1. Product
    let product = Product {
        id: "prod_01JTESTPROD00000000000000".to_string(),
        title: "Medusa Hoodie".to_string(),
        handle: "medusa-hoodie".to_string(),
        subtitle: Some("Comfortable organic hoodie".to_string()),
        description: Some("Made from 100% organic cotton".to_string()),
        status: ProductStatus::Published.as_str().to_string(),
        thumbnail: Some("https://example.com/hoodie.png".to_string()),
        weight: Some(dec!(500.0)),
        length: Some(dec!(30.0)),
        height: Some(dec!(10.0)),
        width: Some(dec!(20.0)),
        hs_code: Some("6109.10".to_string()),
        origin_country: Some("US".to_string()),
        mid_code: None,
        material: Some("Cotton".to_string()),
        discountable: true,
        external_id: None,
        type_id: Some("ptyp_01JTESTTYPE00000000000000".to_string()),
        collection_id: Some("pcol_01JTESTCOL00000000000000".to_string()),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: Some(json!({"season": "autumn"})),
    };
    let product_json = serde_json::to_value(&product).expect("serialize product");
    assert_eq!(product_json["id"], "prod_01JTESTPROD00000000000000");
    assert_eq!(product_json["handle"], "medusa-hoodie");
    assert_eq!(product_json["status"], "published");

    // 2. ProductVariant
    let variant = ProductVariant {
        id: "variant_01JTESTVAR00000000000000".to_string(),
        product_id: product.id.clone(),
        title: "Large / Black".to_string(),
        sku: Some("HOODIE-L-BLK".to_string()),
        barcode: Some("123456789012".to_string()),
        ean: None,
        upc: None,
        allow_backorder: false,
        manage_inventory: true,
        variant_rank: 0,
        weight: Some(dec!(520.0)),
        length: None,
        height: None,
        width: None,
        hs_code: None,
        origin_country: None,
        mid_code: None,
        material: None,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let variant_json = serde_json::to_value(&variant).expect("serialize variant");
    assert_eq!(variant_json["product_id"], product.id);

    // 3. ProductOption
    let option = ProductOption {
        id: "opt_01JTESTOPT00000000000000".to_string(),
        product_id: product.id.clone(),
        title: "Size".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let option_json = serde_json::to_value(&option).expect("serialize option");
    assert_eq!(option_json["title"], "Size");

    // 4. ProductOptionValue
    let option_value = ProductOptionValue {
        id: "optval_01JTESTOPTVAL00000000000".to_string(),
        option_id: option.id.clone(),
        value: "Large".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let option_val_json = serde_json::to_value(&option_value).expect("serialize option value");
    assert_eq!(option_val_json["value"], "Large");

    // 5. ProductProductOption
    let prod_opt = ProductProductOption {
        id: "propt_01JTESTPROPT000000000000".to_string(),
        product_id: product.id.clone(),
        option_id: option.id.clone(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };
    let prod_opt_json = serde_json::to_value(&prod_opt).expect("serialize product option join");
    assert_eq!(prod_opt_json["option_id"], option.id);

    // 6. ProductProductOptionValue
    let prod_opt_val = ProductProductOptionValue {
        id: "prov_01JTESTPROV0000000000000".to_string(),
        variant_id: variant.id.clone(),
        option_value_id: option_value.id.clone(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };
    let prov_json =
        serde_json::to_value(&prod_opt_val).expect("serialize variant option value join");
    assert_eq!(prov_json["variant_id"], variant.id);

    // 7. ProductImage
    let image = ProductImage {
        id: "img_01JTESTIMG00000000000000".to_string(),
        url: "https://cdn.example.com/hoodie_front.jpg".to_string(),
        rank: 0,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let img_json = serde_json::to_value(&image).expect("serialize image");
    assert_eq!(img_json["rank"], 0);

    // 8. ProductVariantProductImage
    let var_img = ProductVariantProductImage {
        id: "vimg_01JTESTVIMG0000000000000".to_string(),
        variant_id: variant.id.clone(),
        image_id: image.id.clone(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };
    let var_img_json = serde_json::to_value(&var_img).expect("serialize variant image join");
    assert_eq!(var_img_json["image_id"], image.id);

    // 9. ProductCategory
    let category = ProductCategory {
        id: "pcat_01JTESTCAT00000000000000".to_string(),
        name: "Apparel".to_string(),
        handle: "apparel".to_string(),
        parent_category_id: None,
        rank: 0,
        is_active: true,
        is_internal: false,
        mpath: "pcat_01JTESTCAT00000000000000.".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let cat_json = serde_json::to_value(&category).expect("serialize category");
    assert_eq!(cat_json["handle"], "apparel");

    // 10. ProductCollection
    let collection = ProductCollection {
        id: "pcol_01JTESTCOL00000000000000".to_string(),
        title: "Autumn Drop 2026".to_string(),
        handle: "autumn-drop-2026".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let col_json = serde_json::to_value(&collection).expect("serialize collection");
    assert_eq!(col_json["title"], "Autumn Drop 2026");

    // 11. ProductTag
    let tag = ProductTag {
        id: "ptag_01JTESTTAG00000000000000".to_string(),
        value: "EcoFriendly".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let tag_json = serde_json::to_value(&tag).expect("serialize tag");
    assert_eq!(tag_json["value"], "EcoFriendly");

    // 12. ProductType
    let ptype = ProductType {
        id: "ptyp_01JTESTTYPE00000000000000".to_string(),
        value: "Sweatshirt".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let type_json = serde_json::to_value(&ptype).expect("serialize type");
    assert_eq!(type_json["value"], "Sweatshirt");

    // Check variant option values hash computation (P2-04/P2-05)
    let hash1 =
        compute_variant_option_values_hash(vec!["optval_b".to_string(), "optval_a".to_string()]);
    let hash2 =
        compute_variant_option_values_hash(vec!["optval_a".to_string(), "optval_b".to_string()]);
    assert_eq!(
        hash1, hash2,
        "Sorted option value hashing must be deterministic"
    );
}
