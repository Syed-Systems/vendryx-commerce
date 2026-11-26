#!/usr/bin/env bash
set -euo pipefail

GROUP="all"
while [[ $# -gt 0 ]]; do
  case $1 in
    --group)
      GROUP="$2"
      shift 2
      ;;
    --smoke|--record|--replay)
      shift
      ;;
    *)
      shift
      ;;
  esac
done

echo "=== Running Parity Suite (Group: ${GROUP}) ==="

if [[ "${GROUP}" == "store-catalog" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-store --test catalog_parity
fi

if [[ "${GROUP}" == "store-pricing" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-store --test pricing_parity
fi

if [[ "${GROUP}" == "store-cart" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-store --test cart_parity
fi

if [[ "${GROUP}" == "store-order" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-store --test order_parity
fi

if [[ "${GROUP}" == "store-returns" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-store --test return_parity
fi

if [[ "${GROUP}" == "admin-catalog" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_catalog
fi

if [[ "${GROUP}" == "admin-inventory" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_inventory
fi

if [[ "${GROUP}" == "admin-pricing" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_pricing
fi

if [[ "${GROUP}" == "admin-orders" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_orders
fi

if [[ "${GROUP}" == "admin-order-mutations" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_order_mutations
fi

if [[ "${GROUP}" == "admin-draft-orders" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_draft_orders
fi

if [[ "${GROUP}" == "admin-fulfillment" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_fulfillment
fi

if [[ "${GROUP}" == "admin-payments" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_payments
fi

if [[ "${GROUP}" == "admin-customers" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_customers
fi

if [[ "${GROUP}" == "admin-marketing" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_marketing
fi

if [[ "${GROUP}" == "admin-config" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_config
fi

if [[ "${GROUP}" == "admin-access" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_access
fi

if [[ "${GROUP}" == "admin-platform" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_platform
fi

if [[ "${GROUP}" == "admin-feature-flags" || "${GROUP}" == "all" ]]; then
  cargo test -p stheno-api-admin --test admin_parity test_admin_feature_flags
fi

echo "=== Parity Suite PASSED ==="
