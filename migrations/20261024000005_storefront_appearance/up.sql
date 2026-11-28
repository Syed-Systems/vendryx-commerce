-- ADR-035.5: persist merchant-approved storefront archetype and product presentation.
-- Values are allowlisted again at the service boundary; checks protect stored rows.
ALTER TABLE store.tenant_branding
    ADD COLUMN storefront_archetype text,
    ADD COLUMN product_card_style text,
    ADD COLUMN product_image_aspect text;

ALTER TABLE store.tenant_branding
    ADD CONSTRAINT tenant_branding_archetype_check
        CHECK (storefront_archetype IS NULL OR storefront_archetype IN ('baseline', 'editorial', 'dense_catalog')),
    ADD CONSTRAINT tenant_branding_card_style_check
        CHECK (product_card_style IS NULL OR product_card_style IN ('standard', 'editorial', 'dense')),
    ADD CONSTRAINT tenant_branding_image_aspect_check
        CHECK (product_image_aspect IS NULL OR product_image_aspect IN ('square', 'portrait', 'landscape'));
