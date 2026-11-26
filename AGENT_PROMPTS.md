# 🤖 The Stheno Agent Prompting Guide

This repository (Stheno) has extremely strict architectural boundaries enforced by CI. If you tell an agent to "build a feature" or "make a change" without using the exact architectural terminology, the agent might guess the wrong pattern, and the CI will fail the build.

This guide translates **what you want to do** into **what you should tell the agent**. 

---

## 🎨 1. Frontend, Theming, & UI

Stheno uses a **Unified Runtime** (`frontend-default`). We never fork the frontend for new stores.

> **New storefront designs (ADR-038):** ask for a theme folder, not a frontend. "Build this design as `themes/<theme-id>/` following docs/THEME_AUTHORING.md: templates plus `theme.json`, every colour, font, radius, spacing value and piece of copy as a token, no data or figures in the theme, core partials for checkout and sign-in." Work items are on the board in docs/STOREFRONT_THEME_ENGINE_PLAN.md. The rows below apply to the Next.js storefront until its cut-over.

| If you want to... | Tell the agent to... | Why? |
| :--- | :--- | :--- |
| **Change colors/fonts/corners across the whole site** | *"Update the **Design Tokens** in the core branding configuration."* | Instructs the agent to update CSS Custom Properties injected at runtime, not hardcoded CSS. |
| **Make a specific component (like the Hero) look totally different** | *"Create a **Visual Variant Modifier** for the Hero component using data attributes."* | The agent will know to use CSS Grid/Flexbox tied to a `data-*` attribute on the existing component, rather than creating a duplicate React component. |
| **Add a completely new brand/theme aesthetic** | *"Create a new **Theme Variant** with custom Design Tokens and Variant Modifiers."* | Ensures the agent scales up the variant system across the single unified codebase. |

---

## 🧱 2. Backend Modules & Domain Logic

Stheno separates logic into completely isolated modules (`crates/modules/`). They are not allowed to depend on each other.

| If you want to... | Tell the agent to... | Why? |
| :--- | :--- | :--- |
| **Implement a new feature or domain** | *"Walk the **Ladder (Rung 1-7)** before writing new code. If needed, build a new isolated **Module**."* | Forces the agent to check for existing standard library or native Postgres solutions before adding bloat (as required by `AGENTS.md §1`). |
| **Read data from two different modules** | *"Use the **Query Engine** (or **Links**) to join the data across schemas."* | Raw SQL is banned from crossing schema boundaries. The query engine is the only legal way to join cross-module data. |
| **Make modules react to each other** | *"Emit a local event and handle it in an **Event Subscriber** (`crates/subscribers/`)."* | Modules cannot call each other directly. The subscriber handles the reaction. |

---

## 🔄 3. Workflows & State (Mutations)

Whenever you are writing data across multiple modules (like placing an order that touches Cart, Inventory, and Payments), it must be a **Workflow**.

| If you want to... | Tell the agent to... | Why? |
| :--- | :--- | :--- |
| **Execute a complex action across domains** | *"Create a **Workflow** in `crates/flows/` with **Idempotent Steps**."* | Workflows are strictly isolated from the modules. Every step must be safely retry-able. |
| **Handle failure in a workflow** | *"Ensure every state-mutating step has a **Compensation**."* | If a workflow crashes halfway, the engine runs the "compensations" backwards to undo the work (e.g., releasing reserved inventory). |
| **Call an external API (like Stripe/Payments)** | *"Wrap the API call in an **Idempotent Call**."* | Prevents double-charging customers if the network times out and the workflow retries. |

---

## ⚡ 4. Caching & CDN

Stheno is a multi-tenant platform. Caching must be handled carefully so Store A doesn't accidentally see Store B's data.

| If you want to... | Tell the agent to... | Why? |
| :--- | :--- | :--- |
| **Cache data in the backend** | *"Use `stheno-caching` and ensure the cache key is **Partitioned by Tenant/Host**."* | A strict requirement (TRD §1.2). If you don't mention the tenant boundary, the agent might create a global cache bug. |
| **Improve frontend speed** | *"Rely on **ISR (Incremental Static Regeneration)** and the **CDN** in front of the origin."* | We do not build edge-compute runtimes. The backend is centralized; we rely on the CDN to cache the rendered ISR pages. |
| **Manage external state/locks** | *"Use the configured **Redis Provider** for caching and session pressure."* | Redis is an optional dependency for scaling, while Postgres is the default lock manager. |

---

## 🚀 5. Deployment & Infrastructure

Stheno is designed as a majestic monolith with a single binary deployment.

| If you want to... | Tell the agent to... | Why? |
| :--- | :--- | :--- |
| **Deploy a background worker** | *"Start the server with the `--role=worker` flag."* | We do not have a separate codebase for workers. The exact same binary runs the API (`--role=api`) and the background tasks (`--role=worker`). |
| **Handle database connections** | *"Rely on **PgBouncer** in transaction mode."* | The architecture expects PgBouncer to manage pooling, and the code uses explicit `&mut Conn<'_>` to track transactions. |
| **Add a new environment/deployment** | *"Remember that **One Deploy Serves Everyone**. We do not spin up separate servers per merchant."* | Prevents the agent from trying to write Terraform/Docker scripts to spin up 50 isolated databases and servers. |

---

## 🛡️ 6. Security & APIs

| If you want to... | Tell the agent to... | Why? |
| :--- | :--- | :--- |
| **Fetch a customer's specific data** | *"Use the **Owned Resource** extractor (`Owned<T, Scope>`)."* | Looking up resources by raw ID is banned to prevent IDOR (Insecure Direct Object Reference) vulnerabilities. |
| **Restrict a feature to higher paying tiers** | *"Gating must be done at the **Router Level**, never inside the domain logic."* | The domain logic cannot know about pricing tiers. If a store isn't entitled to a feature, the route simply isn't mounted (returns 404, not 403). |
| **Accept data from a user request** | *"Create a strictly validated **DTO (Data Transfer Object)**. Never deserialize directly into an Entity."* | Mass-assignment vulnerabilities are prevented by strictly separating the database Entity from the Request DTO. |

---

> [!TIP]
> **Golden Rule of Prompting Stheno Agents:**
> *Always tell the agent **WHAT** architectural pattern to use. The more specific the noun (e.g. "Visual Variant Modifier", "Workflow Compensation", "Owned Resource"), the more perfectly the agent will write the code.*
