#!/usr/bin/env bash
# Preflight: enough free space on the drive holding the cargo target directory.
#
# This exists because running out of space does not look like running out of
# space. It surfaces as `os error 112` on a random crate, as
# `linking with ... failed`, or -- worst -- as a Postgres container dying
# mid-suite and every persistence test reporting `error connecting to server`.
# All three read as code defects. Each one cost real debugging time before the
# cause was found, so the condition is now checked up front and named.
#
# A full workspace debug build of this repo measures ~10 GB with
# `debug = "line-tables-only"` and ~20 GB with full debug info, and cargo never
# garbage-collects superseded artifacts, so the directory grows across a session.
set -euo pipefail

REQUIRED_GB="${STHENO_DISK_REQUIRED_GB:-15}"
COMFORT_GB="${STHENO_DISK_COMFORT_GB:-30}"

# Resolve the target directory the way cargo does: CARGO_TARGET_DIR wins,
# then build.target-dir in .cargo/config.toml, then ./target.
target_dir() {
    if [ -n "${CARGO_TARGET_DIR:-}" ]; then
        echo "$CARGO_TARGET_DIR"
        return
    fi
    if [ -f .cargo/config.toml ]; then
        local configured
        configured=$(grep -E '^\s*target-dir\s*=' .cargo/config.toml 2>/dev/null \
            | head -1 | sed -E 's/.*=\s*"?([^"]*)"?\s*$/\1/')
        if [ -n "$configured" ]; then
            echo "$configured"
            return
        fi
    fi
    echo "./target"
}

TARGET=$(target_dir)
echo "==> Checking free space for the build directory..."
echo "    target-dir: $TARGET"

# `df` needs a path that exists; walk up until one does.
probe="$TARGET"
while [ ! -e "$probe" ] && [ "$probe" != "/" ] && [ "$probe" != "." ]; do
    probe=$(dirname "$probe")
done

AVAIL_KB=$(df -Pk "$probe" 2>/dev/null | awk 'NR==2 {print $4}')
if [ -z "$AVAIL_KB" ]; then
    echo "FAIL: could not determine free space for '$probe'." >&2
    exit 1
fi

AVAIL_GB=$(( AVAIL_KB / 1024 / 1024 ))
echo "    available: ${AVAIL_GB} GB (require ${REQUIRED_GB}, comfortable ${COMFORT_GB})"

if [ "$AVAIL_GB" -lt "$REQUIRED_GB" ]; then
    cat >&2 <<EOF
FAIL: only ${AVAIL_GB} GB free where the build directory lives, need ${REQUIRED_GB} GB.

Do not debug the build failure this would have caused -- reclaim space first:

  # cheapest: drops the incremental cache, keeps compiled artifacts
  rm -rf "${TARGET}/debug/incremental"

  # bigger: drops all build output, costs a full rebuild
  rm -rf "${TARGET}/debug"

  # Docker's virtual disk does not shrink when you prune inside it;
  # reclaiming it needs a deliberate compaction with Docker stopped.
EOF
    exit 1
fi

if [ "$AVAIL_GB" -lt "$COMFORT_GB" ]; then
    echo "WARNING: ${AVAIL_GB} GB free is enough to build but not to build twice." >&2
    echo "         Consider 'rm -rf ${TARGET}/debug/incremental' before a long run." >&2
fi

echo "Disk space check PASSED: ${AVAIL_GB} GB available."
exit 0
