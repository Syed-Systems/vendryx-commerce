#!/usr/bin/env node
/**
 * Storefront Lighthouse / CWV — NOT IMPLEMENTED.
 *
 * This file previously printed a hardcoded array — "Performance 98/100", "LCP 1.1s",
 * "Accessibility 100/100" — and exited zero. No browser was launched and nothing was
 * measured. PHASE2_TASKS P19-13 requires "Lighthouse >= 95 and CWV gates on the PDP,
 * measured"; those numbers were typed, not measured.
 *
 * Exits non-zero until a real run exists. `seo-unlighthouse` (Unlighthouse CLI) or
 * `lighthouse` against a built storefront are both reasonable ways to do it.
 */
console.error(
  "storefront lighthouse audit is not implemented — the previous runner printed fabricated metrics.\n" +
    "P19-13 requires a measured run."
)
process.exit(1)
