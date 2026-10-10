# Vendryx Brand Guidelines & Design System Specification

**Company:** Syed Systems  
**Product:** Vendryx  
**Authority:** Single Source of Truth for the Vendryx corporate brand identity and design system  
**Governed by:** [AGENTS.md](../AGENTS.md) · [DECISIONS.md ADR-028](DECISIONS.md#adr-028) · [DECISIONS.md ADR-031](DECISIONS.md#adr-031)

---

## 1. Brand Identity & Core Naming Rules

### 1.1 Product Name & Attribution
- **Primary public name:** **Vendryx**
- **Full corporate name:** **Vendryx by Syed Systems**
- **Tagline:** "SELL MORE. WORRY LESS."
- **Pronunciation:** *VEN-driks*
- **Capitalization:** Always Titlecase `Vendryx` in prose. Never `VENDRYX` in running body text; never lowercase except in URLs/domains.

### 1.2 Naming Boundary ([ADR-028](DECISIONS.md#adr-028))
- **Customer-Facing Surfaces:** All marketing pages, onboarding flows, dashboard navigation, transaction emails, notifications, browser metadata, and public documentation strictly display **Vendryx**. Customer-facing surfaces never mention internal engine names or Medusa.
- **Internal / Engine Surfaces:** Rust crates, Cargo packages, database schemas, internal environment variables (`STHENO_*`), and Medusa v2 compatibility headers retain `stheno-*` identifiers for stability and wire parity.

### 1.3 Architectural Distinction: Corporate Brand vs Merchant Theming
1. **Core Corporate Brand (This Specification):** Governs all company-owned surfaces: the public marketing site (`apps/marketing`), the merchant admin dashboard (`apps/admin`), the platform control plane (`apps/platform`), and official corporate collateral.
2. **Dynamic Merchant Storefront Theming ([ADR-031](DECISIONS.md#adr-031)):** Merchant storefronts running on the unified Next.js runtime receive dynamic brand tokens injected on SSR into `<style id="stheno-brand-tokens">:root { ... }</style>`. Merchant customizations are scoped to their store and never alter corporate branding.

---

## 2. Core Corporate Design Philosophy: Simplicity is Elegance

- **STRICT RULE — NO UI GRADIENTS:** Gradients are exclusively reserved for the 3D logo mark. There are **NO** linear or radial gradients permitted in web layouts, cards, buttons, backgrounds, or UI chrome.
- **Flat, Clean, and Authoritative:** The interface relies on generous whitespace, crisp typography, subtle hairlines, and solid, high-contrast colors.
- **Apple Human Interface Guidelines (HIG) Baseline:** 8pt spatial grid, smooth squircle corner radii, accessible touch targets (minimum 44px on interactive elements), and fluid spring transitions.

---

## 3. Authoritative Corporate Color Palette

### 3.1 Primary Corporate Colors (Solid Colors Only)

| Token | CSS Variable | Hex Value | Role & Usage |
|---|---|---|---|
| Vendryx Indigo | `--brand-vendryx-primary` / `--vx-brand-primary` | `#4F46E5` | Primary brand accent, primary action buttons, active states |
| Vendryx Violet | `--brand-vendryx-violet` / `--vx-brand-violet` | `#5B21B6` | Deep accents, interactive hover states |
| Vendryx Bright Blue | `--brand-vendryx-bright` / `--vx-brand-bright` | `#3B82F6` | Informational highlights, secondary focus indicators |

### 3.2 Neutral & Surface Backgrounds

| Token | CSS Variable | Hex Value | Role & Usage |
|---|---|---|---|
| Corporate Navy | `--brand-corporate-navy` / `--vx-fg-primary` | `#0F172A` | Primary text and dark-mode UI. Rich, deep blue-black (never pure `#000000`) |
| Navy Light | `--brand-navy-light` / `--vx-fg-secondary` | `#1E293B` | Secondary headers, subheadings, dark borders |
| Muted Gray | `--vx-fg-muted` | `#64748B` | Captions, timestamps, disabled placeholders |
| Border Hairline | `--vx-border` | `#E2E8F0` | Default card borders, dividers, subtle boundaries |
| Strong Border | `--vx-border-strong` | `#CBD5E1` | Input borders, active tabs, focused cards |
| Subtle Background | `--vx-bg-subtle` | `#F8FAFC` | Table headers, secondary buttons, input backgrounds |
| Surface White | `--vx-bg-surface` | `#FFFFFF` | Raised surfaces, card bodies, dialogs, dropdowns |
| Canvas Background | `--vx-bg-canvas` | `#FCFBF9` | Warm default canvas background behind cards |

### 3.3 Semantic Status States (Universal)

Status colors remain semantic and must never be overridden by brand accents:
- **Success:** `#16A34A` (Background: `#F0FDF4`, Border: `#BBF7D0`) — Delivered orders, verified domains, active webhooks.
- **Warning:** `#D97706` (Background: `#FFFBEB`, Border: `#FDE68A`) — Low inventory, pending settlement, unverified courier.
- **Danger / Destructive:** `#DC2626` (Background: `#FEF2F2`, Border: `#FECACA`) — Failed charges, cancelled orders, revocations.
- **Info:** `#2563EB` (Background: `#EFF6FF`, Border: `#BFDBFE`) — Neutral informational notices, sync progress.

---

## 4. Typography

- **Primary Font Family:** `Plus Jakarta Sans`, with fallback `Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`.
- **Monospace Stack:** `ui-monospace, "SF Mono", Menlo, Monaco, Consolas, monospace` (used for Order IDs, transaction hashes, webhook secrets, currency amounts).
- **Type Scale & Weights:**
  - **Display / Hero:** 36px–44px (Font-weight: 700 or 800, tight tracking: -0.02em)
  - **Page Title:** 28px–32px (Font-weight: 700)
  - **Section Title / Modal Header:** 20px–24px (Font-weight: 600)
  - **Card Title / Subheading:** 16px–18px (Font-weight: 600)
  - **Body Text:** 14px–15px (Font-weight: 400 or 500, line-height: 1.55)
  - **Captions / Microcopy:** 12px–13px (Font-weight: 500)
  - **Taglines:** Uppercase with wide letter spacing (tracking: 0.15em–0.2em)

---

## 5. Spacing, Shapes, and Elevation

- **Spatial Grid:** 8px base spacing grid (`8px`, `16px`, `24px`, `32px`, `48px`, `64px`).
- **Corner Radii:**
  - Interactive inputs and standard buttons: `10px` (`--brand-radius-input: 10px`)
  - Cards and container panels: `16px` (`--brand-radius-card: 16px`)
  - Status badges and pill buttons: `9999px` (Full pill)
- **Shadows & Elevation:**
  - Flat and clean: Rely primarily on 1px borders (`--vx-border`) rather than muddy drop shadows.
  - Floating dialogs and popovers use crisp, subtle elevation:
    `box-shadow: 0 10px 25px -5px rgba(15, 23, 42, 0.08), 0 8px 10px -6px rgba(15, 23, 42, 0.04);`

---

## 6. Official Brand Assets (`brand/vendryx/`)

Source assets are maintained in [`brand/vendryx/`](../brand/vendryx/):

| Asset File | Format | Purpose |
|---|---|---|
| `vendryx-logo.png` | PNG (High-Res) | Primary corporate wordmark with folded-layer bag mark |
| `vendryx-logo-dark.png` | PNG (High-Res) | Wordmark for dark mode / contrasting backgrounds |
| `vendryx-mark.png` | PNG (Square) | App icon, favicons, compact navigation headers |
| `favicon.ico` | Multi-size ICO | Browser tab icon |
| `favicon.png` | 512×512 PNG | PWA manifest, Apple touch icon, high-density mobile |
| `brand.json` | JSON | Machine-readable manifest for asset synchronization |

---

## 7. Dynamic Merchant Storefront Theme Tokens ([ADR-031](DECISIONS.md#adr-031))

For multi-tenant merchant storefronts, tenant-specific variables are populated dynamically at runtime:

```css
/* Injected dynamically by storefront-core SSR */
<style id="stheno-brand-tokens">
:root {
  --brand-primary: var(--tenant-primary-color);
  --brand-surface: var(--tenant-surface-color);
  --brand-background: var(--tenant-background-color);
  --brand-text: var(--tenant-text-color);
  --brand-hairline: var(--tenant-border-color);
  --brand-font-family: var(--tenant-font-stack);
  --brand-radius-card: 16px;
  --brand-radius-button: 9999px;
  --brand-radius-input: 10px;
}
</style>
```

Merchant tokens dictate the store aesthetic without altering underlying DOM structures, PCI SAQ-A checkout compliance, or corporate identity.
