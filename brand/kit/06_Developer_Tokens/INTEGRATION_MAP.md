# Suggested Vendryx repo integration (not yet applied)

This kit is a standalone artifact. Paths below are integration suggestions; no GitHub branch was modified.

| Vendryx source path | Recommended files | Notes |
|---|---|---|
| `brand/vendryx/` | source SVGs + palette JSON | Authoritative corporate asset bundle |
| `apps/admin/public/brand/vendryx/` | logo light/reverse SVG, mark PNG, icons | Replace old `/brand/vendryx/vendryx-logo.png` references only after review |
| `apps/marketing/public/brand/vendryx/` | SVG lockups + favicon/OG | Update `VendryxLogo.tsx` and metadata references |
| `apps/platform/public/brand/vendryx/` | main logo, micro mark, favicons | Transitional operations console only |
| `apps/storefront/` | nothing globally enforced | Merchant logo/colors come from tenant brand tokens; do not overwrite |

The repository's existing docs/BRAND.md is authoritative for corporate identity and should be updated via reviewed pull request before considering this kit deployed. Do not rename internal `stheno-*` crates. It is recommended to retain the machine-readable `brand.json` reference and point existing components to versioned final assets.

## Suggested semantic assets

Primary: `vendryx-primary-light.svg`  | Reverse: `vendryx-primary-dark.svg`  | Compact: `vendryx-connected-cart-micro.svg`.

The supplied `site.webmanifest` references `/brand/vendryx/icons/` and is an example. Copy icons there or update paths. Use `branding-integration.html` as a starting point, changing public paths to real deploy paths.
