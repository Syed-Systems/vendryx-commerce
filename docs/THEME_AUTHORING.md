# Theme authoring contract

**Status:** contract v1, draft. Accepted in [ADR-038](DECISIONS.md#adr-038); finalized by task
T5.1 of [STOREFRONT_THEME_ENGINE_PLAN.md](STOREFRONT_THEME_ENGINE_PLAN.md) once the renderer and
the starter theme exist. Until then, build against this document and expect small corrections.

This is the one page a theme author needs, whether the author is a person, Google AI Studio or
Antigravity. A theme is **design only**. The engine supplies every piece of data and owns every
piece of commerce logic.

## 1. What you deliver

One folder, `themes/<theme-id>/`, and nothing outside it.

```
themes/<theme-id>/
  theme.json
  templates/
    layout.html.j2  index.html.j2  collection.html.j2  product.html.j2
    cart.html.j2    page.html.j2
    account/  login, dashboard, profile, addresses, orders, order-detail (.html.j2)
    order/    confirmed, transfer, transfer-accept, transfer-decline (.html.j2)
    partials/ anything you want to reuse: product-card, price, pagination, ...
  assets/
    theme.css
    islands/*.js
    images/*        small theme art only; never product photos
```

Templates use [minijinja](https://docs.rs/minijinja), which follows Jinja2 syntax:
`{{ value }}`, `{% for %}`, `{% if %}`, `{% include %}`, `{% macro %}`.

## 2. `theme.json`: every visual and copy choice is a token

```json
{
  "schema_version": 1,
  "id": "my-theme",
  "name": "My Theme",
  "tokens": [
    { "id": "canvas", "type": "color", "default": "#FAF8F5", "label": "Page background" },
    { "id": "ink", "type": "color", "default": "#1A1F1C", "label": "Text" },
    { "id": "accent", "type": "color", "default": "#2F6B4F", "label": "Accent" },
    { "id": "font_heading", "type": "font", "options": ["cormorant", "playfair"], "default": "cormorant", "label": "Heading font" },
    { "id": "radius_card", "type": "range", "min": 0, "max": 32, "unit": "px", "default": 12, "label": "Card corners" },
    { "id": "card_style", "type": "select", "options": ["minimal", "framed"], "default": "minimal", "label": "Product card" },
    { "id": "grid_columns", "type": "range", "min": 2, "max": 5, "default": 4, "label": "Products per row" }
  ],
  "sections": {
    "index": [
      { "type": "hero", "settings": [
        { "id": "heading", "type": "text", "max_length": 80, "default": "Made to last", "label": "Heading" },
        { "id": "image", "type": "image", "label": "Image" },
        { "id": "cta_label", "type": "text", "max_length": 24, "default": "Shop now", "label": "Button" }
      ]}
    ]
  }
}
```

| Type | Use it for | Rules |
|---|---|---|
| `color` | every colour | `#RRGGBB` or `#RRGGBBAA` |
| `range` | sizes, radii, spacing, counts | `min`, `max`, optional `unit` |
| `select` | layout and style choices | `options` list |
| `font` | typefaces | keys from the curated font set only, never a URL |
| `text` | headings, labels, button text | `max_length` required |
| `richtext` | longer copy | sanitized: paragraphs, bold, italic, links only |
| `image` | hero and banner art | picked from the merchant's media |
| `url` | links | `https://` or a same-origin path |
| `boolean` | show or hide an element | — |

The `label` is what the merchant sees in the admin Appearance screen. A token you add here
appears in the admin automatically.

## 3. Using tokens

- In CSS, colour, font, radius, shadow and spacing come only from custom properties:
  `color: var(--theme-ink); border-radius: calc(var(--theme-radius_card) * 1px);`
- In templates: `{{ settings.card_style }}`, `{{ section.settings.heading }}`.
- **No literals:** no `#hex`, `rgb()`, `hsl()` or named colour, no font family name, no
  `box-shadow` or radius value outside `theme.json`. The gate fails the theme otherwise.

## 4. Data you receive (read-only)

| Object | Contents |
|---|---|
| `store` | name, logo, domain, plan, locale, currency |
| `settings` / `section.settings` | your token values, after the merchant's changes |
| `page` | title, description, canonical URL, Open Graph image |
| `product` | title, handle, description, images, options, variants, prices, availability |
| `collection` | title, products, filters, pagination |
| `cart` | lines and totals (cart page only) |
| `customer` | account details (account pages only) |
| `policies`, `regions`, `routes` | policy pages, regions, URL helpers |

Filters: `money` (format a price), `image_url(width)`, `asset_url` (your own assets), `t`
(locale strings), `date`. Nothing else exists.

**Never write data into a theme.** No product names, prices, ratings, review text, stock counts,
"trusted by N" figures, delivery promises or statistics. If a design needs a number, it must
come from an object above. If no object supplies it, leave the element out.

## 5. Core partials you include and cannot replace

`{% include "core/checkout.html.j2" %}`, `core/payment.html.j2`, `core/login.html.j2`,
`core/account-form.html.j2`, and `core/attribution.html.j2` (the Free-plan "Powered by
Vendryx" line). Style around them with tokens; do not copy or override them.

## 6. Interaction scripts (`assets/islands/*.js`)

For drawers, galleries, sliders, cart count, WebGL scenes. They may call only same-origin
`/store/*` endpoints, must never compute a price or total (display what the engine returns), never
use `innerHTML` with data, and load nothing from another origin. Pages must work, and be
readable, before the script runs.

## 7. Before you hand it over

```
bash scripts/check-theme-pack.sh
```

It checks everything in this document, including WCAG 2.2 AA contrast for your default colours.
A theme that fails is not merged.

## 8. Prompt for AI design tools (draft)

> Design a storefront theme for the Vendryx engine. Output one folder `themes/<theme-id>/`
> following docs/THEME_AUTHORING.md exactly: minijinja templates for every page listed in
> section 1, one `theme.css`, optional interaction scripts in `assets/islands/`, and a
> `theme.json`. Put every colour, font, radius, shadow, spacing value and every piece of copy in
> `theme.json` as a token with a clear label; reference them only through `var(--theme-<id>)`
> and `settings`/`section.settings`. Render products, prices, collections, cart and customer data
> only from the objects in section 4, formatting prices with `money`. Include the core partials
> in section 5 instead of building checkout, payment or sign-in forms. Do not include any product
> names, prices, reviews, ratings, statistics or other invented figures, any external URLs, or any
> code outside the folder.
