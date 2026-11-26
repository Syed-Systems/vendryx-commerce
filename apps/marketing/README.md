# Vendryx Marketing & Interactive Showcase

> Public marketing application and interactive merchant simulator for Vendryx Commerce, built with React 18, Vite, Tailwind CSS 4, Motion, and TypeScript. Governed by [ADR-028](../../docs/DECISIONS.md#adr-028), [docs/BRAND.md](../../docs/BRAND.md), and [docs/DOCS_WORKFLOW.md](../../docs/DOCS_WORKFLOW.md).

---

## 1. Overview & Capabilities

The Vendryx marketing application provides prospective merchants with an authoritative, interactive overview of the Vendryx Commerce Operating System. It communicates who the product is built for (social commerce merchants operating across Facebook, Instagram, and WhatsApp) and enables them to simulate store operations directly in the browser.

### Key Capabilities

1. **Interactive Commerce Simulator (`src/components/demo/`):**
   - Live client-side simulation showcasing catalog creation, social checkout generation, and courier dispatch.
   - Demonstrates the end-to-end flow from social chat message to automated delivery consignment.
2. **Canonical Pricing Calculator:**
   - Displays real-time subscription pricing strictly mapped to canonical truth in `config/tiers.toml` and [COMMERCIAL_IMPLEMENTATION_PLAN.md](../../docs/COMMERCIAL_IMPLEMENTATION_PLAN.md):
     - **Free Tier:** 100 products, 3 staff seats, 30-day analytics window, unlimited orders, 0% platform transaction fees.
     - **Pro Tier:** USD 20.00 / month (or local BDT equivalent), unlimited products, 10 staff seats, full gateway & courier integrations, and a **10% flat discount** for terms of 12+ months.
3. **AI Search & Discoverability (`public/llms.txt`):**
   - Exposes structured markdown manifests (`llms.txt` and `llms-full.txt`) for AI crawlers and LLM search agents (ChatGPT, Claude, Perplexity), strictly aligned with platform facts.
4. **Corporate Brand Compliance ([docs/BRAND.md](../../docs/BRAND.md)):**
   - Uses the official Vendryx corporate color palette: Indigo (`#4F46E5`), Violet (`#5B21B6`), Bright Blue (`#3B82F6`), and Corporate Navy (`#0F172A`).
   - Enforces the **NO UI GRADIENTS** rule: Clean, flat, high-contrast layouts powered by generous whitespace and Plus Jakarta Sans typography.

---

## 2. Directory Structure

```
apps/marketing/
├── index.html                            # Application entry point with metadata & OpenGraph tags
├── package.json                          # Package scripts and dependencies
├── vite.config.ts                        # Vite bundler configuration
├── tsconfig.json                         # TypeScript configuration
├── public/
│   ├── llms.txt                          # Concise AI crawler summary
│   ├── llms-full.txt                     # Detailed platform capabilities for AI search
│   ├── robots.txt                        # Crawler access directives
│   └── brand/                            # Corporate logos and favicons
└── src/
    ├── App.tsx                           # Main landing page composition
    ├── components/                       # UI primitives, hero, feature rails, pricing tables
    │   ├── demo/                         # Interactive merchant operational simulator
    │   └── ui/                           # Accessible buttons, badges, dialogs
    ├── data/                             # Canonical tier data and feature comparison matrices
    └── styles/                           # Tailwind CSS 4 style layers
```

---

## 3. Environment Variables

Create `.env.local` in `apps/marketing/`:

```bash
# Public marketing app port (default 5175 or 3000)
PORT=5175

# Optional: Live merchant admin URL for "Get Started" redirects
VITE_ADMIN_APP_URL="http://localhost:5173"

# Optional: Storefront preview URL for live demo links
VITE_STOREFRONT_URL="http://localhost:8000"
```

---

## 4. Development & Build Commands

All commands are run from the `apps/marketing/` directory:

| Command | Action |
|---|---|
| `npm run dev` | Starts local Vite dev server on `http://127.0.0.1:5175` |
| `npm run build` | Compiles production assets to `dist/` |
| `npm run typecheck` | Validates TypeScript types (`tsc --noEmit`) |
| `npm test` | Runs interactive demo session test suite |

---

## 5. Verification Invariants

The marketing site is gated by automated CI checks:
- **Pricing Invariants:** Verified by `python scripts/doc-sync-workflow.py check`. Any modification to prices or quotas that diverges from `config/tiers.toml` will fail CI immediately.
- **Brand Surfaces:** Verified by `powershell -ExecutionPolicy Bypass -File scripts/check-brand-surface.ps1`. Public surfaces must strictly present **Vendryx** per ADR-028.
