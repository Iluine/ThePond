# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Lis ceci d'abord

Avant toute action, lis **`PROJECT.md`** : c'est la bible du projet (contexte,
vocabulaire imposé, stack, architecture temps réel, schéma DB, design system,
phases de dev). Tout ce qui suit ici n'est qu'un complément orienté Claude Code,
pas un substitut.

## État actuel du dépôt

Le projet n'est **pas encore bootstrappé**. Le dépôt contient uniquement :

- `PROJECT.md` — bible du projet
- `BOOTSTRAP.md` — prompt à coller pour la session de bootstrap (à supprimer une
  fois `backend/`, `frontend/`, `themes/`, `docker-compose.yml` créés)
- `design/` — maquettes HTML Claude Design (références visuelles, voir
  `design/INDEX.md` pour l'inventaire commenté)
- `README.md` — squelette

Si on te demande "lance le backend / le frontend", c'est qu'il n'existe pas
encore : démarre par les étapes de `BOOTSTRAP.md`.

## Vocabulaire NON négociable

Ces termes sont du branding figé. Tout écart est un bug à corriger :

- **"la mare"** (jamais "l'étang") — plusieurs maquettes HTML dans `design/`
  utilisent encore "l'étang", à remplacer systématiquement à l'intégration
- **The Pond.** — nom de marque, en Caprasimo, point inclus dans le logo
- **Mare TV** — l'écran de diffusion
- **barboter** (photos), **cancaner** (clips), **coin-coin** (vocaux),
  **plonger** (entrer dans la mare), **les barbotages** (contenus collectifs)
- **témoin** — modérateur sans pouvoir de modération (orchestration paliers)

Microcopy figée et états de galerie : voir `PROJECT.md` § "Microcopy figée" et
"États de la galerie". À ne pas reformuler librement.

## Architecture en deux principes

1. **SSE avec snapshot complet, pas de deltas.** Le client reçoit l'état entier
   à chaque changement et remplace son store. Pas de gestion d'événements
   manqués. Debounce 300 ms côté backend. Détails dans `PROJECT.md`
   § "Architecture temps réel".

2. **Paliers calculés à la demande, pas via job.** L'état "current/visible/
   locked" d'un palier se déduit de `target_time <= NOW()`. Forcer un palier =
   `UPDATE target_time = NOW()`, surtout pas de champ `forced_at`. Détails dans
   `PROJECT.md` § "Mécanique des paliers".

Ces deux choix sont structurants. Ne propose pas WebSocket, deltas, ou
scheduler de paliers sans en discuter explicitement.

## Stack (rappel court)

- **Backend** : Rust stable, Axum 0.7, Tokio, SQLx (SQLite, requêtes vérifiées
  à la compilation — `SQLX_OFFLINE` + `.sqlx/` à committer), `tracing`,
  `thiserror`
- **Frontend** : Vue 3 Composition API exclusive (`<script setup lang="ts">`),
  Vite, Tailwind avec tokens custom, Vue Router 4, Pinia, vite-plugin-pwa,
  EventSource natif
- **Infra** : Docker Compose (backend + Caddy), Caddy 2 avec DNS-01 DuckDNS,
  cible NAS Synology DS718+ (Celeron J3355, 2 cœurs, Quick Sync)

Pas d'Options API, pas de mixins, pas de props drilling. Pinia ou
provide/inject pour l'état partagé.

## Commandes de dev (post-bootstrap)

À utiliser une fois `backend/` et `frontend/` créés :

```bash
# Backend
cd backend
cargo run                              # démarre sur :3000
cargo test                             # tests
cargo test <nom_du_test>               # un test précis
cargo clippy -- -D warnings            # lint (warnings = échec)
cargo fmt                              # formatage
cargo sqlx prepare                     # régénère .sqlx/ après changement de requête

# Frontend
cd frontend
npm run dev                            # serveur Vite
npm run build                          # build prod (sortie statique pour Caddy)
npm run lint                           # ESLint config Vue 3 + TS
npm run test -- <pattern>              # un test précis (selon le runner choisi)

# Stack complète
docker compose up --build              # backend + Caddy + frontend statique
```

Si une commande ci-dessus échoue avec "command not found" ou "no such file",
c'est probablement que le bootstrap n'a pas encore été fait — vérifie l'état du
dépôt avant d'aller plus loin.

## Design system v0.2

La référence canonique est **`design/The Pond - Design System.html`** : palette
9 couleurs avec ratios de contraste, typographies (Caprasimo / DM Sans /
DM Mono), composants signatures, SVG des canards. À ouvrir dans un navigateur
pour visualiser ; à parser pour récupérer les valeurs exactes (couleurs, SVG)
quand on construit `tokens.css` et `tailwind.config.js`.

Les autres HTML de `design/` sont des écrans validés ou DRAFT — voir
`design/INDEX.md` pour le statut de chacun, la vue Vue cible, et les
ajustements à faire en code (ex : la galerie publique simple dans
`Mosaic 4 screens.html` ne respecte PAS le concept "appareil photo jetable",
la vraie galerie 4 états est dans `Mosaic 6 screens.html`).

Couleurs réservées :
- **Champagne** (`#E8C77A`) — UNIQUEMENT pour les mariées
- **Coral** (`#E07A5F`) — UNIQUEMENT pour le destructif

Composant signature : bouton primaire physique avec
`box-shadow: 0 4px 0 var(--duck-deep)`.

## Conventions code & commit

- Code et commentaires en **anglais**, textes UI en **français**
- Conventional Commits (`feat:`, `fix:`, `chore:`, …)
- Branche unique `main`, dev solo, pas de PR — mais commits atomiques
- Components Vue en PascalCase, composables préfixés `use`
- Privilégie la simplicité : pas d'abstraction prématurée, pas de patterns
  over-engineered

## Phases de livraison

Le découpage en V1 (avant le 5 juin 2026), P1, P2 est dans `PROJECT.md`
§ "Phases de développement". Avant d'implémenter, vérifie que la fonctionnalité
demandée est bien V1 — sinon, signale-le.

Les prompts thématiques 1 à 16 (post-bootstrap) sont dans `BOOTSTRAP.md`
§ "Après cette session". Suis-les dans l'ordre.
