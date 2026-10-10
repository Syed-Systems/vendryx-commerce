# Project Status

**Last consolidated:** 2026-09-13  
**Authority:** This file is the single source of truth for project completion status. Plans, task lists, handovers, and work logs may contain historical notes, but any current "how much is done" answer should start here.

---

## 1. Product Summary

Vendryx Commerce is split into two bodies of work:

| Area | Status | Current read |
|---|---:|---|
| Commerce engine, merchant admin, storefront, providers, couriers, branding, theme packages, hardening | **GA complete through Phase 3** | Phase 3 records **95 of 95 tasks complete** across P21-P28. |
| In-repo transitional control plane (`apps/platform`, `stheno-api-platform`) | **Partially complete** | Most routes exist, but some still need real backing behavior. |
| Long-term company control plane | **Moved to Space** | New company-wide control-plane work belongs in `D:\Syed Systems\Space`. This repo keeps the Vendryx transitional surface and connector baseline. |

---

## 2. Phase Status

| Phase group | Scope | Status | Evidence document |
|---|---|---:|---|
| P0-P10 | Core engine, storefront, admin parity baseline | Historical completed baseline | [TASKS.md](TASKS.md) |
| P11-P20 | Phase 2 durability, persistence, multi-tenant foundations | Historical completed baseline | [PHASE2_TASKS.md](archive/PHASE2_TASKS.md) |
| P21-P28 | Providers, BD gateways, couriers, COD settlement, branding, theme packages, GA hardening | **95 / 95 done** | [PHASE3_STATUS.md](archive/PHASE3_STATUS.md) |
| P29 | Control-plane operator identity and audit | **10 / 10 done** | [PHASE4_TASKS.md](PHASE4_TASKS.md) |
| P30-P36 | Billing, tenant lifecycle, oversight, analytics, support, config, hardening | **Not complete** | [PHASE4_TASKS.md](PHASE4_TASKS.md), [PLATFORM_COMPLETION_TASKS.md](PLATFORM_COMPLETION_TASKS.md) |

---

## 3. Phase 3 Detail

Phase 3 is closed:

| Phase | Done | Gate |
|---|---:|---|
| P21 provider platform | 14 / 14 | `./scripts/gate-p21.sh` |
| P22 BD gateways | 18 / 18 | `./scripts/gate-p22.sh` |
| P23 couriers | 13 / 13 | `./scripts/gate-p23.sh` |
| P24 COD settlement | 12 / 12 | `./scripts/gate-p24.sh` |
| P25 storefront core/theme split | 13 / 13 | `./scripts/gate-p25.sh` |
| P26 branding tier | 10 / 10 | `./scripts/gate-p26.sh` |
| P27 theme packages | 10 / 10 | `./scripts/gate-p27.sh` |
| P28 hardening, PCI, GA | 5 / 5 | `./scripts/gate-p28.sh` |

Current answer: **0 of 95 Phase 3 tasks remain.**

---

## 4. Control Plane Detail

The control-plane documents used to mix three different stories: original in-repo platform scope, Phase 4 remediation, and the later Space extraction. The current rule is:

- **Space** is the permanent company-wide control plane.
- **This repo's `apps/platform`** is transitional and should be completed only where it provides Vendryx operations or connector baseline value.
- **New company-wide features** go to `D:\Syed Systems\Space`, not into this repository.

Current transitional control-plane score:

| Measure | Value | Meaning |
|---|---:|---|
| Surface coverage | **59 / 63 (94%)** | Capabilities with a live route or screen. |
| Substance | **66 / 89 (74%)** | Handlers that read or write real data instead of returning literals. |

Known completion gap:

| Work | State |
|---|---|
| P29 operator identity and audit | Complete and verified. |
| P30 subscription billing | Started; status must be verified task-by-task before claiming complete. |
| Platform handler remediation | 23 handlers identified as the gap between route coverage and real data. |
| Long-term Space implementation | Tracked in the sibling Space project. |
| Commercial pricing/payment/courier/CDN plan | Source-of-truth plan written in [COMMERCIAL_IMPLEMENTATION_PLAN.md](COMMERCIAL_IMPLEMENTATION_PLAN.md); implementation remains in P30-P36/Space work. |

Authoritative next-work lists:

- [PHASE4_TASKS.md](PHASE4_TASKS.md) for P29-P36 task order.
- [PLATFORM_COMPLETION_TASKS.md](PLATFORM_COMPLETION_TASKS.md) for the 23 transitional platform handlers.
- `D:\Syed Systems\Space\docs\IMPLEMENTATION_BACKLOG.md` for new Space work.

---

## 5. Source-Of-Truth Rules

1. Update this file whenever a phase, task group, or project-wide completion percentage changes.
2. Do not repeat live completion percentages in plan documents. Link here instead.
3. Keep [WORK_LOG.md](WORK_LOG.md) chronological; it is evidence history, not current status.
4. Keep task files as acceptance criteria; they should say what must pass, not reinterpret current progress.
5. If this file conflicts with an ADR, the ADR wins for architecture and this file must be corrected.
