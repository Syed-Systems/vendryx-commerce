# syntax=docker/dockerfile:1.4
# Multi-stage Distroless Container Build for Stheno Server (TASKS.md P10-11)

FROM rust:1.98.1-bookworm as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin stheno-server && \
    strip target/release/stheno-server

# Bootstrap stage: release binary plus the shell tooling (curl, postgres-client)
# deploy/docker/bootstrap.sh needs to create + seed the dev tenants. Kept as a
# separate stage so the production runtime below stays distroless (<= 60 MB).
FROM debian:12-slim as bootstrap
RUN apt-get update && apt-get install -y --no-install-recommends curl postgresql-client && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/stheno-server /app/target/release/stheno-server
COPY deploy/docker/bootstrap.sh /app/bootstrap.sh
RUN chmod +x /app/bootstrap.sh

# Debian slim runtime. The binary is dynamically linked (interpreter
# /lib64/ld-linux-x86-64.so.2), which the previous `static-debian12` base does
# not ship — the container died with `no such file or directory` despite the
# binary being present and executable. A fully static musl build would restore
# the distroless option; until then this is the correct base.
# Distroless with glibc. The binary is dynamically linked against glibc only (no OpenSSL, no
# libpq: TLS is rustls and Postgres is tokio-postgres), which `distroless/static` cannot run
# but `distroless/cc` can. Same Debian 12 base as the bookworm builder, no shell, no package
# manager, and a built-in non-root user 65532.
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app
COPY --from=builder --chown=65532:65532 /app/target/release/stheno-server /app/stheno-server

# Run as non-root user (security ASVS L2 / P10-11)
USER 65532:65532

EXPOSE 9000
ENTRYPOINT ["/app/stheno-server"]

