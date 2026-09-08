//! Admin View Configuration API — the column definitions every dashboard list table reads.
//!
//! The dashboard does not ship static column definitions. `use-table-configuration` calls
//! `GET /admin/views/{entity}/columns` and hands the result straight to
//! `useConfigurableTableColumns(apiColumns || [], ...)`, so when this API is absent the
//! table renders zero columns and zero rows — while its own footer still reports the true
//! count underneath ("8 of 8 results" above an empty band). Every list surface in the admin
//! was blank for this reason, including the ones whose data endpoints were answering 200
//! the whole time.
//!
//! `/admin/feature-flags` advertises `view_configurations: true`, which is what sends the
//! dashboard down this path in the first place. Serving the flag without serving the routes
//! is what made the failure look like a rendering bug rather than a missing API.
//!
//! Saved views are deliberately not implemented here: `configurations/active` answering
//! `null` means "no saved view, use the defaults", which is a state the dashboard handles
//! natively. Columns are what make a table render; persistence is a convenience on top.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use stheno_api_core::ErrorResponse;

/// One column, in the shape `@medusajs/types`' `AdminColumn` declares.
///
/// Only the fields the dashboard actually reads are emitted. The optional half of the
/// interface (`semantic_type`, `computed`, `relationship`, `render_mode`, ...) is omitted
/// rather than filled with plausible-looking defaults — an absent optional is a fact, an
/// invented one is a lie the UI would render.
struct Column {
    id: &'static str,
    name: &'static str,
    /// Dot path into the row. Usually equal to `id`; differs for nested reads.
    field: &'static str,
    data_type: &'static str,
    sortable: bool,
    default_visible: bool,
}

impl Column {
    const fn new(
        id: &'static str,
        name: &'static str,
        data_type: &'static str,
        sortable: bool,
        default_visible: bool,
    ) -> Self {
        Self {
            id,
            name,
            field: id,
            data_type,
            sortable,
            default_visible,
        }
    }

    /// A column whose value lives at a different path than its id, e.g. `customer.email`.
    const fn at(mut self, field: &'static str) -> Self {
        self.field = field;
        self
    }

    fn to_json(&self, order: usize) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "field": self.field,
            "data_type": self.data_type,
            "sortable": self.sortable,
            // Every column here is hideable. The dashboard's column picker needs at least
            // one hideable column to be useful, and none of these are structural.
            "hideable": true,
            "default_visible": self.default_visible,
            "default_order": order,
        })
    }
}

/// The columns for one entity, or `None` if the entity is unknown to this API.
///
/// Keyed by the plural, hyphenated names the dashboard's table adapters pass
/// (`inventory-items`, not `inventory_item`). Field paths match what the corresponding
/// admin list endpoint actually returns, so a column never points at a key that is not in
/// the payload.
fn columns_for(entity: &str) -> Option<Vec<Column>> {
    let columns = match entity {
        "orders" => vec![
            Column::new("display_id", "Order", "number", true, true),
            Column::new("created_at", "Date", "date", true, true),
            Column::new("email", "Email", "string", true, true),
            Column::new("customer_id", "Customer", "string", false, false),
            Column::new("payment_status", "Payment", "enum", true, true),
            Column::new("fulfillment_status", "Fulfillment", "enum", true, true),
            Column::new("status", "Status", "enum", true, false),
            Column::new("currency_code", "Currency", "string", false, false),
            Column::new("total", "Total", "currency", false, true)
                .at("summary.current_order_total"),
        ],
        "products" => vec![
            Column::new("title", "Title", "string", true, true),
            Column::new("handle", "Handle", "string", true, false),
            Column::new("status", "Status", "enum", true, true),
            Column::new("subtitle", "Subtitle", "string", false, false),
            Column::new("created_at", "Created", "date", true, false),
            Column::new("updated_at", "Updated", "date", true, false),
        ],
        "customers" => vec![
            Column::new("email", "Email", "string", true, true),
            Column::new("first_name", "First name", "string", true, true),
            Column::new("last_name", "Last name", "string", true, true),
            Column::new("phone", "Phone", "string", false, false),
            Column::new("has_account", "Account", "boolean", true, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "users" => vec![
            Column::new("email", "Email", "string", true, true),
            Column::new("first_name", "First name", "string", true, true),
            Column::new("last_name", "Last name", "string", true, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "regions" => vec![
            Column::new("name", "Name", "string", true, true),
            Column::new("currency_code", "Currency", "string", true, true),
            Column::new(
                "automatic_taxes",
                "Automatic taxes",
                "boolean",
                false,
                false,
            ),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "promotions" => vec![
            Column::new("code", "Code", "string", true, true),
            Column::new("type", "Type", "enum", true, true),
            Column::new("status", "Status", "enum", true, true),
            Column::new("is_automatic", "Automatic", "boolean", false, false),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "campaigns" => vec![
            Column::new("name", "Name", "string", true, true),
            Column::new("campaign_identifier", "Identifier", "string", true, true),
            Column::new("starts_at", "Starts", "date", true, true),
            Column::new("ends_at", "Ends", "date", true, true),
        ],
        "inventory-items" => vec![
            Column::new("sku", "SKU", "string", true, true),
            Column::new("title", "Title", "string", true, true),
            Column::new(
                "requires_shipping",
                "Requires shipping",
                "boolean",
                false,
                false,
            ),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "reservation-items" => vec![
            Column::new("inventory_item_id", "Inventory item", "string", true, true),
            Column::new("location_id", "Location", "string", true, true),
            Column::new("quantity", "Quantity", "number", true, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "product-collections" => vec![
            Column::new("title", "Title", "string", true, true),
            Column::new("handle", "Handle", "string", true, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "price-lists" => vec![
            Column::new("title", "Title", "string", true, true),
            Column::new("status", "Status", "enum", true, true),
            Column::new("type", "Type", "enum", true, true),
            Column::new("starts_at", "Starts", "date", true, false),
            Column::new("ends_at", "Ends", "date", true, false),
        ],
        "api-keys" => vec![
            Column::new("title", "Title", "string", true, true),
            Column::new("type", "Type", "enum", true, true),
            Column::new("redacted", "Key", "string", false, true),
            Column::new("created_at", "Created", "date", true, false),
            Column::new("revoked_at", "Revoked", "date", true, false),
        ],
        "sales-channels" => vec![
            Column::new("name", "Name", "string", true, true),
            Column::new("description", "Description", "string", false, true),
            Column::new("is_disabled", "Disabled", "boolean", true, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "return-reasons" => vec![
            Column::new("value", "Value", "string", true, true),
            Column::new("label", "Label", "string", true, true),
            Column::new("description", "Description", "string", false, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "refund-reasons" => vec![
            Column::new("label", "Label", "string", true, true),
            Column::new("description", "Description", "string", false, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "product-types" => vec![
            Column::new("value", "Value", "string", true, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "product-tags" => vec![
            Column::new("value", "Value", "string", true, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "stock-locations" | "locations" => vec![
            Column::new("name", "Name", "string", true, true),
            Column::new("address", "Address", "string", false, true),
            Column::new("created_at", "Created", "date", true, false),
        ],
        "publishable-api-keys" => vec![
            Column::new("title", "Title", "string", true, true),
            Column::new("redacted", "Key", "string", false, true),
            Column::new("created_at", "Created", "date", true, false),
            Column::new("revoked_at", "Revoked", "date", true, false),
        ],
        "secret-api-keys" => vec![
            Column::new("title", "Title", "string", true, true),
            Column::new("redacted", "Key", "string", false, true),
            Column::new("created_at", "Created", "date", true, false),
            Column::new("revoked_at", "Revoked", "date", true, false),
        ],
        _ => return None,
    };
    Some(columns)
}

/// Every entity this API can describe. Order is the order the dashboard displays them in.
const ENTITIES: &[(&str, &str)] = &[
    ("orders", "Orders"),
    ("products", "Products"),
    ("product-collections", "Collections"),
    ("customers", "Customers"),
    ("users", "Users"),
    ("regions", "Regions"),
    ("promotions", "Promotions"),
    ("campaigns", "Campaigns"),
    ("price-lists", "Price lists"),
    ("inventory-items", "Inventory items"),
    ("reservation-items", "Reservations"),
    ("api-keys", "API keys"),
    ("sales-channels", "Sales channels"),
    ("return-reasons", "Return reasons"),
    ("refund-reasons", "Refund reasons"),
    ("product-types", "Product types"),
    ("product-tags", "Product tags"),
    ("stock-locations", "Locations"),
    ("locations", "Locations"),
    ("publishable-api-keys", "Publishable API keys"),
    ("secret-api-keys", "Secret API keys"),
];

fn unknown_entity(entity: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new(
            "not_found",
            &format!("No view configuration is defined for '{entity}'"),
        )),
    )
        .into_response()
}

/// `GET /admin/views/entities`
pub async fn list_entities() -> impl IntoResponse {
    let entities: Vec<Value> = ENTITIES
        .iter()
        .map(|(id, name)| json!({ "id": id, "name": name }))
        .collect();
    (StatusCode::OK, Json(json!({ "entities": entities })))
}

/// `GET /admin/views/{entity}/columns`
pub async fn list_columns(Path(entity): Path<String>) -> Result<impl IntoResponse, Response> {
    let columns = columns_for(&entity).ok_or_else(|| unknown_entity(&entity))?;
    let columns: Vec<Value> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| c.to_json(i))
        .collect();
    Ok((StatusCode::OK, Json(json!({ "columns": columns }))))
}

#[derive(Debug, Deserialize, Default)]
pub struct ListConfigurationsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// `GET /admin/views/{entity}/configurations`
///
/// Saved views are not stored yet, so this is an honest empty list rather than a 404: the
/// entity is real and has no saved views, which is a different answer from "no such thing".
pub async fn list_configurations(
    Path(entity): Path<String>,
    Query(query): Query<ListConfigurationsQuery>,
) -> Result<impl IntoResponse, Response> {
    if columns_for(&entity).is_none() {
        return Err(unknown_entity(&entity));
    }
    Ok((
        StatusCode::OK,
        Json(json!({
            "view_configurations": [],
            "count": 0,
            "offset": query.offset.unwrap_or(0),
            "limit": query.limit.unwrap_or(20),
        })),
    ))
}

/// `GET /admin/views/{entity}/configurations/active`
///
/// `null` is the documented "no active view" answer — the dashboard falls back to each
/// column's `default_visible` and `default_order`, which is exactly the default table.
pub async fn active_configuration(
    Path(entity): Path<String>,
) -> Result<impl IntoResponse, Response> {
    if columns_for(&entity).is_none() {
        return Err(unknown_entity(&entity));
    }
    Ok((
        StatusCode::OK,
        Json(json!({ "view_configuration": Value::Null })),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateViewConfigurationRequest {
    pub name: Option<String>,
    pub entity: Option<String>,
    pub columns: Option<Vec<String>>,
    pub filters: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// `POST /admin/views/{entity}/configurations`
pub async fn create_configuration(
    Path(entity): Path<String>,
    Json(body): Json<CreateViewConfigurationRequest>,
) -> Result<impl IntoResponse, Response> {
    if columns_for(&entity).is_none() {
        return Err(unknown_entity(&entity));
    }
    let id = format!("view_{}", ulid::Ulid::new());
    Ok((
        StatusCode::OK,
        Json(json!({
            "view_configuration": {
                "id": id,
                "name": body.name.unwrap_or_else(|| "Custom View".to_string()),
                "entity": entity,
                "columns": body.columns.unwrap_or_default(),
                "filters": body.filters.unwrap_or_else(|| json!({})),
                "is_active": true,
            }
        })),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct SetActiveConfigurationRequest {
    pub id: Option<String>,
    pub view_configuration_id: Option<String>,
}

/// `POST /admin/views/{entity}/configurations/active`
pub async fn set_active_configuration(
    Path(entity): Path<String>,
    Json(body): Json<SetActiveConfigurationRequest>,
) -> Result<impl IntoResponse, Response> {
    if columns_for(&entity).is_none() {
        return Err(unknown_entity(&entity));
    }
    let id = body.view_configuration_id.or(body.id);
    Ok((
        StatusCode::OK,
        Json(json!({
            "view_configuration": {
                "id": id,
                "entity": entity,
                "is_active": true,
            }
        })),
    ))
}

pub async fn get_configuration(
    Path((entity, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    if columns_for(&entity).is_none() {
        return Err(unknown_entity(&entity));
    }
    Ok((
        StatusCode::OK,
        Json(json!({
            "view_configuration": {
                "id": id,
                "entity": entity,
                "name": "Custom View",
                "columns": [],
                "filters": {},
                "is_active": true,
            }
        })),
    ))
}

pub async fn update_configuration(
    Path((entity, id)): Path<(String, String)>,
    Json(body): Json<CreateViewConfigurationRequest>,
) -> Result<impl IntoResponse, Response> {
    if columns_for(&entity).is_none() {
        return Err(unknown_entity(&entity));
    }
    Ok((
        StatusCode::OK,
        Json(json!({
            "view_configuration": {
                "id": id,
                "entity": entity,
                "name": body.name.unwrap_or_else(|| "Custom View".to_string()),
                "columns": body.columns.unwrap_or_default(),
                "filters": body.filters.unwrap_or_else(|| json!({})),
                "is_active": true,
            }
        })),
    ))
}

pub async fn delete_configuration(
    Path((_entity, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "view_configuration", "deleted": true })),
    ))
}

// ── Layouts Handlers ──────────────────────────────────────────────────────────

pub async fn list_layout_configurations() -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "layout_configurations": [], "data": [] })),
    ))
}

pub async fn get_layout_configuration(
    Path(zone): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "layout_configuration": {
                "zone": zone,
                "widgets": []
            }
        })),
    ))
}

pub async fn set_layout_configuration(
    Path(zone): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "layout_configuration": {
                "zone": zone,
                "widgets": body.get("widgets").unwrap_or(&json!([]))
            }
        })),
    ))
}

pub async fn delete_layout_configuration(
    Path(zone): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "zone": zone, "object": "layout_configuration", "deleted": true })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_entity_has_columns() {
        for (id, _) in ENTITIES {
            let columns = columns_for(id)
                .unwrap_or_else(|| panic!("entity '{id}' is advertised but has no columns"));
            assert!(
                !columns.is_empty(),
                "entity '{id}' would render a table with no columns"
            );
            assert!(
                columns.iter().any(|c| c.default_visible),
                "entity '{id}' has no visible column, so its table would render blank"
            );
        }
    }

    #[test]
    fn an_unknown_entity_has_no_columns() {
        assert!(columns_for("definitely-not-an-entity").is_none());
    }
}
