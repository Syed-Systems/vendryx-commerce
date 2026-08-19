//! Service layer for `stheno-pricing`, backed by Postgres (P12-03).
//!
//! Multi-attribute specificity ranking, quantity tiers, price lists, and batch
//! resolution — as before. What changed is where the candidates come from.
//!
//! # Why the ranking still runs in Rust
//!
//! `calculate_prices_batch` loads every candidate price for the requested sets,
//! plus their rules, lists, list rules and preferences, then ranks them exactly
//! as the in-memory version did. The ranking is domain logic, not a query
//! concern, and moving it into SQL is a *measured* decision, not an assumed one
//! — that is what P12-04 benchmarks and P12-05 implements.
//!
//! The loads are batched by design: one query per entity kind for the whole
//! batch, never one per price set. An N+1 here would be the first thing the
//! latency budget catches.

use chrono::Utc;
use std::collections::HashMap;
use stheno_db::filter::MAX_IN_ELEMENTS;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{Price, PriceList, PriceListRule, PricePreference, PriceRule, PriceSet};

/// Page size for internal listings. `FilterSpec` clamps to this anyway; naming
/// it makes the ceiling visible at the call site.
const LIST_LIMIT: usize = stheno_db::filter::MAX_PAGE_LIMIT;

#[derive(Clone)]
pub struct PricingService {
    price_sets: PgCrudRepository<PriceSet>,
    prices: PgCrudRepository<Price>,
    price_rules: PgCrudRepository<PriceRule>,
    price_lists: PgCrudRepository<PriceList>,
    price_list_rules: PgCrudRepository<PriceListRule>,
    preferences: PgCrudRepository<PricePreference>,
    /// A variant does not hold a price; it links to a price set. Resolving that link is
    /// the first half of "what does this variant cost", so it belongs here rather than in
    /// each caller.
    links: stheno_links::PgLinkStore,
}

impl PricingService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            price_sets: PgCrudRepository::new(pool.clone()),
            prices: PgCrudRepository::new(pool.clone()),
            price_rules: PgCrudRepository::new(pool.clone()),
            price_lists: PgCrudRepository::new(pool.clone()),
            price_list_rules: PgCrudRepository::new(pool.clone()),
            preferences: PgCrudRepository::new(pool.clone()),
            links: stheno_links::PgLinkStore::new(pool),
        }
    }

    fn live(limit: usize) -> FilterSpec {
        FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, limit)
    }

    /// Reads every row matching `filter`, following pagination to the end.
    ///
    /// `FilterSpec::page_limit` clamps to `MAX_PAGE_LIMIT`, which is a cap on
    /// what a *client* may request. An internal load must not inherit it: one
    /// page of 100 rows silently truncated the candidate set for a 100-item
    /// cart, and the missing prices simply did not resolve.
    async fn load_all<T: stheno_db::CrudEntity>(
        repo: &PgCrudRepository<T>,
        filter: &FilterSpec,
    ) -> Result<Vec<T>> {
        let page_size = LIST_LIMIT;
        let mut out = Vec::new();
        let mut offset = 0;
        loop {
            let page = repo
                .list(None, &filter.clone().with_pagination(offset, page_size))
                .await?;
            let received = page.len();
            out.extend(page);
            if received < page_size {
                return Ok(out);
            }
            offset += page_size;
        }
    }

    /// Loads rows whose `column` is in `values`, in chunks.
    ///
    /// `FilterSpec` caps `$in` at `MAX_IN_ELEMENTS` to keep query plans sane, so
    /// more values than that must be chunked rather than rejected.
    async fn load_in<T: stheno_db::CrudEntity>(
        repo: &PgCrudRepository<T>,
        column: &str,
        values: &[String],
    ) -> Result<Vec<T>> {
        let mut out = Vec::new();
        for chunk in values.chunks(MAX_IN_ELEMENTS) {
            let filter =
                Self::live(LIST_LIMIT).with_predicate(column, FilterOp::In(chunk.to_vec()));
            out.extend(Self::load_all(repo, &filter).await?);
        }
        Ok(out)
    }

    /// Builds the price rows and their rules for one input price.
    fn build_price(
        p_in: &CreatePriceInput,
        price_set_id: &str,
        price_list_id: Option<String>,
    ) -> (Price, Vec<PriceRule>) {
        let pid: Id<Price> = Id::new();
        let pid_str = pid.to_string();
        let now = Utc::now();
        let rules_count = p_in.rules.as_ref().map(|r| r.len() as i32).unwrap_or(0);

        let price = Price {
            id: pid,
            price_set_id: price_set_id.to_string(),
            price_list_id,
            currency_code: p_in.currency_code.to_lowercase(),
            amount: p_in.amount,
            min_quantity: p_in.min_quantity,
            max_quantity: p_in.max_quantity,
            rules_count,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let rules = p_in
            .rules
            .as_ref()
            .map(|r_map| {
                r_map
                    .iter()
                    .map(|(attr, val)| PriceRule {
                        id: Id::new(),
                        price_id: pid_str.clone(),
                        attribute: attr.clone(),
                        value: val.clone(),
                        priority: 0,
                        created_at: now,
                        updated_at: now,
                        deleted_at: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        (price, rules)
    }

    async fn insert_prices(
        &self,
        inputs: &[CreatePriceInput],
        price_set_id: &str,
        list_id_override: Option<&str>,
    ) -> Result<()> {
        let mut prices = Vec::new();
        let mut rules = Vec::new();
        for p_in in inputs {
            let list_id = list_id_override
                .map(str::to_string)
                .or_else(|| p_in.price_list_id.clone());
            let (price, mut price_rules) = Self::build_price(p_in, price_set_id, list_id);
            prices.push(price);
            rules.append(&mut price_rules);
        }

        if !prices.is_empty() {
            self.prices.create_many(None, prices).await?;
        }
        if !rules.is_empty() {
            self.price_rules.create_many(None, rules).await?;
        }
        Ok(())
    }

    // ---- price sets -----------------------------------------------------

    pub async fn create_price_set(&self, input: CreatePriceSetInput) -> Result<PriceSetDTO> {
        let now = Utc::now();
        let set = PriceSet {
            id: Id::new(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let set = self.price_sets.create(None, set).await?;
        self.insert_prices(&input.prices, &set.id.to_string(), None)
            .await?;

        Ok(PriceSetDTO {
            id: Id::from_ulid(set.id.ulid()),
            created_at: set.created_at,
        })
    }

    pub async fn get_price_set(&self, id: &str) -> Result<PriceSetDTO> {
        let set = self.price_sets.retrieve(None, id).await?;
        Ok(PriceSetDTO {
            id: Id::from_ulid(set.id.ulid()),
            created_at: set.created_at,
        })
    }

    pub async fn update_price_set(
        &self,
        id: &str,
        input: UpdatePriceSetInput,
    ) -> Result<PriceSetDTO> {
        let mut set = self.price_sets.retrieve(None, id).await?;
        set.updated_at = Utc::now();
        let set_id = set.id.to_string();
        let set = self.price_sets.update(None, &set_id, set).await?;

        if let Some(new_prices) = input.prices {
            // Replace the set's prices. Deleting a price cascades to its rules
            // in the schema, so the rules do not need clearing separately.
            let existing =
                Self::load_in(&self.prices, "price_set_id", std::slice::from_ref(&set_id)).await?;
            for price in existing {
                self.prices.delete(None, &price.id.to_string()).await?;
            }
            self.insert_prices(&new_prices, &set_id, None).await?;
        }

        Ok(PriceSetDTO {
            id: Id::from_ulid(set.id.ulid()),
            created_at: set.created_at,
        })
    }

    pub async fn delete_price_set(&self, id: &str) -> Result<()> {
        let set = self.price_sets.retrieve(None, id).await?;
        self.price_sets.soft_delete(None, &set.id.to_string()).await
    }

    // ---- price lists ----------------------------------------------------

    pub async fn create_price_list(&self, input: CreatePriceListInput) -> Result<PriceListDTO> {
        let now = Utc::now();
        let list = PriceList {
            id: Id::new(),
            title: input.title.clone(),
            description: input.description.clone(),
            status: "active".to_string(),
            r#type: input.r#type.clone(),
            starts_at: input.starts_at,
            ends_at: input.ends_at,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let list = self.price_lists.create(None, list).await?;
        self.replace_list_rules(&list.id.to_string(), input.rules.as_ref())
            .await?;

        Ok(price_list_dto(&list))
    }

    async fn replace_list_rules(
        &self,
        price_list_id: &str,
        rules: Option<&HashMap<String, String>>,
    ) -> Result<()> {
        let Some(r_map) = rules else {
            return Ok(());
        };

        for existing in Self::load_in(
            &self.price_list_rules,
            "price_list_id",
            &[price_list_id.to_string()],
        )
        .await?
        {
            self.price_list_rules
                .delete(None, &existing.id.to_string())
                .await?;
        }

        let now = Utc::now();
        let replacements: Vec<PriceListRule> = r_map
            .iter()
            .map(|(attr, val)| PriceListRule {
                id: Id::new(),
                price_list_id: price_list_id.to_string(),
                attribute: attr.clone(),
                value: val.clone(),
                created_at: now,
                updated_at: now,
                deleted_at: None,
            })
            .collect();

        if !replacements.is_empty() {
            self.price_list_rules
                .create_many(None, replacements)
                .await?;
        }
        Ok(())
    }

    pub async fn get_price_list(&self, id: &str) -> Result<PriceListDTO> {
        self.price_lists
            .retrieve(None, id)
            .await
            .map(|l| price_list_dto(&l))
    }

    pub async fn list_price_lists(&self) -> Result<Vec<PriceListDTO>> {
        let filter = Self::live(LIST_LIMIT).with_order("created_at", OrderDirection::Asc);
        Ok(self
            .price_lists
            .list(None, &filter)
            .await?
            .iter()
            .map(price_list_dto)
            .collect())
    }

    pub async fn update_price_list(
        &self,
        id: &str,
        input: UpdatePriceListInput,
    ) -> Result<PriceListDTO> {
        let mut list = self.price_lists.retrieve(None, id).await?;

        if let Some(t) = input.title {
            list.title = t;
        }
        if let Some(d) = input.description {
            list.description = Some(d);
        }
        if let Some(s) = input.status {
            list.status = s;
        }
        if let Some(tp) = input.r#type {
            list.r#type = tp;
        }
        if input.starts_at.is_some() {
            list.starts_at = input.starts_at;
        }
        if input.ends_at.is_some() {
            list.ends_at = input.ends_at;
        }
        list.updated_at = Utc::now();

        let list_id = list.id.to_string();
        let list = self.price_lists.update(None, &list_id, list).await?;
        self.replace_list_rules(&list_id, input.rules.as_ref())
            .await?;

        Ok(price_list_dto(&list))
    }

    pub async fn delete_price_list(&self, id: &str) -> Result<()> {
        let list = self.price_lists.retrieve(None, id).await?;
        self.price_lists
            .soft_delete(None, &list.id.to_string())
            .await
    }

    pub async fn batch_price_list_prices(
        &self,
        price_list_id: &str,
        input: BatchPriceListPricesInput,
    ) -> Result<()> {
        if let Some(del_ids) = input.delete {
            for pid in del_ids {
                // Tolerant of an id that is already gone: a batch delete is
                // replayed on retry, and the second run must not fail.
                let _ = self.prices.delete(None, &pid).await;
            }
        }

        if let Some(creates) = input.create {
            if !creates.is_empty() {
                // ponytail: prices created through this endpoint are attached to
                // a placeholder set, inherited from the in-memory version which
                // had no foreign key to satisfy. The real fix is for the batch
                // API to name its price set; that is an API_SPEC change, not a
                // persistence one.
                let set_id = self.default_price_set().await?;
                self.insert_prices(&creates, &set_id, Some(price_list_id))
                    .await?;
            }
        }

        Ok(())
    }

    /// The placeholder price set used by `batch_price_list_prices`, created on
    /// first use so the foreign key is always satisfiable.
    async fn default_price_set(&self) -> Result<String> {
        const DEFAULT_ID: &str = "pset_00000000000000000000000000";

        if self.price_sets.retrieve(None, DEFAULT_ID).await.is_ok() {
            return Ok(DEFAULT_ID.to_string());
        }

        let now = Utc::now();
        let set = PriceSet {
            id: Id::from_string(DEFAULT_ID)?,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        match self.price_sets.create(None, set).await {
            Ok(_) => Ok(DEFAULT_ID.to_string()),
            // Another writer created it between the read and the write.
            Err(Error::Conflict { .. }) => Ok(DEFAULT_ID.to_string()),
            Err(other) => Err(other),
        }
    }

    // ---- price preferences ----------------------------------------------

    pub async fn set_price_preference(
        &self,
        attribute: &str,
        value: &str,
        is_tax_inclusive: bool,
    ) -> Result<()> {
        let existing = self.find_preference(attribute, value).await?;
        let now = Utc::now();

        match existing {
            Some(mut pref) => {
                pref.is_tax_inclusive = is_tax_inclusive;
                pref.updated_at = now;
                let id = pref.id.to_string();
                self.preferences.update(None, &id, pref).await?;
            }
            None => {
                let pref = PricePreference {
                    id: Id::new(),
                    attribute: attribute.to_string(),
                    value: value.to_string(),
                    is_tax_inclusive,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };
                // A unique index covers (attribute, value), so a concurrent
                // writer losing this race gets a Conflict rather than a second
                // row that would make tax inclusivity order-dependent.
                self.preferences.create(None, pref).await?;
            }
        }
        Ok(())
    }

    pub async fn delete_price_preference(&self, attribute: &str, value: &str) -> Result<()> {
        if let Some(pref) = self.find_preference(attribute, value).await? {
            self.preferences.delete(None, &pref.id.to_string()).await?;
        }
        Ok(())
    }

    async fn find_preference(
        &self,
        attribute: &str,
        value: &str,
    ) -> Result<Option<PricePreference>> {
        let filter = Self::live(1)
            .with_predicate("attribute", FilterOp::Eq(attribute.to_string()))
            .with_predicate("value", FilterOp::Eq(value.to_string()));
        Ok(self
            .preferences
            .list(None, &filter)
            .await?
            .into_iter()
            .next())
    }

    // ---- calculation ----------------------------------------------------

    /// Sets a variant's prices, creating and linking its price set on first use.
    ///
    /// The write half of `calculate_for_variants`. The admin already accepted a `prices`
    /// array on variant create and threw it away, so a merchant could set a price, get a
    /// 200, and have the storefront show nothing.
    ///
    /// Replaces rather than appends: the caller sends the price list it wants the variant
    /// to have, which is what the admin form means by saving.
    pub async fn set_variant_prices(
        &self,
        variant_id: &str,
        prices: Vec<CreatePriceInput>,
    ) -> Result<String> {
        self.set_linked_prices("product_variant_price_set", variant_id, prices)
            .await
    }

    /// `set_variant_prices` for a shipping option.
    pub async fn set_shipping_option_prices(
        &self,
        option_id: &str,
        prices: Vec<CreatePriceInput>,
    ) -> Result<String> {
        self.set_linked_prices("shipping_option_price_set", option_id, prices)
            .await
    }

    /// Prices any entity that reaches its price set through a one-to-one link.
    ///
    /// Variants and shipping options are the same problem — an id, a link table, a price
    /// set — so they are one implementation rather than two that drift apart.
    async fn set_linked_prices(
        &self,
        link_name: &str,
        entity_id: &str,
        prices: Vec<CreatePriceInput>,
    ) -> Result<String> {
        let existing = self.links.get_by_left(link_name, entity_id).await?;

        match existing.first() {
            Some(set_id) => {
                self.update_price_set(
                    set_id,
                    UpdatePriceSetInput {
                        prices: Some(prices),
                    },
                )
                .await?;
                Ok(set_id.clone())
            }
            None => {
                let set = self
                    .create_price_set(CreatePriceSetInput { prices })
                    .await?;
                let set_id = set.id.to_string();
                self.links.link(link_name, entity_id, &set_id).await?;
                Ok(set_id)
            }
        }
    }

    /// Resolves a price for each variant, keyed by variant id.
    ///
    /// This is the join the storefront could not make. `GET /store/products` returned
    /// products with no variants and no prices at all, so a product page had nothing to
    /// price and nothing to add to a cart — and the cart took `unit_price` from the
    /// request body, defaulting to zero.
    ///
    /// Variants with no linked price set are simply absent from the result rather than
    /// priced at zero: "unpriced" and "free" must not be the same value.
    pub async fn calculate_for_variants(
        &self,
        variant_ids: &[String],
        context: &PricingContext,
    ) -> Result<HashMap<String, CalculatedPriceDTO>> {
        self.calculate_for_linked("product_variant_price_set", variant_ids, context)
            .await
    }

    /// Resolves a price for each shipping option, keyed by option id.
    ///
    /// `POST /store/carts/{id}/shipping-methods` hardcoded `amount: Decimal::ZERO`, so
    /// every delivery was free however the merchant had priced it.
    pub async fn calculate_for_shipping_options(
        &self,
        option_ids: &[String],
        context: &PricingContext,
    ) -> Result<HashMap<String, CalculatedPriceDTO>> {
        self.calculate_for_linked("shipping_option_price_set", option_ids, context)
            .await
    }

    /// Entities with no linked price set are absent from the result rather than priced at
    /// zero: "unpriced" and "free" must not be the same value.
    async fn calculate_for_linked(
        &self,
        link_name: &str,
        ids: &[String],
        context: &PricingContext,
    ) -> Result<HashMap<String, CalculatedPriceDTO>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let sets_by_entity = self.links.get_by_left_many(link_name, ids).await?;

        let price_set_ids: Vec<String> = sets_by_entity.values().flatten().cloned().collect();
        if price_set_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // One evaluation for the whole batch, not one per entity.
        let calculated = self.calculate_prices_batch(&price_set_ids, context).await?;

        let mut out = HashMap::new();
        for (entity_id, set_ids) in sets_by_entity {
            // The link is OneToOne, so the first is the only one.
            if let Some(price) = set_ids.first().and_then(|id| calculated.get(id)) {
                out.insert(entity_id, price.clone());
            }
        }
        Ok(out)
    }

    pub async fn calculate_price(
        &self,
        price_set_id: &str,
        context: &PricingContext,
    ) -> Result<CalculatedPriceDTO> {
        let batch = self
            .calculate_prices_batch(&[price_set_id.to_string()], context)
            .await?;

        batch
            .get(price_set_id)
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: "price",
                id: format!("No price found for price_set_id {}", price_set_id),
            })
    }

    /// Resolves `calculated_price` for multiple price sets in a single
    /// evaluation (P3-09, P3-10).
    ///
    /// Follows Medusa precedence: Override Price List > Sale Price List >
    /// Specific Rules > Default Base Price.
    pub async fn calculate_prices_batch(
        &self,
        price_set_ids: &[String],
        context: &PricingContext,
    ) -> Result<HashMap<String, CalculatedPriceDTO>> {
        let curr_lower = context.currency_code.to_lowercase();
        let qty = context.quantity.unwrap_or(1);
        let now = context.as_of.unwrap_or_else(Utc::now);

        // One query per entity kind for the whole batch, never one per price
        // set. The candidate set is narrowed in SQL by price set and currency;
        // everything after this point is pure ranking over what was loaded.
        let candidates: Vec<Price> = {
            let mut out = Vec::new();
            for chunk in price_set_ids.chunks(MAX_IN_ELEMENTS) {
                let filter = Self::live(LIST_LIMIT)
                    .with_predicate("price_set_id", FilterOp::In(chunk.to_vec()))
                    .with_predicate("currency_code", FilterOp::Eq(curr_lower.clone()));
                out.extend(Self::load_all(&self.prices, &filter).await?);
            }
            out
        };

        let price_ids: Vec<String> = candidates.iter().map(|p| p.id.to_string()).collect();
        let mut rules_by_price: HashMap<String, Vec<PriceRule>> = HashMap::new();
        for rule in Self::load_in(&self.price_rules, "price_id", &price_ids).await? {
            rules_by_price
                .entry(rule.price_id.clone())
                .or_default()
                .push(rule);
        }

        let list_ids: Vec<String> = {
            let mut ids: Vec<String> = candidates
                .iter()
                .filter_map(|p| p.price_list_id.clone())
                .collect();
            ids.sort();
            ids.dedup();
            ids
        };
        let lists_by_id: HashMap<String, PriceList> =
            Self::load_in(&self.price_lists, "id", &list_ids)
                .await?
                .into_iter()
                .map(|l| (l.id.to_string(), l))
                .collect();

        let mut list_rules_by_list: HashMap<String, Vec<PriceListRule>> = HashMap::new();
        for rule in Self::load_in(&self.price_list_rules, "price_list_id", &list_ids).await? {
            list_rules_by_list
                .entry(rule.price_list_id.clone())
                .or_default()
                .push(rule);
        }

        // The preference table is small (one row per attribute/value pair a
        // deployment configures), so it is loaded whole rather than probed.
        let prefs: HashMap<String, PricePreference> =
            Self::load_all(&self.preferences, &Self::live(LIST_LIMIT))
                .await?
                .into_iter()
                .map(|p| (format!("{}::{}", p.attribute, p.value), p))
                .collect();

        let mut results = HashMap::new();

        for set_id in price_set_ids {
            let matching_prices: Vec<&Price> = candidates
                .iter()
                .filter(|p| {
                    p.price_set_id == *set_id
                        && p.deleted_at.is_none()
                        && p.currency_code == curr_lower
                        && p.min_quantity.map(|min| qty >= min).unwrap_or(true)
                        && p.max_quantity.map(|max| qty <= max).unwrap_or(true)
                })
                .collect();

            let mut best_original: Option<(&Price, i32, i32)> = None; // (price, score, rules_matched)
            let mut best_plist_price: Option<(&Price, &PriceList, i32)> = None;

            for p in &matching_prices {
                let p_rules = rules_by_price.get(&p.id.to_string());
                let mut rules_matched = 0;
                let mut rule_disqualified = false;

                if let Some(r_list) = p_rules {
                    for r in r_list {
                        match r.attribute.as_str() {
                            "region_id" => {
                                if context.region_id.as_deref() == Some(&r.value) {
                                    rules_matched += 1;
                                } else {
                                    rule_disqualified = true;
                                }
                            }
                            "customer_group_id" => {
                                if context.customer_group_id.as_deref() == Some(&r.value) {
                                    rules_matched += 1;
                                } else {
                                    rule_disqualified = true;
                                }
                            }
                            "sales_channel_id" => {
                                if context.sales_channel_id.as_deref() == Some(&r.value) {
                                    rules_matched += 1;
                                } else {
                                    rule_disqualified = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if rule_disqualified {
                    continue;
                }

                if let Some(plist_id) = &p.price_list_id {
                    if let Some(plist) = lists_by_id.get(plist_id) {
                        let mut plist_matches = plist.status == "active"
                            && plist.starts_at.map(|s| now >= s).unwrap_or(true)
                            && plist.ends_at.map(|e| now <= e).unwrap_or(true);

                        let mut pl_rules_matched = 0;
                        if let Some(pl_rules) = list_rules_by_list.get(plist_id) {
                            for lr in pl_rules {
                                match lr.attribute.as_str() {
                                    "customer_group_id" => {
                                        if context.customer_group_id.as_deref() == Some(&lr.value) {
                                            pl_rules_matched += 1;
                                        } else {
                                            plist_matches = false;
                                        }
                                    }
                                    "sales_channel_id" => {
                                        if context.sales_channel_id.as_deref() == Some(&lr.value) {
                                            pl_rules_matched += 1;
                                        } else {
                                            plist_matches = false;
                                        }
                                    }
                                    "region_id" => {
                                        if context.region_id.as_deref() == Some(&lr.value) {
                                            pl_rules_matched += 1;
                                        } else {
                                            plist_matches = false;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if plist_matches {
                            let base_score = if plist.r#type == "override" { 100 } else { 10 };
                            let score = rules_matched + pl_rules_matched * 10 + base_score;
                            if best_plist_price
                                .as_ref()
                                .map(|(_, _, s)| score > *s)
                                .unwrap_or(true)
                            {
                                best_plist_price = Some((p, plist, score));
                            }
                        }
                    }
                } else {
                    // Quantity specificity: a price with min_quantity is more
                    // specific than one without, when rules_matched is equal.
                    let qty_bonus = if p.min_quantity.is_some() { 1 } else { 0 };
                    let score = rules_matched * 10 + qty_bonus;
                    if best_original
                        .as_ref()
                        .map(|(_, s, _)| score > *s)
                        .unwrap_or(true)
                    {
                        best_original = Some((p, score, rules_matched));
                    }
                }
            }

            if let Some((orig_price, _score, orig_matched_count)) = best_original {
                let tax_inclusive = context
                    .region_id
                    .as_ref()
                    .and_then(|r| prefs.get(&format!("region_id::{}", r)))
                    .or_else(|| prefs.get(&format!("currency_code::{}", curr_lower)))
                    .map(|pref| pref.is_tax_inclusive)
                    .unwrap_or(false);

                if let Some((plist_p, plist, _)) = best_plist_price {
                    results.insert(
                        set_id.clone(),
                        CalculatedPriceDTO {
                            price_set_id: set_id.clone(),
                            calculated_amount: wire_amount(plist_p.amount),
                            original_amount: wire_amount(orig_price.amount),
                            currency_code: curr_lower.clone(),
                            is_tax_inclusive: tax_inclusive,
                            price_list_type: Some(plist.r#type.clone()),
                            matched_rules_count: plist_p.rules_count,
                            min_quantity: plist_p.min_quantity,
                        },
                    );
                } else {
                    results.insert(
                        set_id.clone(),
                        CalculatedPriceDTO {
                            price_set_id: set_id.clone(),
                            calculated_amount: wire_amount(orig_price.amount),
                            original_amount: wire_amount(orig_price.amount),
                            currency_code: curr_lower.clone(),
                            is_tax_inclusive: tax_inclusive,
                            price_list_type: None,
                            matched_rules_count: orig_matched_count,
                            min_quantity: orig_price.min_quantity,
                        },
                    );
                }
            }
        }

        Ok(results)
    }
}

/// Strips the trailing zeros a `numeric(20,6)` column adds.
///
/// The stored scale is a property of the column, not of the price: a value
/// entered as 110.00 comes back as 110.000000 and would otherwise render that
/// way on the wire. `normalize` changes no value, only its scale, so precision
/// (ADR-006) is untouched.
fn wire_amount(amount: rust_decimal::Decimal) -> rust_decimal::Decimal {
    amount.normalize()
}

fn price_list_dto(l: &PriceList) -> PriceListDTO {
    PriceListDTO {
        id: Id::from_ulid(l.id.ulid()),
        title: l.title.clone(),
        r#type: l.r#type.clone(),
        status: l.status.clone(),
        starts_at: l.starts_at,
        ends_at: l.ends_at,
        created_at: l.created_at,
    }
}
