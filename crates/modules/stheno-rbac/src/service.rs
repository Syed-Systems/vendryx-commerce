//! RBAC Policy Engine: roles, inheritance graph closure, deny-beats-allow & sensitive permissions (P7-18, P7-19).

use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    AuditRecordDTO, CreatePolicyInput, CreateRoleInput, EvaluatePermissionInput,
    PermissionDecision, PolicyDTO, RoleDTO,
};
use crate::models::{AuditRecord, Policy, Role, RoleInheritance};

#[derive(Clone)]
pub struct RbacService {
    roles: Arc<RwLock<HashMap<String, Role>>>,
    inheritances: Arc<RwLock<Vec<RoleInheritance>>>,
    policies: Arc<RwLock<HashMap<String, Policy>>>,
    audit_log: Arc<RwLock<Vec<AuditRecord>>>,
}

impl Default for RbacService {
    fn default() -> Self {
        let service = Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            inheritances: Arc::new(RwLock::new(Vec::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        };

        // Seed default system roles
        service.seed_default_roles();
        service
    }
}

impl RbacService {
    pub fn new() -> Self {
        Self::default()
    }

    fn seed_default_roles(&self) {
        let mut r_map = self.roles.write().unwrap();
        let mut p_map = self.policies.write().unwrap();
        let mut inh = self.inheritances.write().unwrap();

        // 1. superadmin
        let superadmin = Role {
            id: Id::new(),
            name: "superadmin".to_string(),
            description: Some("Full platform super-administrator".to_string()),
            is_system_role: true,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        r_map.insert("superadmin".to_string(), superadmin);

        // Policy: superadmin allows everything
        let pol_super = Policy {
            id: Id::new(),
            role: "superadmin".to_string(),
            resource: "*".to_string(),
            action: "*".to_string(),
            effect: "allow".to_string(),
            is_sensitive: false,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        p_map.insert(pol_super.id.to_string(), pol_super);

        // 2. admin inherits manager
        let admin = Role {
            id: Id::new(),
            name: "admin".to_string(),
            description: Some("Store administrator".to_string()),
            is_system_role: true,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        r_map.insert("admin".to_string(), admin);

        // 3. manager inherits member
        let manager = Role {
            id: Id::new(),
            name: "manager".to_string(),
            description: Some("Store manager".to_string()),
            is_system_role: true,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        r_map.insert("manager".to_string(), manager);

        // 4. member
        let member = Role {
            id: Id::new(),
            name: "member".to_string(),
            description: Some("Store team member".to_string()),
            is_system_role: true,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        r_map.insert("member".to_string(), member);

        // Set inheritance: superadmin -> admin -> manager -> member
        inh.push(RoleInheritance {
            parent_role: "admin".to_string(),
            child_role: "superadmin".to_string(),
            created_at: Utc::now(),
        });
        inh.push(RoleInheritance {
            parent_role: "manager".to_string(),
            child_role: "admin".to_string(),
            created_at: Utc::now(),
        });
        inh.push(RoleInheritance {
            parent_role: "member".to_string(),
            child_role: "manager".to_string(),
            created_at: Utc::now(),
        });
    }

    // ==========================================
    // 1. Roles & Inheritance (P7-18)
    // ==========================================

    pub async fn create_role(&self, input: CreateRoleInput) -> Result<RoleDTO> {
        let id = Id::new();
        let now = Utc::now();

        {
            let mut r_map = self.roles.write().unwrap();
            if r_map.contains_key(&input.name) {
                return Err(Error::Conflict {
                    code: "role_exists",
                    message: format!("Role '{}' already exists", input.name),
                });
            }

            let role = Role {
                id,
                name: input.name.clone(),
                description: input.description,
                is_system_role: input.is_system_role.unwrap_or(false),
                metadata: input.metadata,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };

            r_map.insert(input.name.clone(), role);
        }

        if let Some(parents) = input.parent_roles {
            let mut inh = self.inheritances.write().unwrap();
            for parent in &parents {
                inh.push(RoleInheritance {
                    parent_role: parent.clone(),
                    child_role: input.name.clone(),
                    created_at: now,
                });
            }
        }

        self.get_role(&input.name).await
    }

    pub async fn get_role(&self, name: &str) -> Result<RoleDTO> {
        let role = {
            let r_map = self.roles.read().unwrap();
            r_map
                .get(name)
                .filter(|r| r.deleted_at.is_none())
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "role",
                    id: name.to_string(),
                })?
        };

        let parent_roles = {
            let inh = self.inheritances.read().unwrap();
            inh.iter()
                .filter(|i| i.child_role == name)
                .map(|i| i.parent_role.clone())
                .collect()
        };

        Ok(RoleDTO {
            id: role.id.to_string(),
            name: role.name,
            description: role.description,
            is_system_role: role.is_system_role,
            parent_roles,
            metadata: role.metadata,
            created_at: role.created_at,
            updated_at: role.updated_at,
        })
    }

    /// Computes the transitive closure of all roles inherited by the given roles (P7-18).
    pub fn get_effective_roles(&self, direct_roles: &[String]) -> HashSet<String> {
        let inh = self.inheritances.read().unwrap();
        let mut effective = HashSet::new();
        let mut queue: Vec<String> = direct_roles.to_vec();

        while let Some(current) = queue.pop() {
            if effective.insert(current.clone()) {
                // Find all parent roles inherited by current
                for edge in inh.iter() {
                    if edge.child_role == current && !effective.contains(&edge.parent_role) {
                        queue.push(edge.parent_role.clone());
                    }
                }
            }
        }

        effective
    }

    // ==========================================
    // 2. Policies & Deny-Beats-Allow Evaluation (P7-18, P7-19)
    // ==========================================

    pub async fn create_policy(&self, input: CreatePolicyInput) -> Result<PolicyDTO> {
        let id = Id::new();
        let now = Utc::now();

        let policy = Policy {
            id,
            role: input.role,
            resource: input.resource,
            action: input.action,
            effect: input.effect.to_lowercase(),
            is_sensitive: input.is_sensitive.unwrap_or(false),
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
        };

        let dto = PolicyDTO {
            id: id.to_string(),
            role: policy.role.clone(),
            resource: policy.resource.clone(),
            action: policy.action.clone(),
            effect: policy.effect.clone(),
            is_sensitive: policy.is_sensitive,
            metadata: policy.metadata.clone(),
            created_at: policy.created_at,
            updated_at: policy.updated_at,
        };

        let mut p_map = self.policies.write().unwrap();
        p_map.insert(id.to_string(), policy);

        Ok(dto)
    }

    pub fn evaluate(&self, input: EvaluatePermissionInput) -> PermissionDecision {
        let effective_roles = self.get_effective_roles(&input.roles);
        let p_map = self.policies.read().unwrap();

        let mut has_explicit_allow = false;
        let mut has_explicit_deny = false;
        let mut requires_mfa = false;

        for policy in p_map.values() {
            if effective_roles.contains(&policy.role)
                && self.matches_pattern(&policy.resource, &input.resource)
                && self.matches_pattern(&policy.action, &input.action)
            {
                if policy.effect == "deny" {
                    has_explicit_deny = true;
                    break; // DENY BEATS ALLOW: immediate exit on explicit deny
                } else if policy.effect == "allow" {
                    has_explicit_allow = true;
                    if policy.is_sensitive {
                        requires_mfa = true;
                    }
                }
            }
        }

        let mfa_verified = input.mfa_verified.unwrap_or(false);

        let (is_allowed, reason) = if has_explicit_deny {
            (
                false,
                "Explicitly denied by policy (deny-beats-allow)".to_string(),
            )
        } else if !has_explicit_allow {
            (false, "No matching allow policy found".to_string())
        } else if requires_mfa && !mfa_verified {
            (
                false,
                "Sensitive permission requires MFA verification".to_string(),
            )
        } else {
            (true, "Permission granted".to_string())
        };

        // Emit audit record (P7-19)
        if let Some(actor) = &input.actor_id {
            let record = AuditRecord {
                id: Id::new(),
                actor_id: actor.clone(),
                role: input.roles.join(","),
                action: input.action.clone(),
                resource: input.resource.clone(),
                mfa_verified,
                decision: if is_allowed {
                    "allowed".to_string()
                } else {
                    "denied".to_string()
                },
                timestamp: Utc::now(),
            };

            let mut log = self.audit_log.write().unwrap();
            log.push(record);
        }

        PermissionDecision {
            is_allowed,
            reason,
            requires_mfa,
        }
    }

    pub fn get_audit_records(&self, actor_id: Option<&str>) -> Vec<AuditRecordDTO> {
        let log = self.audit_log.read().unwrap();
        log.iter()
            .filter(|r| actor_id.map(|a| r.actor_id == a).unwrap_or(true))
            .map(|r| AuditRecordDTO {
                id: r.id.to_string(),
                actor_id: r.actor_id.clone(),
                role: r.role.clone(),
                action: r.action.clone(),
                resource: r.resource.clone(),
                mfa_verified: r.mfa_verified,
                decision: r.decision.clone(),
                timestamp: r.timestamp,
            })
            .collect()
    }

    fn matches_pattern(&self, pattern: &str, target: &str) -> bool {
        if pattern == "*" || pattern == target {
            true
        } else if let Some(prefix) = pattern.strip_suffix(":*") {
            target.starts_with(prefix)
        } else {
            false
        }
    }
}
