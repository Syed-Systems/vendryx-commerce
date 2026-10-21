# syntax=docker/dockerfile:1.4
# Vendryx storefront — Next.js 15 App Router (production: build + next start).
# Mirrors run-dev.bat: npm (not yarn), Node 20, webpack resolver (no --turbopack),
# server-side fetches hit the engine over Docker DNS.

FROM node:20-alpine AS deps
WORKDIR /app
# Workspace packages must be present at install time so npm links them.
COPY package.json package-lock.json ./
COPY packages/frontend-default/package.json ./packages/frontend-default/package.json
COPY packages/frontend-fixture-violating/package.json ./packages/frontend-fixture-violating/package.json
COPY packages/storefront-core/package.json ./packages/storefront-core/package.json
RUN npm install --no-audit --no-fund

FROM node:20-alpine AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .

# Build-time only. NEXT_PUBLIC_* is baked into the bundle; STHENO_BACKEND_URL is
# read server-side at request time but must exist at build for prerendering.
ARG NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY=pk_test
ARG NEXT_PUBLIC_BASE_URL=http://localhost:8000
ARG NEXT_PUBLIC_DEFAULT_REGION=us
ARG NEXT_PUBLIC_STRIPE_KEY=
# NOTE: no STHENO_BACKEND_URL here on purpose. The builder container is outside
# the compose network, so `engine` never resolves at build time. Leaving it
# unset lets config.ts fall through to STHENO_BUILD_BACKEND_URL (host loopback)
# for prerender fetches. Runtime still gets it via compose `environment:`.
# Build-time only: the build container is outside the compose network, so `engine`
# does not resolve here. Prerender fetches (generateStaticParams) use this host
# loopback address, where the engine must already be up with demo data seeded
# (see docker-compose.yml + README Docker section). Docker Desktop resolves
# host.docker.internal to the host; plain Docker Engine on Linux needs
# --add-host=host.docker.internal:host-gateway on the storefront build.
ARG STHENO_BUILD_BACKEND_URL=http://host.docker.internal:9000
ARG STHENO_FRONTEND=default

ENV NEXT_TELEMETRY_DISABLED=1
# run-dev.bat sets --no-experimental-webstorage for Node 25 (AGENTS.md §0). It is
# rejected in NODE_OPTIONS on Node 20, which this image pins — so this Dockerfile
# intentionally does NOT set it. If the base is ever bumped to Node 25+, the flag
# must come back or every SSR render 500s on `localStorage.getItem`.
ENV NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY=$NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY \
    NEXT_PUBLIC_BASE_URL=$NEXT_PUBLIC_BASE_URL \
    NEXT_PUBLIC_DEFAULT_REGION=$NEXT_PUBLIC_DEFAULT_REGION \
    NEXT_PUBLIC_STRIPE_KEY=$NEXT_PUBLIC_STRIPE_KEY \
    STHENO_BUILD_BACKEND_URL=$STHENO_BUILD_BACKEND_URL \
    STHENO_FRONTEND=$STHENO_FRONTEND

RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app
# Re-declared: an ARG is scoped to the stage that names it, so the builder's
# values are invisible here. Without this the runner boots with no
# NEXT_PUBLIC_* set and `next start` aborts with "Missing required environment
# variables: NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY" before serving anything.
ARG NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY=pk_test
ARG NEXT_PUBLIC_BASE_URL=http://localhost:8000
ARG NEXT_PUBLIC_DEFAULT_REGION=us
ARG NEXT_PUBLIC_STRIPE_KEY=
ENV NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY=$NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY \
    NEXT_PUBLIC_BASE_URL=$NEXT_PUBLIC_BASE_URL \
    NEXT_PUBLIC_DEFAULT_REGION=$NEXT_PUBLIC_DEFAULT_REGION \
    NEXT_PUBLIC_STRIPE_KEY=$NEXT_PUBLIC_STRIPE_KEY
ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    STHENO_BACKEND_URL=http://engine:9000
# Whole app dir (not standalone): next.config.js resolves @frontend/@modules via
# the webpack seam at build, but `next start` still reads next.config.js, so it
# ships along with .next, public and node_modules.
COPY --from=builder /app /app
EXPOSE 8000
CMD ["npm", "run", "start"]
