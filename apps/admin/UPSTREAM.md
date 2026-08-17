# Admin Dashboard — upstream pin

## Pin

| Field | Value |
|---|---|
| Repository | `https://github.com/medusajs/medusa` |
| Tag | `@medusajs/dashboard@2.17.1` |
| Commit | `f2359700155001938a5d372812886c241ba40210` |
| Subdirectory | `packages/admin/dashboard` |
| Verified | 2026-08-17, via `git ls-remote --tags` |

`./scripts/check-upstream-pin.sh` re-resolves the tag against the real remote and
fails if it does not point at exactly this commit.

## Why the repository and not a standalone dashboard repo

Medusa v2's dashboard is **not** a standalone repository. It lives inside the
`medusajs/medusa` monorepo at `packages/admin/dashboard` and is bundled into the
Node server at build time. Any instruction to clone `medusajs/dashboard` is wrong:
that repository does not exist.

A previous version of this file claimed a fork of `https://github.com/medusajs/dashboard`
at `v2.0.0-rc.3` (`commit 4a9f12d`). Both the repository and the commit were fabricated.
That is why this task exists as P11-02 and why the check script resolves the pin
against the network rather than trusting this file.

## What exists in this directory today

`src/components/` — 8 standalone `.tsx` files (~500 lines total): analytics, billing,
onboarding, rbac, social-queue, spreadsheet-import, sso, workflows.

They are component sources only. There is no entry point, no router, no bundler
config, and no `node_modules`. **Nothing here builds or runs yet** — establishing
that is P18-01, whose exit command is `cd apps/admin && npm ci && npm run build`.

The previous `package.json` scripts were placeholders that printed success without
executing anything, and were removed rather than left to report green.

## Scope of the port

Because the upstream dashboard is bundled into a Node Medusa server rather than
shipped as a generic client, wiring it to the Stheno engine is a **port, not a
clone**. It talks to `/admin/*` over `@medusajs/js-sdk`, which is why ADR-002's
wire-compatibility requirement is what makes the port possible at all.

RBAC and SSO screens are **not** taken from upstream — they are Medusa Enterprise
and are rebuilt clean-room per ADR-008 (P18-20, P18-21).
