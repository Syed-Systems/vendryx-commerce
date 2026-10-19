# syntax=docker/dockerfile:1.4
# Vendryx marketing site — Vite build served as static files behind nginx.

FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install --no-audit --no-fund
COPY . .

ARG VITE_ADMIN_URL=http://localhost:6100
ENV VITE_ADMIN_URL=$VITE_ADMIN_URL

RUN npm run build

FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
