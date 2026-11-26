# Build, test, and artifact policy

This policy covers all first-party projects under this directory. Follow a project's own architecture, security, and gate instructions as well.

## Development loop

1. Read the nearest `AGENTS.md`, build scripts, and Git status. Confirm whether the relevant server/build is already running.
2. Run the narrowest useful checks first: formatting/lint/typecheck, focused tests, then affected package/module checks and direct consumer tests when needed.
3. Reuse the existing checkout, dependencies, incremental outputs, and local build cache. Avoid routine clean builds, cache purges, repeated installs, redundant packaging, unrelated platform builds, and duplicate concurrent jobs.
4. Use full builds for packaging/deployment validation, release readiness, build-system/toolchain/dependency changes, or a required project/CI gate. Run clean reproducibility builds at the appropriate CI/release boundary.
5. Keep outputs and reports in documented ignored build/output directories. Don't make project copies for tests; use a worktree only when isolation is actually needed.
6. Preserve useful caches. If disk cleanup is necessary, measure exact generated/cache paths, ensure no process is using them, and use their build tool's cleanup mechanism. Never delete source, user data, dependencies, or release artifacts as generic cleanup.
7. Report the checks actually run and distinguish focused checks from full app/platform/repository validation.

## Why incremental testing works

A test run compiles the changed code and the dependencies needed by its test target. Build systems can skip work whose declared inputs have not changed and can reuse cached outputs. A small code change therefore does not require packaging every platform or rebuilding every unrelated module. Compilation of the changed code is still necessary to run meaningful tests.

## Current directory scope

These instructions are for active source projects. Archives, downloaded upstream repositories, backup snapshots, generated output, and unrelated design assets should not be changed merely because they are located under this directory. Apply the policy when actively working in them; don't bulk-rewrite vendored/upstream repositories.