# Frontend Mahal Image-to-Code Analysis

**Status:** Extraction complete for implementation baseline  
**Design source priority:** supplied screenshots first, generated missing-state references second  
**Brand substitution:** Vendryx identity only

## 1. Direction lock

- Theme paradigm: Pristine Light Mode.
- Background character: full-bleed cinematic imagery on a plain white commerce canvas.
- Typography character: Swiss rational hierarchy for commerce labels, with the approved Vendryx
  wordmark as the only identity treatment.
- Hero architecture: massive image-first hero with restrained text.
- Section system: gallery-led cadence.
- Signature components: pristine gapless product grid, hover-accordion navigation slice, off-grid
  editorial merchandising, and layered image crop frames.
- Motion cues: smooth accordion expansion and cinematic fade-through. Motion is subordinate to the
  static screenshot geometry.

The source screenshots are reconstructions, not inspiration. No layout modernization is permitted.

## 2. Pixel normalization

| Source | Bitmap | Working CSS viewport |
|---|---:|---:|
| Jewelry menu | 1178 x 812 | 1178 x 812 |
| Product detail | 2476 x 6806 | approximately 1238 x 3403 at DPR 2 |
| Men's catalog | 2476 x 16384 | approximately 1238 x 8192 at DPR 2 |
| Fashion home | 2476 x 16384 | approximately 1238 x 8192 at DPR 2 |
| Alternate catalog | 2476 x 12760 | approximately 1238 x 6380 at DPR 2 |

The DPR interpretation is validated by readable control sizes after halving. Visual comparison must
still use the original bitmap dimensions so raster output can be overlaid without resampling drift.

## 3. Extracted design tokens

These are initial implementation tokens. Final channel values are tuned by screenshot overlay.

```css
--mahal-canvas: #ffffff;
--mahal-ink: #111111;
--mahal-muted: #707070;
--mahal-hairline: #e5e5e5;
--mahal-well: #f5f5f5;
--mahal-inverse: #050505;
--mahal-danger: #d71920;
--mahal-brand: var(--brand-action, #c2410c);
--mahal-content: 1160px;
--mahal-gutter-desktop: 28px;
--mahal-gutter-mobile: 16px;
--mahal-control-height: 48px;
--mahal-radius: 0px;
```

- Body and navigation use a neutral grotesk at regular to medium weight.
- Commerce labels are 11 to 14px desktop and 12 to 15px mobile.
- Section headings are uppercase, medium weight, and centered or left-aligned exactly as shown.
- Product prices use normal weight and tabular numerals where available.
- Identity spacing follows the Vendryx SVG's intrinsic ratio; the source logo's serif character is
  not copied.
- Product and section surfaces are square. Rounded geometry is reserved for the source's small sale
  control or chat control, neither of which is reproduced unless functional.

## 4. Supplied desktop screens

### 4.1 Global header and jewelry mega-menu

Visible DOM regions in order:

1. White utility header with centered identity and account, search, bag, and BDT region controls.
2. One-line primary navigation centered beneath it.
3. Hairline divider and selected navigation underline.
4. Mega-menu with a narrow left category list and two equal portrait campaign panels.
5. Full-width product/editorial strip beginning immediately beneath the menu.

Measured behavior:

- Header plus navigation occupies about 165px at 1178px width.
- Main navigation remains one line with roughly 36px between labels.
- Mega-menu content is centered inside about 805px of usable width.
- Left category rail occupies about 145px; the two campaign panels are about 300 x 375px each.
- Campaign captions are white rectangular overlays near the lower edge, not pills.
- Menu uses whitespace and one divider rather than a card container or shadow.

### 4.2 Fashion home

Visible DOM cadence:

1. Slim announcement strip and header.
2. Wide campaign hero.
3. Full-width cinematic campaign banner.
4. Designer spotlight: narrative tile plus finite product rail.
5. Three-up merchandising row.
6. Shop by style heading and two-row four-column image matrix.
7. Inverse black shop-by-occasion section with three-up editorial panels.
8. Two-up designer and celebrity editorial tiles.
9. Bridal edit banner, then a three-up product/editorial rail.
10. Another wide campaign banner.
11. Two-up accessory edit.
12. Lower sticky navigation and the beginning of the menswear edit.

Rules:

- Hero and campaign art run nearly edge to edge inside a narrow page gutter.
- Headings do not carry explanatory subcopy unless it exists in the source image.
- Image captions sit as flat black or white strips attached to the media edge.
- Major vertical gaps are created by section padding, not rounded outer wrappers.
- Repeated grids retain equal tracks and narrow gutters; they are intentionally dense.

### 4.3 Men's catalog

Visible DOM cadence:

1. Announcement and compact site header.
2. Breadcrumb and centered category title.
3. Three-product featured rail.
4. Sticky main navigation and catalog utility row.
5. Left filter rail and three-column product grid.
6. Long catalog continuation and pagination.

Rules:

- Catalog content is approximately 900px wide within the 1238px CSS viewport.
- Desktop filter rail is approximately 190px; three equal product tracks fill the rest.
- Product tiles have no enclosing card border. Image, small brand, title, and price stack directly.
- Heart sits at image top-right. Quick-add is a small square attached to the image bottom-right.
- Sale badges are compact white/red rectangles.
- Empty media positions visible late in the source preserve product metadata rhythm rather than
  collapsing the grid.

### 4.4 Alternate clothing catalog

- Four equal product columns sit inside a wide centered container.
- Product wells use a near-white neutral distinct from the page.
- The media ratio is close to 3:4, with the model kept centered and feet near the lower edge.
- Text is smaller and tighter than the men's catalog, but DOM order is identical.
- Footer is full-width near-black with four information groups and a newsletter field.

The Mahal catalog structure governs the new frontend when the two sources disagree; the alternate
catalog is used only to validate long-grid rhythm and four-column behavior.

### 4.5 Product detail

Visible DOM cadence:

1. Slim announcement region followed by generous whitespace at the captured scroll position.
2. Vertical thumbnail rail and one large portrait product image.
3. Sticky purchase information column aligned to the media's lower half in the supplied capture.
4. Large grey description disclosure spanning the media column.
5. Sticky global navigation crossing the captured page during scroll.
6. Related-products heading and four-up product row.
7. Newsletter block.
8. Three-column service strip.
9. Large near-black footer with link groups and lower legal/payment rows.

Rules:

- Desktop media region takes roughly two thirds of the content width; purchase information takes one
  third.
- Thumbnails are small portrait rectangles with 16 to 20px vertical gaps.
- Main media is portrait and uncropped enough to retain the full garment silhouette.
- Option boxes and purchase buttons are square-edged.
- The description block's large grey surface is source-specific and retained.
- Sticky navigation is a real layer with reserved geometry, not a duplicate screenshot fragment.

## 5. Generated missing-screen references

Generated files remain in the Codex generated-image directory for traceability.

### 5.1 Mobile home

- Intended CSS width: 390px.
- Announcement: 34px; utility header: approximately 86px.
- Hero is full bleed under the header.
- Section gutters: approximately 16px; major section spacing: 28 to 36px.
- Designer spotlight uses one large feature image followed by an edge-visible horizontal rail.
- Shop by style becomes a two-column grid with 8px gaps.

### 5.2 Mobile catalog

- Featured products remain three-up in a horizontal overflow rail.
- Sticky control row is approximately 62px with FILTER, count, and SORT in three tracks.
- Main catalog becomes a two-column grid with 8px to 10px gutter.
- Generated announcement text is rejected as an unsourced hallucination. The region is omitted when
  core supplies no approved message.

### 5.3 Mobile product detail

- Header, full-width gallery, page dots, product facts, options, primary action, secondary action,
  accordions, related grid, newsletter, footer, then sticky bottom purchase bar.
- Generated description, care, shipping, returns, and service claims are rejected unless equivalent
  values arrive through core props.
- The duplicate `S` option in the generated image is a generation defect. Actual options render from
  variant data without mutation.

### 5.4 Mobile cart and drawer

- Cart lines use a 158px portrait image beside title, options, price, stepper, and remove action.
- Discount is a disclosure rather than an always-open card.
- Totals are unboxed and separated with hairlines.
- Drawer occupies the right side of the viewport over a neutral dimmed scrim and restores focus when
  closed.

### 5.5 Mobile checkout

- Step order is ADDRESS, DELIVERY, PAYMENT, REVIEW.
- Inputs are square, approximately 48px high, and use always-visible labels.
- Inline errors combine border, text, and programmatic description.
- Core controls the actual field set, state, validation, and progression.

### 5.6 Mobile account

- Plain horizontal account navigation, labeled account detail rows, simple order rows, sign-out
  action, and black footer.
- No statistic cards or loyalty inventions.
- Empty order state uses one heading, one explanatory line, and one browse action.

## 6. Truth and rejection ledger

The following generated or photographed strings are not production facts by themselves:

- free or international shipping;
- early-delivery WhatsApp promises;
- easy returns;
- secure payments;
- continuous customer support;
- warranty or authentication windows;
- product stock counts and product counts;
- policy, contact, size-guide, and track-order routes;
- generated product descriptions, fabric, care, and fit values;
- fixture customer identity, order ids, dates, statuses, and totals.

They render only from core data or approved merchant configuration. Fixture values are confined to
the Vite harness and are labeled as fixtures in source.

## 7. Visual verification protocol

1. Render deterministic fixture routes at the normalized CSS viewport.
2. Capture at the same DPR and bitmap size as each canonical screenshot.
3. Overlay source and output at 50 percent opacity.
4. Correct mismatches in this order: page bounds, section order, primary grid, media frame, typography,
   spacing, border, color, then icon detail.
5. Run Playwright screenshot assertions in a pinned browser/environment.
6. Treat content-order or geometry drift as failure. Review font antialiasing and image encoding noise
   separately.
7. Repeat at 1280 x 800, 768 x 1024, and 390 x 844 for responsive evidence.

No implementation is accepted because it feels similar. It must survive the overlay.
