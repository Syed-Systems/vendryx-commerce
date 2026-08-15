//! `stheno-types::entitlements`: Feature entitlement flags, tier configurations, and fail-closed boot checks.

use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    Promotions,
    MultiCurrency,
    CustomFulfillment,
    Analytics,
    B2B,
    Automations,
    AuditLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanTier {
    Trial,
    Free,
    Pro,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlements {
    pub tier: PlanTier,
    pub active_features: HashSet<Feature>,
    pub max_products: Option<usize>,
    pub max_orders_per_month: Option<usize>,
}

impl Entitlements {
    pub fn for_tier(tier: PlanTier) -> Self {
        let mut active_features = HashSet::new();

        match tier {
            PlanTier::Free => {
                // Base tier features
            }
            PlanTier::Trial => {
                active_features.insert(Feature::Promotions);
                active_features.insert(Feature::MultiCurrency);
                active_features.insert(Feature::Analytics);
            }
            PlanTier::Pro => {
                active_features.insert(Feature::Promotions);
                active_features.insert(Feature::MultiCurrency);
                active_features.insert(Feature::CustomFulfillment);
                active_features.insert(Feature::Analytics);
                active_features.insert(Feature::AuditLog);
            }
            PlanTier::Enterprise => {
                active_features.insert(Feature::Promotions);
                active_features.insert(Feature::MultiCurrency);
                active_features.insert(Feature::CustomFulfillment);
                active_features.insert(Feature::Analytics);
                active_features.insert(Feature::B2B);
                active_features.insert(Feature::Automations);
                active_features.insert(Feature::AuditLog);
            }
        }

        Self {
            tier,
            active_features,
            max_products: match tier {
                PlanTier::Free => Some(100),
                PlanTier::Trial => Some(1000),
                _ => None,
            },
            max_orders_per_month: match tier {
                PlanTier::Free => Some(500),
                _ => None,
            },
        }
    }

    pub fn has(&self, feature: Feature) -> bool {
        self.active_features.contains(&feature)
    }

    /// Fail-closed validation at server boot (ENTITLEMENTS.md §5.1)
    pub fn validate_at_boot(&self) -> Result<(), Error> {
        if self.tier == PlanTier::Free && self.has(Feature::B2B) {
            return Err(Error::Invalid {
                code: "invalid_tier_configuration",
                message: "Enterprise feature B2B cannot be enabled on Free tier".to_string(),
                field: None,
            });
        }
        Ok(())
    }
}
