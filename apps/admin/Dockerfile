# syntax=docker/dockerfile:1.4
# Vendryx admin dashboard — Vite build served as static files behind nginx.
# nginx replicates the vite dev-proxy contract (vite.config.mts): the SDK issues
# same-origin requests (VITE_STHENO_BACKEND_URL empty) and /auth + /admin are
# forwarded to the engine with Host rewritten from the stheno_tenant_host cookie,
# because the engine resolves the merchant from Host (ADR-022).

FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install --no-audit --no-fund
COPY . .

# Empty on purpose (see above). Baked at build time — rebuilding with a different
# value is how the target backend changes.
ARG VITE_STHENO_BACKEND_URL=
ARG VITE_STHENO_STOREFRONT_URL=http://localhost:8000
ENV VITE_STHENO_BACKEND_URL=$VITE_STHENO_BACKEND_URL \
    VITE_STHENO_STOREFRONT_URL=$VITE_STHENO_STOREFRONT_URL

RUN npm run build

FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
