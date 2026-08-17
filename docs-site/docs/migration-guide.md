# Medusa to Stheno Migration Guide

Stheno provides native tooling to migrate existing Medusa v2 databases with 100% entity and financial fidelity.

---

## 1. Migration Steps

### Step 1: Run Dry-Run Simulation
Inspect source database schema and generate a migration fidelity report:
```bash
cargo run -p stheno-cli -- import-medusa --dry-run --source "postgres://user:pass@localhost:5432/medusa_db"
```

### Step 2: Execute Migration
Migrate all entities with IDs preserved identically:
```bash
cargo run -p stheno-cli -- import-medusa --source "postgres://user:pass@localhost:5432/medusa_db"
```

### Step 3: Rebuild Search & Indexes
```bash
cargo run -p stheno-cli -- reindex
```

### Step 4: Verify Financial Totals
Run automated reconciliation assertions across all orders, payment captures, and refunds.
