use chrono::Utc;
use stheno_product::models::{ProductImage, ProductVariantProductImage};

#[test]
fn test_product_and_variant_image_assignments() {
    let now = Utc::now();

    let img1 = ProductImage {
        id: "img_front".to_string(),
        url: "https://example.com/front.jpg".to_string(),
        rank: 0,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let img2 = ProductImage {
        id: "img_back".to_string(),
        url: "https://example.com/back.jpg".to_string(),
        rank: 1,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };

    let var_img = ProductVariantProductImage {
        id: "vimg_1".to_string(),
        variant_id: "variant_blue_42".to_string(),
        image_id: img1.id.clone(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };

    assert_eq!(var_img.variant_id, "variant_blue_42");
    assert_eq!(var_img.image_id, img1.id);
    assert_eq!(img2.rank, 1);
}
