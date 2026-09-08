# Merchant Admin Dashboard — Apple HIG Audit and Functionalization Plan

**Product:** Vendryx / Stheno merchant admin  
**Audit date:** 2026-09-08  
**Scope:** `apps/admin`, its mounted merchant routes, the supporting Admin/Auth/Platform APIs,
and the documentation that claims those screens are complete  
**Deliverable type:** evidence-backed audit and implementation plan; no production behavior is
changed by this document

## 1. Executive verdict

The dashboard already has a reasonably polished visual shell, consistent component styling, useful
core commerce tables, and a mostly coherent dark appearance. The core Orders, Products, Inventory,
Customers, Promotions, Price Lists, payment-provider management, MFA, and most Analytics data paths
are materially connected to APIs.

It is not ready to be described as a fully functional premium merchant dashboard.

The largest issue is **product truth**, not decoration. Several new, visually complete screens
simulate backend behavior with `localStorage`, `setTimeout`, `Math.random`, hard-coded records, and
success toasts. The most serious examples include courier credentials, custom-domain verification
and TLS, support tickets, account sessions, password rotation, subscription billing, store policies,
and COD rules. These screens invite owners to trust outcomes that did not happen.

The settings experience is also too flat. More than 20 destinations are placed in a long sidebar
with text labels but little explanation, status, search, or icon-assisted recognition. On mobile,
settings lose context and some data tables or long values become cramped or clipped. This does not
meet the requested standard of simple, calm, progressively disclosed administration.

Finally, the current documentation overstates completion. `docs/VENDRYX_SCREEN_MATRIX.md` marks
screens as **PASS** while runtime and source evidence contradict several claims. This audit does not
infer intent; it records that the PASS labels are not supported by the current implementation.

### Release posture

- **Block release claims:** courier connection, custom domains/TLS, billing history/payment methods,
  support tickets, active-device sessions, policies, and COD rules.
- **Continue using:** API-backed core commerce routes, MFA, entitlement reads, and most analytics,
  subject to the issues below.
- **Do first:** remove or feature-hide simulated success paths, establish one route/data truth
  registry, and reconcile the requested redesign with the current PRD.
- **Then:** restructure settings, apply the Apple-inspired interaction system, and finish each
  backend workflow with honest state and recovery.

## 2. Scope contract and the implementation ladder

### 2.1 Project-spec conflict that must be decided openly

`docs/PRD.md` §6.1 currently says the 1.0 admin is a Medusa fork with **rebranding only** and **no
layout redesign**, except accessibility fixes. The requested Apple HIG redesign—especially settings
hierarchy, navigation, layout, and interaction changes—is broader than that contract.

Before implementation, record one of these decisions in `docs/DECISIONS.md`:

1. Amend the 1.0 scope to permit the merchant-admin redesign described here; or
2. Keep 1.0 at functional parity and truth remediation, then ship the Apple-inspired information
   architecture as a post-1.0 additive phase.

The plan below separates truth/functionality from visual restructuring so either decision remains
possible. No developer should silently override the PRD.

### 2.2 Ladder result

**Stopped at rung 2 — reuse what already exists.**

The codebase already contains the admin fork, design primitives, entitlement endpoints, a real
password-update flow, MFA flows, payment provider APIs, platform invoice data, domain-management
data, analytics endpoints, routing infrastructure, and test harnesses. The correct plan is to bind,
bridge, consolidate, or hide those capabilities. It is not to build another dashboard, another
platform module, or another disconnected settings store.

Where capability is genuinely absent—merchant courier integrations, merchant support tickets,
public policy publishing, and cross-device session records—the screen must remain unmounted or
clearly unavailable until the backend contract exists.

## 3. Audit method and evidence

The audit combined:

- Full source inspection of merchant routes, layouts, nav configuration, query hooks, mutation
  handlers, and production uses of `localStorage`, timers, random values, and hard-coded records.
- Cross-checking against the PRD, sitemap, entitlements, adaptation plan, route matrix, API audit,
  auth routes, admin-platform routes, and platform service routes.
- An authenticated browser pass against the running Pro tenant on desktop and mobile-sized
  viewports, including navigation, settings, light/dark appearance, route failures, console/network
  failures, and responsive behavior.
- A direct Playwright route walk covering 35 merchant/settings destinations.
- A filtered run of the existing admin E2E suite.
- The project `apple-design` skill and the source-backed `apple-hig` guidance for design principles,
  accessibility, settings, icons, layout, materials, typography, color, dark mode, motion, writing,
  sidebars, lists and tables, menus, search, sheets, data entry, feedback, loading, keyboard,
  pointer, and touch interaction.

Official Apple references used as design guidance:

- [Design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles)
- [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility)
- [Settings](https://developer.apple.com/design/human-interface-guidelines/settings)
- [Icons](https://developer.apple.com/design/human-interface-guidelines/icons)
- [Layout](https://developer.apple.com/design/human-interface-guidelines/layout)
- [Dark Mode](https://developer.apple.com/design/human-interface-guidelines/dark-mode)
- [Motion](https://developer.apple.com/design/human-interface-guidelines/motion)

Apple HIG is used here as a quality and interaction reference for a responsive web application. The
plan does not copy Apple assets, claim the site is a native Apple app, or require proprietary SF Pro
or SF Symbols licensing.

### 3.1 Audit limitations

- The automated browser bridge could not attach because its Chrome extension was unavailable. The
  project-local Playwright installation was used instead.
- The repository does not currently include an axe accessibility runner, so this is not a complete
  WCAG conformance audit. Keyboard/semantic/source checks and visual inspection identified issues;
  automated axe, screen-reader, zoom, and contrast verification remain required work.
- Trial and Free E2E login failed in the existing tier test, so the full visual pass used the Pro
  tenant. Tier-specific route visibility remains unverified.
- This was a read-only product audit. Full Rust/frontend release gates were not run because no
  implementation was performed.

## 4. Severity model

| Severity | Meaning |
|---|---|
| **P0 — Trust/security blocker** | The UI can expose secrets, report a false success, or show invented business/security state. Hide or disable before release. |
| **P1 — Broken primary workflow** | A documented route, permission, state transition, or core responsive/accessibility path fails. |
| **P2 — Significant usability gap** | A task works but is hard to discover, understand, operate responsively, or recover from. |
| **P3 — Polish/consistency** | Does not block the task but weakens coherence or perceived quality. |

## 5. Critical findings

### 5.1 P0 — simulated functionality and false state

| ID | Surface | Evidence | User impact | Required disposition |
|---|---|---|---|---|
| TRUTH-01 | Couriers | `couriers-list.tsx` seeds Pathao, Steadfast, and RedX, reads credentials/rates from browser storage, and uses a timer as a connection probe. The credential modal stores form values in plaintext browser storage and reports a successful sandbox handshake. | Secrets are exposed to any script with origin access; “connected” does not mean authenticated. | Remove credential storage and simulated probe immediately. Keep the route unmounted until a tenant-scoped courier provider contract and encrypted server-side vault exist. |
| TRUTH-02 | Domains & TLS | `domains-view.tsx` creates a random verification string locally, persists the domain locally, and marks DNS/TLS verified after a timer. | A merchant can believe a public domain and certificate are active when no DNS or certificate operation occurred. | Bind to the existing platform domain service through a tenant-scoped admin bridge; poll real verification state; never promote pending to verified in the client. |
| TRUTH-03 | Support tickets | `help-center.tsx` ships invented tickets, replies, operational claims, and random ticket IDs. Submit/reply only changes React state. | Support requests disappear on refresh and merchants can mistake sample conversations for real support history. | Separate static help articles from support operations. Remove tickets until a persistent ticket API, authorization model, notifications, and service ownership exist. |
| TRUTH-04 | Billing | `entitlements-detail.tsx` combines real entitlement reads with sample invoices, fixed pricing, fixed payment-method data, a fabricated standing/renewal state, toast-only invoice download, and toast-only plan/payment actions. It defaults missing tier data to `pro`. | Merchants can see the wrong plan, money, renewal, payment, and invoice facts. | Split Entitlements from Billing. Render no billing fact without a server field. Add a tenant billing bridge over existing subscription/invoice data. Treat loading/error as unknown, never Pro. |
| TRUTH-05 | Password | `profile-password-section.tsx` uses a timer and success toast without calling the existing authenticated password-update route. It asserts hash details independent of runtime configuration. | A compromised password may remain active after the UI says it changed. | Call `POST /auth/user/{auth_provider}/update`; invalidate/query appropriately; remove implementation-specific hash claims. |
| TRUTH-06 | Sessions/devices | `profile-sessions-section.tsx` invents device, IP, and Dhaka location rows and persists them locally. Revoke only filters local state. The backend is intentionally stateless JWT and has no device-session store. | Security posture and revocation are fictitious. | For 1.0, show only the authenticated current token and real “sign out all tokens” behavior, or first build a server-side refresh/session registry with per-device revocation. Do not display invented devices. |
| TRUTH-07 | Store policies | Policy content and “published” state are seeded and saved only to browser storage; Preview only raises a toast. | Legal/store promises may appear saved or published when they are not. | Hide until the data model, merchant admin endpoints, storefront route, audit history, and ADR for public policy URLs are approved. |
| TRUTH-08 | COD rules | COD eligibility, amount, Dhaka restriction, and phone confirmation are local-only despite claiming checkout behavior. | Checkout will not enforce what the owner configured. | Reuse the existing offline payment provider, then add server-owned COD policy and enforcement at checkout. UI is a client of that contract only. |
| TRUTH-09 | Release notes | Hard-coded releases claim courier and domain capabilities that the current UI simulates. | Product documentation reinforces false capability claims. | Generate from a reviewed, versioned release manifest. Publish only capabilities whose acceptance gates passed. |
| TRUTH-10 | Completion matrix | `VENDRYX_SCREEN_MATRIX.md` marks all audited additions PASS, including claims of real session revocation, encrypted credentials, backend courier contracts, live DNS, TLS, billing history, and support tickets. No required viewport/a11y evidence is linked. | Planning and release decisions are made from inaccurate status. | Replace PASS with evidence states: `not-started`, `mock-only`, `partial`, `verified`. Require artifact links and a verification SHA. |

Six production files across five route areas contain `localStorage` behavior: courier setup and its
credential modal, domains, COD rules, profile sessions, and store policies. Browser storage is
acceptable for harmless UI preferences or drafts labeled as local. It is not acceptable as the
source of truth for secrets, business rules, publish state, security state, or operational status.

### 5.2 P1 — broken or misleading routes and API contracts

| ID | Finding | Evidence and impact |
|---|---|---|
| ROUTE-01 | Missing settings routes | `/settings/notifications`, `/settings/brand-kit`, and `/settings/bespoke-theme` are allow-listed/documented but render the not-found experience. |
| ROUTE-02 | Missing onboarding | `/onboarding` is documented and allow-listed but renders not found. |
| ROUTE-03 | Broken Social Orders | `/orders/social` resolves as an order ID and requests `/admin/orders/social`, producing a blank result. The documented filter contract is `GET /admin/orders?status=draft&is_social=true`. |
| ROUTE-04 | Profile path drift | The runtime route is `/settings/profile`; `/profile` is not found while the sitemap and screen matrix claim `/profile`. |
| ROUTE-05 | No merchant home | `/` redirects directly to `/orders`; there is no owner overview, action queue, setup state, or recovery surface. |
| API-01 | RBAC permission failure | Every sampled authenticated page requested `/admin/rbac/me/permissions` and received 404. Permission-aware controls cannot be trusted until this resolves. |
| API-02 | View configuration failure | Return/refund reasons, product types/tags, locations, and both API-key lists request `/admin/views/{entity}/configurations`, `/active`, and `/columns`; those requests return 404. |
| TIER-01 | Tier test failure | Existing Trial and Free analytics E2E cases could not log in. Entitlement visibility and route mounting are therefore not proven for those tiers. |
| DOC-01 | Navigation truth drift | `MERCHANT_WORKSPACE_ROUTES`, `MERCHANT_SETTINGS_ROUTES`, `SITEMAP.md`, the router, and the visible sidebar each describe a different product. |

The 2026-09-02 API audit artifact reports 407 Admin API specification endpoints, 217 mounted Admin
API endpoints, 169 matched endpoints, 48 Stheno extensions, and 238 missing/unbuilt endpoints. That
artifact is useful baseline evidence, not a current release assertion; it must be regenerated in CI.

### 5.3 P1/P2 — accessibility and responsive operation

- The settings sidebar is an `aside` but did not expose a navigation landmark during the runtime
  pass. It needs a named `nav` and an identifiable current destination.
- `ToggleSidebar` desktop buttons and the mobile close icon are icon-only controls without an
  explicit accessible name in the local source. Every icon-only action needs an accessible name and
  visible tooltip; critical actions also need adjacent text at normal desktop widths.
- Settings/store on a 390 px viewport visually clips or constrains the currencies table. Horizontal
  overflow may be contained inside the component, but the content is not comfortably operable.
- Payment-provider webhooks and action/status rows become cramped on narrow screens.
- The mobile breadcrumb collapses to an ellipsis, reducing orientation inside deep settings.
- Touch targets need a 44 × 44 CSS-pixel minimum for primary/icon interactions, with adequate
  spacing between destructive or adjacent actions.
- A complete WCAG 2.2 AA proof still needs automated axe checks, keyboard traversal, focus-order and
  focus-visibility checks, 200% zoom/reflow, contrast testing, and NVDA/VoiceOver passes.

### 5.4 P2/P3 — visual hierarchy and consistency

- The shell hard-codes a `Trial` store badge and `Vendryx Core · Live`, so even the global frame can
  lie about entitlement and system state.
- Analytics uses a sparkle icon rather than a familiar chart/report symbol. Sparkles imply AI or a
  novelty feature, not reporting.
- The settings list is long, low-context, text-only, and internally inconsistent with routes that
  exist elsewhere in the shell.
- Most document titles still use “Stheno” while the visible brand is Vendryx.
- Analytics defaults to USD, offers only USD/EUR/GBP, and renders a literal `$` for summary cards
  even after currency changes. Currency choices should come from store configuration and all output
  should use one formatter.
- “Live”, “Tracked”, and “AOV” badges in Analytics do not communicate a useful state. A status badge
  must name real freshness, coverage, or warning state—or be removed.
- Revenue/orders/AOV do not all have equally explicit loading, empty, failure, stale, and retry
  presentations.
- Dark appearance is visually coherent in the sampled screens, but it was added despite the current
  adaptation plan saying dark mode was not approved. It needs a recorded scope decision and full
  semantic-token/accessibility testing.

## 6. Current functional truth matrix

This matrix is the basis for triage. “API-backed” means the inspected primary data path uses a real
endpoint; it does not certify every state or action on the page.

| Surface | Current status | Notes |
|---|---|---|
| Orders, Products, Inventory, Customers, Promotions, Price Lists | **API-backed, audit further** | Core fork routes render and use SDK/API paths. Still require full parity, permissions, view config, empty/error, and tier tests. |
| Analytics | **Mostly API-backed** | Revenue, order count, top products, and AOV use report endpoints. Currency and state handling are incomplete. |
| Payment provider management | **Mostly API-backed** | Provider list, verification, credentials, and regions use server paths. COD child configuration is local-only. |
| MFA | **API-backed** | Start, verify, disable, and recovery-code hooks have matching auth flows. |
| Entitlements | **API-backed read** | `/admin/entitlements` and feature flags exist. Billing facts and plan actions around them are not real. |
| Profile identity | **Partial** | User data is present; password and sessions sections are not connected honestly. |
| Store settings | **Partial** | Core store fields/currencies/locales use established admin behavior; Policies are local-only. |
| Domains | **Mock-only in merchant UI** | Platform domain service exists, but the merchant screen does not use it. |
| Couriers | **Mock-only** | No merchant courier implementation was found; only operator oversight tests/routes exist. |
| Support tickets | **Mock-only** | Operator impersonation/support operations exist, but no merchant ticket persistence/API was found. |
| Release notes/help articles | **Static content** | Static content is valid only after claims are reviewed and clearly identified as documentation. |
| Billing/invoices/payment method | **Mixed, unsafe** | Entitlements are real; displayed invoices, standing, method, renewal, and actions are not. |
| Notifications, Brand Kit, Bespoke Theme | **Missing route** | Documented/allow-listed but unmounted. |
| Onboarding | **Missing route** | Documented/allow-listed but unmounted. |
| Social Orders | **Broken route** | Static route collision/fall-through to order detail. |
| RBAC current permissions and view configurations | **Missing API behavior** | Repeated runtime 404s across otherwise rendered screens. |

## 7. Apple-inspired product principles for this web admin

The premium feeling should come from clarity, restraint, direct manipulation, and trustworthy
feedback—not more gradients, blur, animation, or icon decoration.

### 7.1 The six governing rules

1. **Truth before beauty.** Unknown is displayed as unknown. Pending remains pending. A success
   state requires a confirmed server result.
2. **Content leads controls.** Put the owner’s object and current state first; actions are secondary
   and remain close to what they affect.
3. **Progressive disclosure.** Show common settings and health at the category level; reveal
   credentials, DNS records, advanced rules, and irreversible actions only when requested.
4. **Familiarity over novelty.** Use conventional web navigation, labels, standard shortcuts, and
   recognizable icons. Never require icon interpretation alone.
5. **Feedback includes recovery.** Every mutation defines in-progress, confirmed, failed, retry,
   and reconciliation behavior before the happy-state design is accepted.
6. **Accessibility is the component contract.** Semantics, keyboard behavior, target size, focus,
   contrast, text scaling, reduced motion, and screen-reader output are part of “done.”

### 7.2 Platform/input pass

| Context | Required behavior |
|---|---|
| Desktop pointer | Persistent sidebar, hover as enhancement only, visible tooltips, precise table controls, command search, multi-select where useful. |
| Desktop keyboard | Logical tab order, skip link, visible focus, Enter/Space activation, Escape closes sheets/dialogs, arrow-key menus/listboxes, documented shortcuts that do not conflict with browser/OS. |
| Tablet/touch | Collapsible split navigation, 44 px targets, no hover dependency, sheets sized for touch, sticky primary action where long forms require it. |
| Mobile | Push-style navigation from Settings hub to category to detail, visible back title rather than only ellipsis, cards/definition lists instead of squeezed tables, bottom-safe action area. |
| Assistive technology | Correct landmarks/headings, current-page semantics, accessible names/descriptions, announced async results, errors associated with fields, chart summaries/data tables. |

## 8. Settings information architecture

### 8.1 Replace the redirect with a real Settings hub

`/settings` should not immediately redirect to Store. It should be a searchable overview with:

- Recently used settings (locally remembered navigation is acceptable because it is a UI
  preference, not business state).
- Setup/attention cards sourced from real backend status.
- Category rows with an icon, clear label, one-line explanation, and real status/accessory.
- Search over settings names and synonyms, including “password”, “API key”, “courier”, “domain”,
  “invoice”, “tax”, and “language”.
- No marketing copy and no fabricated completion percentages.

### 8.2 Proposed category model

Keep navigation to three conceptual layers:

```text
Settings hub
├── Storefront
│   ├── Store identity
│   ├── Brand and theme
│   ├── Domains
│   └── Languages and translations
├── Selling
│   ├── Payments and COD
│   ├── Regions and taxes
│   ├── Sales channels
│   └── Return and refund reasons
├── Fulfillment
│   ├── Locations
│   ├── Shipping configuration
│   └── Couriers
├── Catalog
│   ├── Product types and tags
│   └── Property labels
├── Team and security
│   ├── Users and roles
│   ├── Profile and password
│   ├── MFA
│   └── Sessions or global sign-out
├── Plan and usage
│   ├── Entitlements and quotas
│   └── Billing and invoices
└── Developer
    ├── Publishable keys
    ├── Secret keys
    └── Workflows
```

If a category has one child, navigate directly. If a capability is not mounted for the current
tenant, omit it from navigation and search; do not show a link that ends in 404. Preserve data and
explain unavailable existing configuration at the point of downgrade, per the entitlement rules.

### 8.3 Settings row anatomy

Every settings destination uses the same anatomy:

1. Leading icon in the standard neutral color.
2. Concise noun label (“Domains”, not “Manage your domains”).
3. One-line summary in secondary text.
4. Optional server-sourced state (“2 users”, “MFA on”, “Verification pending”).
5. Trailing chevron for navigation, switch only for an immediate reversible boolean, or button for
   an explicit action.

Do not put destructive actions, credential fields, or multi-step setup directly in the hub.

### 8.4 Icon system

Continue with the already-declared icon family unless licensing or bundle constraints require a
change. Do not mix families. Do not import SF Symbols for the web product.

| Destination | Semantic icon concept | Notes |
|---|---|---|
| Store identity | storefront/building | Use one metaphor everywhere. |
| Brand/theme | palette/paintbrush | Do not use sparkle unless the feature is genuinely generative. |
| Domains | globe/link | Pair pending/verified/error with text, not icon/color alone. |
| Payments | card/wallet | COD can use cash/receipt as a labeled sub-item. |
| Taxes/regions | map/globe | Keep regions and taxes distinguishable in label and tooltip. |
| Fulfillment/couriers | truck/box | Connection status is a separate badge with text. |
| Catalog metadata | tag/list | Product types, tags, and property labels need related but distinct symbols. |
| Team | users | Roles/permissions use shield/key only when security semantics apply. |
| Security | lock/shield | MFA, password, and sessions must not share identical unlabeled icons. |
| Billing | receipt | Entitlements/usage uses gauge/chart; invoices use receipt/document. |
| Developer | terminal/key/workflow | Secret keys require a stronger warning treatment than publishable keys. |
| Analytics | chart | Replace sparkle with a chart symbol. |

Icon rules:

- Standardize optical sizes at 16, 20, and 24 px; use 20 px for settings rows.
- Maintain at least 3:1 non-text contrast where the icon communicates state.
- Add `aria-hidden="true"` when the adjacent label supplies meaning.
- Icon-only buttons require an accessible name and visible tooltip.
- Never rely on red/green or shape alone for connection, billing, or verification state.
- Do not place every icon in a colorful tile. Reserve color for semantic status and selection.

## 9. Visual and interaction system

### 9.1 Hierarchy and density

- Use one clear page title, a short optional explanation, then task content.
- Keep the primary action in a consistent top-right location on desktop; on mobile keep it visible
  without covering content.
- Use compact tables for repeated comparable data. Use cards or definition lists for mobile detail.
- Use section dividers and whitespace before elevation. Shadows should communicate layering, not
  decorate every container.
- Favor an 8 px spacing rhythm with a smaller 4 px substep for icon/text alignment.
- Keep body copy at a comfortable web size and line height; do not shrink settings navigation to fit
  more links.

### 9.2 Color, materials, and dark appearance

- Move all screens to semantic tokens: canvas, surface, raised surface, separator, primary/secondary
  text, accent, success, warning, critical, focus, and disabled.
- Never hard-code a color that communicates state in one appearance only.
- Use translucency/blur only for a true layered surface such as a sticky toolbar or sheet. Provide an
  opaque fallback for reduced-transparency/high-contrast users.
- Verify light and dark independently. Dark mode is not an inverted palette and shadows cannot be
  the only layer separator.
- Persist theme only as a UI preference; support system, light, and dark rather than a two-state
  surprise toggle.

### 9.3 Motion

- Use motion to explain navigation, disclosure, or state change; avoid ambient motion in an
  operational dashboard.
- Keep most micro-transitions in a restrained 120–220 ms range as a product token, not a promise of
  Apple-native motion.
- Respect `prefers-reduced-motion`; replace spatial transitions with instant or opacity-only state
  change.
- Never use animation to delay confirmation of a completed server operation.

### 9.4 Forms and actions

- Save explicitly for credentials, billing, legal policies, tax, and fulfillment rules.
- Autosave only small, reversible preferences; show “Saving”, “Saved”, and “Couldn’t save” states.
- Validate on blur/submit without erasing input. Put the error next to the field and summarize at the
  top for long forms.
- Disable only when the reason is evident; otherwise allow the action and return an actionable
  explanation.
- Use confirmation for destructive/irreversible work. Prefer Undo for reversible list removal.
- Preserve drafts only when labeled (“Saved on this device”) and never include secrets.

## 10. Required state contract for every screen

No route can be marked verified until the relevant states below are designed and tested.

| State | Required presentation |
|---|---|
| Initial loading | Stable skeleton matching final layout; no false default values. |
| Loaded | Server-sourced data and freshness where relevant. |
| Empty | Explain what is absent, why it matters, and the allowed next action. |
| Validation error | Field association, clear correction, preserved input. |
| Request failure | Human explanation, retry, and support/reference ID when available. |
| Pending/async | Real server status, last check, safe polling/backoff, manual retry. |
| Partial/stale | Mark stale/partial explicitly; never silently merge samples with live data. |
| Forbidden | Hide controls the role cannot use; direct access returns the API’s correct safe response. |
| Not entitled | Backend route remains unmounted/404; navigation/search omit the feature. |
| Offline | Preserve safe drafts where applicable; block false success; explain reconnection. |
| Success | Confirm only after server acknowledgement; update query cache from the returned object. |
| Destructive success | Remove/update the object, announce result, offer Undo if the backend supports it. |

## 11. Functional architecture plan

### 11.1 One route and capability truth registry

Create one typed merchant-route registry that is consumed by:

```text
route definition
  ├── sidebar and Settings hub
  ├── command/search palette
  ├── breadcrumbs and document title
  ├── feature/entitlement visibility
  ├── required permission metadata
  └── automated route smoke tests
```

The backend remains authoritative. Feature-gated route groups continue to be unmounted per
`ENTITLEMENTS.md`; the client registry controls discoverability, not authorization. A route cannot
be labeled available solely because it appears in a JavaScript set.

### 11.2 Client/server boundary

```text
React route
  → typed query or mutation hook
  → tenant-scoped /admin endpoint
  → existing module/service/workflow
  → tenant-owned database/provider
  → event/audit where required
  → authoritative response
  → cache update and accessible feedback
```

Do not call operator `/platform/*` routes from the merchant browser. For existing platform-owned
data such as domains, subscriptions, and invoices, add a narrowly scoped authenticated `/admin/*`
bridge that derives the tenant from request context. Do not accept tenant IDs from the merchant
body/path where ownership can be implied.

All Admin API changes must be additive because the Medusa v2-compatible Admin surface is frozen.

### 11.3 Capability-specific backend decisions

| Capability | Reuse | Gap to implement |
|---|---|---|
| Password | Existing authenticated update flow | Wire UI mutation, validation, cache/session behavior, tests. |
| MFA | Existing auth MFA routes/hooks | Accessibility, recovery UX, error/retry tests. |
| Entitlements | Existing `/admin/entitlements` and feature flags | Split from billing; remove Pro fallback. |
| Billing | Existing platform subscriptions/invoices/billing attempts | Tenant-admin read bridge, real plan catalog/pricing, invoice download artifact, approved upgrade/payment workflow. |
| Domains | Existing platform tenant domain and verification service | Tenant-admin bridge, provider/ACME status, safe polling, DNS copy affordances, entitlement mounting. |
| Payment providers | Existing admin provider list/verify/credentials/regions | Remove hard-coded callback fallbacks; keep status tied to remote health. |
| COD | Existing offline payment provider | Persistent COD policy, checkout enforcement, validation, audit/event, admin endpoints. |
| Couriers | No merchant implementation found | Complete the planned courier provider/module/workflow contracts before mounting UI; encrypted secrets and real health probes. |
| Sessions | Existing stateless JWT current-session/global invalidation | Choose honest current-token/global sign-out UI, or design a persistent refresh/device session registry. |
| Support tickets | No merchant ticket domain/API found | Decide external support-system integration vs internal module, persistence, ownership, attachments, notifications, SLA copy source. |
| Policies | No persistent merchant/public policy contract found | ADR for public URLs, data model/versioning, admin CRUD, publish workflow, storefront rendering, audit log. |
| Release notes | Version exists at build time | Reviewed release manifest/changelog generator and capability-gate link. |
| Notifications | Sitemap contract only | Build/verify VAPID subscription API and preferences before mounting. |
| Brand/theme | Platform theme catalog/assignment exists for operators | Define safe merchant choices and tenant-admin bridge; do not expose rollout/operator controls. |

## 12. Phased implementation plan

### Phase 0 — Truth freeze and scope decision

**Goal:** prevent more false completion while preserving usable core commerce work.

1. Record the PRD/ADR decision for the redesign scope and dark appearance.
2. Replace the screen matrix’s binary PASS with `verified`, `partial`, `mock-only`, `blocked`, and
   `not-started`; attach route, API, viewport, accessibility, and verification evidence.
3. Feature-hide or place a clearly non-interactive development guard on all P0 mock-only surfaces.
4. Remove release-note/help claims about capabilities not verified in production.
5. Add a CI check that blocks production route code from using browser storage for secrets,
   business rules, publish state, billing state, or security sessions.

**Exit:** no production UI claims an operational or security success that cannot be traced to an
authoritative response.

### Phase 1 — Navigation, route truth, and contract baseline

1. Consolidate route/nav/search/breadcrumb/title metadata into the typed registry.
2. Correct `/profile` vs `/settings/profile` with one canonical route and an intentional redirect if
   compatibility requires it.
3. Mount `/orders/social` before dynamic order detail and implement it as the documented filtered
   orders view.
4. Decide and implement or remove `/onboarding`, notifications, brand kit, and bespoke theme from
   all registries/docs together.
5. Fix `/admin/rbac/me/permissions` and missing view-configuration endpoints, or remove the client
   requests only if the fork truly does not require those features and parity evidence supports it.
6. Regenerate the API audit and make missing/matched counts a dated artifact.
7. Fix Trial/Free seeded auth and tier-route tests.

**Exit:** every visible link resolves, every registered route has its required API contract, and no
sampled page produces an unexplained 404/5xx.

### Phase 2 — P0 functional replacements

Implement in this order because later settings design must present real states:

1. Password mutation and honest token/session controls.
2. Billing read model and invoices; remove all samples and fallback Pro state.
3. Domain read/write/verification bridge and real status polling.
4. COD persistent policy plus checkout enforcement.
5. Courier backend/provider work; mount UI only after contract and encrypted credential storage.
6. Policy data/publish workflow and storefront route, after ADR approval.
7. Support-ticket persistence/integration, or remove ticket UI and retain reviewed help content.
8. Release manifest and capability verification link.

**Exit:** refresh, a second browser, and an authorized API read all agree with every displayed
mutation.

### Phase 3 — Settings redesign and component system

1. Build the Settings hub and category structure.
2. Add search with synonyms and entitlement-aware results.
3. Introduce `SettingsRow`, `SettingsSection`, `StatusAccessory`, `FormFooter`, `AsyncState`,
   `ResponsiveDataList`, and accessible icon-button wrappers using existing dependencies.
4. Apply the icon taxonomy and remove ambiguous/decorative status icons.
5. Implement desktop split navigation, tablet collapse, and mobile push navigation.
6. Replace cramped mobile tables with responsive rows/cards while retaining desktop comparison.
7. Bind tier and shell service status; use no fallback claim.

**Exit:** common settings are findable by category or search, all destinations explain purpose/state,
and all controls are usable by keyboard, pointer, and touch.

### Phase 4 — Owner home and guided operations

1. Replace the `/` redirect with a store-owner home focused on actionable exceptions—not decorative
   vanity metrics.
2. Source cards from real data: orders needing action, low stock, failed payments/fulfillment,
   setup blockers, quota pressure, and integration failures.
3. Build onboarding only from persisted setup state; make every step resumable and independently
   verifiable.
4. Deep-link every alert to the exact recovery task.
5. Never display a count, time, percentage, health, or trend unless sourced and labeled.

**Exit:** an owner can identify and enter the day’s highest-priority task without navigating a
catalog of settings.

### Phase 5 — Accessibility, responsive, dark, and localization hardening

1. Add axe-based route checks and fail on serious/critical violations.
2. Perform manual keyboard and screen-reader checks on every layout/component pattern.
3. Verify reflow at 200% and viewports 390, 768, 1024, and 1440 CSS px.
4. Validate touch targets, focus rings, text scaling, reduced motion, reduced transparency, high
   contrast, light, dark, and system appearance.
5. Add accessible chart summaries and an equivalent data table/download.
6. Remove string concatenation and hard-coded currency symbols; use store locale/currency and
   message catalogs.
7. Align document titles, labels, and help text with Vendryx naming.

**Exit:** WCAG 2.2 AA evidence is attached per pattern and all supported viewports/appearances pass.

### Phase 6 — Verification and release evidence

1. Run route smoke tests for every nav/search destination and each entitlement tier.
2. Run mutation round trips in two browser contexts and verify server persistence.
3. Test loading, empty, error, pending, retry, stale, permission, entitlement, offline, success, and
   destructive flows where applicable.
4. Run all frontend and affected Rust gates.
5. Capture desktop/tablet/mobile light/dark artifacts only after automated gates pass.
6. Update the screen matrix with exact commit SHA, route, API fixture/environment, test result, and
   artifact links.

## 13. Implementation backlog

Sizes are relative complexity, not calendar commitments: **S** local/contained, **M** cross-layer,
**L** new workflow or service boundary.

| Ticket | Size | Outcome | Depends on | Acceptance summary |
|---|---:|---|---|---|
| ADM-000 | S | Record PRD/ADR scope and dark-mode decision | Owner/product decision | PRD, decision record, implementation plan, and route matrix agree. |
| ADM-001 | M | Replace completion matrix with evidence ledger | ADM-000 | No PASS without route/API/a11y/viewport evidence and SHA. |
| ADM-002 | S | Remove/hide simulated success paths | None | P0 surfaces cannot submit or claim success in production. |
| ADM-010 | M | Typed route/capability registry | ADM-000 | Router, nav, settings search, breadcrumbs, titles, and smoke tests share metadata. |
| ADM-011 | M | Repair route drift/collisions | ADM-010 | Social Orders, profile, and all visible links resolve intentionally. |
| ADM-012 | M | Close RBAC/view-configuration 404s | API audit | No sampled route emits these missing-contract errors. |
| ADM-013 | M | Trial/Free auth and route matrix | Seed/auth owners | Trial, Free, Pro mount only declared routes and tests log in reliably. |
| ADM-020 | S | Real password update | Existing auth route | Correct password changes; wrong old password fails; success is server-confirmed. |
| ADM-021 | M/L | Honest sessions design | Security decision | Either current/global token controls only, or real device registry/revocation—never invented rows. |
| ADM-022 | M | Billing/entitlement split | Platform invoice service | No sample money/dates; real invoices and plan data; error never defaults to Pro. |
| ADM-023 | M | Merchant domain bridge | Platform domain service | Add/remove/primary/verification are tenant-scoped; real pending/verified/failed state. |
| ADM-024 | M | Persistent COD policy | Offline provider | Admin rules persist server-side and checkout enforces identical policy. |
| ADM-025 | L | Merchant courier integration | Planned courier backend | Encrypted credentials, real probe, rate/zone persistence, webhook/auth tests. |
| ADM-026 | L | Policy authoring/publishing | Public-route ADR | Versioned content persists, preview renders real draft, publish updates storefront. |
| ADM-027 | L | Support tickets or external integration | Ownership decision | Ticket/reply persists, authorization enforced, notifications and truthful SLA source. |
| ADM-028 | S/M | Release manifest | Release process | UI reads reviewed releases; capabilities link to verification evidence. |
| ADM-030 | M | Settings hub/category/search | ADM-010 | Common tasks discoverable; absent/gated routes omitted; current state explained. |
| ADM-031 | M | Settings/icon/accessibility primitives | Design tokens | Shared rows, async states, accessible icon buttons, form footer, responsive lists. |
| ADM-032 | M | Responsive settings navigation | ADM-030/031 | Desktop/tablet/mobile hierarchy preserves context and back navigation. |
| ADM-033 | M | Semantic themes | ADM-000/031 | System/light/dark and accessibility preferences pass token/contrast checks. |
| ADM-034 | S/M | Analytics currency/state repair | Store settings API | Store currencies only, one formatter, honest badges, complete async states. |
| ADM-040 | L | Owner action home | Real aggregate queries | Every card is sourced, actionable, permission-aware, and has empty/error state. |
| ADM-041 | L | Persisted onboarding | ADM-040 + workflow APIs | Resumable steps reflect actual setup, no fake completion. |
| ADM-050 | M | Automated accessibility suite | ADM-031 | axe + keyboard checks cover shared patterns and critical routes. |
| ADM-051 | M | Route/API truth smoke suite | ADM-010/012/013 | Every tier/nav route has expected status; no unexpected 404/5xx/console errors. |
| ADM-052 | M | Visual evidence matrix | ADM-050/051 | 390/768/1024/1440, light/dark/system, reduced motion artifacts linked to SHA. |

## 14. Acceptance criteria by high-risk workflow

### 14.1 Courier credential setup

- Secret fields are write-only after save and never returned plaintext.
- Browser storage, logs, analytics, URLs, and error messages never contain secrets.
- Provider mutation is idempotent where applicable and audited.
- “Connected” requires a successful server-side provider probe with timestamp and safe reason on
  failure.
- Timeout is `unknown`, not success or hard failure; retry is safe.
- Rate zones persist server-side and fulfillment quotes use the same source.

### 14.2 Domain verification

- Domain normalization and ownership validation happen server-side.
- Verification token is issued and stored server-side, stable until rotated.
- UI can copy exact DNS records and reports last checked time.
- Polling uses backoff and stops in terminal state/background.
- TLS shows `not requested`, `provisioning`, `active`, `renewal warning`, or `failed` from provider
  state; the client never promotes it.
- Cross-tenant domain reads and mutations return 404 without existence disclosure.

### 14.3 Billing

- Plan, price, currency, billing interval, status, term/trial dates, quota usage, invoices, tax, and
  payment method all trace to named response fields.
- No invoice download success before a signed/authorized artifact is returned.
- Upgrade/downgrade explains effective date, proration, data retention, and confirmation.
- Money stays decimal/minor-unit correct end to end; no floating point.
- The merchant cannot call operator-only credit/retry/tier mutation routes.

### 14.4 Account security

- Password mutation requires the old password and current authenticated identity.
- Error copy does not expose account/provider internals.
- Global sign-out invalidates existing tokens and returns the user to login.
- Per-device session UI ships only with a real persistent session model.
- MFA setup/recovery codes are operable with keyboard/screen reader and cannot be redisplayed after
  their allowed disclosure window.

### 14.5 Support and policies

- Ticket IDs, actors, timestamps, status, and messages persist and are tenant-owned.
- Any promised response hours/SLA come from a configured, named support policy.
- Policy drafts and published versions are distinct, timestamped, and attributable.
- Public preview uses real draft content with authorization; published URLs are core-owned and
  covered by the sitemap/ADR.

## 15. Verification gates

### 15.1 Frontend gates for each admin change

```powershell
npm.cmd --prefix apps/admin run typecheck
npm.cmd --prefix apps/admin run test
npm.cmd --prefix apps/admin run build
npm.cmd --prefix apps/admin run test:e2e
```

Add these targeted gates:

- Route-link smoke test for every visible nav/search link and every tier.
- Browser test that fails on unexpected `/admin/*` 404/5xx or uncaught console errors.
- Mutation persistence test: change → reload → second context → API read.
- Production-bundle test forbidding known mock storage keys and hard-coded sample records.
- axe scan and keyboard tests for shared shell, Settings hub, forms, dialog/sheet, table/list, and
  charts.
- Snapshot or structural check for accessible name on every icon-only button.

### 15.2 Backend gates for affected Rust work

Run the full project gates in `AGENTS.md`, including formatting, clippy, workspace tests, dependency
policy, module isolation, SQL boundaries, shared-kernel, module contracts, entitlement boundaries,
schema drift, OpenAPI drift, and documentation sync. New endpoints additionally require:

- authorization matrix: role × ownership × entitlement;
- filter/sort/pagination tests for lists;
- migration apply and schema drift checks;
- parity fixture or written additive-extension treatment for Admin API changes;
- idempotency/concurrency/ambiguity tests for money, provider, inventory, or external mutation;
- no customer/tenant-owned lookup by raw ID without ownership enforcement.

### 15.3 Evidence required to mark a screen `verified`

1. Exact route and commit SHA.
2. Named API contracts and whether data is fixture/staging/production-like.
3. Happy, loading, empty, error, permission, entitlement, and recovery checks.
4. Desktop/tablet/mobile artifacts.
5. Light/dark/system and reduced-motion evidence when supported.
6. Keyboard and automated accessibility result; screen-reader result for new patterns.
7. No unexpected console errors or network 404/5xx.
8. Mutation persistence proof for every write action.
9. Product/content approval for legal, billing, support, and release claims.

## 16. Audit run results

### Browser/runtime findings

- Pro tenant login and the 35-route authenticated walk succeeded as a harness operation.
- Core commerce and most sampled settings routes rendered.
- `/profile`, `/onboarding`, `/settings/notifications`, and `/settings/brand-kit` rendered not found.
- `/orders/social` fell through to an order-detail API request and produced a blank screen.
- Every sampled route produced a 404 for `/admin/rbac/me/permissions`.
- Several list screens produced additional 404s for view-configuration endpoints.
- Mobile Settings/Store and Payment Providers showed table/long-value compression issues.

### Existing filtered E2E run

Command:

```powershell
npm.cmd run test:e2e -- --grep "analytics|window control|chart survives"
```

Result: **4 passed, 2 failed (14.2s)**. The analytics browser tests and Pro-tier analytics clamp
passed. Trial and Free tier cases failed during login.

### Gates not claimed

No full typecheck, unit suite, build, Rust workspace gates, axe suite, or screen-reader suite is
claimed by this audit. Those are implementation/release gates, not evidence that can be inferred
from source inspection.

## 17. Definition of done

The merchant admin is ready to call functional, premium, and easy to use only when:

- Every visible fact traces to backend data or a named reviewed content source.
- Every visible action either completes a real workflow or is clearly unavailable; no simulated
  success remains.
- No secret or business/security source of truth is stored in the browser.
- The route registry, router, navigation, sitemap, entitlement matrix, and screen evidence agree.
- Settings support category browsing, search, progressive disclosure, clear state, and mobile
  navigation without losing context.
- Icons are consistent, labeled, accessible, and semantically recognizable.
- All shared patterns meet WCAG 2.2 AA evidence requirements across keyboard, screen reader, zoom,
  contrast, and supported appearances.
- Every asynchronous and destructive workflow has honest loading, pending, failure, retry,
  reconciliation, confirmation, and recovery behavior.
- Trial, Free, and Pro are verified separately; hidden/gated routes do not leak through direct
  navigation.
- Full frontend, backend, API, documentation, and evidence gates pass on the same commit.

The desired premium quality is the result of this consistency: fewer claims, clearer hierarchy,
real status, predictable controls, accessible detail, and calm recovery when something fails.
