//! RBAC Policy Engine: roles, inheritance graph closure, deny-beats-allow & sensitive permissions (P7-18, P7-19, P14-06).

use chrono::Utc;
use std::collections::HashSet;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    AuditRecordDTO, CreatePolicyInput, CreateRoleInput, EvaluatePermissionInput,
    PermissionDecision, PolicyDTO, RoleDTO,
};
use crate::models::{AuditRecord, Policy, Role, RoleInheritance};

#[derive(Clone)]
pub struct RbacService {
    roles: PgCrudRepository<Role>,
    inheritances: PgCrudRepository<RoleInheritance>,
    policies: PgCrudRepository<Policy>,
    audit_log: PgCrudRepository<AuditRecord>,
}

impl RbacService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            roles: PgCrudRepository::new(pool.clone()),
            inheritances: PgCrudRepository::new(pool.clone()),
            policies: PgCrudRepository::new(pool.clone()),
            audit_log: PgCrudRepository::new(pool),
        }
    }

    pub async fn seed_default_roles(&self) -> Result<()> {
        let filter = FilterSpec::new().with_pagination(0, 100);
        let existing = self.roles.list(None, &filter).await?;
        if !existing.is_empty() {
            return Ok(());
        }

        let now = Utc::now();

        // 1. superadmin
        let superadmin = Role {
            id: Id::new(),
            name: "superadmin".to_string(),
            description: Some("Full platform super-administrator".to_string()),
            is_system_role: true,
            metadata: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        self.roles.create(None, superadmin).await?;

        let pol_super = Policy {
            id: Id::new(),
            role: "superadmin".to_string(),
            resource: "*".to_string(),
            action: "*".to_string(),
            effect: "allow".to_string(),
            is_sensitive: false,
            metadata: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        self.policies.create(None, pol_super).await?;

        // 2. admin
        let admin = Role {
            id: Id::new(),
            name: "admin".to_string(),
            description: Some("Store administrator".to_string()),
            is_system_role: true,
            metadata: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        self.roles.create(None, admin).await?;

        // 3. manager
        let manager = Role {
            id: Id::new(),
            name: "manager".to_string(),
            description: Some("Store manager".to_string()),
            is_system_role: true,
            metadata: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        self.roles.create(None, manager).await?;

        // 4. member
        let member = Role {
            id: Id::new(),
            name: "member".to_string(),
            description: Some("Store team member".to_string()),
            is_system_role: true,
            metadata: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        self.roles.create(None, member).await?;

        // Set inheritance: superadmin -> admin -> manager -> member
        self.inheritances
            .create(
                None,
                RoleInheritance {
                    parent_role: "admin".to_string(),
                    child_role: "superadmin".to_string(),
                    created_at: now,
                    deleted_at: None,
                },
            )
            .await?;
        self.inheritances
            .create(
                None,
                RoleInheritance {
                    parent_role: "manager".to_string(),
                    child_role: "admin".to_string(),
                    created_at: now,
                    deleted_at: None,
                },
            )
            .await?;
        self.inheritances
            .create(
                None,
                RoleInheritance {
                    parent_role: "member".to_string(),
                    child_role: "manager".to_string(),
                    created_at: now,
                    deleted_at: None,
                },
            )
            .await?;

        Ok(())
    }

    // ==========================================
    // 1. Roles & Inheritance (P7-18)
    // ==========================================

    pub async fn create_role(&self, input: CreateRoleInput) -> Result<RoleDTO> {
        let filter = FilterSpec::new()
            .with_predicate("name", FilterOp::Eq(input.name.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.roles.list(None, &filter).await?;
        if !existing.is_empty() {
            return Err(Error::Conflict {
                code: "role_exists",
                message: format!("Role '{}' already exists", input.name),
            });
        }

        let id = Id::new();
        let now = Utc::now();

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

        self.roles.create(None, role).await?;

        if let Some(parents) = input.parent_roles {
            for parent in &parents {
                self.inheritances
                    .create(
                        None,
                        RoleInheritance {
                            parent_role: parent.clone(),
                            child_role: input.name.clone(),
                            created_at: now,
                            deleted_at: None,
                        },
                    )
                    .await?;
            }
        }

        self.get_role(&input.name).await
    }

    pub async fn get_role(&self, name: &str) -> Result<RoleDTO> {
        let filter = FilterSpec::new()
            .with_predicate("name", FilterOp::Eq(name.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.roles.list(None, &filter).await?;
        let role = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "role",
            id: name.to_string(),
        })?;

        let inh_filter = FilterSpec::new()
            .with_predicate("child_role", FilterOp::Eq(name.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let inheritances = self.inheritances.list(None, &inh_filter).await?;
        let parent_roles = inheritances.into_iter().map(|i| i.parent_role).collect();

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
    pub async fn get_effective_roles(&self, direct_roles: &[String]) -> Result<HashSet<String>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        let inh = self.inheritances.list(None, &filter).await?;

        let mut effective = HashSet::new();
        let mut queue: Vec<String> = direct_roles.to_vec();

        while let Some(current) = queue.pop() {
            if effective.insert(current.clone()) {
                for edge in &inh {
                    if edge.child_role == current && !effective.contains(&edge.parent_role) {
                        queue.push(edge.parent_role.clone());
                    }
                }
            }
        }

        Ok(effective)
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
            deleted_at: None,
        };

        let created = self.policies.create(None, policy).await?;

        Ok(PolicyDTO {
            id: created.id.to_string(),
            role: created.role,
            resource: created.resource,
            action: created.action,
            effect: created.effect,
            is_sensitive: created.is_sensitive,
            metadata: created.metadata,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn evaluate(&self, input: EvaluatePermissionInput) -> Result<PermissionDecision> {
        let effective_roles = self.get_effective_roles(&input.roles).await?;

        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        let policies = self.policies.list(None, &filter).await?;

        let mut has_explicit_allow = false;
        let mut has_explicit_deny = false;
        let mut requires_mfa = false;

        for policy in policies {
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
                deleted_at: None,
            };

            let _ = self.audit_log.create(None, record).await;
        }

        Ok(PermissionDecision {
            is_allowed,
            reason,
            requires_mfa,
        })
    }

    pub async fn get_audit_records(&self, actor_id: Option<&str>) -> Result<Vec<AuditRecordDTO>> {
        let mut filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("timestamp", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        if let Some(actor) = actor_id {
            filter = filter.with_predicate("actor_id", FilterOp::Eq(actor.to_string()));
        }

        let list = self.audit_log.list(None, &filter).await?;
        Ok(list
            .into_iter()
            .map(|r| AuditRecordDTO {
                id: r.id.to_string(),
                actor_id: r.actor_id,
                role: r.role,
                action: r.action,
                resource: r.resource,
                mfa_verified: r.mfa_verified,
                decision: r.decision,
                timestamp: r.timestamp,
            })
            .collect())
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
