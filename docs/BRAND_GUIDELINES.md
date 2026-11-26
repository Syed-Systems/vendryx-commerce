# Vendryx Brand Guidelines (Syed Systems)

This document is the **Single Source of Truth** for the core Vendryx and Syed Systems corporate brand identity.
See [docs/BRAND.md](BRAND.md) for the complete design system specification and dynamic merchant token rules.

**IMPORTANT DISTINCTION:**
1. **Core Corporate Brand:** The guidelines below govern all internal/corporate products (Admin Dashboards, Marketing Sites, Syed Systems corporate pages, and default Vendryx properties).
2. **Merchant Branding:** Separate from this document. Merchants on the Vendryx platform will have their own distinct brand identities injected dynamically at runtime via tenant-specific design tokens (per `AGENTS.md` Rule 12.2). Merchant themes must *never* pollute this core Vendryx brand file.

---

## 1. Logo & Core Identity
**Vendryx** is the flagship product of **Syed Systems**. The brand represents modern, frictionless, and premium commerce ("Sell More. Worry Less.").

- **Primary Logo Mark:** The stylized 'V' resembling a shopping bag. It uses a dynamic violet-to-blue gradient with 3D overlapping folded layers.
- **Tagline:** "SELL MORE. WORRY LESS." (Used with high letter-spacing in uppercase).

## 2. Core Design Philosophy: Simplicity is Elegance
- **STRICT RULE - NO UI GRADIENTS:** Gradients are exclusively reserved for the Logo. There must be **NO** gradients used in web page design, theme design, or UI elements.
- **Plain and Simple:** The interface must remain plain, simple, and elegant. We rely on solid colors, excellent typography, and generous whitespace.

## 3. Color Palette
The Vendryx UI color system is vibrant but flat, grounded by deep, trustworthy navys.

### Primary UI Colors (Solid Colors Only)
- **Vendryx Orange:** `#FF7A00` (Brand accent, the mark, and highlights; pair with charcoal text)
- **Orange Action:** `#C2410C` (Buttons with white text and active states)
- **Orange Hover:** `#9A3412` (Hover and pressed states)

### Neutral & Background Colors
- **Corporate Navy (Text & Dark Mode UI):** `#0F172A` (Rich, deep blue-black. Never use pure `#000000`)
- **Navy Light:** `#1E293B`
- **Surface White:** `#FFFFFF`
- **Subtle Gray:** `#F8FAFC`

## 4. Typography
The brand requires a modern, geometric sans-serif to match the premium tech aesthetic. (Recommended: **Inter** or **Plus Jakarta Sans**).

- **Brand Logotype / Headings:** Extra Bold (800) or Bold (700). Very tight letter spacing (tracking).
- **Taglines / Microcopy:** Medium (500) or SemiBold (600). Uppercase with extremely wide letter spacing (tracking: 0.15em - 0.2em).
- **Body Text:** Regular (400) or Medium (500). Optimized for readability with a line height of 1.5 - 1.6.

## 5. Design Tokens (Implementation)
These values must be mapped to our CSS variables in the root style files (`index.css` / `globals.css`) for all corporate apps (marketing, admin, internal).

```css
:root {
  --brand-vendryx-primary: #FF7A00;
  --brand-vendryx-action: #C2410C;
  --brand-vendryx-hover: #9A3412;
  --brand-corporate-navy: #0F172A;

  --brand-font-family: 'Plus Jakarta Sans', 'Inter', sans-serif;
  --brand-radius-card: 16px;
}
```

## 6. Usage Rules
- **No Hallucination:** Agents and developers must strictly reference the variables defined in section 5. Do not invent new shades of purple or blue.
- **Flat & Clean:** UI elements must be flat and clean. Do not use gradients or overly complex shadows. Let the solid colors and typography speak for themselves.
