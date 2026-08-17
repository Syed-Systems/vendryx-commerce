# Module Authoring Guide

Stheno enforces strict module isolation (AGENTS.md §2). Each domain module is a standalone leaf crate owning its private PostgreSQL schema.

---

## 1. Scaffolding a Module

```bash
cargo run -p stheno-cli -- new-module loyalty
```

This generates:
- `MODULE.toml`: Declares owned tables, published events, and links.
- `src/models.rs`: Entities defined as `pub(crate)`.
- `src/service.rs`: Service implementation with explicit `&mut Conn<'_>`.
- `src/dto.rs`: Public DTOs for safe boundary crossing.
- `src/events.rs`: Published domain events.
- `migrations/`: Module's dedicated schema migrations.

---

## 2. The Four Interaction Channels
Modules never depend directly on other domain modules. Cross-module communication uses:
1. **Workflows**: Multi-step orchestrations in `crates/flows/`.
2. **Queries**: Cross-module reads through `stheno-query`.
3. **Events**: Asynchronous notifications dispatched to `crates/subscribers/`.
4. **Links**: Relationship associations defined via `stheno-links`.
