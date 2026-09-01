//! RBAC Policy Engine: roles, inheritance graph closure, deny-beats-allow & sensitive permissions (P7-18, P7-19, P14-06).

use chrono::Utc;
use std::collections::HashSet;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    AuditRecordDTO, CreatePolicyInput, CreateRoleInput, EvaluatePermissionInput,
    PermissionDecision, PolicyDTO, RoleDTO, UpdatePolicyInput, UpdateRoleInput,
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

    /// Every role, with its inherited parents resolved.
    ///
    /// The admin's roles screen needs the list; the API previously returned three
    /// hardcoded fakes because this method did not exist and the module was wired to
    /// nothing.
    pub async fn list_roles(&self) -> Result<Vec<RoleDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("name", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        let roles = self.roles.list(None, &filter).await?;

        // One inheritance read for all roles rather than one per role.
        let inh_filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        let inheritances = self.inheritances.list(None, &inh_filter).await?;

        Ok(roles
            .into_iter()
            .map(|role| RoleDTO {
                parent_roles: inheritances
                    .iter()
                    .filter(|i| i.child_role == role.name)
                    .map(|i| i.parent_role.clone())
                    .collect(),
                id: role.id.to_string(),
                name: role.name,
                description: role.description,
                is_system_role: role.is_system_role,
                metadata: role.metadata,
                created_at: role.created_at,
                updated_at: role.updated_at,
            })
            .collect())
    }

    /// Every policy, optionally narrowed to one role.
    pub async fn list_policies(&self, role: Option<&str>) -> Result<Vec<PolicyDTO>> {
        let mut filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        if let Some(role) = role {
            filter = filter.with_predicate("role", FilterOp::Eq(role.to_string()));
        }

        Ok(self
            .policies
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|p| PolicyDTO {
                id: p.id.to_string(),
                role: p.role,
                resource: p.resource,
                action: p.action,
                effect: p.effect,
                is_sensitive: p.is_sensitive,
                metadata: p.metadata,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect())
    }

    /// Soft-deletes a role. System roles are refused: removing them would silently strip
    /// permissions from every user holding one.
    /// Partial update for a role, addressed by name.
    ///
    /// System roles are immutable here for the same reason `delete_role` refuses them:
    /// they are seeded by `seed_default_roles` and the authorisation model assumes they
    /// exist with the shape it seeded. Editing one through the admin would let a merchant
    /// quietly widen their own permissions.
    pub async fn update_role(&self, name: &str, input: UpdateRoleInput) -> Result<RoleDTO> {
        let existing = self.get_role(name).await?;
        if existing.is_system_role {
            return Err(Error::Invalid {
                code: "system_role_immutable",
                message: format!("{name} is a system role and cannot be edited"),
                field: Some("name".to_string()),
            });
        }

        let mut role = self.roles.retrieve(None, &existing.id).await?;
        if let Some(description) = input.description {
            role.description = Some(description);
        }
        if let Some(metadata) = input.metadata {
            role.metadata = Some(metadata);
        }
        role.updated_at = Utc::now();
        self.roles.update(None, &existing.id, role).await?;

        self.get_role(name).await
    }

    pub async fn get_policy(&self, id: &str) -> Result<PolicyDTO> {
        let p = self.policies.retrieve(None, id).await?;
        Ok(PolicyDTO {
            id: p.id.to_string(),
            role: p.role,
            resource: p.resource,
            action: p.action,
            effect: p.effect,
            is_sensitive: p.is_sensitive,
            metadata: p.metadata,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
    }

    /// Partial update for a policy.
    ///
    /// `effect` is validated rather than trusted: the evaluator treats anything that is
    /// not "deny" as an allow, so a typo here would silently widen access.
    pub async fn update_policy(&self, id: &str, input: UpdatePolicyInput) -> Result<PolicyDTO> {
        let mut policy = self.policies.retrieve(None, id).await?;

        if let Some(effect) = &input.effect {
            if effect != "allow" && effect != "deny" {
                return Err(Error::Invalid {
                    code: "invalid_effect",
                    message: format!("effect must be 'allow' or 'deny', got '{effect}'"),
                    field: Some("effect".to_string()),
                });
            }
        }

        if let Some(role) = input.role {
            policy.role = role;
        }
        if let Some(resource) = input.resource {
            policy.resource = resource;
        }
        if let Some(action) = input.action {
            policy.action = action;
        }
        if let Some(effect) = input.effect {
            policy.effect = effect;
        }
        if let Some(is_sensitive) = input.is_sensitive {
            policy.is_sensitive = is_sensitive;
        }
        if let Some(metadata) = input.metadata {
            policy.metadata = Some(metadata);
        }
        policy.updated_at = Utc::now();

        self.policies.update(None, id, policy).await?;
        self.get_policy(id).await
    }

    pub async fn delete_role(&self, name: &str) -> Result<()> {
        let role = self.get_role(name).await?;
        if role.is_system_role {
            return Err(Error::Invalid {
                code: "system_role_immutable",
                message: format!("{name} is a system role and cannot be deleted"),
                field: Some("name".to_string()),
            });
        }
        self.roles.soft_delete(None, &role.id).await?;
        Ok(())
    }

    /// Soft-deletes a policy.
    pub async fn delete_policy(&self, id: &str) -> Result<()> {
        self.policies.soft_delete(None, id).await?;
        Ok(())
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
