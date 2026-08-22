# Frontend Versioning Policy

**Status:** Accepted · **Date:** 2026-08-20 · **Task:** P25-11 ·
**Governed by:** [ADR-013](DECISIONS.md#adr-013)

ADR-013 names this as one of the two costs it accepted:

> **Themes must be versioned against core.** Additive-within-a-major is a discipline, not a
> mechanism — nothing in the type system enforces it. A core major bump is an N-frontend migration,
> and that cost grows with every frontend sold.

This document is the discipline. `scripts/check-frontend-pin.sh` is the part of it a build can check.

---

## 1. The rule

**A frontend pins one `storefront-core` major. Core is additive within a major.**

| Change to core | Allowed in | Why |
|---|---|---|
| New optional slot in `FrontendContract` | minor | Existing frontends still type-check |
| New optional prop on an existing slot | minor | Existing components ignore it |
| New required slot | **major** | Every frontend must now supply it |
| Removing or renaming a slot | **major** | Silently drops a surface |
| Making an optional prop required | **major** | Existing components break |
| Tightening a prop's type | **major** | Previously valid frontends stop compiling |
| Widening a prop's type | minor | Existing components still accept it |
| Route, data or checkout changes | any | Not frontend surface — [ADR-013](DECISIONS.md#adr-013) decision 4 |

The last row is the point of the split. Most engine work is invisible to frontends, which is what
makes the tenth frontend cost what the second did.

---

## 2. How a frontend pins

Every frontend package declares the core major it was built against:

```json
{
  "name": "@stheno/frontend-default",
  "peerDependencies": { "@stheno/storefront-core": "^0.1.0" }
}
```

`peerDependencies`, not `dependencies` — a frontend does not carry its own copy of core. There is one
core in the runtime, and the pin declares which one the frontend expects to find.

`scripts/check-frontend-pin.sh` fails the build when a frontend package omits the pin, or when its pinned
major does not match the core actually present in the workspace. That is mechanical enforcement of
the *pin*; it does not and cannot enforce that a minor was truly additive. That part stays human,
which is exactly what ADR-013 warned about.

---

## 3. What a major bump costs

A core major is an N-frontend migration, and N grows with sales. Before proposing one:

1. **Check the change is really a major.** Most "we need a new component" changes are a new
   *optional* slot with a core-supplied default, which is a minor.
2. **Count the frontends.** Every Pro tenant on the old major needs a frontend update and a deploy.
3. **Do not batch a major with unrelated work.** A major that also changes checkout behaviour makes
   the migration impossible to review.

The P27-07 core-delta budget exists for the same reason from the other direction: if building
frontend B needs more than two core changes, the contract was drawn wrong, and fixing the contract is
cheaper than shipping a major.

---

## 4. Pre-1.0

`storefront-core` is `0.x` while P25–P27 are in flight, and `0.x` minors may break. The pin is
enforced from now regardless, so the discipline is in place before the first external frontend exists
— which is the only time it is cheap to establish.

The first frontend built by someone outside this repo is what makes `1.0` meaningful. Until then the
policy is being rehearsed, not relied on.
