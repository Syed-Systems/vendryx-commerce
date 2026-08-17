#!/usr/bin/env bash
# scripts/soak.sh — Sustained load soak and memory budget check (TASKS.md P10-14)
#
# This gate MEASURES. It drives real traffic at the configured rate against a running
# stheno-server and samples that process's real RSS. It reports what it observed.
#
# Usage: ./scripts/soak.sh --duration 1h --rps 500
set -euo pipefail

DURATION_VAL="1h"
RPS_VAL=500
TARGET_URL="${TARGET_URL:-http://127.0.0.1:9000}"
RSS_BUDGET_MB=400

while [[ $# -gt 0 ]]; do
    case "$1" in
        --duration) DURATION_VAL="$2"; shift 2 ;;
        --rps)      RPS_VAL="$2";      shift 2 ;;
        *)          shift ;;
    esac
done

# Normalise 1h / 30m / 90s into seconds.
case "$DURATION_VAL" in
    *h) DURATION_SEC=$(( ${DURATION_VAL%h} * 3600 )) ;;
    *m) DURATION_SEC=$(( ${DURATION_VAL%m} * 60 )) ;;
    *s) DURATION_SEC=${DURATION_VAL%s} ;;
    *)  DURATION_SEC="$DURATION_VAL" ;;
esac

echo "=========================================================="
echo "    STHENO SOAK & SUSTAINED LOAD TEST (P10-14)            "
echo "=========================================================="
echo "Target = ${TARGET_URL} | Rate = ${RPS_VAL} rps | Duration = ${DURATION_VAL} (${DURATION_SEC}s)"

echo "[1/4] Checking the target is up..."
if ! curl -fsS -m 5 "${TARGET_URL}/health" >/dev/null 2>&1; then
    echo "FAIL: no server answering at ${TARGET_URL}/health." >&2
    echo "      Start it first:  cargo run -p stheno-server -- --bind 127.0.0.1:9000" >&2
    exit 1
fi
echo "  ok: target healthy."

echo "[2/4] Locating the server process to sample RSS from..."
SERVER_PID=""
if command -v powershell.exe >/dev/null 2>&1; then
    SERVER_PID=$(powershell.exe -NoProfile -Command \
        "(Get-Process stheno-server -ErrorAction SilentlyContinue | Select-Object -First 1).Id" \
        2>/dev/null | tr -d '\r\n ')
elif command -v pgrep >/dev/null 2>&1; then
    SERVER_PID=$(pgrep -n stheno-server || true)
fi
if [ -z "$SERVER_PID" ]; then
    echo "FAIL: could not find the stheno-server process, so RSS cannot be measured." >&2
    echo "      P10-14 is UNVERIFIED without a real memory sample." >&2
    exit 1
fi
echo "  ok: sampling pid ${SERVER_PID}."

echo "[3/4] Driving ${RPS_VAL} rps for ${DURATION_SEC}s..."
PEAK_RSS_MB=0
sample_rss() {
    if command -v powershell.exe >/dev/null 2>&1; then
        powershell.exe -NoProfile -Command \
            "[math]::Round((Get-Process -Id ${SERVER_PID} -ErrorAction SilentlyContinue).WorkingSet64/1MB)" \
            2>/dev/null | tr -d '\r\n '
    else
        ps -o rss= -p "$SERVER_PID" 2>/dev/null | awk '{print int($1/1024)}'
    fi
}

if command -v k6 >/dev/null 2>&1; then
    k6 run --quiet --vus 50 --duration "${DURATION_SEC}s" tests/load/all.js &
else
    echo "  note: k6 not installed, using scripts/load-driver.js"
    node scripts/load-driver.js --url "$TARGET_URL" --rps "$RPS_VAL" \
        --duration "$DURATION_SEC" > /tmp/stheno-soak-result.json &
fi
LOAD_PID=$!

while kill -0 "$LOAD_PID" 2>/dev/null; do
    RSS=$(sample_rss)
    if [ -n "$RSS" ] && [ "$RSS" -gt "$PEAK_RSS_MB" ] 2>/dev/null; then
        PEAK_RSS_MB="$RSS"
    fi
    sleep 5
done

LOAD_STATUS=0
wait "$LOAD_PID" || LOAD_STATUS=$?

if [ -f /tmp/stheno-soak-result.json ]; then
    echo "  driver result: $(cat /tmp/stheno-soak-result.json)"
fi
if [ "$LOAD_STATUS" -ne 0 ]; then
    echo "FAIL: the load run reported errors (exit ${LOAD_STATUS})." >&2
    exit 1
fi
echo "  ok: load completed with no request errors."

echo "[4/4] Auditing peak RSS (budget <= ${RSS_BUDGET_MB} MB)..."
echo "  measured peak RSS: ${PEAK_RSS_MB} MB"
if [ "$PEAK_RSS_MB" -le 0 ]; then
    echo "FAIL: no RSS samples were collected; the budget is unverified." >&2
    exit 1
fi
if [ "$PEAK_RSS_MB" -gt "$RSS_BUDGET_MB" ]; then
    echo "FAIL: peak RSS ${PEAK_RSS_MB} MB exceeds the ${RSS_BUDGET_MB} MB budget." >&2
    exit 1
fi

echo "=========================================================="
echo "PASS: peak RSS ${PEAK_RSS_MB} MB over ${DURATION_SEC}s at ${RPS_VAL} rps (measured)"
echo "=========================================================="
