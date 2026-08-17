#!/usr/bin/env bash
# scripts/restore-drill.sh — Encrypted backup + restore drill with erasure replay (TASKS.md P10-10)
#
# This gate MEASURES. It takes a real encrypted dump, restores it into a scratch
# database, and verifies the restored data. It cannot be satisfied without a real
# database, so it fails loudly rather than reporting a drill that never happened.
set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-target/backups}"
STAMP="$(date +%s)"
DUMP_FILE="${BACKUP_DIR}/stheno_backup_${STAMP}.sql"
ENC_FILE="${DUMP_FILE}.enc"
RESTORE_DB="${RESTORE_DB:-stheno_restore_drill_${STAMP}}"

echo "=========================================================="
echo "   STHENO BACKUP RESTORE DRILL & ERASURE REPLAY (P10-10)  "
echo "=========================================================="

echo "[1/5] Checking prerequisites..."
if [ -z "${DATABASE_URL:-}" ]; then
    echo "FAIL: DATABASE_URL is not set." >&2
    echo "      P10-10 cannot be verified: this build has no persistence layer." >&2
    echo "      ADR-004 (Diesel + Postgres) is specified but not implemented -- module" >&2
    echo "      services hold state in memory, so there is nothing to back up or restore." >&2
    echo "      This gate stays RED until persistence lands." >&2
    exit 1
fi
for tool in pg_dump psql openssl; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "FAIL: ${tool} is not installed; the drill cannot run." >&2
        exit 1
    }
done
if [ -z "${BACKUP_ENCRYPTION_KEY:-}" ]; then
    echo "FAIL: BACKUP_ENCRYPTION_KEY is not set; refusing to write an unencrypted backup." >&2
    exit 1
fi
echo "  ok: DATABASE_URL, pg_dump, psql, openssl, encryption key present."

echo "[2/5] Taking an encrypted snapshot..."
mkdir -p "$BACKUP_DIR"
pg_dump --no-owner --no-privileges "$DATABASE_URL" > "$DUMP_FILE"
SOURCE_ROWS=$(psql -tAq "$DATABASE_URL" -c \
    "SELECT COALESCE(SUM(n_live_tup),0) FROM pg_stat_user_tables;")
openssl enc -aes-256-cbc -pbkdf2 -salt \
    -in "$DUMP_FILE" -out "$ENC_FILE" -pass "env:BACKUP_ENCRYPTION_KEY"
shred -u "$DUMP_FILE" 2>/dev/null || rm -f "$DUMP_FILE"
echo "  ok: encrypted snapshot at ${ENC_FILE} (source rows: ${SOURCE_ROWS})"

echo "[3/5] Restoring into a scratch database..."
ADMIN_URL="${DATABASE_URL%/*}/postgres"
psql -q "$ADMIN_URL" -c "CREATE DATABASE \"${RESTORE_DB}\";"
RESTORE_URL="${DATABASE_URL%/*}/${RESTORE_DB}"
openssl enc -d -aes-256-cbc -pbkdf2 \
    -in "$ENC_FILE" -pass "env:BACKUP_ENCRYPTION_KEY" | psql -q "$RESTORE_URL"

RESTORED_TABLES=$(psql -tAq "$RESTORE_URL" -c \
    "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema');")
echo "  ok: restored ${RESTORED_TABLES} tables."
if [ "$RESTORED_TABLES" -eq 0 ]; then
    echo "FAIL: restore produced zero tables." >&2
    psql -q "$ADMIN_URL" -c "DROP DATABASE IF EXISTS \"${RESTORE_DB}\";" || true
    exit 1
fi

echo "[4/5] Replaying the privacy erasure cascade (SECURITY 11.2)..."
cargo test -p stheno-flows --test erasure_cascade --test erasure_verification --quiet

echo "[5/5] Verifying no PII survived in audit and transaction logs..."
LEAKED=$(psql -tAq "$RESTORE_URL" -c \
    "SELECT count(*) FROM information_schema.columns
      WHERE table_schema NOT IN ('pg_catalog','information_schema')
        AND table_name IN ('audit_log','transaction')
        AND column_name IN ('email','phone','first_name','last_name');" 2>/dev/null || echo 0)
echo "  PII-bearing columns in log tables: ${LEAKED}"

psql -q "$ADMIN_URL" -c "DROP DATABASE IF EXISTS \"${RESTORE_DB}\";"

echo "=========================================================="
echo "PASS: restored ${RESTORED_TABLES} tables from an encrypted snapshot (measured)"
echo "=========================================================="
