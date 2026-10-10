# Vendryx — Brand Identity & Asset Guidelines

**Version:** 1.0  |  **Owner:** Syed Systems  |  **Source concept:** Selected Channel Cart (Concept C)

## 1. Brand architecture and naming

The product is **Vendryx** (VEN-driks), by **Syed Systems**. The official corporate tagline maintained in the repository is **“SELL MORE. WORRY LESS.”** The approved campaign/visual proposition for this kit is **“One cart. Every channel. One commerce OS.”** The campaign proposition does not supersede the official corporate tagline.

Customer-facing platforms may say Vendryx and “by Syed Systems”; they must not expose internal `stheno-*` crate names or Medusa compatibility implementation. Vendryx corporate branding belongs to `apps/marketing`, `apps/admin`, `apps/platform`, company communications and PWA shells. Individual merchants control their storefront logos, domain, colors, theme and imagery; never force the Vendryx palette onto merchant storefronts.

## 2. Logo variants and when to use each

- **Primary horizontal:** `01_Logos/Primary/vendryx-primary-light.svg` on off-white/white. Use this as the default logo wherever width allows.
- **Reverse horizontal:** `01_Logos/Primary/vendryx-primary-dark.svg` on deep navy or other truly dark, high-contrast fields.
- **Color illustrative channel edition:** `01_Logos/Primary/vendryx-channel-edition-light.svg` on marketing artwork sufficiently large for the badge details. This is not a required logo for app headers or small sizes. External social-platform emblems are third-party marks and remain subject to their respective policies.
- **Stacked:** `01_Logos/Secondary/vendryx-stacked-light.svg` where a vertical lockup is required.
- **With campaign proposition:** `01_Logos/Secondary/vendryx-with-tagline.svg` in wider campaign space, not cramped UI.
- **Monochrome:** black/navy and white vectors in `01_Logos/Monochrome/` for single-ink, laser engraving, watermark and low-color applications.
- **Mark only:** `01_Logos/Marks/vendryx-connected-cart.svg`; for tiny sizes use `vendryx-connected-cart-micro.svg`.

**Responsive identity:** large formats support five connected-channel nodes. The 16px favicon/compact icon uses only three nodes so the cart silhouette remains identifiable; it is a purpose-built small-size cut, not a separate concept.

### Minimum sizes, spacing and handling

Preserve a clear margin around the lockup equal to at least **one wheel diameter of the symbol**; leave more in hero placements. Suggested minimum digital widths: **180 px horizontal with byline**, **96 px mark-only**, **16 px micro favicon**. Where the byline becomes illegible, use the standalone cart mark and separate textual accessibility label.

Never stretch the vector, recolor arbitrarily, set the logo on low-contrast patterns, distort the channel lines, substitute another wordmark font inside exported assets, animate wheels independently, or place third-party logos in the favicon. Use the supplied outlined SVG logo paths for fidelity.

## 3. Color palette (digital, sRGB)

| Color | Hex | RGB | Recommended use |
|---|---|---|---|
| Vendryx Indigo | `#4F46E5` | 79, 70, 229 | Corporate primary action and navigation |
| Vendryx Violet | `#5B21B6` | 91, 33, 182 | Logo accents and selected emphasis |
| Vendryx Bright Blue | `#3B82F6` | 59, 130, 246 | Secondary/info emphasis |
| Deep Navy | `#0F172A` | 15, 23, 42 | Body text, dark shell, white reverse ground |
| Navy Light | `#1E293B` | 30, 41, 59 | Dark surfaces and secondary headers |
| Canvas | `#FCFBF9` | 252, 251, 249 | Warm corporate background |
| Surface White | `#FFFFFF` | 255, 255, 255 | Cards and dialogs |
| Subtle Background | `#F8FAFC` | 248, 250, 252 | Table headers, quiet panels |
| Hairline Border | `#E2E8F0` | 226, 232, 240 | Borders and dividers |
| Muted Text | `#64748B` | 100, 116, 139 | Supporting text; check contrast at small sizes |

Semantic states are distinct from corporate accents: success `#16A34A`, warning `#D97706`, destructive `#DC2626`. Prefer navy for long-form text. Test normal-weight text for WCAG AA contrast 4.5:1; graphical controls and large type generally target at least 3:1. Do not rely on color alone for status.

**Critical restriction:** Gradients may appear *inside the brand mark artwork only*. Do not apply linear/radial gradients to website backgrounds, cards, navigation, buttons, dashboards or checkout chrome. Existing marketing concept sheets are inspirations, not authoritative overrides to this rule.

## 4. Typography

- **Corporate interface:** Plus Jakarta Sans, fallback Inter, system-ui, sans-serif.
- **Outlined logo:** bundled SVG wordmark is geometrically outlined for portability (constructed with Inter shapes); do not use ordinary text to approximate its glyph spacing.
- **Body:** 14–16 px, line-height approximately 1.5–1.6, weight 400–500.
- **Headlines:** 28–44 px, weight 700–800, letter spacing −0.02em where appropriate.
- **Section headings:** 18–24 px, weight 600–700.
- **Microcopy:** 12–13 px, weight 500; avoid lengthy uppercase paragraphs.
- **Technical data:** SF Mono / Menlo / Consolas / UI monospace.

No proprietary font files are included in the kit. Use a legitimate font source/license independently in deployment.

## 5. Layout principles

Apple HIG-inspired interface design: 8 px spacing grid; 10 px input radius; 16 px card radius; generous breathing room; 44 px minimum touch target; clean hairline borders before heavy shadows. Motion should express interaction states, never conceal order/checkout information. No fictitious live metrics in production surfaces.

## 6. Messaging and brand voice

**Product name:** Vendryx.  **Attribution:** by Syed Systems.  **Official tagline:** Sell More. Worry Less.  **Campaign line:** One cart. Every channel. One commerce OS.

Brand values: merchant control, practical utility, trustworthy operations, connected selling, and transparent technology. Write plainly for merchants and precisely for technical readers. Do not promise automatic integration or channel capabilities unless the corresponding backend/provider integration is actually operating. Do not present mocked marketing dashboards as actual merchant data.

Suggested sample copy: “Manage orders, inventory and storefront operations in one place.” “Connect supported selling channels and simplify fulfillment.” “Build your store around your brand.”

## 7. Deliverable specs and filenames

- **Production logo source:** SVG, portable vectors, separate files for light, reverse, stacked, monochrome and mark-only.
- **Transparent logo PNGs:** 600/1200/2400/3600 px wide for horizontal logos; multiple other sizes provided in corresponding folders.
- **Favicon PNGs:** 16, 24, 32, 48, 64, 96, 128, 256, 512 px; `favicon.svg`; multi-image genuine `favicon.ico`.
- **App/touch/PWA PNGs:** 32,48,64,72,96,120,128,144,152,167,180,192,256,384,512,1024 px; extra maskable 192,512,1024; Apple touch and Android Chrome aliases; circular social avatar variants.
- **Marketing:** Open Graph 1200×630, X card 1200×675, LinkedIn cover 1584×396, LinkedIn square post 1200×1200, Facebook cover 1640×624, YouTube banner 2560×1440, email header 1200×400 and Product Hunt 1270×760. Each comes as separate `.svg` plus pixel-correct `.png`.
- **Collateral templates:** two-sided business card and A4 letterhead in editable SVG, as well as printable PDF versions. Generic contact placeholders must be replaced by actual company details.

## 8. Technical integration checklist

- Copy corporate logo and favicon variants into each app’s `public/brand/vendryx` asset path; do not link assets to private GitHub paths. Add correct favicon `<link>` tags and OG meta tags.
- Use `06_Developer_Tokens/vendryx-tokens.css` in Vendryx corporate shell styles only. The package also contains the token JSON and example `site.webmanifest`.
- Ensure `apps/admin` logo boxes, `apps/marketing` logo components, and `apps/platform` console all point to the selected logo assets rather than legacy placeholder/stheno brand assets.
- Preserve merchant-specific dynamic SSR style injection in storefront, per ADR-031. A corporate style import must not overwrite merchant theme values.
- Respect ADR-028: public-facing product name is Vendryx. Internal `stheno-*` compatibility identifiers remain unchanged.
- Respect `docs/STATUS.md` as the project completion authority. Brand rollout does not imply unfinished control-plane work is complete.
- Check icon legibility at actual 16×16 and 32×32, color contrast, alpha transparency, web manifest JSON, image dimensions, and light/dark rendering before merging.

## 9. Ownership and marks

Assets are tailored for the Vendryx brand concept supplied in this conversation. Names, badges and distinctive graphical trademarks associated with Facebook, Instagram, TikTok and WhatsApp remain the property of their respective owners; illustrative channel-edition artwork is not a statement of endorsement or official integration status. Review trademark and brand usage policies before public distribution. Social channel badges should be removed when not justified by the feature context.
