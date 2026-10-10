# Vendryx Brand Kit v1.0

**Approved direction:** Connected-channel shopping cart (selected Concept C).
**Parent company:** Syed Systems.
**Product:** Vendryx, a multi-tenant commerce operating system.
**Official product tagline:** Sell More. Worry Less.
**Campaign/visual proposition:** One cart. Every channel. One commerce OS.

This folder contains **actual independent, export-ready files**, not mockup boards. Each PNG's named dimensions are its actual pixel dimensions. Vector SVG files are independent, resolution-free assets with the main wordmark paths outlined to remove external font dependencies. The font recommendation applies to surrounding text/UI; no font binaries are included.

## Folder map

1. `01_Logos` - individual horizontal/stacked/color/reverse/monochrome SVGs and transparent PNGs; 600-3600 px wide; symbol PNGs at 128-2048 px. `vendryx-channel-edition` is an illustrative channel-badge edition for large marketing use only. The regular primary lockup has generic nodes and is the official default.
2. `02_Favicons` - independent `favicon.svg`, favicon PNG 16/24/32/48/64/96/128/256/512, multi-resolution real `.ico`.
3. `03_App_Icons` - separate PNG icons at 32,48,64,72,96,120,128,144,152,167,180,192,256,384,512,1024; SVG originals; maskable variants and round social avatars; Apple/Android aliases.
4. `04_Social_and_Marketing` - independent, correctly-sized SVG and PNG campaign banners, OG image, X card, LinkedIn, Facebook, YouTube, email, Product Hunt.
5. `05_Brand_Guidelines` - full written guidelines and illustrated PDF.
6. `06_Developer_Tokens` - machine-readable CSS/JSON, example PWA manifest, HTML `<link>` snippets.
7. `07_Collateral_Templates` - separate editable SVG business card and A4 letterhead templates, with placeholders (no invented contact details).
8. `08_Previews` - overall contact sheet (not a substitute for independent assets).

## Key usage choices

- The approved *idea* is a cart receiving channels. The primary logo uses five abstract colored input nodes (not other platforms' trademarks). The illustrative `channel-edition` restores platform-symbol badges for contextual large-format graphics; do not make those badges a required element in app icons or favicons.
- Small favicons use a simplified three-node cart to retain legibility at 16 px. This is a responsive logo family, not an unrelated logo.
- All main PNG logos and raw marks have **true alpha transparency**. App icons intentionally have a solid indigo/navy background.
- UI uses **no gradients**. Only logo artwork uses an indigo-violet / blue blend, matching repository `docs/BRAND.md`.
- The bundled files are design deliverables. They are **not yet written to the GitHub repository** or deployed to any hosted app.

## Quick integration

- Use `01_Logos/Primary/vendryx-primary-light.svg` in light app headers and marketing pages.
- Use `01_Logos/Primary/vendryx-primary-dark.svg` in dark headers (text is white).
- For menus and tiny spaces prefer `01_Logos/Marks/vendryx-connected-cart-micro.svg` or the favicon set.
- Copy `02_Favicons/favicon.ico`, `favicon.svg`, etc. to frontend public assets, respecting their names and paths.
- Copy the app icons for PWA/iOS/Android. Update `site.webmanifest` paths to match the destination.
- Use CSS tokens for **Vendryx corporate shells**: marketing, admin, and operator console only. Keep tenant-specific storefront themes isolated, per ADR-031.

## Caveats

These deliverables are new vector adaptations of the chosen cart+channel concept, not pixel-perfect vector tracings of the earlier generated presentation image. SVG wordmark shapes are built from the installed Inter typeface as closed vector outlines. Surrounding interface typography is specified as Plus Jakarta Sans with Inter fallback. Review trademark usage and third-party channel logo requirements before external launch.
