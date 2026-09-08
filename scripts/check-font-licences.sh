#!/usr/bin/env bash
# scripts/check-font-licences.sh
# P26-04: Curated font set licences and self-hosted enforcement.
#
# Validates that:
# 1. docs/FONTS.md exists and registers all curated font keys with their open source licences.
# 2. Only approved font licences (OFL-1.1, Apache-2.0, MIT) are recorded.
# 3. No external font CDN URLs (fonts.googleapis.com, etc.) are loaded on checkout routes.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> Checking curated font set licences (P26-04)..."

FONTS_DOC="docs/FONTS.md"
if [ ! -f "$FONTS_DOC" ]; then
    echo "FAIL: $FONTS_DOC does not exist." >&2
    exit 1
fi

REQUIRED_FONTS=("inter" "geist" "noto_sans_bengali" "playfair" "plus_jakarta_sans" "roboto" "cinzel")

for font_key in "${REQUIRED_FONTS[@]}"; do
    if ! grep -q "\`$font_key\`" "$FONTS_DOC"; then
        echo "FAIL: Required font key '$font_key' is missing from $FONTS_DOC" >&2
        exit 1
    fi
done

# Ensure every listed font has an approved open licence
UNAPPROVED=$(grep -E '\|\s*`[a-z_]+`\s*\|' "$FONTS_DOC" | grep -v -E '(SIL Open Font License 1\.1|Apache License 2\.0|MIT)' || true)
if [ -n "$UNAPPROVED" ]; then
    echo "FAIL: Found unapproved or missing font licence in $FONTS_DOC:" >&2
    echo "$UNAPPROVED" >&2
    exit 1
fi

# Ensure checkout routes never reference third-party font CDNs (ADR-013 Decision 2a)
CHECKOUT_DIR="apps/storefront"
if [ -d "$CHECKOUT_DIR" ]; then
    CDN_LEAKS=$(grep -rnE '(fonts\.googleapis\.com|fonts\.gstatic\.com|use\.typekit\.net)' "$CHECKOUT_DIR/src/checkout" 2>/dev/null || true)
    if [ -n "$CDN_LEAKS" ]; then
        echo "FAIL: Third-party font CDN reference found on checkout surface:" >&2
        echo "$CDN_LEAKS" >&2
        exit 1
    fi
fi

echo "Curated font set licences PASSED: All keys registered, licence-cleared, and self-hosted."
exit 0
