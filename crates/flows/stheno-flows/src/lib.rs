//! `stheno-flows`: SAGA cross-module workflows (WORKFLOWS.md).

pub mod api_key;
pub mod auth;
pub mod cart;
pub mod change_control;
pub mod claims;
pub mod complete_cart;
pub mod customer;
pub mod draft_orders;
pub mod exchanges;
pub mod file;
pub mod fulfillment;
pub mod order;
pub mod order_edit;
pub mod payment;
pub mod platform;
pub mod pricing;
pub mod promotion;
pub mod quota;
pub mod rbac;
pub mod region;
pub mod returns;
pub mod settings;
pub mod store;
pub mod tax;
pub mod translation;
pub mod user;

pub use api_key::*;
pub use auth::*;
pub use cart::*;
pub use change_control::*;
pub use claims::*;
pub use complete_cart::*;
pub use customer::*;
pub use draft_orders::*;
pub use exchanges::*;
pub use file::*;
pub use fulfillment::*;
pub use order::*;
pub use order_edit::*;
pub use payment::*;
pub use platform::*;
pub use pricing::*;
pub use promotion::*;
pub use quota::*;
pub use rbac::*;
pub use region::*;
pub use returns::*;
pub use settings::*;
pub use store::*;
pub use tax::*;
pub use translation::*;
pub use user::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_product::{CreateProductInput, ProductService};
use stheno_types::{Error, Result};
use stheno_workflow::{SagaCoordinator, Step, StepConfig, StepContext};

pub struct CreateProductStep {
    product_service: Arc<ProductService>,
}

impl CreateProductStep {
    pub fn new(product_service: Arc<ProductService>) -> Self {
        Self { product_service }
    }
}

#[async_trait]
impl Step for CreateProductStep {
    fn name(&self) -> &'static str {
        "CreateProductStep"
    }

    fn config(&self) -> StepConfig {
        StepConfig {
            retries: 3,
            retry_interval_ms: 10,
            ..Default::default()
        }
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let title = input["title"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_title",
            message: "Missing product title in workflow input".to_string(),
            field: Some("title".to_string()),
        })?;

        let product = self
            .product_service
            .create_product(CreateProductInput {
                title: title.to_string(),
                handle: input["handle"].as_str().map(|s| s.to_string()),
                subtitle: input["subtitle"].as_str().map(|s| s.to_string()),
                description: input["description"].as_str().map(|s| s.to_string()),
                discountable: input["discountable"].as_bool(),
            })
            .await?;

        Ok(json!({
            "product_id": product.id.to_string(),
            "title": product.title,
            "handle": product.handle,
            "status": product.status,
        }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        // Compensation logic if downstream steps fail
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProductResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<BatchErrorItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchErrorItem {
    pub index: usize,
    pub error: String,
}

pub struct BatchProductService {
    product_service: Arc<ProductService>,
}

impl BatchProductService {
    pub fn new(product_service: Arc<ProductService>) -> Self {
        Self { product_service }
    }

    /// Executes batch product creation over 1,000 items with per-row errors (P2-18)
    pub async fn batch_create_products(
        &self,
        items: Vec<CreateProductInput>,
    ) -> Result<BatchProductResult> {
        let total = items.len();
        let mut succeeded = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for (index, item) in items.into_iter().enumerate() {
            if item.title.trim().is_empty() {
                failed += 1;
                errors.push(BatchErrorItem {
                    index,
                    error: "Title cannot be empty".to_string(),
                });
                continue;
            }

            match self.product_service.create_product(item).await {
                Ok(_) => succeeded += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(BatchErrorItem {
                        index,
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchProductResult {
            total,
            succeeded,
            failed,
            errors,
        })
    }
}

use stheno_db::QuotaCounter;
use stheno_user::{CreateUserInput, UserService};

pub struct CreateUserStep {
    user_service: Arc<UserService>,
}

impl CreateUserStep {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
}

#[async_trait]
impl Step for CreateUserStep {
    fn name(&self) -> &'static str {
        "CreateUserStep"
    }

    fn config(&self) -> StepConfig {
        StepConfig {
            retries: 3,
            retry_interval_ms: 10,
            ..Default::default()
        }
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let email = input["email"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_email",
            message: "Missing user email in workflow input".to_string(),
            field: Some("email".to_string()),
        })?;

        let user = self
            .user_service
            .create_user(CreateUserInput {
                email: email.to_string(),
                first_name: input["first_name"].as_str().map(|s| s.to_string()),
                last_name: input["last_name"].as_str().map(|s| s.to_string()),
                avatar_url: input["avatar_url"].as_str().map(|s| s.to_string()),
                role: input["role"].as_str().map(|s| s.to_string()),
                metadata: None,
            })
            .await?;

        Ok(json!({
            "user_id": user.id,
            "email": user.email,
            "role": user.role,
        }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

pub fn build_create_product_saga(product_service: Arc<ProductService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreateProductStep::new(product_service)));
    coordinator
}

pub fn build_create_product_quota_saga(
    product_service: Arc<ProductService>,
    quota_counter: QuotaCounter,
    limit: Option<i64>,
) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CheckQuotaStep::new(
        quota_counter,
        "products",
        limit,
    )));
    coordinator.add_step(Arc::new(CreateProductStep::new(product_service)));
    coordinator
}

pub fn build_create_seat_quota_saga(
    user_service: Arc<UserService>,
    quota_counter: QuotaCounter,
    limit: Option<i64>,
) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CheckQuotaStep::new(quota_counter, "users", limit)));
    coordinator.add_step(Arc::new(CreateUserStep::new(user_service)));
    coordinator
}
