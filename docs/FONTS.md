# Curated Font Set — Licences & Specifications

> **Rule (ADR-013 Decision 2a):**
> Checkout is core and strictly audited. Fonts on checkout routes must be self-hosted. Pulling
> fonts from a third-party CDN (such as Google Fonts or Adobe Fonts) introduces external network
> dependencies and script/origin risks, violating `check-checkout-scripts.sh` and SAQ-A eligibility.
> Merchants choose from this curated, licence-cleared font set by key. Merchants do not upload
> font files.

---

## 1. Curated Font Catalog

All curated fonts are licensed under permissive open-source font licences (SIL Open Font License 1.1
or Apache 2.0) permitting self-hosting, redistribution, and commercial usage with storefronts.

| Font Key | Family Name | Version | Licence | Upstream Source | Primary Use Case |
|---|---|---|---|---|---|
| `inter` | Inter Variable | 4.0 | SIL Open Font License 1.1 | [rsms/inter](https://github.com/rsms/inter) | Default clean, modern UI typeface |
| `geist` | Geist Sans | 1.3 | SIL Open Font License 1.1 | [vercel/geist-font](https://github.com/vercel/geist-font) | High-tech, minimal display & body |
| `noto_sans_bengali` | Noto Sans Bengali | 2.003 | SIL Open Font License 1.1 | [googlefonts/noto-fonts](https://github.com/googlefonts/noto-fonts) | Native script support for Bangladesh commerce |
| `playfair` | Playfair Display | 2.904 | SIL Open Font License 1.1 | [clauseggers/Playfair-Display](https://github.com/clauseggers/Playfair-Display) | Boutique, editorial, luxury watch/fashion |
| `plus_jakarta_sans` | Plus Jakarta Sans | 2.8 | SIL Open Font License 1.1 | [tokotype/Plus-Jakarta-Sans](https://github.com/tokotype/Plus-Jakarta-Sans) | Modern geometric consumer brands |
| `roboto` | Roboto | 2.138 | Apache License 2.0 | [googlefonts/roboto](https://github.com/googlefonts/roboto) | Universal industrial sans |
| `cinzel` | Cinzel | 2.001 | SIL Open Font License 1.1 | [NDISCOVER/Cinzel](https://github.com/NDISCOVER/Cinzel) | Classical serif, high-end jewellery |

---

## 2. Technical Invariants

1. **Self-Hosted WOFF2:**
   All fonts are distributed as compressed `.woff2` files located within the storefront public assets:
   `apps/storefront/public/fonts/<font_key>/`
2. **Font-Display Swap:**
   Every `@font-face` declaration uses `font-display: swap;` with standardized metrics to eliminate
   Cumulative Layout Shift (CLS) during web font resolution (satisfies P26-09 Lighthouse budget).
3. **No Dynamic Uploads:**
   Merchants select a `font_key` string stored in `tenant_branding.font_key`. Binary file uploads for
   fonts are rejected at the API boundary — fonts are built-in static assets.
4. **Fallback Safety:**
   If `font_key` is `None` or an unrecognised value, the CSS property `--brand-font-family` falls back
   to the system font stack:
   `Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`

---

## 3. Licence Compliance Verification

Run the font license gate before shipping:
```bash
./scripts/check-font-licences.sh
```
The script validates that every registered font key maps to an approved licence in this registry and
that no external font URLs exist on checkout routes.
