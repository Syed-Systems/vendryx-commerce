-- Seed Stheno Store with demo catalog, regions, currencies, prices, and inventory

-- 1. Region & Country
INSERT INTO region.region (id, name, currency_code, automatic_taxes)
VALUES ('reg_01', 'United States', 'usd', true)
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, currency_code = EXCLUDED.currency_code;

INSERT INTO region.country (id, iso_2, iso_3, num_code, name, display_name, region_id)
VALUES ('c_us', 'us', 'usa', '840', 'United States', 'United States', 'reg_01')
ON CONFLICT (iso_2) DO UPDATE SET region_id = 'reg_01';

-- 2. Sales Channel & Store
INSERT INTO sales_channel.sales_channel (id, name, description, is_disabled)
VALUES ('sc_default', 'Default Web Channel', 'Primary web storefront', false)
ON CONFLICT (id) DO NOTHING;

INSERT INTO stock_location.stock_location (id, name)
VALUES ('loc_main', 'Main Fulfillment Center')
ON CONFLICT (id) DO NOTHING;

INSERT INTO store.store (id, name, default_sales_channel_id, default_region_id, default_location_id)
VALUES ('store_01', 'Stheno Store', 'sc_default', 'reg_01', 'loc_main')
ON CONFLICT (id) DO NOTHING;

INSERT INTO store.store_currency (id, store_id, currency_code, is_default)
VALUES ('scurr_usd', 'store_01', 'usd', true)
ON CONFLICT (id) DO NOTHING;

-- 3. Collections
INSERT INTO product.product_collection (id, title, handle)
VALUES 
  ('col_summer', 'Summer 2026', 'summer-2026'),
  ('col_apparel', 'Core Apparel', 'core-apparel'),
  ('col_accessories', 'Accessories', 'accessories')
ON CONFLICT (id) DO NOTHING;

-- 4. Products
INSERT INTO product.product (id, title, handle, subtitle, description, status, thumbnail, discountable, collection_id)
VALUES 
  (
    'prod_tshirt', 
    'Heavyweight Cotton Crewneck', 
    'heavyweight-cotton-crewneck', 
    '100% Organic Combed Cotton', 
    'A tailored everyday classic engineered for unmatched breathability and comfort. Pre-shrunk 260 GSM organic cotton with reinforced collar ribbing.',
    'published',
    'https://images.unsplash.com/photo-1521572267360-ee0c2909d518?auto=format&fit=crop&w=800&q=80',
    true,
    'col_apparel'
  ),
  (
    'prod_jacket', 
    'Selvedge Denim Overshirt', 
    'selvedge-denim-overshirt', 
    'Japanese 13.5oz Raw Indigo', 
    'Crafted from vintage shuttle-loom Japanese selvedge denim. Features antique brass hardware and interior utility pockets.',
    'published',
    'https://images.unsplash.com/photo-1576995853123-5a10305d93c0?auto=format&fit=crop&w=800&q=80',
    true,
    'col_summer'
  ),
  (
    'prod_hoodie', 
    'Merino Terry Pullover Hoodie', 
    'merino-terry-pullover-hoodie', 
    'Ultra-fine 19.5 Micron Merino Wool', 
    'Naturally odor-resistant and temperature regulating. French terry knit with double-lined hood and minimal metal accents.',
    'published',
    'https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=800&q=80',
    true,
    'col_apparel'
  ),
  (
    'prod_sneakers', 
    'Vulcanized Low-Top Canvas Shoes', 
    'vulcanized-low-top-canvas', 
    'Eco Canvas & Natural Rubber', 
    'Ortholite recycled foam insoles paired with flexible vulcanized rubber outsoles for all-day urban support.',
    'published',
    'https://images.unsplash.com/photo-1525966222134-fcfa99b8ae77?auto=format&fit=crop&w=800&q=80',
    true,
    'col_summer'
  ),
  (
    'prod_backpack', 
    'Full-Grain Leather Commuter Pack', 
    'full-grain-leather-commuter-pack', 
    'Vegetable-Tanned Italian Leather', 
    'Padded 16-inch laptop compartment, water-resistant YKK Excella zippers, and ergonomic contoured leather shoulder straps.',
    'published',
    'https://images.unsplash.com/photo-1548036328-c9fa89d128fa?auto=format&fit=crop&w=800&q=80',
    true,
    'col_accessories'
  )
ON CONFLICT (id) DO UPDATE SET status = 'published', thumbnail = EXCLUDED.thumbnail;

-- 5. Product Variants
DELETE FROM product.product_variant WHERE product_id IN ('prod_tshirt', 'prod_jacket', 'prod_hoodie', 'prod_sneakers', 'prod_backpack');

INSERT INTO product.product_variant (id, product_id, title, sku, allow_backorder, manage_inventory, variant_rank, option_values_hash)
VALUES
  ('var_tshirt_s', 'prod_tshirt', 'S / Black', 'TSHIRT-BLK-S', true, true, 0, 'size:s_color:blk'),
  ('var_tshirt_m', 'prod_tshirt', 'M / Black', 'TSHIRT-BLK-M', true, true, 1, 'size:m_color:blk'),
  ('var_tshirt_l', 'prod_tshirt', 'L / Black', 'TSHIRT-BLK-L', true, true, 2, 'size:l_color:blk'),
  ('var_jacket_m', 'prod_jacket', 'M / Indigo', 'JKT-IND-M', true, true, 0, 'size:m_color:ind'),
  ('var_jacket_l', 'prod_jacket', 'L / Indigo', 'JKT-IND-L', true, true, 1, 'size:l_color:ind'),
  ('var_hoodie_m', 'prod_hoodie', 'M / Heather Grey', 'HD-GRY-M', true, true, 0, 'size:m_color:gry'),
  ('var_hoodie_l', 'prod_hoodie', 'L / Heather Grey', 'HD-GRY-L', true, true, 1, 'size:l_color:gry'),
  ('var_sneakers_9', 'prod_sneakers', 'US 9 / Off-White', 'SNK-OW-9', true, true, 0, 'size:9_color:wht'),
  ('var_sneakers_10', 'prod_sneakers', 'US 10 / Off-White', 'SNK-OW-10', true, true, 1, 'size:10_color:wht'),
  ('var_backpack_std', 'prod_backpack', 'One Size / Cognac', 'BPK-COG-OS', true, true, 0, 'size:os_color:cog')
ON CONFLICT (id) DO NOTHING;

-- 6. Price Sets & Prices
INSERT INTO pricing.price_set (id) VALUES
  ('pset_tshirt_s'),
  ('pset_tshirt_m'),
  ('pset_tshirt_l'),
  ('pset_jacket_m'),
  ('pset_jacket_l'),
  ('pset_hoodie_m'),
  ('pset_hoodie_l'),
  ('pset_sneakers_9'),
  ('pset_sneakers_10'),
  ('pset_backpack_std')
ON CONFLICT (id) DO NOTHING;

INSERT INTO pricing.price (id, price_set_id, currency_code, amount, rules_count) VALUES
  ('pr_tshirt_s', 'pset_tshirt_s', 'usd', 35.00, 0),
  ('pr_tshirt_m', 'pset_tshirt_m', 'usd', 35.00, 0),
  ('pr_tshirt_l', 'pset_tshirt_l', 'usd', 35.00, 0),
  ('pr_jacket_m', 'pset_jacket_m', 'usd', 125.00, 0),
  ('pr_jacket_l', 'pset_jacket_l', 'usd', 125.00, 0),
  ('pr_hoodie_m', 'pset_hoodie_m', 'usd', 95.00, 0),
  ('pr_hoodie_l', 'pset_hoodie_l', 'usd', 95.00, 0),
  ('pr_sneakers_9', 'pset_sneakers_9', 'usd', 85.00, 0),
  ('pr_sneakers_10', 'pset_sneakers_10', 'usd', 85.00, 0),
  ('pr_backpack_std', 'pset_backpack_std', 'usd', 185.00, 0)
ON CONFLICT (id) DO UPDATE SET amount = EXCLUDED.amount;

-- 7. Inventory Items & Levels
INSERT INTO inventory.inventory_item (id, sku, requires_shipping) VALUES
  ('inv_tshirt_s', 'TSHIRT-BLK-S', true),
  ('inv_tshirt_m', 'TSHIRT-BLK-M', true),
  ('inv_tshirt_l', 'TSHIRT-BLK-L', true),
  ('inv_jacket_m', 'JKT-IND-M', true),
  ('inv_jacket_l', 'JKT-IND-L', true),
  ('inv_hoodie_m', 'HD-GRY-M', true),
  ('inv_hoodie_l', 'HD-GRY-L', true),
  ('inv_sneakers_9', 'SNK-OW-9', true),
  ('inv_sneakers_10', 'SNK-OW-10', true),
  ('inv_backpack_std', 'BPK-COG-OS', true)
ON CONFLICT (id) DO NOTHING;

INSERT INTO inventory.inventory_level (id, inventory_item_id, location_id, stocked_quantity, reserved_quantity, incoming_quantity) VALUES
  ('lvl_tshirt_s', 'inv_tshirt_s', 'loc_main', 50, 0, 0),
  ('lvl_tshirt_m', 'inv_tshirt_m', 'loc_main', 100, 0, 0),
  ('lvl_tshirt_l', 'inv_tshirt_l', 'loc_main', 75, 0, 0),
  ('lvl_jacket_m', 'inv_jacket_m', 'loc_main', 30, 0, 0),
  ('lvl_jacket_l', 'inv_jacket_l', 'loc_main', 25, 0, 0),
  ('lvl_hoodie_m', 'inv_hoodie_m', 'loc_main', 45, 0, 0),
  ('lvl_hoodie_l', 'inv_hoodie_l', 'loc_main', 40, 0, 0),
  ('lvl_sneakers_9', 'inv_sneakers_9', 'loc_main', 20, 0, 0),
  ('lvl_sneakers_10', 'inv_sneakers_10', 'loc_main', 25, 0, 0),
  ('lvl_backpack_std', 'inv_backpack_std', 'loc_main', 15, 0, 0)
ON CONFLICT (id) DO NOTHING;

-- 8. Links
-- Variant <-> Price Set
INSERT INTO links.link_product_variant_price_set (id, left_id, right_id) VALUES
  ('l_vp_tshirt_s', 'var_tshirt_s', 'pset_tshirt_s'),
  ('l_vp_tshirt_m', 'var_tshirt_m', 'pset_tshirt_m'),
  ('l_vp_tshirt_l', 'var_tshirt_l', 'pset_tshirt_l'),
  ('l_vp_jacket_m', 'var_jacket_m', 'pset_jacket_m'),
  ('l_vp_jacket_l', 'var_jacket_l', 'pset_jacket_l'),
  ('l_vp_hoodie_m', 'var_hoodie_m', 'pset_hoodie_m'),
  ('l_vp_hoodie_l', 'var_hoodie_l', 'pset_hoodie_l'),
  ('l_vp_sneakers_9', 'var_sneakers_9', 'pset_sneakers_9'),
  ('l_vp_sneakers_10', 'var_sneakers_10', 'pset_sneakers_10'),
  ('l_vp_backpack_std', 'var_backpack_std', 'pset_backpack_std')
ON CONFLICT (id) DO NOTHING;

-- Variant <-> Inventory Item
INSERT INTO links.link_product_variant_inventory_item (id, left_id, right_id) VALUES
  ('l_vi_tshirt_s', 'var_tshirt_s', 'inv_tshirt_s'),
  ('l_vi_tshirt_m', 'var_tshirt_m', 'inv_tshirt_m'),
  ('l_vi_tshirt_l', 'var_tshirt_l', 'inv_tshirt_l'),
  ('l_vi_jacket_m', 'var_jacket_m', 'inv_jacket_m'),
  ('l_vi_jacket_l', 'var_jacket_l', 'inv_jacket_l'),
  ('l_vi_hoodie_m', 'var_hoodie_m', 'inv_hoodie_m'),
  ('l_vi_hoodie_l', 'var_hoodie_l', 'inv_hoodie_l'),
  ('l_vi_sneakers_9', 'var_sneakers_9', 'inv_sneakers_9'),
  ('l_vi_sneakers_10', 'var_sneakers_10', 'inv_sneakers_10'),
  ('l_vi_backpack_std', 'var_backpack_std', 'inv_backpack_std')
ON CONFLICT (id) DO NOTHING;

-- Product <-> Sales Channel
INSERT INTO links.link_product_sales_channel (id, left_id, right_id) VALUES
  ('l_ps_tshirt', 'prod_tshirt', 'sc_default'),
  ('l_ps_jacket', 'prod_jacket', 'sc_default'),
  ('l_ps_hoodie', 'prod_hoodie', 'sc_default'),
  ('l_ps_sneakers', 'prod_sneakers', 'sc_default'),
  ('l_ps_backpack', 'prod_backpack', 'sc_default')
ON CONFLICT (id) DO NOTHING;
