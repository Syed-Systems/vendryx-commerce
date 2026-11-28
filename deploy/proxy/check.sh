#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT"
COMPOSE="docker compose -f deploy/proxy/compose.check.yml"
cleanup() {
  $COMPOSE down --remove-orphans
}
trap cleanup EXIT INT TERM

$COMPOSE up -d
python3 deploy/proxy/check.py
