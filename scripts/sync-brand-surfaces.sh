#!/usr/bin/env bash
# scripts/sync-brand-surfaces.sh
# Synchronizes all customer-facing brand assets, tokens, and titles from
# Vendryx_Approved_Complete_Brand_Package/Vendryx_Brand_Kit (single source of truth).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

KIT_ROOT="Vendryx_Approved_Complete_Brand_Package/Vendryx_Brand_Kit"
if [ ! -d "$KIT_ROOT" ]; then
    echo "FAIL: Canonical brand kit not found at $KIT_ROOT" >&2
    exit 1
fi

MANIFEST="brand/vendryx/brand.json"
if [ ! -f "$MANIFEST" ]; then
    echo "FAIL: Brand manifest not found at $MANIFEST" >&2
    exit 1
fi

MODE="${1:-}"
if [ "$MODE" != "--write" ] && [ "$MODE" != "--check" ]; then
    echo "Usage: $0 --write | --check" >&2
    exit 1
fi

PRODUCT_NAME=$(grep -oE '"productName": "[^"]+"' "$MANIFEST" | head -n 1 | cut -d'"' -f4)
LEGAL_NAME=$(grep -oE '"legalName": "[^"]+"' "$MANIFEST" | head -n 1 | cut -d'"' -f4)
DOMAIN=$(grep -oE '"canonicalDomain": "[^"]+"' "$MANIFEST" | head -n 1 | cut -d'"' -f4)

echo "==> Canonical brand source: $KIT_ROOT"
echo "==> Brand: $PRODUCT_NAME ($LEGAL_NAME), domain: $DOMAIN"

DRIFT=0

sync_asset() {
    local src="$KIT_ROOT/$1"
    shift
    if [ ! -f "$src" ]; then
        echo "FAIL: Missing canonical asset: $src" >&2
        exit 1
    fi
    for dst in "$@"; do
        if [ ! -f "$dst" ] || ! cmp -s "$src" "$dst"; then
            DRIFT=1
            if [ "$MODE" == "--write" ]; then
                mkdir -p "$(dirname "$dst")"
                cp "$src" "$dst"
                echo "  Synced $dst <- $1"
            else
                echo "  Drift in $dst (does not match $src)"
            fi
        fi
    done
}

sync_asset "09_Developer/tokens.css" \
    "brand/vendryx/vendryx-tokens.css"
sync_asset "09_Developer/tokens.json" \
    "brand/vendryx/vendryx-tokens.json"

sync_asset "01_Primary_Logos/vendryx_primary_light_with_by_syed_systems.svg" \
    "brand/vendryx/vendryx-primary-light.svg" \
    "apps/admin/public/brand/vendryx/vendryx-primary-light.svg" \
    "apps/platform/public/brand/vendryx/vendryx-primary-light.svg" \
    "apps/marketing/public/brand/vendryx/vendryx-primary-light.svg" \
    "apps/marketing/public/branding/vendryx-primary-light.svg"

sync_asset "01_Primary_Logos/vendryx_primary_dark_with_by_syed_systems.svg" \
    "brand/vendryx/vendryx-primary-dark.svg" \
    "apps/admin/public/brand/vendryx/vendryx-primary-dark.svg" \
    "apps/platform/public/brand/vendryx/vendryx-primary-dark.svg" \
    "apps/marketing/public/brand/vendryx/vendryx-primary-dark.svg" \
    "apps/marketing/public/branding/vendryx-primary-dark.svg"

sync_asset "01_Primary_Logos/vendryx_primary_light_no_tagline_1200w.png" \
    "brand/vendryx/vendryx-logo.png" \
    "apps/admin/public/brand/vendryx/vendryx-logo.png" \
    "apps/platform/public/brand/vendryx/vendryx-logo.png" \
    "apps/marketing/public/brand/vendryx/vendryx-logo.png"

sync_asset "01_Primary_Logos/vendryx_primary_dark_no_tagline_1200w.png" \
    "brand/vendryx/vendryx-logo-dark.png" \
    "apps/admin/public/brand/vendryx/vendryx-logo-dark.png" \
    "apps/platform/public/brand/vendryx/vendryx-logo-dark.png" \
    "apps/marketing/public/brand/vendryx/vendryx-logo-dark.png"

sync_asset "01_Primary_Logos/vendryx_primary_light_with_by_syed_systems.svg" \
    "brand/vendryx/vendryx-with-tagline.svg" \
    "apps/marketing/public/brand/vendryx/vendryx-with-tagline.svg"

sync_asset "02_Secondary_Logos/vendryx_stacked_light_with_by_syed_systems.svg" \
    "brand/vendryx/vendryx-stacked-light.svg" \
    "apps/marketing/public/brand/vendryx/vendryx-stacked-light.svg"

sync_asset "02_Secondary_Logos/vendryx_stacked_dark_with_by_syed_systems.svg" \
    "brand/vendryx/vendryx-stacked-dark.svg" \
    "apps/marketing/public/brand/vendryx/vendryx-stacked-dark.svg"

sync_asset "04_Symbol/vendryx_tile_dark.svg" \
    "brand/vendryx/vendryx-mark.svg" \
    "apps/admin/public/brand/vendryx/vendryx-mark.svg" \
    "apps/platform/public/brand/vendryx/vendryx-mark.svg" \
    "apps/marketing/public/brand/vendryx/vendryx-mark.svg" \
    "apps/marketing/public/branding/vendryx-mark-dark.svg"

sync_asset "04_Symbol/vendryx_tile_dark_512w.png" \
    "brand/vendryx/vendryx-mark.png" \
    "apps/admin/public/brand/vendryx/vendryx-mark.png" \
    "apps/platform/public/brand/vendryx/vendryx-mark.png" \
    "apps/marketing/public/brand/vendryx/vendryx-mark.png"

sync_asset "04_Symbol/vendryx_mark_black_transparent.svg" \
    "brand/vendryx/vendryx-black.svg"
sync_asset "04_Symbol/vendryx_mark_white_transparent.svg" \
    "brand/vendryx/vendryx-white.svg"

sync_asset "05_Favicons/favicon.ico" \
    "brand/vendryx/favicon.ico" \
    "apps/admin/public/favicon.ico" \
    "apps/admin/public/brand/vendryx/favicon.ico" \
    "apps/platform/public/favicon.ico" \
    "apps/platform/public/brand/vendryx/favicon.ico" \
    "apps/marketing/public/favicon.ico" \
    "apps/marketing/public/brand/vendryx/favicon.ico"

sync_asset "05_Favicons/favicon.svg" \
    "brand/vendryx/favicon.svg" \
    "apps/admin/public/brand/vendryx/favicon.svg" \
    "apps/platform/public/brand/vendryx/favicon.svg" \
    "apps/marketing/public/brand/vendryx/favicon.svg" \
    "apps/marketing/public/branding/favicon.svg"

sync_asset "05_Favicons/favicon-16x16.png" \
    "apps/admin/public/brand/vendryx/favicon-16x16.png" \
    "apps/platform/public/brand/vendryx/favicon-16x16.png" \
    "apps/marketing/public/brand/vendryx/favicon-16x16.png"
sync_asset "05_Favicons/favicon-32x32.png" \
    "apps/admin/public/brand/vendryx/favicon-32x32.png" \
    "apps/platform/public/brand/vendryx/favicon-32x32.png" \
    "apps/marketing/public/brand/vendryx/favicon-32x32.png"
sync_asset "05_Favicons/favicon-48x48.png" \
    "apps/admin/public/brand/vendryx/favicon-48x48.png" \
    "apps/platform/public/brand/vendryx/favicon-48x48.png" \
    "apps/marketing/public/brand/vendryx/favicon-48x48.png"
sync_asset "05_Favicons/favicon-64x64.png" \
    "apps/admin/public/brand/vendryx/favicon-64x64.png" \
    "apps/platform/public/brand/vendryx/favicon-64x64.png" \
    "apps/marketing/public/brand/vendryx/favicon-64x64.png"
sync_asset "05_Favicons/favicon-96x96.png" \
    "apps/admin/public/brand/vendryx/favicon-96x96.png" \
    "apps/platform/public/brand/vendryx/favicon-96x96.png" \
    "apps/marketing/public/brand/vendryx/favicon-96x96.png"

sync_asset "04_Symbol/vendryx_tile_dark_512w.png" \
    "brand/vendryx/favicon.png" \
    "apps/admin/public/favicon.png" \
    "apps/admin/public/brand/vendryx/favicon.png" \
    "apps/platform/public/favicon.png" \
    "apps/platform/public/brand/vendryx/favicon.png" \
    "apps/marketing/public/favicon.png" \
    "apps/marketing/public/brand/vendryx/favicon.png"
sync_asset "04_Symbol/vendryx_tile_dark_128w.png" \
    "apps/admin/public/brand/vendryx/favicon-128x128.png" \
    "apps/platform/public/brand/vendryx/favicon-128x128.png" \
    "apps/marketing/public/brand/vendryx/favicon-128x128.png"
sync_asset "04_Symbol/vendryx_tile_dark_256w.png" \
    "apps/admin/public/brand/vendryx/favicon-256x256.png" \
    "apps/platform/public/brand/vendryx/favicon-256x256.png" \
    "apps/marketing/public/brand/vendryx/favicon-256x256.png"
sync_asset "04_Symbol/vendryx_tile_dark_512w.png" \
    "apps/admin/public/brand/vendryx/favicon-512x512.png" \
    "apps/platform/public/brand/vendryx/favicon-512x512.png" \
    "apps/marketing/public/brand/vendryx/favicon-512x512.png"

sync_asset "06_App_PWA/app-icon-dark-180.png" \
    "brand/vendryx/apple-touch-icon.png" \
    "apps/admin/public/apple-touch-icon.png" \
    "apps/admin/public/brand/vendryx/apple-touch-icon.png" \
    "apps/platform/public/apple-touch-icon.png" \
    "apps/platform/public/brand/vendryx/apple-touch-icon.png" \
    "apps/marketing/public/apple-touch-icon.png" \
    "apps/marketing/public/brand/vendryx/apple-touch-icon.png" \
    "apps/marketing/public/branding/apple-touch-icon.png"

sync_asset "06_App_PWA/app-icon-dark-192.png" \
    "brand/vendryx/app-icon-192.png" \
    "apps/admin/public/brand/vendryx/app-icon-192.png" \
    "apps/platform/public/brand/vendryx/app-icon-192.png" \
    "apps/marketing/public/brand/vendryx/app-icon-192.png"
sync_asset "06_App_PWA/app-icon-dark-512.png" \
    "brand/vendryx/app-icon-512.png" \
    "apps/admin/public/brand/vendryx/app-icon-512.png" \
    "apps/platform/public/brand/vendryx/app-icon-512.png" \
    "apps/marketing/public/brand/vendryx/app-icon-512.png"
sync_asset "06_App_PWA/maskable-icon-512.png" \
    "brand/vendryx/maskable-icon-512.png" \
    "apps/admin/public/brand/vendryx/maskable-icon-512.png" \
    "apps/platform/public/brand/vendryx/maskable-icon-512.png" \
    "apps/marketing/public/brand/vendryx/maskable-icon-512.png"

sync_asset "07_Social_Marketing/open-graph.png" \
    "apps/marketing/public/brand/vendryx/open-graph-1200x630.png" \
    "apps/marketing/public/branding/open-graph.png"
sync_asset "07_Social_Marketing/open-graph.svg" \
    "apps/marketing/public/brand/vendryx/open-graph-1200x630.svg"

check_title() {
    local target="$1"
    local desired="$2"
    if [ ! -f "$target" ]; then return; fi
    local current
    current=$(grep -oE '<title>[^<]+</title>' "$target" | sed -E 's/<title>(.*)<\/title>/\1/' || echo "")
    if [ "$current" != "$desired" ]; then
        DRIFT=1
        if [ "$MODE" == "--write" ]; then
            sed -i -E "s|<title>[^<]+</title>|<title>$desired</title>|g" "$target"
            echo "  Updated title in $target -> $desired"
        else
            echo "  Drift in $target: title is '$current', expected '$desired'"
        fi
    fi
}

check_title "apps/marketing/index.html" "$PRODUCT_NAME | Your Online Store, Your Brand"
check_title "apps/admin/index.html" "$PRODUCT_NAME Dashboard"
check_title "apps/platform/index.html" "$PRODUCT_NAME Platform"

if [ "$MODE" == "--check" ]; then
    if [ "$DRIFT" -ne 0 ]; then
        echo "FAIL: Brand surface drift detected. Run $0 --write to synchronize." >&2
        exit 1
    else
        echo "Brand surfaces check PASSED: All targets match $KIT_ROOT"
        exit 0
    fi
else
    echo "Brand surfaces synchronized successfully."
    exit 0
fi
