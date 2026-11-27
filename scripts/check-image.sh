#!/usr/bin/env bash
# scripts/check-image.sh — Distroless image and cold-boot budget checks (TASKS.md P10-11)
#
# This gate MEASURES. It builds the image, reads its real size, boots a container and
# times it to first healthy response. It does not assert numbers it did not observe.
set -euo pipefail

IMAGE_TAG="${IMAGE_TAG:-stheno-server:gate-p10-11}"
SIZE_BUDGET_MB=60
COLD_START_BUDGET_MS=1000
CONTAINER_NAME="stheno-p10-11-$$"
DB_CONTAINER_NAME="stheno-p10-11-db-$$"
NETWORK_NAME="stheno-p10-11-net-$$"

echo "=========================================================="
echo "      STHENO CONTAINER IMAGE & SECURITY AUDIT (P10-11)    "
echo "=========================================================="

cleanup() {
    docker rm -f "$CONTAINER_NAME" "${CONTAINER_NAME}-migrate" "$DB_CONTAINER_NAME" >/dev/null 2>&1 || true
    docker network rm "$NETWORK_NAME" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "[1/4] Checking Dockerfile presence and base image..."
if [ ! -f Dockerfile ]; then
    echo "FAIL: Dockerfile not found." >&2
    exit 1
fi
# The runtime stage must be distroless: no shell, no package manager. `distroless/cc` is
# the distroless image for glibc-linked binaries, which stheno-server is; `distroless/static`
# is accepted for a future fully static (musl) build.
grep -qE "^FROM gcr.io/distroless/(static|cc)" Dockerfile || {
    echo "FAIL: Dockerfile's runtime stage must use a distroless base (static or cc)." >&2
    exit 1
}
echo "  ok: distroless base declared."

echo "[2/4] Verifying non-root execution policy..."
grep -q "USER 65532:65532" Dockerfile || {
    echo "FAIL: Dockerfile must run as non-root user 65532:65532." >&2
    exit 1
}
echo "  ok: non-root user declared."

if ! command -v docker >/dev/null 2>&1; then
    echo "FAIL: docker is not installed, so image size and cold start cannot be measured." >&2
    echo "      P10-11 is UNVERIFIED. Install Docker and re-run." >&2
    exit 1
fi
if ! docker info >/dev/null 2>&1; then
    echo "FAIL: the Docker daemon is not reachable, so this gate cannot measure anything." >&2
    echo "      P10-11 is UNVERIFIED. Start Docker and re-run." >&2
    exit 1
fi

echo "[3/4] Building image and measuring size (budget <= ${SIZE_BUDGET_MB} MB)..."
docker build -t "$IMAGE_TAG" . >/dev/null

SIZE_BYTES=$(docker image inspect "$IMAGE_TAG" --format '{{.Size}}')
SIZE_MB=$(( SIZE_BYTES / 1000000 ))
echo "  measured: ${SIZE_MB} MB"
if [ "$SIZE_MB" -gt "$SIZE_BUDGET_MB" ]; then
    echo "FAIL: image is ${SIZE_MB} MB, over the ${SIZE_BUDGET_MB} MB budget." >&2
    exit 1
fi
echo "  ok: within budget."

echo "[4/4] Measuring cold start (budget <= ${COLD_START_BUDGET_MS} ms)..."
# The server refuses to boot without DATABASE_URL (crates/stheno-server/src/db.rs), so the
# gate gives it a throwaway Postgres on a private network. The database is started and ready
# before the clock starts; only the server's own boot is measured.
docker network create "$NETWORK_NAME" >/dev/null
docker run -d --name "$DB_CONTAINER_NAME" --network "$NETWORK_NAME" \
    -e POSTGRES_USER=stheno -e POSTGRES_PASSWORD=stheno -e POSTGRES_DB=stheno \
    postgres:16-alpine >/dev/null
DB_READY=0
for _ in $(seq 1 120); do
    if docker exec "$DB_CONTAINER_NAME" pg_isready -U stheno -d stheno >/dev/null 2>&1; then
        DB_READY=1
        break
    fi
    sleep 0.5
done
if [ "$DB_READY" -ne 1 ]; then
    echo "FAIL: the gate's Postgres never became ready, so cold start cannot be measured." >&2
    docker logs "$DB_CONTAINER_NAME" >&2 || true
    exit 1
fi

# The server does not migrate on boot; `--migrate` applies migrations and then keeps
# serving. Run it once, wait for /health (migrations done), remove it, and only then time a
# fresh boot against the migrated database.
MIGRATE_NAME="${CONTAINER_NAME}-migrate"
docker run -d --name "$MIGRATE_NAME" --network "$NETWORK_NAME" \
    -e DATABASE_URL="postgres://stheno:stheno@${DB_CONTAINER_NAME}:5432/stheno" \
    "$IMAGE_TAG" --migrate --bind 0.0.0.0:9000 >/dev/null
MIGRATED=0
for _ in $(seq 1 240); do
    if docker exec "$DB_CONTAINER_NAME" psql -U stheno -d stheno -tAc \
        "SELECT 1 FROM pg_roles WHERE rolname = 'stheno_app'" 2>/dev/null | grep -q 1 \
        && docker logs "$MIGRATE_NAME" 2>&1 | grep -q "Applied "; then
        MIGRATED=1
        break
    fi
    if ! docker ps -q --filter "name=$MIGRATE_NAME" | grep -q .; then
        echo "FAIL: the migration run exited. Logs:" >&2
        docker logs "$MIGRATE_NAME" >&2 || true
        exit 1
    fi
    sleep 0.5
done
docker rm -f "$MIGRATE_NAME" >/dev/null 2>&1 || true
if [ "$MIGRATED" -ne 1 ]; then
    echo "FAIL: migrations did not complete, so cold start cannot be measured." >&2
    exit 1
fi

HOST_PORT=$(( 19000 + (RANDOM % 1000) ))
START_NS=$(date +%s%N)
docker run -d --name "$CONTAINER_NAME" --network "$NETWORK_NAME" -p "${HOST_PORT}:9000" \
    -e DATABASE_URL="postgres://stheno:stheno@${DB_CONTAINER_NAME}:5432/stheno" \
    "$IMAGE_TAG" --bind 0.0.0.0:9000 >/dev/null

READY=0
# Poll far faster than the budget so the measurement resolution is meaningful.
for _ in $(seq 1 600); do
    if curl -fsS -m 1 "http://127.0.0.1:${HOST_PORT}/health" >/dev/null 2>&1; then
        READY=1
        break
    fi
    if ! docker ps -q --filter "name=$CONTAINER_NAME" | grep -q .; then
        echo "FAIL: container exited before becoming healthy. Logs:" >&2
        docker logs "$CONTAINER_NAME" >&2 || true
        exit 1
    fi
    sleep 0.05
done
END_NS=$(date +%s%N)

if [ "$READY" -ne 1 ]; then
    echo "FAIL: container never answered /health. Logs:" >&2
    docker logs "$CONTAINER_NAME" >&2 || true
    exit 1
fi

COLD_START_MS=$(( (END_NS - START_NS) / 1000000 ))
echo "  measured: ${COLD_START_MS} ms"
if [ "$COLD_START_MS" -gt "$COLD_START_BUDGET_MS" ]; then
    echo "FAIL: cold start ${COLD_START_MS} ms exceeds the ${COLD_START_BUDGET_MS} ms budget." >&2
    exit 1
fi
echo "  ok: within budget."

echo "=========================================================="
echo "PASS: image ${SIZE_MB} MB, cold start ${COLD_START_MS} ms (measured)"
echo "=========================================================="
