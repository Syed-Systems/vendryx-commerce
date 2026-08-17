# syntax=docker/dockerfile:1.4
# Multi-stage Distroless Container Build for Stheno Server (TASKS.md P10-11)

FROM rust:1.80-bullseye as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin stheno-server && \
    strip target/release/stheno-server

# Minimal Distroless Final Stage (<= 60 MB)
FROM gcr.io/distroless/static-debian12:nonroot

WORKDIR /app
COPY --from=builder /app/target/release/stheno-server /app/stheno-server

# Run as non-root user (security ASVS L2 / P10-11)
USER 65532:65532

EXPOSE 9000
ENTRYPOINT ["/app/stheno-server"]
