# Stheno Deployment & Infrastructure Guide

This guide covers building, configuring, deploying, scaling, and operating Stheno in production environments.

---

## 1. Architecture Overview

Stheno is architected as a lightweight, single-binary modular monolith written in Rust. It requires minimal external infrastructure while supporting horizontal scaling and physical tenant isolation:

```
                      [ Load Balancer / Ingress (TLS Termination) ]
                                            │
                     ┌──────────────────────┴──────────────────────┐
                     ▼                                             ▼
          [ Stheno Server Pod 1 ]                       [ Stheno Server Pod 2 ]
          (Distroless, Non-Root)                        (Distroless, Non-Root)
                     │                                             │
         ┌───────────┴───────────┐                     ┌───────────┴───────────┐
         ▼                       ▼                     ▼                       ▼
   [ PostgreSQL 15+ ]     [ Redis 7+ ]           [ PostgreSQL 15+ ]     [ Redis 7+ ]
   (Primary / Replica)    (Cache / Queue)        (Primary / Replica)    (Cache / Queue)
```

---

## 2. Container Image Specification

Stheno builds into a distroless, non-root Linux container based on `gcr.io/distroless/static-debian12:nonroot`:

- **Binary Size**: ~40 MB static binary (musl / static glibc).
- **Final Container Size**: < 60 MB uncompressed.
- **Security Footprint**: Zero OS package managers, zero shell interpreters (`/bin/sh` absent), running as UID `65532:65532`.
- **Base Memory Usage**: ≤ 128 MB resident set size (RSS) at idle.

### Building the Image

```bash
# Build multi-stage optimized production image
docker build -t stheno-server:latest -f Dockerfile .

# Inspect image size and vulnerability status
docker images stheno-server:latest
```

### Running Locally via Docker

```bash
docker run -d \
  --name stheno \
  -p 9000:9000 \
  -e DATABASE_URL="postgres://stheno:password@localhost:5432/stheno" \
  -e REDIS_URL="redis://localhost:6379" \
  -e JWT_SECRET="your-256-bit-secret-key-here-minimum-32-chars" \
  -e COOKIE_SECRET="your-256-bit-cookie-secret-key-min-32-chars" \
  stheno-server:latest
```

---

## 3. Environment Variables Reference

All runtime configuration is supplied via environment variables. Sensitive values must be injected via Kubernetes Secrets or a secrets manager (e.g., HashiCorp Vault, AWS Secrets Manager).

| Variable | Type | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | String (URI) | *Required* | Postgres connection string (`postgres://user:pass@host:5432/db`). |
| `DATABASE_MAX_CONNECTIONS` | Integer | `20` | Maximum connection pool size per pod. |
| `DATABASE_MIN_CONNECTIONS` | Integer | `2` | Minimum idle connections retained in pool. |
| `DATABASE_TIMEOUT_SECONDS` | Integer | `30` | Connection checkout timeout. |
| `REDIS_URL` | String (URI) | `redis://127.0.0.1:6379` | Redis connection URI for caching and idempotency locks. |
| `PORT` | Integer | `9000` | HTTP listen port. |
| `BIND_ADDRESS` | String | `0.0.0.0` | Bind IP address. |
| `JWT_SECRET` | String (Hex/Base64) | *Required* | HMAC-SHA256 secret for customer and admin session tokens (min 32 bytes). |
| `COOKIE_SECRET` | String | *Required* | At least 32 bytes; signs session/CSRF cookies and encrypts storefront cart cookies. |
| `COOKIE_SECRET_PREVIOUS` | String | Optional | Previous cookie key accepted during a rotation window so encrypted active carts remain readable. |
| `CSRF_PROTECTION_ENABLED` | Boolean | `true` | Enforces double-submit cookie + custom header check on state mutations. |
| `ENVIRONMENT` | String | `production` | Environment tier (`production`, `staging`, `development`). |
| `LOG_LEVEL` | String | `info` | Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`). |
| `LOG_FORMAT` | String | `json` | Structured logging format (`json` or `compact`). |
| `STRIPE_API_KEY` | String | Optional | Secret key for Stripe payment gateway operations. |
| `STRIPE_WEBHOOK_SECRET` | String | Optional | Webhook signing secret for asynchronous Stripe events. |
| `SENDGRID_API_KEY` | String | Optional | API key for transactional email delivery via SendGrid. |
| `S3_BUCKET` | String | Optional | Object storage bucket name for media and document assets. |
| `S3_REGION` | String | Optional | AWS / S3-compatible cloud region. |
| `S3_ACCESS_KEY_ID` | String | Optional | Storage IAM access key. |
| `S3_SECRET_ACCESS_KEY` | String | Optional | Storage IAM secret key. |
| `S3_ENDPOINT` | String (URI) | Optional | Custom S3 endpoint (e.g. MinIO, Cloudflare R2). |
| `STHENO_PLATFORM_TOKEN` | String | Optional | Bootstrap credential for the control plane. **Unset disables `/platform/*` entirely** — the surface 404s rather than opening. See §3.1. |
| `STHENO_TENANT` | String | Optional | Fallback tenant for a single-merchant self-hosted install with no domain registry (ADR-009 shape, still supported). |

---

### 3.1 The control-plane credential

> **Note (2026-09-08 — [ADR-030](DECISIONS.md#adr-030)):** The primary company control plane is **Space**
> (`D:\Syed Systems\Space`), which manages fleet-wide operations. The credential below applies to the legacy
> in-repo `/platform/*` surface during the transitional phase.

`/platform/*` reads and writes across **every** tenant, outside the RLS layer
([ADR-012](DECISIONS.md#adr-012)). It is the highest-privilege surface in the deployment,
and it is credentialed separately from every merchant account.

Two credential kinds reach it, and they are not equivalent:

| | Bootstrap token | Named operator |
|---|---|---|
| Storage | `STHENO_PLATFORM_TOKEN` in the environment | `platform.operator`, SHA-256 of the token |
| Identifies | a secret | a person |
| Audited as | `op_bootstrap` | their name |
| Revoke one holder | not possible | `DELETE /platform/operators/{id}` |
| Rotate | changes it for everyone at once | issue new, revoke old, nobody else disturbed |

**Operational rules:**

1. **Inject `STHENO_PLATFORM_TOKEN` as a Secret**, never as a plain env value in a manifest
   or a `values.yaml`. It is a break-glass credential to the whole estate.
2. **Issue a named credential per person on first deploy**, and use those. A row recorded
   against `op_bootstrap` cannot say which of three people made the change.
3. **Leave the bootstrap token configured.** It is the way back in if every named
   credential is revoked or lost — a control plane that can lock its owner out of itself
   is a worse failure than a shared secret. Retiring it once the team is established is
   [PHASE4_TASKS.md](PHASE4_TASKS.md) P36-05.
4. **Unsetting it does not disable named operators** — it only closes the bootstrap path.
   The surface stays reachable by anyone holding a live `op_...` credential.
5. **On suspected compromise**, revoke the affected credential first (it takes effect on
   the next request, with no cache to expire), then read
   `GET /platform/audit` to establish what it did. Revocation does not delete its rows.

The token cannot be recovered after issue. Only its SHA-256 is stored, so a database read,
a backup, or a support request cannot produce it; a lost credential is replaced, not
recovered.

---

## 4. Kubernetes Deployment via Helm

Stheno provides a production-grade Helm chart located in `deploy/chart`.

### Prerequisites

- Kubernetes cluster v1.26+
- PostgreSQL 15+ (managed or self-hosted with `uuid-ossp` or ULID support)
- Redis 7+ (cluster, Sentinel, or standalone)
- NGINX Ingress Controller or AWS ALB Controller
- `cert-manager` for automated Let's Encrypt TLS certificates

### Chart Structure & Configuration

```yaml
# deploy/chart/values.yaml snippet
replicaCount: 3

image:
  repository: gcr.io/stheno-commerce/stheno-server
  pullPolicy: IfNotPresent
  tag: "1.0.0"

resources:
  limits:
    cpu: 1000m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 75
  targetMemoryUtilizationPercentage: 80

podDisruptionBudget:
  minAvailable: 2

securityContext:
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  runAsUser: 65532
  runAsGroup: 65532
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL

probes:
  liveness:
    httpGet:
      path: /health/live
      port: 9000
    initialDelaySeconds: 5
    periodSeconds: 10
  readiness:
    httpGet:
      path: /health/ready
      port: 9000
    initialDelaySeconds: 2
    periodSeconds: 5
```

### Installation Commands

```bash
# 1. Lint the Helm chart
helm lint deploy/chart

# 2. Perform dry-run template rendering
helm template stheno-prod deploy/chart \
  --namespace commerce \
  -f deploy/chart/values.yaml

# 3. Deploy or Upgrade Release
helm upgrade --install stheno-prod deploy/chart \
  --namespace commerce \
  --create-namespace \
  -f deploy/chart/values.yaml \
  --set env.JWT_SECRET="$JWT_SECRET" \
  --set env.DATABASE_URL="$DATABASE_URL"
```

---

## 5. High Availability & Operational Resilience

### 5.1 Zero-Downtime Rolling Upgrades
- Kubernetes rolling update strategy: `maxUnavailable: 0`, `maxSurge: 1`.
- Database schema changes follow the **Expand/Contract** phased migration pattern (see §6).
- New pods must pass `/health/ready` (verifying database connectivity and schema readiness) before old pods are terminated.

### 5.2 Health Probes
- **`/health/live`**: Fast, lightweight liveness check verifying the HTTP worker process is responsive.
- **`/health/ready`**: Thorough readiness probe verifying database connectivity pool and Redis connectivity before admitting traffic.
- **`/metrics`**: Prometheus-formatted telemetry endpoint exporting request latencies, active DB connections, and workflow execution states.

### 5.3 Redis Outage Fallback
- If Redis becomes unavailable, Stheno automatically degrades gracefully:
  - Cache reads fallback to direct PostgreSQL queries.
  - Distributed locks fallback to PostgreSQL advisory locks via `PostgresAdvisoryLockProvider` on dedicated pinned connections.

---

## 6. Phased Database Migrations

Destructive schema migrations (`DROP COLUMN`, `ALTER COLUMN TYPE`, `NOT NULL` without default) are strictly forbidden in a single step. All changes follow two distinct deployment phases:

1. **Expand Phase (`-- phase: expand`)**:
   - Add new columns as nullable or with defaults.
   - Dual-write to both legacy and new schema locations.
   - Deploy new application version.
2. **Contract Phase (`-- phase: contract`)**:
   - Backfill historical data if needed.
   - Switch application reads strictly to new schema.
   - Remove legacy columns/tables in a subsequent migration release.

Migrations are applied automatically at startup or via CI pipeline job using `stheno-cli migrate`.

---

## 7. Backup & Disaster Recovery (DR)

| Metric | Target | Implementation |
|---|---|---|
| **Recovery Point Objective (RPO)** | < 1 minute | Continuous WAL archiving to S3/GCS via pgBackRest or AWS RDS Automated Backups. |
| **Recovery Time Objective (RTO)** | < 15 minutes | Automated point-in-time recovery (PITR) with automated restore drill validation (`scripts/restore-drill.sh`). |

### Backup Verification Protocol
- Daily automated restore drill executes against an ephemeral test cluster.
- Validates cryptographic integrity of database dump and schema consistency.

---

## 8. Cloudflare edge

Cloudflare terminates public TLS and connects through the Tunnel to the origin proxy. The Rust
engine, storefront SSR process, and admin dashboard remain on the origin. The proxy has no public
host port; do not publish the engine, storefront, or dashboard services directly. Set
`STHENO_ADMIN_DOMAIN_SUFFIX` at runtime. The corresponding admin suffix must serve no
merchant-authored content. See [ADR-037](DECISIONS.md#adr-037).

Complete these steps in order:

1. **Create zones.** Add the default storefront zone, the separate admin-hostname zone, and the
   media zone. Keep the marketing site's zone separate from tenant storefront content.
2. **Set TLS mode.** Set each web zone to **Full (strict)**. Keep the origin proxy on HTTP inside
   the private Tunnel network; Caddy automatic HTTPS is disabled because TLS ends at the edge.
3. **Add the wildcard store record.** Associate the proxied wildcard record for the configured
   store subdomains with the Tunnel. Do not add a public A or AAAA record for the origin.
4. **Configure the admin domain and certificate.** Choose a registrable domain that serves no
   merchant-authored content, set its suffix as `STHENO_ADMIN_DOMAIN_SUFFIX`, and issue a
   certificate covering one tenant label beneath that suffix. Verify certificate depth for a
   name such as `merchant.admin.example.test` before fixing the production suffix.
5. **Create the Tunnel.** Store its connector token as `CLOUDFLARE_TUNNEL_TOKEN`. Configure the
   Tunnel's storefront wildcard and admin wildcard public hostnames to target `http://proxy:8080`
   (in Compose) or the chart's `<release>-proxy:8080` Service. The Tunnel is the only public
   route to the origin. In Compose, attach the engine, storefront, and admin containers to the
   external `STHENO_ORIGIN_NETWORK` (default `stheno-origin`) and configure the three upstream
   service names if they differ from `web`, `storefront`, and `admin`. The root Compose file does
   not define those application services; create the external network and attach the separately
   deployed containers to it. The Helm chart creates a ClusterIP service for the proxy and no
   Ingress resource; point the Tunnel at that service. Preserve the incoming `Host`; the proxy
   uses it for tenant resolution.
6. **Add the cache rule.** Use the origin `Cache-Control` when it is present and bypass caching
   when it is absent. Bypass `/admin/*`, `/auth/*`, `/store/carts*`, `/store/customers*`,
   `/store/payment-collections*`, `/store/orders/*`, `/*/cart*`, `/*/checkout*`, `/*/account*`,
   and `/*/order*`.
   Also bypass `/hooks/*` and `/webhooks/meta/*`. Do not configure an edge TTL override: it could
   cache a response that the origin did not mark public.
7. **Create the R2 buckets.** Configure a public product-media bucket with its media hostname
   and a separate private bucket with no public hostname for payment proofs and other private
   documents. Configure the S3-compatible endpoint with region `auto`. Do not give private
   objects a public URL.
8. **Set the custom-hostname fallback origin.** Space owns custom-hostname registration and
   certificate status. Point the fallback origin to the Tunnel/proxy route so custom domains
   still reach the same Host-based tenant resolver; never expose the origin directly.
9. **Deploy marketing as static content.** Publish `apps/marketing` as static assets only. Do not
   enable Functions or move application runtime to the edge.

### Edge firewall baseline

Rate-limit the following requests by client address and hostname:

| Purpose | Route patterns read from the current route definitions |
|---|---|
| Sign-in | `POST /auth/user/{auth_provider}`; `POST /auth/{actor_type}/{provider_id}` |
| Password reset | `POST /auth/user/{auth_provider}/reset-password`; `POST /auth/{actor_type}/{provider_id}/reset-password`; `POST /auth/{actor_type}/{provider_id}/confirm-reset-password` |
| Account creation | `POST /auth/user/{auth_provider}/register`; `POST /auth/{actor_type}/{provider_id}/register`; `POST /store/customers` |
| Checkout submission | `POST /store/carts/{id}/complete`; `POST /store/payment-collections`; `POST /store/payment-collections/{id}/payment-sessions`; `POST /store/payment-collections/{id}/payment-sessions/{session_id}` |
| Payment-proof submission | `POST /store/orders/{id}/payment-proof` |

Never apply browser challenges or bot checks to the machine callbacks below. Keep the rate limit
and the engine's signature verification enabled:

- `/hooks/payment/{provider_id}`
- `/hooks/fulfillment/{provider_id}`
- `/webhooks/meta/{webhook_id}`

The paths above come from `crates/api/stheno-api-auth/src/routes.rs`,
`crates/api/stheno-api-store/src/lib.rs`, `crates/api/stheno-api-store/src/hooks.rs`, and
`crates/api/stheno-api-store/src/social_webhook.rs`.

### Acceptance checklist from ADR-037

| Check | Result |
|---|---|
| Tag and hostname purge availability and rate limits on the selected plan tier | |
| `Cache-Tag` list format and size limits | |
| `stale-while-revalidate` behavior | |
| Custom-hostname pricing | |
| Wildcard certificate depth for admin hostnames | |
| Wildcard Tunnel routing preserves `Host` | |
| `stheno-file-s3` compatibility with R2 | |

The plan tier and production admin suffix are runtime/owner configuration and are not chosen by
this runbook.
