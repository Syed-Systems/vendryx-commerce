CREATE SCHEMA IF NOT EXISTS inventory;

CREATE TABLE inventory.inventory_item (
    id text PRIMARY KEY,
    sku text NULL,
    hs_code text NULL,
    origin_country text NULL,
    mid_code text NULL,
    material text NULL,
    requires_shipping boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/inventory-items?sku=
CREATE INDEX idx_inventory_item_sku ON inventory.inventory_item(sku) WHERE deleted_at IS NULL;

CREATE TABLE inventory.inventory_level (
    id text PRIMARY KEY,
    inventory_item_id text NOT NULL,
    location_id text NOT NULL,
    stocked_quantity integer NOT NULL DEFAULT 0,
    reserved_quantity integer NOT NULL DEFAULT 0,
    incoming_quantity integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    CONSTRAINT unq_inventory_level_item_location UNIQUE (inventory_item_id, location_id)
);

-- serves: GET /admin/inventory-items/:id/location-levels
CREATE INDEX idx_inventory_level_item ON inventory.inventory_level(inventory_item_id) WHERE deleted_at IS NULL;

-- serves: GET /admin/stock-locations/:id/inventory-levels
CREATE INDEX idx_inventory_level_location ON inventory.inventory_level(location_id) WHERE deleted_at IS NULL;

CREATE TABLE inventory.reservation_item (
    id text PRIMARY KEY,
    line_item_id text NULL,
    inventory_item_id text NOT NULL,
    location_id text NOT NULL,
    quantity integer NOT NULL,
    description text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/reservations?line_item_id=
CREATE INDEX idx_reservation_item_line_item ON inventory.reservation_item(line_item_id) WHERE deleted_at IS NULL;

-- serves: GET /admin/reservations?inventory_item_id=
CREATE INDEX idx_reservation_item_inventory_item ON inventory.reservation_item(inventory_item_id) WHERE deleted_at IS NULL;
