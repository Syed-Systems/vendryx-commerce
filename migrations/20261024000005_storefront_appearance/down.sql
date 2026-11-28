ALTER TABLE store.tenant_branding
    DROP CONSTRAINT tenant_branding_archetype_check,
    DROP CONSTRAINT tenant_branding_card_style_check,
    DROP CONSTRAINT tenant_branding_image_aspect_check,
    DROP COLUMN storefront_archetype,
    DROP COLUMN product_card_style,
    DROP COLUMN product_image_aspect;
