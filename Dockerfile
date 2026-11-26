# syntax=docker/dockerfile:1.4
# Multi-stage Distroless Container Build for Stheno Server (TASKS.md P10-11)

FROM rust:1.89-bullseye as builder
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
FROM debian:12-slim

WORKDIR /app
COPY --from=builder /app/target/release/stheno-server /app/stheno-server

# Non-root runtime user (replaces the distroless `nonroot` user).
RUN useradd -r -u 65532 -g nogroup stheno && chown 65532:nogroup /app/stheno-server

# Run as non-root user (security ASVS L2 / P10-11)
USER 65532:nogroup

EXPOSE 9000
ENTRYPOINT ["/app/stheno-server"]

