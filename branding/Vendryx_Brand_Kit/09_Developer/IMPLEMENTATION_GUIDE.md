# Vendryx brand asset integration (for coding agents)

## Canonical choice
Use the **approved third revised direction**: the orange folded-wing/ribbon emblem in the charcoal app tile. This pack contains a **clean editable vector reconstruction** of the approved image concept. All production exports are derived from the same vector geometry. The provided original concept preview is not the vector source; see `11_References/`.

## Asset selection
- Site navbar on light UI: `01_Primary_Logos/vendryx_primary_light_with_by_syed_systems.svg`.
- Site navbar on dark UI: `01_Primary_Logos/vendryx_primary_dark_with_by_syed_systems.svg`.
- Narrow sidebars: `04_Symbol/vendryx_tile_dark.svg`.
- Footer / wordmark-only: `03_Wordmark/vendryx_wordmark_light.svg` or `_dark.svg`.
- Dedicated brand pages: `02_Secondary_Logos/vendryx_stacked_light_with_by_syed_systems.svg`.
- Favicon: `05_Favicons/favicon.ico`, `favicon.svg`, or size-specific PNGs.
- iOS home-screen: `06_App_PWA/app-icon-dark-180.png`.
- PWA: `06_App_PWA/manifest.webmanifest` (copy icons to paths in manifest or rewrite paths).
- Sharing cards: `07_Social_Marketing/open-graph.png`.

## Workflow
1. Copy the intended SVG and PNG assets to the repository's `public/branding` and `public/icons` folders, preserving filename clarity.
2. Wire favicon, Apple touch icon, and manifest in `app/layout.tsx` metadata or existing head implementation.
3. Use CSS custom properties from `tokens.css` and extend existing Tailwind theme with `vendryxTheme`; never replace your whole theme config blindly.
4. Select the light/dark wordmark programmatically or through CSS; **do not recolor the dark icon tile by CSS filters**.
5. Maintain natural logo aspect ratio; do not stretch. Avoid tagline in compact UI where unreadable.
6. Confirm against real UI on desktop/mobile and run a dark/light screenshot QA pass.
7. Ensure all high-level metadata and title strings spell **Vendryx**, never Vendrix or Vedrix.
8. Check responsive share preview crops and YouTube channel art safe area before publishing.

## Accessibility
- Treat images purely decorative only when adjacent visible text already names Vendryx. Otherwise use alt="Vendryx".
- `#FF7A00` is a brand accent but **not** the default fill for white-text CTA buttons; prefer darker `#B74600` for small white button text or charcoal text on the vivid orange.
- Preserve focus ring and other keyboard navigation styling.

## Source rebuild
See `12_Source/build_brand.py`. Export SVGs are fully self-contained and use glyph outlines; no font files are distributed. The rebuild script requires a licensed local Inter Display font installation.
