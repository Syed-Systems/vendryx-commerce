use chrono::Utc;
use stheno_product::models::{
    ProductCollection, ProductOption, ProductOptionValue, ProductTag, ProductType,
};

#[test]
fn test_taxonomy_collections_tags_types_options_lifecycle() {
    let now = Utc::now();

    let collection = ProductCollection {
        id: "pcol_summer_2026".to_string(),
        title: "Summer Collection 2026".to_string(),
        handle: "summer-2026".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    assert_eq!(collection.handle, "summer-2026");

    let tag = ProductTag {
        id: "ptag_eco".to_string(),
        value: "EcoFriendly".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    assert_eq!(tag.value, "EcoFriendly");

    let ptype = ProductType {
        id: "ptyp_apparel".to_string(),
        value: "Apparel".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    assert_eq!(ptype.value, "Apparel");

    let option = ProductOption {
        id: "opt_color".to_string(),
        product_id: "prod_sample".to_string(),
        title: "Color".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let opt_val = ProductOptionValue {
        id: "optval_navy".to_string(),
        option_id: option.id.clone(),
        value: "Navy Blue".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    assert_eq!(opt_val.value, "Navy Blue");
}
