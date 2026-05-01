# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Lis ceci d'abord

Avant toute action, lis **`PROJECT.md`** (la bible) puis `design/INDEX.md`
(carte des maquettes). Tout ce qui suit ici n'est qu'un complément orienté
Claude Code — ça ne remplace pas la bible.

## État actuel

Stack complète et fonctionnelle de bout en bout pour le parcours invité +
témoin. `git log --oneline` pour les détails ; topologie courante :

- **Backend Rust** (Axum + SQLx + SQLite) : routes auth (`POST /api/users`,
  `PATCH /api/users/:id`, `GET /api/pseudo`), upload (`POST /api/{media,clips,voice}`),
  SSE (`GET /api/events`), orchestration témoin (`/api/orchestration/*`
  protégé par token HMAC), export ZIP. Snapshot polymorphe (UNION des 3 tables
  media/clips/voice avec join users). `tower-http::ServeDir` sur `/uploads/*`.
- **Frontend Vue 3** : Welcome, Upload + capture in-app (clip + voice via
  MediaRecorder), Confirmation (3-frame strip + optimistic UI), Gallery
  (4 états calculés depuis le snapshot), ClipPlayer/VoicePlayer, Orchestration
  témoin. 3 stores Pinia (user, snapshot, uploadQueue) + 1 (witness) pour les
  témoins. SSE auto-mounté dans `App.vue` via `useEventStream()`.
- **Mare TV** : `MareTVView` (kiosque, rotation client-side) et
  `MareTVInstructionsView` (QR de pairing via `qrcode` lib) livrés au
  prompt 13.
- **Restant V1** : prompt 14 (PWA install + iOS install sheet — actif sur
  la branche, `InstallSheet.vue` en cours), prompt 15 (polish + ErrorView),
  prompt 16 (déploiement NAS). Likes / défis / réponses vocales en chaîne
  → P1 explicite.

## Vocabulaire NON négociable

- **"la mare"** (jamais "l'étang") — design HTML peut encore avoir des
  résidus, à corriger
- **The Pond.** — nom de marque, Caprasimo, point inclus
- **Mare TV** — l'écran de diffusion
- **barboter** (photos) / **cancaner** (clips) / **coin-coin** (vocaux) /
  **plonger** (entrer) / **les barbotages** (contenus collectifs)
- **témoin** — modérateur SANS pouvoir de modération (orchestre les paliers)

Microcopies figées dans `themes/duck-pond/strings.ron` et reprises (hardcodées
pour V1) dans les vues correspondantes. Pas reformuler librement.

## Trois principes architecturaux structurants

1. **SSE avec snapshot complet, pas de deltas.** Le client remplace son store
   à chaque réception. Pas de gestion d'événements manqués. Debounce 300 ms
   trailing-edge dans `services/broadcast.rs::SnapshotHub`.

2. **Paliers calculés à la demande, pas via job.** L'état current/visible/
   locked se déduit en mémoire de `target_time <= NOW()`
   (`services/phase_logic.rs`, fonctions pures + 5 tests). Forcer un palier =
   `UPDATE target_time = NOW()` (PAS de champ `forced_at`).

3. **Toute mutation appelle `state.hub.mark_dirty()`**. C'est le contrat
   qui fait que les écrans se mettent à jour live. Si tu ajoutes une route
   qui modifie quoi que ce soit, n'oublie pas ce dernier appel.

Ne propose pas WebSocket, deltas, ou scheduler de paliers sans en discuter.

## Stack & conventions

- Rust stable, Axum 0.7, Tokio, SQLx 0.8 — **runtime queries uniquement**
  (feature `macros` activé dans `Cargo.toml` mais ON N'UTILISE PAS
  `query!`/`query_as!`, donc pas de `.sqlx/` à committer ni `SQLX_OFFLINE`
  à régler en dev)
- Vue 3 Composition API exclusive (`<script setup lang="ts">`), Vite 7,
  Tailwind 3.4, Vue Router 4, Pinia, vite-plugin-pwa, EventSource natif
- Code et commentaires en **anglais**, textes UI en **français**
- Conventional Commits, branche unique `main`, dev solo, commits atomiques
- Components Vue en PascalCase, composables préfixés `use`

## Commandes courantes

```bash
# Backend
cd backend
cargo run                              # :3000, lit theme.ron + pseudo.ron
cargo test                             # 7 tests (5 phase_logic + 2 pseudo)
cargo test phase_logic                 # un module
cargo build --quiet                    # build release-debug
cargo fmt                              # formatage rustfmt
cargo clippy -- -D warnings            # lint strict (convention projet)

# Frontend
cd frontend
npm run dev                            # :5173, proxy /api + /events + /uploads
npm run build                          # bundle prod (10x environ ~250 KB précachés)

# Stack complète
docker compose up --build              # backend + Caddy + dist statique
```

**Pour démarrer une vraie session de dev local** : il faut backend ET vite
en marche. Pour tester l'orchestration, set `WITNESS_SECRET=test-secret`
côté backend ; le token apparaît dans les logs (`witness token = ...`)
ou tu peux le calculer :

```python
python3 -c "
import hmac, hashlib
print(hmac.new(b'test-secret', b'thepond:witness:event:1', hashlib.sha256).hexdigest())
"
```

URL témoin → `/orchestration?token=<ce_token>`.

**Seed manuel des phases** (sqlite3 CLI absent ; passer par Python) :

```python
python3 << 'EOF'
import sqlite3
c = sqlite3.connect('thepond.db')
c.execute("INSERT INTO phases (event_id, phase_order, name, target_time, is_final_reveal) VALUES (1, 0, 'Apéro', '2024-01-01T19:00:00+00:00', 0)")
c.commit()
EOF
```

Au prompt 12 (orchestration) on pourra créer les phases via UI à la place.

## Routes & ce qu'elles touchent

| Route | Type | Touche |
|---|---|---|
| `GET /health` | unauthed | rien (état+version) |
| `GET /api/events` | unauthed | SSE snapshot |
| `GET /api/pseudo` | unauthed | retourne un pseudo généré (sans persister) |
| `POST /api/users` | unauthed | INSERT user, mark_dirty |
| `PATCH /api/users/:id` | unauthed | UPDATE custom_name, mark_dirty |
| `POST /api/media` | unauthed* | INSERT media + thumb 600x600 JPEG, mark_dirty |
| `POST /api/clips` | unauthed* | INSERT clip + placeholder thumb partagé, mark_dirty |
| `POST /api/voice` | unauthed* | INSERT voice + waveform JSON placeholder, mark_dirty |
| `GET /uploads/*` | unauthed | ServeDir |
| `GET /api/orchestration/state` | WitnessAuth | snapshot orchestrateur |
| `POST /api/orchestration/phases` | WitnessAuth | CREATE, mark_dirty |
| `PATCH /api/orchestration/phases/:id` | WitnessAuth | UPDATE name/target_time, mark_dirty |
| `DELETE /api/orchestration/phases/:id` | WitnessAuth | DELETE, mark_dirty |
| `POST /api/orchestration/phases/:id/trigger` | WitnessAuth | force déclenchement, mark_dirty |
| `POST /api/orchestration/phases/reorder` | WitnessAuth | bulk reorder, mark_dirty |
| `GET /api/orchestration/export` | WitnessAuth | ZIP attachment |

\* Vérification "user existe" via SELECT (renvoie 401 si inconnu),
   mais pas d'auth cryptographique côté invité — l'UUID seul fait foi.

## Détails non-obvious à connaître

- **WHITE = warm cream `#EADFB8`** (PAS `#FAF3E3`) — décision documentée dans
  `Duck.vue`. La version "true white" de `Slideshow TV v2.html` était
  contextuelle ; les composants TV peuvent réinjecter via les CSS vars
  `--body / --belly / --shade`.
- **Bec hardcodé `#F2B400`** dans Duck.vue pour TOUTES les variantes
  (yellow/white/blue/rainbow). C'est une signature visuelle.
- **`--duck`** dans `tokens.css` (pas `--duck-yellow` comme PROJECT.md le
  suggère — le HTML maquette fait foi).
- **Optimistic UI uploads** : les capture views et UploadView appellent
  `uploadQueue.enqueue()` puis `router.push('/confirmation?itemId=…')`
  IMMÉDIATEMENT. Le store auto-run le POST en background. ConfirmationView
  observe le statut. Cf `stores/uploadQueue.ts` pour le runner.
- **`sessionStorage` pour le witness token** (pas localStorage) : on veut
  que ça expire à la fermeture d'onglet.
- **Snapshot polymorphe** : `Media.kind = 'photo' | 'clip' | 'voice'`
  discriminator + champs optionnels. UNION ALL avec sub-SELECT corrélé pour
  user_pseudo/color (un LEFT JOIN unifié sur 3 branches était plus pénible).
- **GalleryView 4 états** détectés depuis snapshot uniquement :
  `phaseCurrent`/`phaseVisible`/`phasesAll`/`is_final_reveal`. Pas d'appel
  HTTP supplémentaire.
- **Showcase design system** vit à `/dev` (déplacé de `/` au prompt 7 quand
  `WelcomeView` a pris la racine). Reste utile pour vérifier visuellement
  un composant.
- **Pseudo flow** : `GET /api/pseudo` à chaque reroll côté Welcome. Sur
  PLONGER, le pseudo affiché est envoyé dans `POST /api/users` — le backend
  l'utilise tel quel si fourni, sinon en génère un.
- **`mark_dirty` est un mpsc unbounded** : appel non-bloquant. Le debouncer
  trailing-edge fait le reste. Si un upload échoue ET que tu veux quand
  même rediffuser le snapshot, mark_dirty quand même (idempotent).
- **Vite proxy** route `/api`, `/events`, `/uploads` → backend localhost:3000.
  En prod, Caddy fait le même routing.
- **MareTVInstructionsView** génère un QR de pairing côté client via la lib
  `qrcode` (npm dep, pas de génération serveur). Le QR encode l'URL
  `/mare-tv` cible — c'est le seul écran qui en utilise pour V1.

## Couleurs réservées (rappel)

- **Champagne** `#E8C77A` — UNIQUEMENT pour les mariées (couronnes Duck,
  border carte couple, halo nuit Mare TV)
- **Coral** `#E07A5F` (et la famille `coral-soft/-deep/-line`) — UNIQUEMENT
  pour le destructif (delete, danger zone) ET pour FAIRE COIN-COIN selon
  la classe CSS (la maquette Upload v2 a un drift inline qui force cream,
  arbitrage à faire au moment où on retravaillera UploadView)

Composant signature : bouton primaire physique avec
`box-shadow: 0 4px 0 var(--duck-deep)`.

## Phases de livraison

V1 (avant 5 juin 2026) / P1 / P2 dans `PROJECT.md` § "Phases de
développement". Les prompts thématiques 1 à 16 dans `BOOTSTRAP.md`
§ "Après cette session" — actuellement faits jusqu'au 13. Prompts 14
(PWA install + iOS sheet), 15 (polish + ErrorView), 16 (déploiement NAS)
restants.
