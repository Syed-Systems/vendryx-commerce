# Documentation & Version Logging Workflow Protocol

**Governed by:** [AGENTS.md](../AGENTS.md) · [DECISIONS.md ADR-028](DECISIONS.md#adr-028) · [WORK_LOG.md](WORK_LOG.md)

This protocol establishes the continuous discipline for keeping all documentation 100% faithful to
the codebase, preventing unsourced or contradictory figures from entering public surfaces, and
recording chronological version-by-version audit logs of working increments.

---

## 1. The Verification Gate

Before concluding any development increment or PR, run the documentation synchronization check:

```bash
python scripts/doc-sync-workflow.py check
```
*(On Windows PowerShell: `.\scripts\doc-sync-workflow.ps1 check`)*

This gate asserts five invariants:

1. **Pricing & Quota Invariants:**
   Every public marketing surface (`apps/marketing/public/llms.txt`, `llms-full.txt`, `index.html`,
   `App.tsx`) must match the canonical quotas in [config/tiers.toml](../config/tiers.toml) and
   [docs/ENTITLEMENTS.md](ENTITLEMENTS.md):
   - Free Tier: `max_products = 100`, `max_staff_seats = 3`, `analytics_window_days = 30`
   - Pro Tier: `price = 1500.00` BDT/mo, `max_products = 0` (unlimited), `max_staff_seats = 10`
   - Fabricated numbers (e.g. `2,800` or `50 products`) fail the build immediately.

2. **Module Coverage:**
   Every crate under `crates/modules/` must be documented in [docs/TRD.md](TRD.md) or
   [docs/DATA_MODEL.md](DATA_MODEL.md). New crates cannot be added without their schema and
   boundary specification.

3. **Provider Synchronization:**
   Executes `cargo run -p stheno-cli -- provider-docs --check` to ensure the compiled provider
   registry in Rust matches [docs/providers/PAYMENT_GATEWAYS.md](providers/PAYMENT_GATEWAYS.md).

4. **Brand Surface Invariants:**
   Executes `scripts/check-brand-surface.ps1` to assert that all customer-facing surfaces display
   **Vendryx** (operated by Syed Systems) per ADR-028, without leaking internal engine codenames.

5. **Relative Link Integrity:**
   Scans all 46+ documentation files to ensure that every relative Markdown link resolves to an
   existing target file.

---

## 2. Version Logging Protocol

Whenever a task, feature increment, or phase milestone is completed, record the change in the
version ledger:

```bash
python scripts/doc-sync-workflow.py log \
  --version "vX.Y.Z" \
  --summary "Concise description of the working increment" \
  --surfaces "Engine, API, Admin, Marketing" \
  --gates "doc-sync-workflow check, openapi check, cargo test"
```

This updates [docs/WORK_LOG.md](WORK_LOG.md) chronologically, maintaining an immutable ledger
of what was changed, what was verified, and what quality gates passed.

---

## 3. Versioning Strategy

- **Major (`vX.0.0`)**: Breaking wire contract modifications, core frontend contract version bumps
  ([docs/FRONTEND_VERSIONING.md](FRONTEND_VERSIONING.md)), or platform architecture migrations.
- **Minor (`vX.Y.0`)**: New domain modules, additional payment providers or courier integrations,
  new customer-facing pages or merchant dashboard features.
- **Patch (`vX.Y.Z`)**: Documentation synchronizations, bug fixes, schema expansions, performance
  tuning, or UI token polishes.

---

## 4. Integration with CI & Quality Gates

The documentation check is part of the standard pre-commit and CI verification suite:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p stheno-cli -- openapi --check
python scripts/check-no-fabricated-handlers.py
python scripts/doc-sync-workflow.py check
```
