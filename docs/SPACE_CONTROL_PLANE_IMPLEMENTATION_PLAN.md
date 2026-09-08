# Space Control Plane Implementation Plan

**Company:** Syed Systems  
**Product name:** Space  
**Document date:** 2026-09-08  
**Status:** Proposed architecture and implementation handoff  
**Current implementation:** `apps/platform`, `stheno-platform`, and `stheno-api-platform`  
**Target:** One company control plane for Vendryx Commerce, ERP, LMS, and future SaaS products

> 2026-09-08 implementation note: Space now lives as a standalone sibling project at
> `D:\Syed Systems\Space`, not inside the Store/Vendryx workspace. References in this plan to
> `apps/space`, `crates/space-server`, and Space migrations now resolve relative to this standalone
> Space project root.

## 1. Decision summary

Space is not a renamed Vendryx control panel. It is a separate company-level product that manages
customers and the commercial relationship across every Syed Systems SaaS product.

Space owns:

- customer organizations and their memberships;
- the Syed Systems product catalog;
- plans, prices, bundles, add-ons, trials, and contracts;
- subscriptions and lifecycle changes;
- entitlement definitions, grants, overrides, usage limits, and resolved snapshots;
- product-instance provisioning intent and observed status;
- invoices, payment attempts, credits, dunning, and subscription revenue;
- workforce access, product-scoped permissions, support access, and audit evidence;
- product connector registration, service credentials, webhooks, and delivery history;
- company-level operational and commercial projections.

Each product remains authoritative for its own domain data. Vendryx owns stores, orders, products,
payments, and fulfillment. An ERP owns ledgers, payroll, and procurement. An LMS owns courses,
enrollments, and learning records. Space does not read or join those product databases directly.

### 1.1 Technology choices

| Area | Decision | Reason |
|---|---|---|
| Frontend framework | **React with Vite** | The existing control plane already uses Vite. Space is an authenticated console with no SEO or server-rendering requirement. Rust remains the only application server. |
| Component system | **shadcn/ui, customized for Space** | The source is owned in the app, supports Vite, and permits a distinct Syed Systems identity. Do not ship the default shadcn theme. |
| Styling | **Tailwind CSS 4 through the Vite plugin** | Matches the current shadcn Vite path and starts the new app without inheriting the old control plane's Tailwind 3 preset. |
| Data tables | **TanStack Table with shadcn table primitives** | Tables need screen-specific sorting, filtering, selection, pagination, and column behavior. |
| Server state | **TanStack Query** | Already used in `apps/platform`; suitable for caching, invalidation, retries, and mutation state. |
| Charts | **Recharts through owned chart wrappers** | Appropriate for non-streaming operational and commercial charts. |
| Routing | **React Router with URL-backed nested routes** | Replaces the current in-memory tab selection and enables deep links, breadcrumbs, access guards, and route tests. |
| Forms | **React Hook Form plus Zod at the client boundary** | Accessible form state and explicit client validation. Rust performs the authoritative validation. |
| Notifications | **Sonner through one owned notification adapter** | Transient feedback only. Inline state remains on the affected object or field. |
| Icons | **One Phosphor icon family** | Broad enterprise vocabulary and consistent optical weight. No hand-drawn interface icons. |
| Backend | **Rust, Axum, PostgreSQL, existing workflow patterns** | Matches the repository and keeps authorization, money, and tenancy in one compiled backend. |
| Async work | **PostgreSQL transactional outbox plus worker** | Reuses the database and avoids a broker before throughput proves one is needed. |
| API contract | **Versioned REST and generated OpenAPI client** | Clear product/service boundary and independently testable connectors. |

Official implementation references:

- [shadcn/ui for Vite](https://ui.shadcn.com/docs/installation/vite)
- [shadcn/ui data tables](https://ui.shadcn.com/docs/components/data-table)
- [TanStack Query](https://tanstack.com/query/latest/docs/framework/react/overview)
- [Vite guide](https://vite.dev/guide/)

### 1.2 Why not Next.js

Next.js would add a second application server and a potential authorization boundary between the
browser and Rust. Space does not need public search indexing, server-rendered marketing pages, or
React Server Components. A Vite SPA served behind the same origin as the Rust API has fewer moving
parts, reuses the current deployment shape, and keeps all sensitive policy enforcement in Rust.

If Syed Systems later needs a public customer account portal with server-rendered public content,
that should be a separate surface. It is not a reason to add Next.js to the internal operator
console.

### 1.3 Implementation ladder

**Rung 2: reuse existing capabilities before creating new ones.**

Reuse verified parts of the current platform service: operator audit patterns, subscription money
logic, invoice idempotency, dunning, entitlement resolution, domain lifecycle patterns, scheduler,
PostgreSQL persistence, provider idempotency, and existing tests. Wrap the Vendryx-specific service
with a Space product connector during migration.

Do not widen `stheno-types`, rename commerce tables into generic concepts, or turn
`stheno-platform` into the company-wide shared kernel. That would couple future ERP and LMS products
to commerce terminology and release cycles.

## 2. Design read

Reading this as: a high-trust B2B operations console for Syed Systems staff, with a precise,
restrained, technical visual language, using customized shadcn/ui and dense but calm data patterns.

The Taste skill is not a dashboard architecture system. For Space it supplies anti-template,
accessibility, token, content-truth, state, and interaction-quality constraints. Dashboard structure
comes from the workflows and data model in this plan.

### Design dials

- `DESIGN_VARIANCE: 4` - stable and recognizable, with selective asymmetry only on overview pages.
- `MOTION_INTENSITY: 2` - state transitions and feedback only, no ambient or scroll-driven motion.
- `VISUAL_DENSITY: 7` - an operational console, not a marketing site or an overloaded cockpit.

Hard visual rules:

- one Space accent color;
- no galaxy background, star field, purple mesh, neon glow, or generic AI gradient;
- no invented metrics, decorative status dots, or fake live states;
- cards only when they represent a real bounded object or elevation layer;
- tables for comparable records, definition lists for single-record facts;
- all loading, empty, error, partial, pending, permission, and recovery states designed before a
  screen is marked complete;
- light, dark, and system appearance from semantic tokens;
- WCAG 2.2 AA minimum and full keyboard operation.

## 3. Current-state assessment

### 3.1 What exists

The current control plane is a React 18 and Vite application in `apps/platform`. It uses Medusa UI,
Medusa icons, TanStack Query, Tailwind 3, and one large tab-based `App.tsx`. It has no URL-backed
screen routing. Authentication is an operator token stored in `sessionStorage`.

The Rust side has a substantial Vendryx-specific platform implementation:

- merchant tenants and domains;
- one subscription per commerce tenant;
- global tier definitions and effective entitlements;
- operators and operator audit;
- invoices, invoice lines, payment attempts, credits, proration, VAT calculations, and dunning;
- launch requests and merchant ownership;
- product-specific provider, courier, settlement, theme, and support operations;
- 74 current route declarations across platform and merchant surfaces.

The existing `platform` model assumes that a tenant is a commerce merchant and that a subscription
points directly to that tenant. It has no product, organization, product instance, multi-product
contract, membership, product-scoped role, or connector model.

### 3.2 What must not be carried into Space

- one shared bootstrap token as the normal sign-in path;
- browser-stored bearer credentials after the bootstrap period;
- in-memory domain registries;
- hard-coded provider records or commercial figures;
- routes that answer successfully without durable behavior;
- one global Trial, Free, or Pro vocabulary shared by every product;
- commerce names such as merchant, store, theme, courier, and gateway in company-level types;
- direct access from Space to a product database;
- UI availability as authorization;
- product-side feature definitions typed manually a second time in Space.

### 3.3 Migration posture

Build Space beside the current control plane. Do not repaint `apps/platform` in place while its
backend contract is being generalized.

- New frontend: `apps/space`.
- New server binary: `crates/space-server`.
- New API root: `/space/v1`.
- New product integration root: `/space/integration/v1`.
- New database schemas owned by bounded modules.
- Existing `apps/platform` remains available and read-only during shadowing and reconciliation.
- Vendryx becomes the first registered product through a connector.
- Retire the old surface only after data reconciliation and workflow parity pass on the same build.

### 3.4 Architecture decisions to record

| Proposed ADR | Decision |
|---|---|
| SPACE-ADR-001 | Space is an independently deployed company control plane, not a widened commerce module. |
| SPACE-ADR-002 | The internal console uses Vite, React Router, customized shadcn/ui, and a Rust API. |
| SPACE-ADR-003 | Human authentication comes from OIDC; Space owns authorization and revocable sessions. |
| SPACE-ADR-004 | Products integrate only through versioned connectors, signed delivery, and service identities. |
| SPACE-ADR-005 | Products own entitlement definitions; Space owns commercial grants and resolved snapshots. |
| SPACE-ADR-006 | Subscription billing is centralized in Space and separated from product transaction money. |
| SPACE-ADR-007 | Vendryx moves through expand, shadow, reconcile, cutover, and contract phases. |
| SPACE-ADR-008 | PostgreSQL outbox/worker is the initial async transport; a broker requires measured need. |

## 4. Product vocabulary

These terms are contractual. Do not use them interchangeably.

| Term | Meaning |
|---|---|
| Organization | A Syed Systems customer or internal business entity that can own subscriptions and product instances. |
| User | A human identity referenced from the configured identity provider. Space does not store a reusable password. |
| Membership | A user's relationship to an organization, including status and organization-level roles. |
| Workforce actor | A Syed Systems operator using the internal Space console. |
| Product | A sellable SaaS system, such as Vendryx Commerce, ERP, or LMS. |
| Product instance | One provisioned or requested deployment/account of a product for an organization. |
| Plan | A versioned package for one product. It defines included entitlements but not the customer's active contract. |
| Price | A currency, amount, interval, and effective period for a plan or add-on. |
| Bundle | A commercial offer containing plans or add-ons from more than one product. |
| Subscription | The customer contract and lifecycle container. It can contain one or more subscription items. |
| Subscription item | One plan or add-on tied to a product instance, quantity, and price. |
| Entitlement definition | A product-owned capability key with type, unit, enforcement mode, and failure policy. |
| Entitlement grant | A plan, add-on, contract, support, or emergency value applied to one product instance. |
| Entitlement snapshot | The versioned, resolved result consumed by a product. |
| Usage event | An idempotent product-reported consumption record for one metric. |
| Provisioning job | The desired transition of a product instance and the connector's observed outcome. |
| Connector | The versioned adapter between Space and one product. |

## 5. Users, roles, and permissions

### 5.1 Identity boundary

Use an OpenID Connect identity provider for human authentication. Space stores the external subject
identifier, profile metadata needed for operations, membership, roles, and audit references. It does
not build another password database.

The first production release requires:

- same-origin Secure, HttpOnly, SameSite session cookie issued by the Rust API;
- short access session with server-side revocation;
- mandatory MFA for workforce actors;
- fresh-MFA step-up for billing, entitlement override, role, impersonation, and service-credential
  operations;
- CSRF protection on cookie-authenticated mutations;
- explicit sign-out and revoke-all-sessions behavior;
- no bearer token in browser storage.

### 5.2 Permission model

Permissions are stable capabilities. Roles are editable bundles of permissions.

Initial permissions:

```text
organizations.read
organizations.manage
memberships.read
memberships.manage
products.read
products.manage
plans.read
plans.manage
subscriptions.read
subscriptions.manage
entitlements.read
entitlements.manage
billing.read
billing.manage
provisioning.read
provisioning.manage
support.access
support.impersonate
audit.read
operators.manage
integrations.manage
system.manage
```

Initial workforce role templates:

| Role | Intended scope |
|---|---|
| Space Owner | All permissions, including role and system management. |
| Product Operator | Product catalog, instances, provisioning, and product-scoped support. |
| Billing Operator | Subscriptions, invoices, credits, and payment attempts. No impersonation. |
| Support Operator | Organization/product reads and tightly scoped support sessions. No billing mutation. |
| Security Auditor | Audit and access reads. No mutation. |
| Read-only Analyst | Commercial and operational projections only. |

An actor may be restricted to specific products. A product-scoped operator for LMS cannot change a
Vendryx plan or inspect ERP support data.

### 5.3 Dangerous-operation policy

The following actions require a reason, fresh MFA, explicit affected scope, and append-only audit:

- suspend or terminate an organization or product instance;
- activate, cancel, or backdate a subscription;
- change a price or plan entitlement;
- create an entitlement override;
- issue a credit or retry an unknown payment;
- start support impersonation;
- reveal or rotate a service credential;
- change a role or permission;
- trigger a product-wide kill switch.

Where possible, show a dry-run impact preview and require `If-Match` revision protection.

## 6. Target architecture

```text
Browser
  Space Vite SPA
       |
       | same-origin HTTPS, session cookie, CSRF token
       v
  space-server (Axum)
       |
       +-- Space API routers
       +-- workflow orchestration
       +-- query projections
       +-- background worker and scheduler
       |
       v
  Space PostgreSQL
       +-- access schema
       +-- catalog schema
       +-- subscription schema
       +-- entitlement schema
       +-- billing schema
       +-- provisioning schema
       +-- audit schema
       +-- outbox and delivery state
       |
       +-- signed connector calls/events --> Vendryx
       +-- signed connector calls/events --> ERP
       +-- signed connector calls/events --> LMS
       +-- payment provider adapters
       +-- identity provider
```

### 6.1 Deployment boundary

Space is deployed independently from every managed product. It can live in this monorepo initially,
but it has its own binary, database credentials, migrations, deployment pipeline, domain, and
availability monitoring.

In development, the Space database may share a PostgreSQL server. It must still use a separate
database or role boundary. Product service credentials cannot query Space tables, and Space cannot
query product tables.

### 6.2 Availability behavior

A temporary Space outage must not take every product offline.

- Products cache a signed last-known-good entitlement snapshot.
- Every entitlement definition states its failure policy and maximum stale age.
- Core read access normally continues during a short control-plane outage.
- Cost-bearing or security-sensitive capabilities fail closed after their defined grace window.
- Product writes that report usage queue locally and replay with idempotency keys.
- Provisioning and subscription changes remain pending until connector acknowledgement.
- The UI never changes pending to active because a timeout elapsed.

## 7. Rust bounded contexts and crate ownership

The new modules remain leaves and never depend on another module crate. Cross-module writes live in
Space workflows. Cross-module reads use query projections outside the modules. Subscribers remain in
subscriber crates.

| Crate | Owned schema | Responsibility |
|---|---|---|
| `space-access` | `space_access` | users, workforce actors, memberships, roles, permissions, sessions, service accounts |
| `space-organization` | `space_organization` | customer organizations, lifecycle, billing identity, ownership links |
| `space-catalog` | `space_catalog` | products, product manifests, plan versions, prices, bundles, add-ons, entitlement definitions |
| `space-subscription` | `space_subscription` | subscriptions, items, lifecycle changes, contract overrides |
| `space-entitlement` | `space_entitlement` | grants, resolution, snapshots, usage events, aggregates, quota state |
| `space-billing` | `space_billing` | billing accounts, invoices, lines, attempts, credits, dunning, provider references |
| `space-provisioning` | `space_provisioning` | product instances, desired/observed state, jobs, connector commands and acknowledgements |
| `space-audit` | `space_audit` | immutable security and business audit records, evidence exports |

Supporting crates:

| Crate | Responsibility |
|---|---|
| `space-types` | Dependency-free primitives only: typed IDs, product key, organization key, revision, idempotency key, error, metadata policy. |
| `space-flows-lifecycle` | Organization, subscription, entitlement, billing, and provisioning workflows. |
| `space-query` | Authorized read models spanning Space schemas. |
| `space-api` | Operator and integration REST DTOs, extractors, authorization, RFC 9457 errors, OpenAPI. |
| `space-connector-vendryx` | Vendryx implementation of the product connector contract. |
| `space-provider-bkash` | Subscription payment adapter where approved and already supported. |
| `space-provider-sslcommerz` | Subscription payment adapter where approved and already supported. |
| `space-subscribers` | Audit, outbox delivery, projections, and notifications. |
| `space-server` | Composition root, config, routers, scheduler, health, telemetry, and graceful shutdown. |

Do not move Product, Plan, Subscription, or Entitlement into `stheno-types`. Those are Space domain
concepts, not shared primitives.

`space-provisioning` owns the product connector trait. Connector crates depend only on that
trait-owning module. `space-billing` owns the subscription payment provider trait. Payment provider
crates depend only on that module. Workflows coordinate those modules without lateral module
dependencies.

## 8. Data model

All IDs are opaque typed IDs. All records have `created_at`, `updated_at` where mutable, and an
integer revision for optimistic concurrency where operators can edit them.

### 8.1 Access

```text
user_account
  id, external_subject, email, display_name, status, last_seen_at

workforce_actor
  user_id, status, mfa_required, revoked_at

organization_membership
  id, organization_id, user_id, status, invited_at, joined_at

role
  id, name, description, system_template, revision

role_permission
  role_id, permission_key

role_assignment
  id, user_id, role_id, organization_id?, product_id?, starts_at?, ends_at?

operator_session
  id, user_id, token_hash, ip_hash, user_agent, expires_at, revoked_at

service_account
  id, product_id?, name, scopes, credential_hash/public_key, expires_at, revoked_at
```

Email uniqueness follows the identity-provider policy. The external subject is the authentication
identity key. Do not make an email address the permanent foreign key.

### 8.2 Customer and product catalog

```text
organization
  id, key, legal_name, display_name, status, billing_email, country_code, tax_id?, revision

product
  id, key, display_name, description, status, connector_kind, manifest_version, revision

product_manifest
  product_id, version, schema_version, document, checksum, accepted_at

plan
  id, product_id, code, version, display_name, status, effective_from, effective_until?, revision

price
  id, plan_id/add_on_id, currency, amount, billing_interval, interval_count,
  effective_from, effective_until?, tax_behavior, revision

bundle
  id, code, version, display_name, status, revision

bundle_item
  bundle_id, product_id, plan_id/add_on_id, quantity

add_on
  id, product_id, code, version, display_name, status, revision

entitlement_definition
  id, product_id, key, value_kind, unit, enforcement_mode, failure_policy,
  maximum_stale_seconds, description, manifest_version

plan_entitlement
  plan_id/add_on_id, entitlement_definition_id, value
```

Money uses the repository `Money` pattern and decimal storage. No floating point appears in prices,
invoices, proration, tax, credits, tests, or fixtures.

Plan versions are immutable after first activation. A changed package creates a new version. Existing
subscriptions remain pinned until an explicit migration.

### 8.3 Subscription and entitlement

```text
subscription
  id, organization_id, status, currency, billing_anchor, trial_ends_at?,
  current_period_start, current_period_end, cancel_at?, revision

subscription_item
  id, subscription_id, product_instance_id, plan_id?, add_on_id?, price_id,
  quantity, starts_at, ends_at?, revision

subscription_change
  id, subscription_id, type, requested_by, reason, effective_at,
  before_revision, after_revision, status, idempotency_key

product_instance
  id, organization_id, product_id, external_reference?, region,
  desired_state, observed_state, connector_version, revision

entitlement_grant
  id, product_instance_id, definition_id, source_type, source_id,
  value, priority, starts_at, ends_at?, reason, revision

entitlement_snapshot
  id, product_instance_id, version, checksum, resolved_document,
  generated_at, valid_until, signature

usage_event
  id, product_instance_id, definition_id, quantity, occurred_at,
  source_event_id, idempotency_key, dimensions

usage_aggregate
  product_instance_id, definition_id, window_key, window_start, quantity, revision
```

`dimensions` is allow-listed by definition. It cannot contain passwords, tokens, card data, message
content, full customer records, or unbounded arbitrary PII.

### 8.4 Billing

```text
billing_account
  id, organization_id, provider_customer_reference?, default_payment_method_reference?, status

invoice
  id, organization_id, subscription_id, currency, status, period_start,
  period_end, subtotal, discount_total, tax_total, total, balance_due,
  issued_at?, due_at?, paid_at?, revision

invoice_line
  id, invoice_id, subscription_item_id?, product_id, description,
  quantity, unit_amount, net_amount, tax_amount, total_amount, signed_metadata

payment_attempt
  id, invoice_id, provider, provider_reference?, idempotency_key,
  requested_amount, status, ambiguity_state, attempted_at, resolved_at?

credit_adjustment
  id, invoice_id, amount, reason, issued_by, created_at

dunning_case
  id, invoice_id, stage, next_attempt_at?, grace_ends_at?, status, revision
```

One invoice has one currency. A bundle or subscription containing mixed-currency items is rejected at
the write boundary.

### 8.5 Provisioning, delivery, and audit

```text
provisioning_job
  id, product_instance_id, operation, desired_revision, status,
  attempt_count, next_attempt_at?, idempotency_key, last_error_code?, started_at?, completed_at?

connector_delivery
  id, product_id, product_instance_id?, direction, event_type,
  payload_checksum, attempt, status, response_code?, next_attempt_at?, delivered_at?

outbox_event
  id, aggregate_type, aggregate_id, event_type, payload, created_at, published_at?

audit_event
  id, occurred_at, actor_type, actor_id, action, resource_type, resource_id,
  organization_id?, product_id?, request_id, reason?, before_redacted?, after_redacted?, outcome
```

Audit rows are append-only. Secret values never enter audit before/after documents.

## 9. Entitlement model

### 9.1 Product-owned definitions

Each product repository owns a versioned entitlement manifest. Space validates and imports it. The
operator may configure plans and grants from registered definitions but cannot invent an unknown
key in a text field.

Example manifest shape:

```json
{
  "schema_version": 1,
  "product_key": "vendryx-commerce",
  "connector_version": "1",
  "entitlements": [
    {
      "key": "analytics",
      "kind": "boolean",
      "enforcement": "route",
      "failure_policy": "last_known_good",
      "maximum_stale_seconds": 86400
    },
    {
      "key": "products",
      "kind": "quota",
      "unit": "product",
      "enforcement": "write_boundary",
      "failure_policy": "last_known_good",
      "maximum_stale_seconds": 86400
    }
  ]
}
```

The values above are contract examples, not production defaults.

### 9.2 Resolution order

Resolution is deterministic and returns an explanation for every value.

1. Product-wide emergency deny or safety switch.
2. Instance-specific emergency deny.
3. Time-bound support or incident grant.
4. Explicit contract override.
5. Active add-on grant.
6. Active plan grant.
7. Definition default, which is deny or zero unless explicitly defined otherwise.

For boolean values, a higher-priority explicit deny wins. For quota/window values, the highest
priority active grant replaces the lower value unless the definition declares additive behavior.
Additive behavior must be explicit in the product manifest and property-tested.

Every resolved entry includes:

- final value;
- definition version;
- source type and source ID;
- effective period;
- resolution reason;
- snapshot version and checksum.

### 9.3 Distribution and enforcement

- Product services fetch snapshots with `ETag` and `If-None-Match`.
- Space publishes `entitlements.snapshot.published` through the transactional outbox.
- Webhooks are signed, timestamped, replay-protected, and retried.
- Products enforce entitlements at their own route/workflow/write boundaries.
- Space is authoritative for the contract; the product is authoritative for the enforcement result.
- Usage events are idempotent and can be replayed without double counting.
- An operator can inspect why a value resolved, but cannot edit the resolved snapshot directly.

## 10. Product connector contract

Every product implements the same minimum connector behavior:

```text
register manifest
validate plan/entitlement compatibility
provision product instance
report observed instance state
receive entitlement snapshot notification
fetch current entitlement snapshot
report usage event
suspend/reactivate/cancel instance
return health and connector version
acknowledge support-access start/end where supported
```

### 10.1 Connector rules

- Product connectors use service accounts, not workforce sessions.
- Service credentials are scoped to one product and environment.
- All mutating commands carry an idempotency key.
- A timeout returns unknown/pending. It never becomes failed or successful by assumption.
- Product callbacks include the command ID and desired revision.
- Space compares desired and observed revision and shows drift explicitly.
- Connector payloads are versioned and contract-tested in both repositories.
- Breaking changes require a new connector major version and a compatibility window.

### 10.2 Vendryx adapter

The first adapter maps existing commerce concepts without making them universal:

| Existing Vendryx concept | Space concept |
|---|---|
| merchant account | organization owner membership |
| platform tenant/store | Vendryx product instance |
| tier definition | versioned Vendryx plan |
| platform subscription | Space subscription item |
| feature/quota/window | Vendryx entitlement definition/grant |
| entitlement usage | Space usage event/aggregate |
| platform invoice | Space invoice with imported external reference |
| launch request | provisioning request and Vendryx connector workflow |
| operator | Space workforce actor |
| operator audit | Space audit event |

Domains, themes, couriers, gateways, orders, and storefront configuration remain Vendryx product
operations. Space may show a connector health summary and deep link, but it does not absorb those
domain models.

## 11. Subscription and billing workflows

### 11.1 Activate subscription

```text
validate organization and offer
  -> lock price and plan versions
  -> create pending subscription and items
  -> authorize/confirm required payment agreement
  -> request product instance provisioning
  -> publish initial entitlement snapshots
  -> await connector acknowledgements
  -> activate subscription
  -> issue audit and notification events
```

Each mutating step is idempotent. Each workflow step has compensation or an explicit
`no_compensation` reason. An instance that provisions after a timeout is reconciled by command ID,
not provisioned twice.

### 11.2 Change plan or bundle

1. Read subscription under revision lock.
2. Calculate a dry-run diff: price, proration, entitlements, quantities, and affected instances.
3. Present the exact effective date and consequences.
4. Commit the change with `If-Match` and an idempotency key.
5. Create signed invoice/credit lines.
6. Publish new snapshots.
7. Reconcile product acknowledgements.
8. Audit before/after without secrets.

### 11.3 Billing cycle and dunning

- Generate an invoice once per subscription and period.
- Use signed line metadata so later catalog edits do not rewrite history.
- Send provider calls only through the provider idempotency wrapper.
- Treat a dropped provider response as unknown and reconcile before retry.
- Advance dunning from scheduled jobs with a single-writer guarantee.
- Apply grace and suspension through subscription state, entitlement snapshots, and connector
  desired state.
- Never delete customer product data because a subscription ends.

### 11.4 Cancellation

Cancellation supports end-of-term and immediate modes. The dry run must show invoice/credit impact,
entitlement expiry, product-instance desired state, retention policy, and reversibility. Destructive
product deletion is a separate workflow and is never implied by cancellation.

## 12. API design

The Space API is independent from the frozen Medusa-compatible `/admin/*` surface.

### 12.1 Operator API

```text
POST   /space/v1/auth/session
DELETE /space/v1/auth/session
GET    /space/v1/me
GET    /space/v1/me/permissions

GET    /space/v1/overview

GET    /space/v1/organizations
POST   /space/v1/organizations
GET    /space/v1/organizations/{organization_id}
PATCH  /space/v1/organizations/{organization_id}
GET    /space/v1/organizations/{organization_id}/memberships
POST   /space/v1/organizations/{organization_id}/memberships

GET    /space/v1/products
POST   /space/v1/products
GET    /space/v1/products/{product_id}
PATCH  /space/v1/products/{product_id}
POST   /space/v1/products/{product_id}/manifests
GET    /space/v1/products/{product_id}/entitlement-definitions

GET    /space/v1/plans
POST   /space/v1/plans
GET    /space/v1/plans/{plan_id}
POST   /space/v1/plans/{plan_id}/activate
POST   /space/v1/plans/{plan_id}/supersede
GET    /space/v1/prices
POST   /space/v1/prices

GET    /space/v1/subscriptions
POST   /space/v1/subscriptions/preview
POST   /space/v1/subscriptions
GET    /space/v1/subscriptions/{subscription_id}
POST   /space/v1/subscriptions/{subscription_id}/changes/preview
POST   /space/v1/subscriptions/{subscription_id}/changes
POST   /space/v1/subscriptions/{subscription_id}/cancel

GET    /space/v1/product-instances
GET    /space/v1/product-instances/{instance_id}
POST   /space/v1/product-instances/{instance_id}/commands
GET    /space/v1/product-instances/{instance_id}/entitlements
POST   /space/v1/product-instances/{instance_id}/entitlement-grants
DELETE /space/v1/product-instances/{instance_id}/entitlement-grants/{grant_id}

GET    /space/v1/invoices
GET    /space/v1/invoices/{invoice_id}
POST   /space/v1/invoices/{invoice_id}/credits
POST   /space/v1/invoices/{invoice_id}/payment-attempts

GET    /space/v1/provisioning-jobs
GET    /space/v1/connector-deliveries
POST   /space/v1/connector-deliveries/{delivery_id}/replay

GET    /space/v1/users
GET    /space/v1/roles
POST   /space/v1/role-assignments
DELETE /space/v1/role-assignments/{assignment_id}

POST   /space/v1/support-sessions
DELETE /space/v1/support-sessions/{session_id}

GET    /space/v1/audit-events
GET    /space/v1/service-accounts
POST   /space/v1/service-accounts
POST   /space/v1/service-accounts/{account_id}/rotate
DELETE /space/v1/service-accounts/{account_id}
```

### 12.2 Product integration API

```text
POST /space/integration/v1/manifests/validate
GET  /space/integration/v1/instances/{external_reference}/entitlements
POST /space/integration/v1/instances/{external_reference}/observed-state
POST /space/integration/v1/usage-events
POST /space/integration/v1/commands/{command_id}/acknowledgements
GET  /space/integration/v1/connector-config
```

### 12.3 API rules

- Cursor pagination on all unbounded lists.
- Explicit filter and sort allow-lists.
- RFC 9457 problem details for errors.
- Request DTOs never reuse entity models.
- Server-derived actor, organization, and product scope where possible.
- `Idempotency-Key` required on external or money-affecting mutations.
- `ETag` and `If-Match` required on high-risk concurrent edits.
- `X-Request-Id` returned and stored in audit/delivery records.
- No secret value returned after initial creation.
- OpenAPI generated and checked for drift in CI.
- TypeScript client generated from the accepted OpenAPI document.

## 13. Frontend information architecture

### 13.1 Route tree

```text
/overview

/organizations
/organizations/:organizationId/overview
/organizations/:organizationId/users
/organizations/:organizationId/products
/organizations/:organizationId/subscriptions
/organizations/:organizationId/invoices
/organizations/:organizationId/audit

/products
/products/:productId/overview
/products/:productId/plans
/products/:productId/entitlements
/products/:productId/instances
/products/:productId/connectors

/subscriptions
/subscriptions/:subscriptionId/overview
/subscriptions/:subscriptionId/items
/subscriptions/:subscriptionId/entitlements
/subscriptions/:subscriptionId/invoices
/subscriptions/:subscriptionId/history

/billing/invoices
/billing/payments
/billing/dunning
/billing/credits

/operations/provisioning
/operations/connectors
/operations/deliveries
/operations/incidents

/access/users
/access/roles
/access/service-accounts
/access/sessions

/audit
/settings/company
/settings/identity
/settings/billing-providers
/settings/webhooks
```

Routes are permission-aware but not permission-enforcing. Rust remains authoritative.

### 13.2 Global shell

Desktop:

- fixed 240 px collapsible sidebar;
- Space mark, environment label, and product-neutral primary navigation;
- top bar with organization/product context, global search, pending-work count, help, and actor menu;
- main content with route title, breadcrumbs, scoped actions, and saved view controls;
- optional right detail sheet for quick inspection, never the only path to a record.

Tablet:

- collapsible sidebar drawer;
- preserved route title and context selector;
- tables retain critical columns and move secondary facts into expandable row detail.

Mobile:

- supported for review, triage, approvals, and emergency action;
- complex catalog/plan construction may require desktop and must say so before entry;
- no squeezed desktop tables;
- destructive actions remain behind full-screen confirmation with scope and reason.

### 13.3 Primary navigation

```text
Overview
Organizations
Products
Subscriptions
Billing
Operations
Access
Audit
Settings
```

Do not place product-specific nouns such as Orders, Courses, Payroll, Couriers, or Storefronts in the
Space primary navigation. Product details can provide deep links to the correct product console.

### 13.4 Overview page

The overview is an action queue, not a wall of vanity metrics.

Priority sections:

1. Failed or ambiguous payment attempts.
2. Provisioning jobs waiting or retrying.
3. Entitlement delivery drift.
4. Subscriptions entering grace or suspension.
5. Connector/service credential expiry.
6. Pending access approvals or suspicious support sessions.
7. Company revenue and product distribution summaries, only from real projections.

Every count links to its filtered records. If a projection is stale, the timestamp and stale state
are visible.

### 13.5 Detail-page pattern

Every primary object follows one pattern:

- identity header with status, revision, and scoped actions;
- concise summary facts;
- nested URL-backed tabs;
- chronological activity/audit section;
- dangerous actions grouped separately;
- no editable fields hidden inside a read-only table cell;
- no mutation that exists only inside a hover menu.

## 14. Space design system

### 14.1 Brand direction

Space should communicate coordination and control across products. It should not look like a space
game, crypto product, AI wrapper, or astronomy app.

Recommended identity direction: **Spaceframe S**.

- a geometric S formed from two interlocking planes or brackets;
- one central negative-space corridor representing the control layer;
- no planet, rocket, star, orbit trail, or gradient nebula;
- recognizable at 16 px and in one color;
- wordmark set in a modern grotesk, with “by Syed Systems” reserved for login, legal, and brand
  contexts rather than every navigation state.

Required assets:

```text
apps/space/public/brand/space/space-mark.svg
apps/space/public/brand/space/space-wordmark.svg
apps/space/public/brand/space/space-lockup.svg
apps/space/public/brand/space/space-lockup-dark.svg
apps/space/public/brand/space/favicon.svg
apps/space/public/brand/space/favicon.ico
apps/space/public/brand/space/apple-touch-icon.png
apps/space/public/brand/space/brand.json
docs/SPACE_BRAND.md
```

The mark is designed and exported in a vector tool. Do not improvise production logo paths inside a
React component. The source design file, grid, clear-space rule, minimum sizes, monochrome use, dark
use, misuse examples, and license ownership are part of the handoff.

“Space” is a common commercial name. Legal/trademark, product-domain, package-name, and social-handle
checks are required before public launch. This plan treats Space as the approved working name.

### 14.2 Proposed token direction

These are design starting values, not approved brand facts:

- cool off-white canvas and graphite text;
- near-black blue-gray dark canvas, never pure black;
- one cobalt accent for selection, focus, links, and primary actions;
- green, amber, red, and cyan reserved for semantic state;
- Geist Sans for interface text and Geist Mono for IDs, revisions, money tables, and timestamps;
- 4 px base spacing with an 8 px primary rhythm;
- 8 px control radius, 12 px dialog/panel radius, full radius only for badges and avatars;
- 1 px neutral separators; elevation only for menus, sheets, dialogs, and sticky layers;
- 120-180 ms opacity/transform transitions, disabled under reduced motion.

All colors live in semantic CSS variables. shadcn component files consume tokens and never carry
product-specific literal colors.

### 14.3 shadcn component inventory

Initialize only what is needed:

```text
alert-dialog, avatar, badge, breadcrumb, button, calendar, checkbox,
collapsible, command, dialog, dropdown-menu, field, input, input-otp,
label, pagination, popover, progress, radio-group, select, separator,
sheet, sidebar, skeleton, switch, table, tabs, textarea, tooltip
```

Owned composite components:

```text
AppShell
SpaceSidebar
ContextSwitcher
GlobalCommand
PageHeader
ObjectHeader
StatusBadge
DataTable
FilterBar
SavedViewMenu
ColumnPicker
AsyncBoundary
EmptyState
ErrorState
DefinitionList
MoneyText
Timestamp
RevisionBadge
ImpactPreview
DangerZone
ReasonDialog
EntitlementValue
EntitlementResolutionTrace
AuditTimeline
ConnectorHealth
ProvisioningTimeline
PermissionGate
```

No wrapper may silently discard accessibility props from its shadcn primitive.

### 14.4 Tables and data density

- Server-side cursor pagination for unbounded lists.
- URL-backed filters and sorting so views are shareable.
- Column visibility may persist as a harmless browser preference.
- Bulk actions appear only after selection and show affected count/scope.
- Sticky headers only inside a correctly labeled scroll region.
- Numeric columns align right and use tabular figures.
- IDs have copy actions and a visible label.
- Status always uses text; color/icon is supplementary.
- Virtualize audit/delivery lists only when measured row counts require it.
- At narrow widths, switch to structured record cards rather than horizontal compression.

### 14.5 State contract

Every route and mutation defines:

| State | Requirement |
|---|---|
| Loading | Shape-matched skeleton without false default values. |
| Loaded | Authoritative data, revision, and freshness where relevant. |
| Empty | Explain the absence and the authorized next action. |
| Partial | Identify which product/projection is unavailable. Do not replace it with zero. |
| Stale | Show last successful update and retry. |
| Validation failure | Field-associated correction with preserved input. |
| Permission denied | Hide unavailable actions and handle direct access safely. |
| Conflict | Show changed revision and offer refresh/compare, never overwrite silently. |
| Pending | Show workflow/command ID and observed progress. |
| Unknown external result | Reconcile before retry; never claim failure or success. |
| Success | Confirm after server acknowledgement and update the exact cache objects. |
| Destructive success | Confirm scope, retain audit reference, and offer recovery only if real. |

## 15. Frontend source layout

```text
apps/space/
  public/brand/space/
  src/
    app/
      router.tsx
      providers.tsx
      route-registry.ts
    components/
      ui/                 # generated and customized shadcn source
      shell/
      data-table/
      forms/
      states/
      audit/
      entitlements/
    features/
      overview/
      organizations/
      products/
      plans/
      subscriptions/
      entitlements/
      billing/
      provisioning/
      access/
      audit/
      settings/
    lib/
      api-client/         # generated types plus small transport adapter
      auth/
      query/
      permissions/
      formatting/
      telemetry/
    styles/
      tokens.css
      globals.css
    test/
```

Feature modules may import shared `components` and `lib`; they do not import another feature's
private components. Shared components must be promoted intentionally, not through relative-path
reach-through.

## 16. Observability and operational safety

Space requires:

- structured logs with request, actor, organization, product, workflow, and connector command IDs;
- metrics for API latency/error, auth failures, workflow backlog, outbox lag, connector success,
  entitlement drift, billing ambiguity, dunning stage, and audit-write failure;
- distributed traces across Space and product connectors without secret/PII payloads;
- readiness that checks database and required worker dependencies;
- liveness that does not fail only because a product connector is down;
- alerts on audit-write failure, entitlement-delivery backlog, repeated unknown payment result,
  failed provisioning, credential expiry, and elevated authorization denial;
- an operator-visible system status page backed by these signals, not literals.

No business mutation succeeds if its required audit or outbox record cannot commit in the same
transaction.

## 17. Migration from the current Vendryx control plane

### 17.1 Expand

1. Add Space crates, schemas, server, API, and empty frontend without changing existing routes.
2. Register Vendryx as product key `vendryx-commerce` with an imported entitlement manifest.
3. Create organization and product-instance mapping tables with immutable legacy IDs.
4. Import plans, prices, subscriptions, entitlements, invoices, operators, and audit references.
5. Add an outbox publisher to current Vendryx platform changes.
6. Implement the Vendryx connector and contract tests.

### 17.2 Shadow

1. Dual-publish platform changes to Space through idempotent events.
2. Resolve Space entitlement snapshots and compare them to current Vendryx effective entitlements.
3. Compare invoice balances, subscription state, and operator audit counts.
4. Run `apps/space` in read-only mode for internal operators.
5. Report reconciliation differences by legacy ID and block cutover while unexplained differences
   remain.

### 17.3 Cut over

1. Freeze old control-plane writes.
2. Run final import and reconciliation.
3. Enable Space subscription and entitlement writes for Vendryx.
4. Switch Vendryx to Space snapshots with last-known-good fallback.
5. Enable Space billing scheduler after an idempotency dry run.
6. Redirect operator access from `apps/platform` to `apps/space`.
7. Keep old read paths available for the agreed rollback window.

### 17.4 Contract

Only after the rollback window and evidence sign-off:

- remove old control-plane write routes;
- archive or remove `apps/platform`;
- retain immutable legacy IDs and audit references;
- use phased destructive migrations marked `phase: contract` and paired with their expansion phase;
- update `CONTROL_PLANE.md`, `CONTROL_PLANE_PLAN.md`, decisions, data model, API spec, workflows,
  security, sitemap, tasks, and work log together.

## 18. Delivery phases and tickets

Sizes are relative complexity, not calendar estimates.

Implementation order:

```text
accepted ADRs and brand
  -> service and frontend foundations
  -> identity, authorization, and audit
  -> organizations and product catalog
  -> subscriptions and entitlement resolution
  -> connector runtime and Vendryx adapter
  -> billing and dunning
  -> shadow migration and reconciliation
  -> write cutover and retirement
```

Do not parallelize a later phase across an unsettled contract from an earlier phase. Frontend work
can run beside its backend ticket only after the OpenAPI examples and state machine are accepted.

### Foundation

| Ticket | Size | Owner area | Deliverable | Acceptance |
|---|---:|---|---|---|
| SPACE-001 | S | Architecture/docs | Accept ADRs for Space boundary, Vite, auth provider, connector transport, billing ownership, and migration. | Decisions and all named docs agree; open choices are not hidden in code. |
| SPACE-002 | S | Brand | Validate name and create Spaceframe S identity system. | Vector source plus required light/dark/mono/favicon assets and `SPACE_BRAND.md`. |
| SPACE-003 | M | Backend/platform | Scaffold `space-server`, config, health, migrations, telemetry, and CI gates. | Empty service builds, migrates, starts, reports health, and shuts down gracefully. |
| SPACE-004 | M | Frontend/platform | Scaffold `apps/space` with Vite, React Router, customized shadcn, tokens, query, auth boundary, and tests. | Shell renders via deep URL, light/dark/system work, no Medusa UI import. |
| SPACE-005 | M | API/tooling | Add versioned OpenAPI generation and TypeScript client pipeline. | Rust contract change fails CI until generated client is updated. |

### Access and audit

| Ticket | Size | Owner area | Deliverable | Acceptance |
|---|---:|---|---|---|
| SPACE-010 | L | Access | OIDC session, logout, revocation, CSRF, MFA claim/step-up. | Unauthorized requests fail closed; no browser-stored bearer token. |
| SPACE-011 | M | Access | Permissions, role templates, product scopes, assignments. | Authorization matrix covers every operator route and scope. |
| SPACE-012 | M | Audit | Append-only audit module and middleware/workflow integration. | Every write is attributed; audit failure rolls back required mutation. |
| SPACE-013 | M | Frontend/access | Login, actor menu, access pages, permission-aware shell. | Keyboard/a11y tests and direct-route permission tests pass. |

### Product catalog and organizations

| Ticket | Size | Owner area | Deliverable | Acceptance |
|---|---:|---|---|---|
| SPACE-020 | M | Catalog | Product, manifest, entitlement-definition persistence and validation. | Unknown or breaking manifests fail with actionable errors. |
| SPACE-021 | L | Catalog | Versioned plans, prices, add-ons, bundles, activation/supersession. | Activated versions are immutable; money/currency rules property-tested. |
| SPACE-022 | M | Access/customer | Organizations, users, memberships, lifecycle. | Cross-organization reads/writes denied; external subject is identity key. |
| SPACE-023 | M | Frontend/catalog | Products, plans, entitlements, organizations, users. | URL-backed lists/details, full state contract, no fabricated figures. |

### Subscription and entitlement core

| Ticket | Size | Owner area | Deliverable | Acceptance |
|---|---:|---|---|---|
| SPACE-030 | L | Subscription | Subscription and item lifecycle with optimistic concurrency. | Create/change/cancel are idempotent and revision-safe. |
| SPACE-031 | L | Entitlement | Grant model and deterministic resolver with explanation trace. | Table/property tests cover precedence, time, quota, add-on, deny, and expiry. |
| SPACE-032 | L | Entitlement | Signed snapshots, ETags, usage ingest, aggregates, stale policy. | Replay does not double-count; product can operate on last-known-good policy. |
| SPACE-033 | M | Frontend/subscription | Subscription detail, impact preview, entitlement trace, override workflow. | Operator sees exact source and consequence before mutation. |

### Product instances and connectors

| Ticket | Size | Owner area | Deliverable | Acceptance |
|---|---:|---|---|---|
| SPACE-040 | L | Provisioning | Product instance, desired/observed state, provisioning jobs, retries. | Kill-and-resume and duplicate-command tests prove idempotency. |
| SPACE-041 | L | Integration | Service accounts, signed delivery, replay protection, outbox worker. | Rotation, expiry, retry, and dead-letter behavior tested. |
| SPACE-042 | L | Vendryx adapter | Manifest, provisioning, snapshot, usage, lifecycle integration. | Bidirectional contract suite passes against current Vendryx. |
| SPACE-043 | M | Frontend/operations | Provisioning, connectors, deliveries, drift, replay screens. | Unknown/pending/failed/recovered states use real job and delivery records. |

### Billing

| Ticket | Size | Owner area | Deliverable | Acceptance |
|---|---:|---|---|---|
| SPACE-050 | L | Billing | Billing accounts, invoices, signed lines, credits, proration. | Exact money properties and period idempotency pass. |
| SPACE-051 | L | Providers | Payment adapter bridge and ambiguity reconciliation. | Dropped response after provider commit cannot double-charge. |
| SPACE-052 | L | Billing | Scheduler, dunning, grace, entitlement/instance suspension. | Single-writer concurrency and complete lifecycle tests pass. |
| SPACE-053 | M | Frontend/billing | Invoice, payment, credit, dunning, revenue views. | Every amount traces to an API field; dangerous actions use impact/reason flow. |

### Migration and release

| Ticket | Size | Owner area | Deliverable | Acceptance |
|---|---:|---|---|---|
| SPACE-060 | M | Migration | Vendryx legacy ID mapping and import commands. | Repeatable import gives identical result and produces reconciliation report. |
| SPACE-061 | L | Migration | Shadow event sync and entitlement/billing comparison. | No unexplained material differences for the agreed observation window. |
| SPACE-062 | M | Frontend | Overview action queue and global command search. | Every item is real, scoped, fresh/stale labeled, and deep-linked. |
| SPACE-063 | M | Quality | Accessibility, responsive, security, performance, and visual evidence suite. | Release matrix passes supported browsers/viewports and WCAG 2.2 AA gates. |
| SPACE-064 | L | Release | Controlled write freeze, cutover, rollback drill, and old-plane retirement. | Restore/cutback runbook rehearsed; data/audit continuity verified. |

### Future product onboarding template

Each ERP/LMS product follows these tickets without changing Space core:

1. `PRODUCT-01`: approve product metadata and connector owner.
2. `PRODUCT-02`: publish and validate entitlement manifest.
3. `PRODUCT-03`: implement provision/suspend/reactivate/cancel connector commands.
4. `PRODUCT-04`: implement snapshot fetch/cache and webhook handling.
5. `PRODUCT-05`: implement idempotent usage reporting.
6. `PRODUCT-06`: create versioned plans, prices, and optional bundle items.
7. `PRODUCT-07`: run contract, failure, stale, and migration tests.
8. `PRODUCT-08`: enable product for selected organizations, then general availability.

## 19. Test strategy

### 19.1 Rust

- Unit tests for validation and state machines.
- Property tests for money, proration, invoice sums, entitlement precedence, quotas, windows, and
  additive grants.
- Authorization matrix for every route: role, product scope, organization scope, and ownership.
- Workflow compensation test at every mutating step.
- Idempotency and concurrency tests for subscription, billing, usage, and provisioning.
- Provider ambiguity tests where the remote side commits and the response is lost.
- Connector contract suite shared by Vendryx, ERP, and LMS adapters.
- Migration apply, rollback where safe, and schema-drift checks.
- Outbox crash/restart, retry, ordering, dead-letter, and replay tests.
- Audit completeness and secret-redaction tests.
- No lateral module dependency and no cross-schema raw SQL checks.

### 19.2 Frontend

- Vitest and Testing Library for owned composites and permission rendering.
- Mock Service Worker scenarios generated from API examples, labeled as test fixtures.
- Playwright for critical workflows and direct URL access.
- axe checks for every shared layout and critical route.
- keyboard tests for sidebar, command menu, tables, dialogs, sheets, tabs, forms, and date controls.
- 390, 768, 1024, and 1440 CSS pixel visual checks.
- light, dark, system, reduced motion, and 200 percent zoom evidence.
- unexpected console error and `/space/*` 4xx/5xx network gate.
- mutation persistence test: write, reload, second browser context, API read.

### 19.3 Required end-to-end journeys

1. Sign in with MFA and product-scoped role.
2. Create an organization and invite a user.
3. Register a product manifest.
4. Create and activate a plan and price.
5. Preview and activate a subscription with two product items.
6. Provision instances and reconcile one delayed acknowledgement.
7. Resolve and inspect entitlement snapshots in each product.
8. Report duplicate usage events without double counting.
9. Change a plan with proration and concurrent-edit conflict.
10. Generate, collect, lose the provider response, reconcile, and settle an invoice once.
11. Enter grace, suspend affected capabilities, pay, and reactivate.
12. Start and end a time-limited support session.
13. Rotate a service credential without breaking an in-flight delivery.
14. Export an audit trail linking actor, request, workflow, invoice, snapshot, and product command.

## 20. CI and release gates

### 20.1 Existing repository gates

Run the full Rust gates from `AGENTS.md` for every affected workspace change:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
module isolation
SQL boundaries
shared-kernel
module contracts
domain entitlement boundaries
schema drift
OpenAPI drift
documentation sync
```

### 20.2 New Space gates

```text
no fabricated API handlers
no process-global domain stores
no unaudited Space writes
no secret fields in DTO responses, logs, audit, or metadata
no product database connection from space-server
no unknown entitlement key outside an accepted product manifest
all product connector contracts compatible
all unbounded list endpoints cursor-paginated
all money paths free of floating point
all external mutations idempotent
all generated clients synchronized with OpenAPI
no @medusajs/ui or competing component system in apps/space
no icon-only control without accessible name
no visible route that lacks permission/API/state tests
```

Frontend commands are finalized when `apps/space/package.json` lands. Minimum scripts:

```text
npm --prefix apps/space run typecheck
npm --prefix apps/space run lint
npm --prefix apps/space run test
npm --prefix apps/space run build
npm --prefix apps/space run test:e2e
```

### 20.3 Dependency review

Before adding each dependency, the implementing PR records:

- why existing code or a small local implementation is insufficient;
- direct and transitive dependency count;
- last release and maintenance status;
- license;
- bundle or binary impact;
- security posture and update owner.

Do not install every library named in this plan in the scaffold PR. Add it when its first real
consumer is implemented.

## 21. Documentation set

The implementation is incomplete until these documents exist and agree:

```text
docs/SPACE_PRD.md
docs/SPACE_TRD.md
docs/SPACE_DATA_MODEL.md
docs/SPACE_API_SPEC.md
docs/SPACE_WORKFLOWS.md
docs/SPACE_ENTITLEMENTS.md
docs/SPACE_BILLING.md
docs/SPACE_CONNECTOR_SPEC.md
docs/SPACE_SECURITY.md
docs/SPACE_SITEMAP.md
docs/SPACE_BRAND.md
docs/SPACE_IMPLEMENTATION_PLAN.md
docs/SPACE_TASKS.md
docs/SPACE_DECISIONS.md
docs/SPACE_RUNBOOK.md
docs/SPACE_WORK_LOG.md
```

This document can become `SPACE_IMPLEMENTATION_PLAN.md` after the architecture decisions are
accepted. Existing Vendryx control-plane documents remain historical/product-specific and link to
the Space migration decision.

## 22. Decisions required before coding

These are the only blocking product decisions. The rest can proceed from this plan.

1. Confirm Space as the public/internal name after legal and domain checks.
2. Select the OpenID Connect provider and MFA policy.
3. Decide whether an organization may have multiple instances of the same product in the first
   release. The data model supports it; the initial UI can limit it.
4. Decide whether one subscription may combine products in the first release or whether bundles
   create separate product subscriptions under one commercial agreement.
5. Approve supported billing countries, currencies, tax behavior, and payment providers.
6. Define cancellation, retention, grace, and deletion policies per product.
7. Define the entitlement stale/failure policy for each product capability.
8. Decide whether customer self-service is a later `Space Account` surface or internal-only for the
   first release.

Until these are decided, implementers use conservative behavior: one currency per subscription,
no destructive deletion, default-deny unknown entitlements, and no customer self-service routes.

## 23. Handoff protocol for the implementing agent

For each ticket:

1. Read this plan and the accepted Space ADRs.
2. State the implementation ladder rung.
3. Name owned files/crates and avoid unrelated dirty worktree changes.
4. Write the smallest failing test for the ticket's contract.
5. Implement only the named bounded context and legal cross-module channel.
6. Run the ticket gates and quote real output.
7. Update OpenAPI, generated client, data model, workflows, tasks, and work log when the contract
   changes.
8. Record deviations explicitly. Never silently redesign the contract.
9. Attach UI state evidence and API persistence proof before marking a screen complete.
10. Leave the ticket partial if any acceptance criterion or gate remains unmet.

## 24. Definition of done

Space is ready to replace the Vendryx-only control plane when:

- Vendryx is managed as one product through the same connector contract intended for ERP and LMS;
- organizations can hold users, memberships, subscriptions, and multiple product instances;
- plans, prices, bundles, add-ons, and entitlement definitions are versioned and product-scoped;
- entitlement resolution is deterministic, explainable, signed, cached, and enforced by products;
- billing is exact, idempotent, provider-ambiguity safe, scheduled, and tied to subscription state;
- every product mutation has desired state, observed state, retry, reconciliation, and audit;
- workforce authentication uses named identities, MFA, revocable sessions, scoped roles, and
  step-up for dangerous actions;
- no company-level business state lives only in a browser, process map, spreadsheet, or hard-coded
  handler;
- the Space UI uses one customized shadcn system, URL-backed navigation, real data, complete states,
  and WCAG 2.2 AA evidence;
- the old control-plane data reconciles exactly and a rollback drill succeeds;
- full backend, frontend, connector, security, migration, documentation, and release gates pass on
  the same commit.

The intended premium quality comes from trustworthy state, precise scope, calm density, predictable
actions, and strong recovery. The name Space should describe the place where Syed Systems products
are coordinated, not dictate a decorative space theme.
