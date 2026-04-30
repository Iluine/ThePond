# The Pond

PWA self-hostée pour le partage de photos, clips vidéo et messages vocaux
pendant un mariage privé. Concept "appareil photo jetable" : les invités
postent pendant la soirée mais ne voient pas les contenus immédiatement —
une mécanique de paliers temporels gère la révélation progressive des
souvenirs.

Open source sous licence beerware. Code librement réutilisable pour d'autres
événements avec une thématisation différente (voir `themes/`).

## Documentation

- **[`PROJECT.md`](./PROJECT.md)** — bible complète : contexte, vocabulaire
  imposé (la mare, barboter, cancaner, coin-coin), architecture temps réel,
  schéma DB, design system v0.2, phases de livraison.
- **[`CLAUDE.md`](./CLAUDE.md)** — guide pour les sessions Claude Code.
- **[`design/`](./design/)** — maquettes HTML de référence, source de vérité
  pour les valeurs du design system (voir `design/INDEX.md`).

## Stack

- **Backend** — Rust stable, Axum 0.7, Tokio, SQLx (SQLite)
- **Frontend** — Vue 3 Composition API, Vite, Tailwind, Pinia, vite-plugin-pwa
- **Real-time** — Server-Sent Events avec snapshot push
- **Infra** — Docker Compose, Caddy 2 (TLS via DNS-01 DuckDNS)

## Développement local

### Backend

```bash
cd backend
cargo run                    # démarre sur :3000, /health répond
cargo test
cargo clippy -- -D warnings
```

Le binaire lit son thème via `THEME_PATH` (défaut :
`../themes/duck-pond/theme.ron`).

### Frontend

```bash
cd frontend
npm install
npm run dev                  # serveur Vite sur :5173
npm run build                # bundle statique dans dist/
```

### Stack complète (avec Caddy en local sans TLS)

```bash
caddy run --config Caddyfile.local
# puis lance backend et frontend dans deux autres terminaux
```

Caddy expose tout sur `:8080` (proxy `/api/*` et `/events` vers le backend,
le reste vers le serveur Vite).

## Déploiement (NAS Synology)

Crée un fichier `.env` à la racine avec :

```
POND_DOMAIN=mariage.duckdns.org
DUCKDNS_TOKEN=...
WITNESS_SECRET=...
POND_DATA_DIR=/volume1/docker/thepond
```

Puis :

```bash
docker compose up -d --build
```

Volumes persistants : `${POND_DATA_DIR}/data` (SQLite + uploads).

## Licence

Beerware — si on se croise, paie-moi une bière.
