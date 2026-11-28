#!/usr/bin/env bash
# scripts/doc-sync-workflow.sh — Documentation sync and version logging wrapper
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

python3 "$REPO_ROOT/scripts/doc-sync-workflow.py" "$@"
