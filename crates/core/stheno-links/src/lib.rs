//! `stheno-links`: Module links registry (37 cross-module link definitions + link CRUD operations).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    pub module: &'static str,
    pub entity: &'static str,
    pub field: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CascadeBehaviour {
    Delete,
    Nullify,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDefinition {
    pub name: &'static str,
    pub left: EntityRef,
    pub right: EntityRef,
    pub table: &'static str,
    pub cardinality: Cardinality,
    pub cascade: CascadeBehaviour,
}

/// The complete 36 cross-module link definitions (TRD §4.3 / DATA_MODEL.md).
pub static ALL_LINKS: &[LinkDefinition] = &[
    // 1. cart -> customer
    LinkDefinition {
        name: "cart_customer",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        right: EntityRef {
            module: "customer",
            entity: "customer",
            field: "id",
        },
        table: "link_cart_customer",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::Nullify,
    },
    // 2. cart -> payment_collection
    LinkDefinition {
        name: "cart_payment_collection",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "payment_collection",
            field: "id",
        },
        table: "link_cart_payment_collection",
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 3. cart -> promotion
    LinkDefinition {
        name: "cart_promotion",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        right: EntityRef {
            module: "promotion",
            entity: "promotion",
            field: "id",
        },
        table: "link_cart_promotion",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 4. cart -> region
    LinkDefinition {
        name: "cart_region",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        right: EntityRef {
            module: "region",
            entity: "region",
            field: "id",
        },
        table: "link_cart_region",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 5. cart -> sales_channel
    LinkDefinition {
        name: "cart_sales_channel",
        left: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        right: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        table: "link_cart_sales_channel",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 6. order -> customer
    LinkDefinition {
        name: "order_customer",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "customer",
            entity: "customer",
            field: "id",
        },
        table: "link_order_customer",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 7. order -> payment_collection
    LinkDefinition {
        name: "order_payment_collection",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "payment_collection",
            field: "id",
        },
        table: "link_order_payment_collection",
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 8. order -> promotion
    LinkDefinition {
        name: "order_promotion",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "promotion",
            entity: "promotion",
            field: "id",
        },
        table: "link_order_promotion",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::None,
    },
    // 9. order -> region
    LinkDefinition {
        name: "order_region",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "region",
            entity: "region",
            field: "id",
        },
        table: "link_order_region",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 10. order -> sales_channel
    LinkDefinition {
        name: "order_sales_channel",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        table: "link_order_sales_channel",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 11. order -> fulfillment
    LinkDefinition {
        name: "order_fulfillment",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "fulfillment",
            entity: "fulfillment",
            field: "id",
        },
        table: "link_order_fulfillment",
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 12. product -> sales_channel
    LinkDefinition {
        name: "product_sales_channel",
        left: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        right: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        table: "link_product_sales_channel",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 13. product_variant -> price_set
    LinkDefinition {
        name: "product_variant_price_set",
        left: EntityRef {
            module: "product",
            entity: "product_variant",
            field: "id",
        },
        right: EntityRef {
            module: "pricing",
            entity: "price_set",
            field: "id",
        },
        table: "link_product_variant_price_set",
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 14. product_variant -> inventory_item
    LinkDefinition {
        name: "product_variant_inventory_item",
        left: EntityRef {
            module: "product",
            entity: "product_variant",
            field: "id",
        },
        right: EntityRef {
            module: "inventory",
            entity: "inventory_item",
            field: "id",
        },
        table: "link_product_variant_inventory_item",
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 15. inventory_item -> stock_location
    LinkDefinition {
        name: "inventory_item_stock_location",
        left: EntityRef {
            module: "inventory",
            entity: "inventory_item",
            field: "id",
        },
        right: EntityRef {
            module: "stock_location",
            entity: "stock_location",
            field: "id",
        },
        table: "link_inventory_item_stock_location",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 16. sales_channel -> stock_location
    LinkDefinition {
        name: "sales_channel_stock_location",
        left: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        right: EntityRef {
            module: "stock_location",
            entity: "stock_location",
            field: "id",
        },
        table: "link_sales_channel_stock_location",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 17. fulfillment_set -> stock_location
    LinkDefinition {
        name: "fulfillment_set_stock_location",
        left: EntityRef {
            module: "fulfillment",
            entity: "fulfillment_set",
            field: "id",
        },
        right: EntityRef {
            module: "stock_location",
            entity: "stock_location",
            field: "id",
        },
        table: "link_fulfillment_set_stock_location",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 18. publishable_api_key -> sales_channel
    LinkDefinition {
        name: "publishable_api_key_sales_channel",
        left: EntityRef {
            module: "api_key",
            entity: "api_key",
            field: "id",
        },
        right: EntityRef {
            module: "sales_channel",
            entity: "sales_channel",
            field: "id",
        },
        table: "link_publishable_api_key_sales_channel",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 19. user -> rbac_role
    LinkDefinition {
        name: "user_rbac_role",
        left: EntityRef {
            module: "user",
            entity: "user",
            field: "id",
        },
        right: EntityRef {
            module: "rbac",
            entity: "role",
            field: "id",
        },
        table: "link_user_rbac_role",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 20. customer -> customer_group
    LinkDefinition {
        name: "customer_customer_group",
        left: EntityRef {
            module: "customer",
            entity: "customer",
            field: "id",
        },
        right: EntityRef {
            module: "customer",
            entity: "customer_group",
            field: "id",
        },
        table: "link_customer_customer_group",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 21. customer_group -> price_set
    LinkDefinition {
        name: "customer_group_price_set",
        left: EntityRef {
            module: "customer",
            entity: "customer_group",
            field: "id",
        },
        right: EntityRef {
            module: "pricing",
            entity: "price_set",
            field: "id",
        },
        table: "link_customer_group_price_set",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 22. region -> payment_provider
    LinkDefinition {
        name: "region_payment_provider",
        left: EntityRef {
            module: "region",
            entity: "region",
            field: "id",
        },
        right: EntityRef {
            module: "payment",
            entity: "payment_provider",
            field: "id",
        },
        table: "link_region_payment_provider",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 23. region -> tax_rate
    LinkDefinition {
        name: "region_tax_rate",
        left: EntityRef {
            module: "region",
            entity: "region",
            field: "id",
        },
        right: EntityRef {
            module: "tax",
            entity: "tax_rate",
            field: "id",
        },
        table: "link_region_tax_rate",
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 24. product_category -> product
    LinkDefinition {
        name: "product_category_product",
        left: EntityRef {
            module: "product",
            entity: "product_category",
            field: "id",
        },
        right: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        table: "link_product_category_product",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 25. product_collection -> product
    LinkDefinition {
        name: "product_collection_product",
        left: EntityRef {
            module: "product",
            entity: "product_collection",
            field: "id",
        },
        right: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        table: "link_product_collection_product",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Nullify,
    },
    // 26. product_tag -> product
    LinkDefinition {
        name: "product_tag_product",
        left: EntityRef {
            module: "product",
            entity: "product_tag",
            field: "id",
        },
        right: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        table: "link_product_tag_product",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 27. product_type -> product
    LinkDefinition {
        name: "product_type_product",
        left: EntityRef {
            module: "product",
            entity: "product_type",
            field: "id",
        },
        right: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        table: "link_product_type_product",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::Nullify,
    },
    // 28. product -> file
    LinkDefinition {
        name: "product_file",
        left: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        right: EntityRef {
            module: "file",
            entity: "file",
            field: "id",
        },
        table: "link_product_file",
        cardinality: Cardinality::ManyToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 29. promotion -> rule
    LinkDefinition {
        name: "promotion_rule",
        left: EntityRef {
            module: "promotion",
            entity: "promotion",
            field: "id",
        },
        right: EntityRef {
            module: "promotion",
            entity: "promotion_rule",
            field: "id",
        },
        table: "link_promotion_rule",
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 30. promotion -> campaign
    LinkDefinition {
        name: "promotion_campaign",
        left: EntityRef {
            module: "promotion",
            entity: "promotion",
            field: "id",
        },
        right: EntityRef {
            module: "promotion",
            entity: "campaign",
            field: "id",
        },
        table: "link_promotion_campaign",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::Nullify,
    },
    // 31. pricing_rule -> price_set
    LinkDefinition {
        name: "pricing_rule_price_set",
        left: EntityRef {
            module: "pricing",
            entity: "price_rule",
            field: "id",
        },
        right: EntityRef {
            module: "pricing",
            entity: "price_set",
            field: "id",
        },
        table: "link_pricing_rule_price_set",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 32. order -> order_change
    LinkDefinition {
        name: "order_order_change",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "order_change",
            entity: "order_change",
            field: "id",
        },
        table: "link_order_order_change",
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 33. customer -> customer_address
    LinkDefinition {
        name: "customer_customer_address",
        left: EntityRef {
            module: "customer",
            entity: "customer",
            field: "id",
        },
        right: EntityRef {
            module: "customer",
            entity: "address",
            field: "id",
        },
        table: "link_customer_customer_address",
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::Delete,
    },
    // 34. return_reason -> order_change
    LinkDefinition {
        name: "return_reason_order_change",
        left: EntityRef {
            module: "order",
            entity: "return_reason",
            field: "id",
        },
        right: EntityRef {
            module: "order_change",
            entity: "order_change",
            field: "id",
        },
        table: "link_return_reason_order_change",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::None,
    },
    // 35. notification -> customer
    LinkDefinition {
        name: "notification_customer",
        left: EntityRef {
            module: "notification",
            entity: "notification",
            field: "id",
        },
        right: EntityRef {
            module: "customer",
            entity: "customer",
            field: "id",
        },
        table: "link_notification_customer",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::Nullify,
    },
    // 36. translation -> entity
    LinkDefinition {
        name: "translation_entity",
        left: EntityRef {
            module: "translation",
            entity: "translation",
            field: "id",
        },
        right: EntityRef {
            module: "product",
            entity: "product",
            field: "id",
        },
        table: "link_translation_entity",
        cardinality: Cardinality::ManyToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 37. shipping_option -> price_set
    LinkDefinition {
        name: "shipping_option_price_set",
        left: EntityRef {
            module: "fulfillment",
            entity: "shipping_option",
            field: "id",
        },
        right: EntityRef {
            module: "pricing",
            entity: "price_set",
            field: "id",
        },
        table: "link_shipping_option_price_set",
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::Delete,
    },
    // 38. order -> cart
    LinkDefinition {
        name: "order_cart",
        left: EntityRef {
            module: "order",
            entity: "order",
            field: "id",
        },
        right: EntityRef {
            module: "cart",
            entity: "cart",
            field: "id",
        },
        table: "link_order_cart",
        cardinality: Cardinality::OneToOne,
        cascade: CascadeBehaviour::None,
    },
    // 39. order.return -> fulfillment
    LinkDefinition {
        name: "order_return_fulfillment",
        left: EntityRef {
            module: "order",
            entity: "return",
            field: "id",
        },
        right: EntityRef {
            module: "fulfillment",
            entity: "fulfillment",
            field: "id",
        },
        table: "link_order_return_fulfillment",
        cardinality: Cardinality::OneToMany,
        cascade: CascadeBehaviour::Delete,
    },
];

pub static STANDARD_LINKS: &[LinkDefinition] = ALL_LINKS;

type LinkIndex = HashMap<String, Vec<String>>;
type LinkTableMap = Arc<RwLock<HashMap<String, LinkIndex>>>;

/// In-memory link store for fast bidirectional resolution and testing.
#[derive(Clone, Default)]
pub struct LinkStore {
    // link_name -> (left_id -> Vec<right_id>)
    forward: LinkTableMap,
    // link_name -> (right_id -> Vec<left_id>)
    reverse: LinkTableMap,
}

impl LinkStore {
    pub fn new() -> Self {
        Self {
            forward: Arc::new(RwLock::new(HashMap::new())),
            reverse: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn link(&self, link_name: &str, left_id: impl Into<String>, right_id: impl Into<String>) {
        let left = left_id.into();
        let right = right_id.into();

        let mut fwd = self.forward.write().unwrap();
        fwd.entry(link_name.to_string())
            .or_default()
            .entry(left.clone())
            .or_default()
            .push(right.clone());

        let mut rev = self.reverse.write().unwrap();
        rev.entry(link_name.to_string())
            .or_default()
            .entry(right)
            .or_default()
            .push(left);
    }

    pub fn get_by_left(&self, link_name: &str, left_id: &str) -> Vec<String> {
        let fwd = self.forward.read().unwrap();
        fwd.get(link_name)
            .and_then(|m| m.get(left_id))
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_by_right(&self, link_name: &str, right_id: &str) -> Vec<String> {
        let rev = self.reverse.read().unwrap();
        rev.get(link_name)
            .and_then(|m| m.get(right_id))
            .cloned()
            .unwrap_or_default()
    }

    pub fn cascade_delete_left(&self, link_name: &str, left_id: &str) -> Vec<String> {
        let mut fwd = self.forward.write().unwrap();
        let mut rev = self.reverse.write().unwrap();

        let rights = fwd
            .get_mut(link_name)
            .and_then(|m| m.remove(left_id))
            .unwrap_or_default();

        if let Some(rev_map) = rev.get_mut(link_name) {
            for r in &rights {
                if let Some(lefts) = rev_map.get_mut(r) {
                    lefts.retain(|l| l != left_id);
                }
            }
        }

        rights
    }

    pub fn unlink(&self, link_name: &str, left_id: &str, right_id: &str) {
        let mut fwd = self.forward.write().unwrap();
        if let Some(map) = fwd.get_mut(link_name) {
            if let Some(list) = map.get_mut(left_id) {
                list.retain(|r| r != right_id);
            }
        }

        let mut rev = self.reverse.write().unwrap();
        if let Some(map) = rev.get_mut(link_name) {
            if let Some(list) = map.get_mut(right_id) {
                list.retain(|l| l != left_id);
            }
        }
    }
}
