# ─── Build stage ─────────────────────────────────────────────────
# Le bundle statique (dist/) est ensuite copié par le service Caddy
# (volume read-only) — il n'y a pas de runtime Node en prod.
FROM node:22-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

# ─── Output stage ────────────────────────────────────────────────
# Image minimale dont seul le contenu de /dist est utilisé via le
# volume mount dans docker-compose.yml.
FROM scratch AS dist
COPY --from=builder /app/dist /dist
