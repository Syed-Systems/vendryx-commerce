# syntax=docker/dockerfile:1.4
# Vendryx transitional platform console — Vite build served as static files.
# The backend URL is baked at build time (Vite define); the browser reaches the
# engine directly, so this defaults to the host-published engine port.

FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install --no-audit --no-fund
COPY . .

ARG VITE_STHENO_BACKEND_URL=http://localhost:9000
ENV VITE_STHENO_BACKEND_URL=$VITE_STHENO_BACKEND_URL

RUN npm run build

FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
