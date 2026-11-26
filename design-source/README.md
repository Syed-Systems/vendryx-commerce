# design-source

Design material kept for reference. **Nothing builds from this folder**, and no app, test or CI
job imports it.

## `ai-studio-2026-11-27/`

The three storefront designs pushed to `main` in commit `12203039` (2026-11-27), built in Google
AI Studio:

| Design | Theme id | Path |
|---|---|---|
| Maison Véla, luxury leather | `maison-cuir` | `apps/storefront/src/themes/maison-cuir/` |
| KOVA, insulated bottles | `hydro-vessel` | `apps/storefront/src/themes/hydro-vessel/` |
| Homedine, eco kitchenware | `homedine-eco` | `apps/storefront/src/themes/homedine-eco/` |

Paths inside this folder mirror where each file sat in the repository. The shared runtime the
designs came with (`src/core/`: theme registry, theme schema, tenant resolver, locked checkout,
attribution footer) is kept as input for the ADR-038 theme contract.

These files were moved here by task T0.1 of `docs/STOREFRONT_THEME_ENGINE_PLAN.md` because they
replaced the engine-connected storefront: tenants, domains, products, prices and reviews are
hardcoded, and the checkout creates no order. Tasks T3.1–T3.3 convert each design into a
`themes/<theme-id>/` package under ADR-038 (`docs/DECISIONS.md`), keeping its look and replacing
every hardcoded value with tokens and engine data.

Do not edit these files to change a live store. Edit the converted theme instead.
