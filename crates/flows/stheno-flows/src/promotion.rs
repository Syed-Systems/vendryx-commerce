//! Promotion and campaign workflows (WORKFLOWS §2, TASKS P4-28).

use rust_decimal::Decimal;
use std::sync::Arc;
use stheno_promotion::*;
use stheno_types::Result;

pub struct PromotionWorkflows {
    promotion_service: Arc<PromotionService>,
}

impl PromotionWorkflows {
    pub fn new(promotion_service: Arc<PromotionService>) -> Self {
        Self { promotion_service }
    }

    // 1. create_promotion_workflow
    pub async fn create_promotion(&self, input: CreatePromotionInput) -> Result<PromotionDTO> {
        self.promotion_service.create_promotion(input).await
    }

    // 2. get_promotion_workflow
    pub async fn get_promotion(&self, id: &str) -> Result<PromotionDTO> {
        self.promotion_service.get_promotion(id).await
    }

    // 3. get_promotion_by_code_workflow
    pub async fn get_promotion_by_code(&self, code: &str) -> Result<PromotionDTO> {
        self.promotion_service.get_promotion_by_code(code).await
    }

    // 4. update_promotion_workflow
    pub async fn update_promotion(
        &self,
        id: &str,
        input: UpdatePromotionInput,
    ) -> Result<PromotionDTO> {
        self.promotion_service.update_promotion(id, input).await
    }

    // 5. delete_promotion_workflow
    pub async fn delete_promotion(&self, id: &str) -> Result<()> {
        self.promotion_service.delete_promotion(id).await
    }

    // 6. create_campaign_workflow
    pub async fn create_campaign(&self, input: CreateCampaignInput) -> Result<CampaignDTO> {
        self.promotion_service.create_campaign(input).await
    }

    // 7. get_campaign_workflow
    pub async fn get_campaign(&self, id: &str) -> Result<CampaignDTO> {
        self.promotion_service.get_campaign(id).await
    }

    // 8. create_campaign_budget_workflow
    pub async fn create_campaign_budget(
        &self,
        campaign_id: &str,
        input: CreateCampaignBudgetInput,
    ) -> Result<CampaignBudgetDTO> {
        self.promotion_service
            .create_campaign_budget(campaign_id, input)
            .await
    }

    // 9. consume_campaign_budget_workflow (atomic under lock)
    pub async fn consume_campaign_budget(
        &self,
        campaign_id: &str,
        amount: Decimal,
        order_id: Option<&str>,
        cart_id: Option<&str>,
    ) -> Result<CampaignBudgetUsageDTO> {
        self.promotion_service
            .consume_campaign_budget(campaign_id, amount, order_id, cart_id)
            .await
    }

    // 10. evaluate_and_apply_promotions_workflow
    pub async fn evaluate_promotions(
        &self,
        promotions: &[PromotionDTO],
        context: &PromotionEvaluationContext,
    ) -> Vec<ComputedDiscountAdjustment> {
        self.promotion_service
            .evaluate_promotions(promotions, context)
            .await
    }

    // 11. validate_promo_code_workflow (with abuse protection F10)
    pub async fn validate_promo_code(
        &self,
        code: &str,
        cart_id: Option<&str>,
        ip: Option<&str>,
    ) -> Result<PromotionDTO> {
        if let Some(c_id) = cart_id {
            if !self.promotion_service.check_cart_promo_abuse_limit(c_id, 5) {
                return Err(stheno_types::Error::Invalid {
                    code: "promo_code_abuse_limit_exceeded",
                    message: "Too many failed promo code attempts on this cart".to_string(),
                    field: None,
                });
            }
        }

        if let Some(client_ip) = ip {
            if !self
                .promotion_service
                .check_ip_promo_rate_limit(client_ip, 20)
            {
                return Err(stheno_types::Error::Invalid {
                    code: "rate_limit_exceeded",
                    message: "Too many promo requests from this IP".to_string(),
                    field: None,
                });
            }
        }

        let promo_res = self.promotion_service.get_promotion_by_code(code).await;
        if promo_res.is_err() {
            if let Some(c_id) = cart_id {
                self.promotion_service.record_failed_promo_attempt(c_id);
            }
        }
        promo_res
    }
}
