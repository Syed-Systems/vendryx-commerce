# Admin Dashboard — upstream pin

## Pin

| Field | Value |
|---|---|
| Repository | `https://github.com/medusajs/medusa` |
| Tag | `v2.19.0` |
| Commit | `87d77fa1b56ec287aa6655aaa2f54245387aa2f2` |
| Subdirectory | `packages/admin/dashboard` |
| Package | `@medusajs/dashboard@2.19.0` |
| Verified | 2026-11-27, via `git ls-remote --tags` and the tagged `packages/admin/dashboard/package.json` |

`./scripts/check-upstream-pin.sh` re-resolves the tag against the real remote and
fails if it does not point at exactly this commit.

## Why the tag is `v2.19.0`

Medusa tagged the dashboard on its own (`@medusajs/dashboard@<version>`) only up to
`2.17.1`. From 2.18 onwards the only tag is the monorepo-wide `v<version>`. At `v2.19.0`
the dashboard's `package.json` declares `@medusajs/dashboard` `2.19.0`, which is the
version this port was taken from: `apps/admin/CHANGELOG.md` is that package's
changelog and starts at 2.19.0, and `apps/admin/package.json` pins the matching
`@medusajs/*` 2.19.0 libraries.

The previous pin (`@medusajs/dashboard@2.17.1`, commit `f2359700…`) described the
earlier port. The source was re-ported from 2.19.0 in `df428235` (2026-08-18), which
deleted this file instead of updating it.

## Why the repository and not a standalone dashboard repo

Medusa v2's dashboard is not a standalone repository. It lives inside the
`medusajs/medusa` monorepo at `packages/admin/dashboard`. Any instruction to clone
`medusajs/dashboard` is wrong: that repository does not exist.

## Scope of the port

The dashboard talks to `/admin/*` over `@medusajs/js-sdk`, so wiring it to the Stheno
engine is a port, not a clone. ADR-002's admin wire compatibility is what makes the
port possible, and ADR-027 keeps that compatibility as an internal economy while the
store wire diverges. Per ADR-027 decision 4, this file is engineering provenance and
stays.

RBAC and SSO screens are not taken from upstream. They are Medusa Enterprise and are
rebuilt clean-room per ADR-008.

There is no upstream to pull from in practice: the fork carries Vendryx routes,
branding and tenant handling. Re-porting a newer upstream version means updating this
pin in the same change.
