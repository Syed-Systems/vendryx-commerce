# Vendryx Unified Multi-Theme Storefront — Design System & Security Architecture

## 1. Architectural Model (Shopify-Style Single Server, Unrecognizable Brands)

`apps/storefront` runs as **one unified Next.js App Router server** serving every merchant storefront while guaranteeing that **no shopper can tell two brands share an engine** and **no theme can corrupt commerce data or leak cross-tenant state**.

```
Incoming Request (Host: maisonvela.com | kovahydration.com | *.vendryx.shop)
   │
   ▼
[src/middleware.ts] ── Strict Host validation (blocks Host Header Injection), Vary: Host
   │
   ▼
[src/core/tenant-resolver.ts] ── Resolves Tenant + Plan + Published/Draft Theme JSON
   │                              Partitioned cache key: sf:<tenantId>:<themeVer>:<currency>:<route>
   ▼
[src/core/theme-schema.ts] ── Zod v1 strict validation (hex colors, safe hrefs, blocks XSS/scripts)
   │                           Falls back safely to last known good config on corrupt JSON
   ▼
[src/core/StorefrontShell.tsx] ── Scopes CSS variables onto theme root, partitions Cart by tenantId
   ├── Mounts isolated Theme Pack from [src/core/theme-registry.ts]
   │      ├── maison-cuir  (Maison Véla — Luxury Leather)
   │      └── hydro-vessel (KOVA — Modern Insulated Bottles)
   ├── [PlatformAttributionFooter.tsx] ── Core-enforced "Powered by Vendryx" on Free plan ONLY
   └── [LockedCheckoutModal.tsx] ── Core-owned checkout + integer money math (themes cannot override)
```

---

## 2. Design Token Taxonomy (Validated via Zod)

All runtime-customizable theme tokens pass through `DesignTokensSchema` (`src/core/theme-schema.ts`) and are injected onto the theme root as scoped CSS custom properties (`--theme-*`):

| Token | CSS Variable | Purpose |
| :--- | :--- | :--- |
| `canvas` | `--theme-canvas` | Primary page background |
| `surface` | `--theme-surface` | Secondary container / card plinth / drawer surface |
| `surfaceAlt` | `--theme-surface-alt` | Hover / elevated pill / subtle inset surface |
| `ink` | `--theme-ink` | Primary high-contrast foreground text |
| `inkMuted` | `--theme-ink-muted` | Secondary captions, specs, metadata |
| `border` | `--theme-border` | Hairline or structural container dividers |
| `accent` | `--theme-accent` | Primary CTA & active selection color |
| `accentInk` | `--theme-accent-ink` | Contrast foreground on accent controls |
| `heroBg` / `heroInk` | `--theme-hero-bg` / `--theme-hero-ink` | Editorial campaign / dark studio hero framing |
| `radiusCardPx` | `--theme-radius-card` | Structural geometry (`0px` luxury vs `16px` consumer D2C) |
| `radiusButtonPx` | `--theme-radius-button` | Control geometry (`0px` sharp vs `999px` pill) |
| `fontPair` | `cormorant-jakarta` \| `outfit-dmsans` | Paired display + body typography (both include `Hind Siliguri` Bengali fallback) |

---

## 3. Theme Divergence Matrix (Zero Shared DNA)

Every theme lives in its own isolated folder (`src/themes/<theme-id>/`) with **its own CSS Module, bespoke SVG icon pack, typography hierarchy, card ratio, PDP layout, and cart drawer**.

| Dimension | `maison-cuir` (Maison Véla — Luxury Leather) | `hydro-vessel` (KOVA — Bottle Brand) |
| :--- | :--- | :--- |
| **Real-World Lineage** | Polène Paris, Métier London, Carl Friedrik | LARQ, Fellow, Hydro Flask, Owala |
| **Color Palette** | Warm Limestone `#F6F3EE`, Espresso `#171513`, Saddle Cognac `#8F4F2B` | Cool Mist `#F7F9FB`, Slate Navy `#0D1B2A`, Cobalt `#0057FF`, Canyon Coral `#DF6747` |
| **Typography** | `Cormorant Garamond` (Editorial Serif) + `Plus Jakarta Sans` | `Outfit` (Geometric Display) + `DM Sans` + `JetBrains Mono` |
| **Corner Geometry** | `0px` razor-sharp architectural plinths & buttons | `16px` tactile studio cards + `999px` full-pill CTAs |
| **Header Architecture** | Centered serif wordmark, hairline top/bottom border, minimal atelier nav | Kinetic split bar, bold left wordmark, rounded utility bag button with counter |
| **Hero Layout** | Full-bleed dark-vignette editorial campaign frame | 2-column slate-navy split hero with live colorway & capacity configurator |
| **Product Cards** | 3:4 tall portrait gallery cards, hover crossfade, quiet finish swatches | 4:3 clean studio cards, inline color swatches + instant `Quick Add` button |
| **Story / Proof Band** | 50/50 Italian tannery provenance narrative with macro leather photography | 4-column engineered thermal performance matrix (`24h Cold`, `12h Hot`, `18/8 Steel`, `100% Seal`) |
| **PDP Interaction** | Sticky Atelier modal with expandable Interior Anatomy accordion | Split Product modal with interactive `"Bundle & Save"` silicone accessory checkbox |
| **SVG Icon Pack** | Custom `1.1px` hairline architectural SVGs (`maison-cuir/icons.tsx`) | Custom `2.2px` rounded industrial SVGs (`hydro-vessel/icons.tsx`) |

---

## 4. Security & Multi-Tenant Hardening Checklist

1. **Host Header Injection Protection:** `src/middleware.ts` + `normalizeAndValidateHost()` reject malformed/injected hosts before SSR.
2. **Tenant-Partitioned Caching:** Every cache key is prefixed `sf:<tenantId>:<themeVersion>:<currency>:<route>` and responses emit `Vary: Host`.
3. **Host-Only Session Cookies:** `buildHostOnlyCookie()` omits `Domain=` so Free tier subdomains (`store-a.vendryx.shop`, `store-b.vendryx.shop`) never leak cookies across tenants.
4. **No Platform Leak in HTTP Headers:** `next.config.mjs` sets `poweredByHeader: false` (strips `X-Powered-By: Next.js`).
5. **Plan-Enforced Attribution:** `PlatformAttributionFooter` lives in `src/core/`, renders `"Powered by Vendryx"` exclusively when `plan === "free"`, and renders `null` for Pro & Scale white-label stores.
6. **Locked Checkout & Integer Money Math:** Themes receive only `StorefrontThemeProps` callbacks — never checkout forms, never payment credentials, never floating-point pricing.
7. **Dynamic Per-Tenant SEO:** `generateMetadata`, `src/app/robots.ts`, and `src/app/sitemap.ts` resolve tenant canonical origins per request host.
