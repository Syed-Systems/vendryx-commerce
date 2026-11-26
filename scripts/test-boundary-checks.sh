#!/usr/bin/env bash
set -euo pipefail

echo "==> Running negative boundary check harness (P0-25)..."

TEMP_FILE="crates/modules/stheno-cart/src/temp_violation.rs"
TEMP_SQL="crates/modules/stheno-cart/migrations/temp_violation.sql"
KERNEL_VIOLATION="crates/core/stheno-types/src/temp_kernel.rs"

cleanup() {
    rm -f "$TEMP_FILE" "$TEMP_SQL" "$KERNEL_VIOLATION"
}
trap cleanup EXIT

# 1. Test check-sql-boundaries fails on cross-schema SQL
echo "Testing check-sql-boundaries on planted violation..."
echo 'pub fn bad() { let _ = "sql_query SELECT * FROM pricing.price"; }' > "$TEMP_FILE"
if ./scripts/check-sql-boundaries.sh > /dev/null 2>&1; then
    echo "FAILED: check-sql-boundaries did not catch planted violation!"
    exit 1
fi
rm -f "$TEMP_FILE"
echo "  Passed: check-sql-boundaries properly failed on violation."

# 2. Test check-shared-kernel fails on domain type in kernel
echo "Testing check-shared-kernel on planted violation..."
echo "pub struct Product { pub id: String }" > "$KERNEL_VIOLATION"
if ./scripts/check-shared-kernel.sh > /dev/null 2>&1; then
    echo "FAILED: check-shared-kernel did not catch planted violation!"
    exit 1
fi
rm -f "$KERNEL_VIOLATION"
echo "  Passed: check-shared-kernel properly failed on violation."

# 3. Test check-no-domain-gates fails on Entitlements in module
echo "Testing check-no-domain-gates on planted violation..."
echo "pub fn check_tier(e: &Entitlements) {}" > "$TEMP_FILE"
if ./scripts/check-no-domain-gates.sh > /dev/null 2>&1; then
    echo "FAILED: check-no-domain-gates did not catch planted violation!"
    exit 1
fi
rm -f "$TEMP_FILE"
echo "  Passed: check-no-domain-gates properly failed on violation."

# 4. Test check-migration-phases fails on unphased DROP COLUMN
echo "Testing check-migration-phases on planted violation..."
mkdir -p crates/modules/stheno-cart/migrations
echo "ALTER TABLE cart DROP COLUMN customer_id;" > "$TEMP_SQL"
if ./scripts/check-migration-phases.sh > /dev/null 2>&1; then
    echo "FAILED: check-migration-phases did not catch planted violation!"
    exit 1
fi
rm -f "$TEMP_SQL"
echo "  Passed: check-migration-phases properly failed on violation."

echo "All negative boundary tests PASSED: All boundary enforcement scripts are verified active and strict."
exit 0
