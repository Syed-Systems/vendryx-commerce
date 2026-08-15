//! Service layer for `stheno-pricing` (TASKS.md P3-04 through P3-11).
//! Implements multi-attribute specificity ranking, quantity tiers, price lists, and single-query batch resolution.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{Price, PriceList, PriceListRule, PricePreference, PriceRule, PriceSet};

#[derive(Clone, Default)]
pub struct PricingService {
    price_sets: Arc<RwLock<HashMap<String, PriceSet>>>,
    prices: Arc<RwLock<HashMap<String, Price>>>,
    price_rules: Arc<RwLock<HashMap<String, Vec<PriceRule>>>>, // price_id -> rules
    price_lists: Arc<RwLock<HashMap<String, PriceList>>>,
    price_list_rules: Arc<RwLock<HashMap<String, Vec<PriceListRule>>>>, // price_list_id -> rules
    preferences: Arc<RwLock<HashMap<String, PricePreference>>>,         // attribute::value -> pref
}

impl PricingService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_price_set(&self, input: CreatePriceSetInput) -> Result<PriceSetDTO> {
        let set_id: Id<PriceSet> = Id::new();
        let set_id_str = set_id.to_string();

        let set = PriceSet {
            id: set_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut prices_map = self.prices.write().unwrap();
        let mut rules_map = self.price_rules.write().unwrap();

        for p_in in input.prices {
            let pid: Id<Price> = Id::new();
            let pid_str = pid.to_string();
            let rules_count = p_in.rules.as_ref().map(|r| r.len() as i32).unwrap_or(0);

            let price = Price {
                id: pid,
                price_set_id: set_id_str.clone(),
                price_list_id: p_in.price_list_id,
                currency_code: p_in.currency_code.to_lowercase(),
                amount: p_in.amount,
                min_quantity: p_in.min_quantity,
                max_quantity: p_in.max_quantity,
                rules_count,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };

            let mut r_rules = Vec::new();
            if let Some(r_map) = &p_in.rules {
                for (attr, val) in r_map {
                    r_rules.push(PriceRule {
                        id: Id::new(),
                        price_id: pid_str.clone(),
                        attribute: attr.clone(),
                        value: val.clone(),
                        priority: 0,
                        created_at: Utc::now(),
                    });
                }
            }

            rules_map.insert(pid_str.clone(), r_rules);
            prices_map.insert(pid_str, price);
        }

        let mut set_map = self.price_sets.write().unwrap();
        set_map.insert(set_id_str.clone(), set.clone());

        Ok(PriceSetDTO {
            id: Id::from_string(set.id.to_string()).unwrap_or_default(),
            created_at: set.created_at,
        })
    }

    pub async fn create_price_list(&self, input: CreatePriceListInput) -> Result<PriceListDTO> {
        let plist_id: Id<PriceList> = Id::new();
        let plist_id_str = plist_id.to_string();

        let list = PriceList {
            id: plist_id,
            title: input.title.clone(),
            description: input.description.clone(),
            status: "active".to_string(),
            r#type: input.r#type.clone(),
            starts_at: input.starts_at,
            ends_at: input.ends_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        if let Some(r_map) = &input.rules {
            let mut pl_rules = Vec::new();
            for (attr, val) in r_map {
                pl_rules.push(PriceListRule {
                    id: Id::new(),
                    price_list_id: plist_id_str.clone(),
                    attribute: attr.clone(),
                    value: val.clone(),
                    created_at: Utc::now(),
                });
            }
            let mut plr_map = self.price_list_rules.write().unwrap();
            plr_map.insert(plist_id_str.clone(), pl_rules);
        }

        let mut plist_map = self.price_lists.write().unwrap();
        plist_map.insert(plist_id_str.clone(), list.clone());

        Ok(PriceListDTO {
            id: Id::from_string(list.id.to_string()).unwrap_or_default(),
            title: list.title,
            r#type: list.r#type,
            status: list.status,
            starts_at: list.starts_at,
            ends_at: list.ends_at,
            created_at: list.created_at,
        })
    }

    pub async fn set_price_preference(
        &self,
        attribute: &str,
        value: &str,
        is_tax_inclusive: bool,
    ) -> Result<()> {
        let mut map = self.preferences.write().unwrap();
        let key = format!("{}::{}", attribute, value);
        map.insert(
            key,
            PricePreference {
                id: Id::new(),
                attribute: attribute.to_string(),
                value: value.to_string(),
                is_tax_inclusive,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );
        Ok(())
    }

    pub async fn delete_price_preference(&self, attribute: &str, value: &str) -> Result<()> {
        let mut map = self.preferences.write().unwrap();
        let key = format!("{}::{}", attribute, value);
        map.remove(&key);
        Ok(())
    }

    pub async fn get_price_set(&self, id: &str) -> Result<PriceSetDTO> {
        let set_map = self.price_sets.read().unwrap();
        let set = set_map
            .values()
            .find(|s| {
                (s.id.to_string() == id || s.id.to_string().ends_with(id)) && s.deleted_at.is_none()
            })
            .or_else(|| set_map.get(id).filter(|s| s.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "price_set",
                id: id.to_string(),
            })?;

        Ok(PriceSetDTO {
            id: Id::from_string(set.id.to_string()).unwrap_or_default(),
            created_at: set.created_at,
        })
    }

    pub async fn update_price_set(
        &self,
        id: &str,
        input: UpdatePriceSetInput,
    ) -> Result<PriceSetDTO> {
        let set_id_str = {
            let mut set_map = self.price_sets.write().unwrap();
            let set = set_map
                .values_mut()
                .find(|s| {
                    (s.id.to_string() == id || s.id.to_string().ends_with(id))
                        && s.deleted_at.is_none()
                })
                .ok_or_else(|| Error::NotFound {
                    entity: "price_set",
                    id: id.to_string(),
                })?;

            set.updated_at = Utc::now();
            let set_id_str = set.id.to_string();

            if let Some(new_prices) = input.prices {
                let mut prices_map = self.prices.write().unwrap();
                let mut rules_map = self.price_rules.write().unwrap();

                // Clear old prices for this price_set
                prices_map.retain(|_, p| p.price_set_id != set_id_str);

                for p_in in new_prices {
                    let pid: Id<Price> = Id::new();
                    let pid_str = pid.to_string();
                    let rules_count = p_in.rules.as_ref().map(|r| r.len() as i32).unwrap_or(0);

                    let price = Price {
                        id: pid,
                        price_set_id: set_id_str.clone(),
                        price_list_id: p_in.price_list_id,
                        currency_code: p_in.currency_code.to_lowercase(),
                        amount: p_in.amount,
                        min_quantity: p_in.min_quantity,
                        max_quantity: p_in.max_quantity,
                        rules_count,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        deleted_at: None,
                    };

                    let mut r_rules = Vec::new();
                    if let Some(r_map) = &p_in.rules {
                        for (attr, val) in r_map {
                            r_rules.push(PriceRule {
                                id: Id::new(),
                                price_id: pid_str.clone(),
                                attribute: attr.clone(),
                                value: val.clone(),
                                priority: 0,
                                created_at: Utc::now(),
                            });
                        }
                    }

                    rules_map.insert(pid_str.clone(), r_rules);
                    prices_map.insert(pid_str, price);
                }
            }

            set_id_str
        };

        self.get_price_set(&set_id_str).await
    }

    pub async fn delete_price_set(&self, id: &str) -> Result<()> {
        let mut set_map = self.price_sets.write().unwrap();
        let set = set_map
            .values_mut()
            .find(|s| {
                (s.id.to_string() == id || s.id.to_string().ends_with(id)) && s.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "price_set",
                id: id.to_string(),
            })?;

        set.deleted_at = Some(Utc::now());
        Ok(())
    }

    pub async fn get_price_list(&self, id: &str) -> Result<PriceListDTO> {
        let plist_map = self.price_lists.read().unwrap();
        let list = plist_map
            .values()
            .find(|l| {
                (l.id.to_string() == id || l.id.to_string().ends_with(id)) && l.deleted_at.is_none()
            })
            .or_else(|| plist_map.get(id).filter(|l| l.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "price_list",
                id: id.to_string(),
            })?;

        Ok(PriceListDTO {
            id: Id::from_string(list.id.to_string()).unwrap_or_default(),
            title: list.title.clone(),
            r#type: list.r#type.clone(),
            status: list.status.clone(),
            starts_at: list.starts_at,
            ends_at: list.ends_at,
            created_at: list.created_at,
        })
    }

    pub async fn list_price_lists(&self) -> Result<Vec<PriceListDTO>> {
        let plist_map = self.price_lists.read().unwrap();
        let list = plist_map
            .values()
            .filter(|l| l.deleted_at.is_none())
            .map(|l| PriceListDTO {
                id: Id::from_string(l.id.to_string()).unwrap_or_default(),
                title: l.title.clone(),
                r#type: l.r#type.clone(),
                status: l.status.clone(),
                starts_at: l.starts_at,
                ends_at: l.ends_at,
                created_at: l.created_at,
            })
            .collect();
        Ok(list)
    }

    pub async fn update_price_list(
        &self,
        id: &str,
        input: UpdatePriceListInput,
    ) -> Result<PriceListDTO> {
        let plist_id_str = {
            let mut plist_map = self.price_lists.write().unwrap();
            let list = plist_map
                .values_mut()
                .find(|l| {
                    (l.id.to_string() == id || l.id.to_string().ends_with(id))
                        && l.deleted_at.is_none()
                })
                .ok_or_else(|| Error::NotFound {
                    entity: "price_list",
                    id: id.to_string(),
                })?;

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
            let plist_id_str = list.id.to_string();

            if let Some(r_map) = input.rules {
                let mut pl_rules = Vec::new();
                for (attr, val) in r_map {
                    pl_rules.push(PriceListRule {
                        id: Id::new(),
                        price_list_id: plist_id_str.clone(),
                        attribute: attr,
                        value: val,
                        created_at: Utc::now(),
                    });
                }
                let mut plr_map = self.price_list_rules.write().unwrap();
                plr_map.insert(plist_id_str.clone(), pl_rules);
            }

            plist_id_str
        };

        self.get_price_list(&plist_id_str).await
    }

    pub async fn delete_price_list(&self, id: &str) -> Result<()> {
        let mut plist_map = self.price_lists.write().unwrap();
        let list = plist_map
            .values_mut()
            .find(|l| {
                (l.id.to_string() == id || l.id.to_string().ends_with(id)) && l.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "price_list",
                id: id.to_string(),
            })?;

        list.deleted_at = Some(Utc::now());
        Ok(())
    }

    pub async fn batch_price_list_prices(
        &self,
        price_list_id: &str,
        input: BatchPriceListPricesInput,
    ) -> Result<()> {
        let mut prices_map = self.prices.write().unwrap();
        let mut rules_map = self.price_rules.write().unwrap();

        if let Some(del_ids) = input.delete {
            for pid in del_ids {
                prices_map.remove(&pid);
                rules_map.remove(&pid);
            }
        }

        if let Some(creates) = input.create {
            for p_in in creates {
                let pid: Id<Price> = Id::new();
                let pid_str = pid.to_string();
                let rules_count = p_in.rules.as_ref().map(|r| r.len() as i32).unwrap_or(0);

                let price = Price {
                    id: pid,
                    price_set_id: "default_set".to_string(),
                    price_list_id: Some(price_list_id.to_string()),
                    currency_code: p_in.currency_code.to_lowercase(),
                    amount: p_in.amount,
                    min_quantity: p_in.min_quantity,
                    max_quantity: p_in.max_quantity,
                    rules_count,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    deleted_at: None,
                };

                let mut r_rules = Vec::new();
                if let Some(r_map) = &p_in.rules {
                    for (attr, val) in r_map {
                        r_rules.push(PriceRule {
                            id: Id::new(),
                            price_id: pid_str.clone(),
                            attribute: attr.clone(),
                            value: val.clone(),
                            priority: 0,
                            created_at: Utc::now(),
                        });
                    }
                }

                rules_map.insert(pid_str.clone(), r_rules);
                prices_map.insert(pid_str, price);
            }
        }

        Ok(())
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

    /// Resolves `calculated_price` for multiple price sets in a single evaluation (P3-09, P3-10).
    /// Follows Medusa precedence: Override Price List > Sale Price List > Specific Rules > Default Base Price.
    pub async fn calculate_prices_batch(
        &self,
        price_set_ids: &[String],
        context: &PricingContext,
    ) -> Result<HashMap<String, CalculatedPriceDTO>> {
        let prices_map = self.prices.read().unwrap();
        let rules_map = self.price_rules.read().unwrap();
        let plists_map = self.price_lists.read().unwrap();
        let plr_map = self.price_list_rules.read().unwrap();
        let prefs_map = self.preferences.read().unwrap();

        let curr_lower = context.currency_code.to_lowercase();
        let qty = context.quantity.unwrap_or(1);
        let now = context.as_of.unwrap_or_else(Utc::now);

        let mut results = HashMap::new();

        for set_id in price_set_ids {
            let matching_prices: Vec<&Price> = prices_map
                .values()
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
                let p_rules = rules_map.get(&p.id.to_string());
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
                    if let Some(plist) = plists_map.get(plist_id) {
                        let mut plist_matches = plist.status == "active"
                            && plist.starts_at.map(|s| now >= s).unwrap_or(true)
                            && plist.ends_at.map(|e| now <= e).unwrap_or(true);

                        let mut pl_rules_matched = 0;
                        if let Some(pl_rules) = plr_map.get(plist_id) {
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
                    // Quantity specificity: a price with min_quantity is more specific
                    // than one without, when rules_matched count is equal.
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
                    .and_then(|r| prefs_map.get(&format!("region_id::{}", r)))
                    .or_else(|| prefs_map.get(&format!("currency_code::{}", curr_lower)))
                    .map(|pref| pref.is_tax_inclusive)
                    .unwrap_or(false);

                if let Some((plist_p, plist, _)) = best_plist_price {
                    results.insert(
                        set_id.clone(),
                        CalculatedPriceDTO {
                            price_set_id: set_id.clone(),
                            calculated_amount: plist_p.amount,
                            original_amount: orig_price.amount,
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
                            calculated_amount: orig_price.amount,
                            original_amount: orig_price.amount,
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
