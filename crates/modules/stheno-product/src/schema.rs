//! Diesel schema table definitions for `stheno-product` (TRD §4.1).

diesel::table! {
    product (id) {
        id -> Text,
        title -> Text,
        handle -> Text,
        subtitle -> Nullable<Text>,
        description -> Nullable<Text>,
        status -> Text,
        thumbnail -> Nullable<Text>,
        weight -> Nullable<Numeric>,
        length -> Nullable<Numeric>,
        height -> Nullable<Numeric>,
        width -> Nullable<Numeric>,
        hs_code -> Nullable<Text>,
        origin_country -> Nullable<Text>,
        mid_code -> Nullable<Text>,
        material -> Nullable<Text>,
        discountable -> Bool,
        external_id -> Nullable<Text>,
        type_id -> Nullable<Text>,
        collection_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_variant (id) {
        id -> Text,
        product_id -> Text,
        title -> Text,
        sku -> Nullable<Text>,
        barcode -> Nullable<Text>,
        ean -> Nullable<Text>,
        upc -> Nullable<Text>,
        allow_backorder -> Bool,
        manage_inventory -> Bool,
        variant_rank -> Integer,
        weight -> Nullable<Numeric>,
        length -> Nullable<Numeric>,
        height -> Nullable<Numeric>,
        width -> Nullable<Numeric>,
        hs_code -> Nullable<Text>,
        origin_country -> Nullable<Text>,
        mid_code -> Nullable<Text>,
        material -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_option (id) {
        id -> Text,
        product_id -> Text,
        title -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_option_value (id) {
        id -> Text,
        option_id -> Text,
        value -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_product_option (id) {
        id -> Text,
        product_id -> Text,
        option_id -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    product_product_option_value (id) {
        id -> Text,
        variant_id -> Text,
        option_value_id -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    product_image (id) {
        id -> Text,
        url -> Text,
        rank -> Integer,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_variant_product_image (id) {
        id -> Text,
        variant_id -> Text,
        image_id -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    product_category (id) {
        id -> Text,
        name -> Text,
        handle -> Text,
        parent_category_id -> Nullable<Text>,
        rank -> Integer,
        is_active -> Bool,
        is_internal -> Bool,
        mpath -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_collection (id) {
        id -> Text,
        title -> Text,
        handle -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_tag (id) {
        id -> Text,
        value -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_type (id) {
        id -> Text,
        value -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}
