//! RBAC workflows (WORKFLOWS §2, TASKS P7-21).

use std::sync::Arc;
use stheno_rbac::*;
use stheno_types::Result;

pub struct RbacWorkflows {
    rbac_service: Arc<RbacService>,
}

impl RbacWorkflows {
    pub fn new(rbac_service: Arc<RbacService>) -> Self {
        Self { rbac_service }
    }

    // 1. create_role_workflow
    pub async fn create_role(&self, input: CreateRoleInput) -> Result<RoleDTO> {
        self.rbac_service.create_role(input).await
    }

    // 2. get_role_workflow
    pub async fn get_role(&self, name: &str) -> Result<RoleDTO> {
        self.rbac_service.get_role(name).await
    }

    // 3. create_policy_workflow
    pub async fn create_policy(&self, input: CreatePolicyInput) -> Result<PolicyDTO> {
        self.rbac_service.create_policy(input).await
    }

    // 4. evaluate_permission_workflow
    pub fn evaluate_permission(&self, input: EvaluatePermissionInput) -> PermissionDecision {
        self.rbac_service.evaluate(input)
    }
}
