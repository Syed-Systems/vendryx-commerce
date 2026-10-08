# Vendryx Brand Guidelines & Design System

This is the specification for the **Vendryx** brand identity, typography, color tokens, asset hierarchy,
and usage rules across all customer-facing surfaces, including the public marketing site, merchant dashboard,
and operator platform.

---

## 1. Brand Identity & Naming Rules

### 1.1 Product Name
- **Primary public name:** **Vendryx**
- **Full legal / attribution name:** **Vendryx by Syed Systems**
- **Pronunciation:** *VEN-driks*
- **Capitalization:** Always Titlecase `Vendryx`. Never `VENDRYX` in running prose; never `vendryx` except in URLs/domains.

### 1.2 Naming Boundary
In accordance with [ADR-028](DECISIONS.md#adr-028):
- **Customer-facing:** All marketing websites, onboarding, dashboard navigation, customer emails, favicons, open graph metadata, and user documentation use **Vendryx**.
- **Internal / Architectural:** Rust crates, Cargo packages, database schemas, environment variables (`STHENO_*`), and compatibility headers retain internal `stheno-*` identifiers to maintain compatibility and stability.

---

## 2. Core Color Tokens

All applications consume the `--vx-*` token layer. Pages and components must never define arbitrary hex values.

| Token | CSS Variable | Hex Value | Role & Usage |
|---|---|---|---|
| Canvas Background | `--vx-bg-canvas` | `#FCFBF9` | Primary page canvas behind cards and containers |
| Sidebar Background | `--vx-bg-sidebar` | `#F7F3ED` | Navigation sidebar background, quiet contrast |
| Surface Background | `--vx-bg-surface` | `#FFFFFF` | Raised surfaces, card bodies, dialogs, dropdowns |
| Subtle Background | `--vx-bg-subtle` | `#F6F5F3` | Table headers, secondary buttons, input backgrounds |
| Primary Text | `--vx-fg-primary` | `#18181B` | Headings, high-emphasis text, active labels |
| Secondary Text | `--vx-fg-secondary` | `#5F6368` | Body text, descriptions, table cells |
| Muted Text | `--vx-fg-muted` | `#7C818A` | Captions, timestamps, inactive placeholders |
| Border | `--vx-border` | `#E6E0D8` | Default card borders, dividers, subtle boundaries |
| Strong Border | `--vx-border-strong` | `#D8CFC4` | Input borders, active tabs, focused cards |
| Brand Display | `--vx-brand-600` | `#EA580C` | Brand accent, decorative highlights, large logos |
| Accessible Action | `--vx-brand-700` | `#C2410C` | Primary buttons, filled call-to-actions (WCAG AA) |
| Brand Hover | `--vx-brand-800` | `#9A3412` | Hover state for primary action buttons |
| Brand Tint | `--vx-brand-50` | `#FFF7ED` | Active nav items, highlighted badge backgrounds |
| Focus Ring | `--vx-focus` | `#C2410C` | 2px solid ring with 2px offset on all interactive items |

### 2.1 Semantic States
Semantic status colors remain semantic and must never be recolored orange:
- **Success:** `#16A34A` (Background: `#F0FDF4`, Border: `#BBF7D0`)
- **Warning:** `#D97706` (Background: `#FFFBEB`, Border: `#FDE68A`)
- **Danger / Destructive:** `#DC2626` (Background: `#FEF2F2`, Border: `#FECACA`)
- **Info:** `#2563EB` (Background: `#EFF6FF`, Border: `#BFDBFE`)

---

## 3. Typography

- **Primary Typeface:** `Geist Variable`, with fallback stack `Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`.
- **Monospace Stack:** `ui-monospace, "SF Mono", Menlo, Monaco, Consolas, monospace` (used for IDs, tokens, webhook secrets, currency codes).
- **Type Scale:**
  - Display / Hero: 36px–44px (Font-weight: 600)
  - Page Title: 28px–32px (Font-weight: 600)
  - Section Title / Modal Header: 20px–24px (Font-weight: 600)
  - Card Title / Subheading: 16px–18px (Font-weight: 500)
  - Body Text: 14px (Font-weight: 400, Line-height: 1.5)
  - Small / Utility: 12px–13px (Font-weight: 400 or 500)
- **Casing:** Sentence case for all buttons, navigation links, tabs, badges, and headers (e.g., "Store settings", "Save and verify", not "Store Settings" or "SAVE AND VERIFY").

---

## 4. Spacing, Shapes, and Elevation

- **Grid Base:** 8px base spacing unit (`8px`, `16px`, `24px`, `32px`, `48px`, `64px`).
- **Corner Radii:**
  - Interactive inputs and buttons: `10px` (`--vx-radius-input: 0.625rem`)
  - Cards and container panels: `14px` (`--vx-radius-card: 0.875rem`)
  - Status badges: Fully rounded pill (`9999px`)
- **Elevation & Shadows:**
  - Avoid heavy, dark drop-shadows. Use borders (`--vx-border`) for structural separation.
  - Floating dialogs and popovers use a warm subtle shadow: `0 10px 25px -5px rgba(24, 24, 27, 0.06), 0 8px 10px -6px rgba(24, 24, 27, 0.04)`.
- **Layout Measurements:**
  - Admin sidebar width: `248px` expanded; drawer on mobile (<1024px).
  - Top bar height: `64px`.
  - Max content width: `1440px` with `24px` desktop gutters.

---

## 5. Asset Directory & Standards

Source-of-truth brand assets reside in `brand/vendryx/`:

| File | Purpose | Variant |
|---|---|---|
| `vendryx-logo.png` | Primary wordmark with mark | Light backgrounds (`#FCFBF9`, `#FFFFFF`) |
| `vendryx-logo-dark.png` | Wordmark with mark | Dark / monochrome backgrounds |
| `vendryx-mark.png` | Square icon / mark alone | Favicon, app icons, compact headers |
| `favicon.ico` | Multi-size ICO (16x16, 32x32, 48x48) | Browser tab icon |
| `favicon.png` | 512x512 PNG | PWA, Apple touch icon, high-res web apps |
| `brand.json` | Machine-readable manifest | Single source of truth for sync scripts |

### Clear Space Rule
- Maintain clear space equal to the height of the "V" glyph on all sides of the logo.
- Minimum display height for wordmark: `28px`.
- Minimum display size for square mark: `20px × 20px`.
