# The Pond — Bible du projet

> Document de référence pour Claude Code et tout dev qui touche au projet.
> Lire ce fichier en début de chaque session de dev.

## Contexte

The Pond est une PWA de partage de photos, clips vidéo et messages vocaux
pour un mariage privé entre amis. ~50 invités, soirée de 6 heures, le 5 juin 2026.

L'app implémente un concept "appareil photo jetable" : les invités postent
pendant la soirée, mais ne voient pas immédiatement les contenus. Une mécanique
de paliers temporels gère la révélation progressive des souvenirs.

Le projet est self-hosté sur un NAS Synology DS718+ (CPU Intel Celeron J3355,
2 cœurs, Quick Sync hardware). Open source sous licence beerware. Code librement
réutilisable pour d'autres mariages avec une thématisation différente.

## Vocabulaire du projet (à respecter strictement)

| Terme | Sens |
|-------|------|
| **The Pond.** | Nom de marque, en anglais. Logo en Caprasimo. |
| **La mare** | Le lieu virtuel où les canards barbotent. Toujours "la mare", JAMAIS "l'étang". |
| **Mare TV** | Le grand écran de diffusion dans la salle de fête. |
| **Barboter** | Verbe pour les photos. "Barboter une photo." |
| **Cancaner** | Verbe pour les clips vidéo. "Cancaner un clip." |
| **Coin-coin** | Verbe pour les messages vocaux. "Faire coin-coin." |
| **Plonger** | Verbe générique pour entrer dans la mare. |
| **Les barbotages** | Nom collectif pour tous les contenus postés. |
| **Témoin** | Utilisateur qui a accès à l'orchestration (modérateur sans pouvoir de modération). |

### Microcopy figée

- Salutation jour (avant 23h) : `"Bonsoir [pseudo]"`
- Salutation nuit (après 23h) : `"Tes plumes scintillent magnifiquement, [pseudo]"`
- Confirmation post-upload : `"Splash !"` + `"Ton coin-coin a rejoint la mare"`
- CTA post-upload : `"CONTINUER À BARBOTER"` / `"CONTINUER À CANCANER"` / `"CONTINUER À COIN-COIN"`
- Lien galerie : `"Voir ce qui se passe dans la mare →"`
- Lien Mare TV : `"Voir Mare TV →"`
- Install PWA : `"Pour ne pas perdre la mare ce soir"`

### États de la galerie (microcopy templatisée)

- **État 1 (mare endormie initiale)** : `"Les premiers barbotages se révèleront avec {nom_palier_2} vers {heure}"`
- **État 2 (mare réveillée, arrivée d'un palier)** : `"{nom_palier_précédent} vient d'arriver · {N} souvenirs"`
- **État 2 (annonce du palier suivant)** : `"Les souvenirs de {palier_en_cours} se révèleront avec {palier_suivant} vers {heure}"`
- **État 3 (mare nocturne)** : `"Plus assez de lumière aujourd'hui · on y verra mieux demain pour les autres barbotages"`
- **État 4 (mare révélée, lendemain)** : `"Les canards sont confits · dégustons maintenant..."`

## Stack technique

### Backend
- **Langage** : Rust stable
- **Framework HTTP** : Axum 0.7
- **Runtime async** : Tokio
- **DB** : SQLite via SQLx (requêtes SQL vérifiées à la compilation)
- **Real-time** : Server-Sent Events (SSE) avec snapshot push
- **Auth invités** : UUID stocké en localStorage côté client
- **Auth témoins** : token HMAC partagé pour tous les témoins, généré au boot
- **Build** : Cargo, Docker multi-stage pour la prod

### Frontend
- **Framework** : Vue 3 + Composition API
- **Build** : Vite + TypeScript
- **Styling** : Tailwind CSS avec config custom (tokens du design system v0.2)
- **Routing** : Vue Router 4
- **State** : Pinia
- **PWA** : vite-plugin-pwa
- **Real-time** : EventSource natif

### Infrastructure
- **Cible** : NAS Synology DS718+
- **Conteneurisation** : Docker + Docker Compose
- **Reverse proxy** : Caddy 2 (dans le compose, dédié au projet)
- **TLS** : DNS-01 via DuckDNS (config dans Caddyfile)
- **Persistance** : volumes Docker mappés sur `/volume1/docker/thepond/`

## Architecture temps réel

### Choix : SSE avec snapshot push (pas WebSocket, pas delta-based)

Le backend expose un endpoint SSE `GET /api/events`. Quand un client se connecte,
il reçoit immédiatement le **snapshot complet** de l'état actuel. Quand un
changement survient (upload, transition de palier, etc.), le backend recalcule
le snapshot et le rediffuse à tous les clients abonnés.

**Le snapshot contient l'état complet, pas des deltas.** Un client qui se
reconnecte n'a rien à reconstruire : il remplace son état local par le snapshot
reçu et c'est terminé.

### Structure du snapshot

```typescript
type Snapshot = {
  server_time: string,           // timestamp ISO du serveur
  phase_current: Phase,          // palier en cours
  phase_visible: Phase | null,   // palier dont les contenus sont visibles
  phases_all: Phase[],           // tous les paliers (pour orchestration)
  media_visible: Media[],        // médias visibles dans la galerie
  media_recent_for_tv: Media[],  // médias du palier en cours pour Mare TV
  counts: {
    total_users: number,
    total_posts: number,
    posts_visible: number,
    posts_pending: number
  }
}
```

### Flux client/serveur

1. Client charge la page → fetch initial via REST classique
2. Client ouvre un EventSource sur `/api/events`
3. Serveur envoie immédiatement le snapshot actuel
4. Client reçoit, met à jour son store Pinia, l'UI se rafraîchit via réactivité
5. Quand un événement modifie l'état (upload, palier, etc.) :
   - Backend persiste le changement
   - Backend recalcule le snapshot
   - Backend diffuse le snapshot via broadcast channel
   - Tous les clients connectés reçoivent et mettent à jour leur état
6. Si le client se déconnecte (perte wifi, app en arrière-plan) :
   - EventSource gère la reconnexion automatiquement
   - Au retour, le serveur envoie le snapshot actuel
   - Le client est immédiatement à jour, pas de gestion d'événements manqués

### Debounce des updates

Quand plusieurs changements surviennent rapidement (5 invités postent en 1 seconde),
le backend agrège avec un debounce de 300ms et envoie un seul snapshot.

## Mécanique des paliers (révélation temporelle)

### Concept fondamental

Pendant le palier N en cours, la galerie affiche **uniquement** les contenus
du palier N-1. Les contenus du palier en cours sont invisibles jusqu'à ce que
le palier suivant ouvre.

À la fin de la soirée (dernier palier), la galerie passe en état "nocturne" :
plus rien n'est visible. Le lendemain, les témoins déclenchent la révélation
finale qui rend visible **tous** les contenus de la soirée d'un coup.

### Configuration des paliers

Les paliers sont configurés par les témoins via la page d'orchestration. Chaque
palier a :
- Un nom (texte libre, ex: "Apéro", "Premières danses")
- Une heure cible (auto-trigger quand l'heure passe)
- Un statut (calculé dynamiquement : locked / current / revealed)

Les témoins peuvent forcer manuellement l'ouverture d'un palier en avance.

### Paliers par défaut

Définis dans `themes/duck-pond/phases.ron` :

```ron
[
  ("Apéro", "19:00"),
  ("Dîner", "20:30"),
  ("Dessert", "22:30"),
  ("Café", "00:00"),
  ("Le réveil de la mare", "11:00 J+1"),
]
```

Les témoins peuvent les renommer, les réorganiser, en ajouter ou supprimer.

### Calcul de l'état des paliers (à la demande, pas via job)

```sql
-- Palier en cours : le dernier dont target_time est passé
SELECT * FROM phases
WHERE event_id = ? AND target_time <= NOW()
ORDER BY phase_order DESC
LIMIT 1;

-- Palier visible : le précédent du palier en cours
SELECT * FROM phases
WHERE event_id = ? AND target_time <= NOW()
ORDER BY phase_order DESC
LIMIT 1 OFFSET 1;

-- Médias visibles : ceux postés dans la fenêtre du palier visible
SELECT * FROM media
WHERE event_id = ?
  AND posted_at >= (heure de début du palier visible)
  AND posted_at < (heure de début du palier en cours);
```

### Forçage manuel d'un palier

Quand un témoin force un palier en avance, on met à jour `target_time = NOW()`
au lieu d'ajouter un champ `forced_at`. La logique d'affichage continue à
fonctionner sans modification.

## Schéma de base de données

```sql
-- Configuration de l'événement (1 seul row)
CREATE TABLE event_config (
  id INTEGER PRIMARY KEY,
  event_name TEXT NOT NULL,        -- "Le mariage de Marie & Sarah"
  expected_guests_count INTEGER,   -- 52
  theme_path TEXT NOT NULL,        -- "themes/duck-pond"
  witness_token_hash TEXT NOT NULL, -- HMAC du secret pour les témoins
  created_at TEXT NOT NULL
);

-- Liste des invités attendus (juste pour le compteur)
-- Importée depuis le CSV au boot, ou simple chiffre
-- Décision finale : juste un nombre dans event_config (pas de table dédiée)

-- Utilisateurs (canards)
CREATE TABLE users (
  id TEXT PRIMARY KEY,             -- UUID généré côté client
  pseudo TEXT NOT NULL,
  custom_name TEXT,                -- "vrai prénom" optionnel
  duck_color TEXT NOT NULL,        -- "yellow" / "white" / "blue" / "rainbow"
  created_at TEXT NOT NULL,
  last_seen_at TEXT
);

-- Paliers (phases)
CREATE TABLE phases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id INTEGER NOT NULL,
  phase_order INTEGER NOT NULL,
  name TEXT NOT NULL,
  target_time TEXT NOT NULL,       -- ISO timestamp
  triggered_at TEXT,               -- ISO timestamp réel quand déclenché
  is_final_reveal BOOLEAN NOT NULL DEFAULT 0,
  UNIQUE(event_id, phase_order)
);

-- Médias photo
CREATE TABLE media (
  id TEXT PRIMARY KEY,             -- UUID
  user_id TEXT NOT NULL REFERENCES users(id),
  filename TEXT NOT NULL,          -- nom du fichier sur disque
  thumb_filename TEXT NOT NULL,
  posted_at TEXT NOT NULL,
  hidden INTEGER NOT NULL DEFAULT 0
);

-- Clips vidéo
CREATE TABLE clips (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id),
  filename TEXT NOT NULL,
  thumb_filename TEXT NOT NULL,    -- frame extraite à 1s
  duration_seconds REAL NOT NULL,
  posted_at TEXT NOT NULL,
  hidden INTEGER NOT NULL DEFAULT 0
);

-- Messages vocaux
CREATE TABLE voice_messages (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id),
  filename TEXT NOT NULL,
  waveform_json TEXT NOT NULL,     -- amplitudes pré-calculées
  duration_seconds REAL NOT NULL,
  caption TEXT,                    -- caption optionnelle
  posted_at TEXT NOT NULL,
  hidden INTEGER NOT NULL DEFAULT 0
);

-- Likes (cœurs)
CREATE TABLE likes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL REFERENCES users(id),
  content_type TEXT NOT NULL CHECK(content_type IN ('media', 'clip', 'voice')),
  content_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(user_id, content_type, content_id)
);
```

## Stockage des fichiers

Structure sur le disque :
```
/volume1/docker/thepond/
├── data/
│   └── thepond.db          # SQLite
└── uploads/
    ├── photos/
    │   ├── original/
    │   │   └── {uuid}.jpg
    │   └── thumb/
    │       └── {uuid}.jpg  # 600x600 max
    ├── clips/
    │   ├── original/
    │   │   └── {uuid}.mp4
    │   └── thumb/
    │       └── {uuid}.jpg  # frame à 1s
    └── voice/
        └── {uuid}.{webm|mp4}
```

### Compression côté client

- **Photos** : pas de compression. Upload de l'original (JPEG/PNG/HEIC).
  HEIC est converti en JPEG côté serveur via libheif.
- **Vidéos** : compression côté client à 1080p maximum, bitrate raisonnable
  (~5 Mbps). Utilisation d'une lib comme `@ffmpeg/ffmpeg` (WASM) ou de l'API
  WebCodecs si supportée. Si compression impossible, upload tel quel.
- **Vocaux** : pas de compression. Format natif MediaRecorder
  (`audio/webm` sur Chrome, `audio/mp4` sur Safari).

### Génération côté serveur

- **Thumbnails photos** : 600x600 max, JPEG qualité 80
- **Thumbnails clips** : extraction frame à 1s via ffmpeg, 600x600 max
- **Waveform vocaux** : 100 amplitudes échantillonnées, sauvegardées en JSON

## Authentification

### Invités

Au scan QR :
1. Le client génère un UUID aléatoire si pas déjà en localStorage
2. Le UUID est envoyé au serveur avec le pseudo généré
3. Le serveur crée la row dans `users`
4. Le UUID est utilisé comme identifiant pour tous les uploads

Pas de session, pas de cookie. Juste un UUID en localStorage.

### Témoins

Un seul token HMAC partagé pour tous les témoins, généré au boot du serveur.
Le token est intégré dans un QR code séparé que tu donnes aux témoins.

URL des témoins : `https://thepond.tld/orchestration?token=XXX`

Le token est vérifié côté serveur sur chaque requête à `/api/orchestration/*`.
Si le token est invalide, 403.

Tu peux invalider tous les tokens en redémarrant le serveur (regénère le secret).

## Architecture monorepo

```
ThePond/
├── README.md
├── PROJECT.md                  # ce fichier
├── BOOTSTRAP.md                # premier prompt pour Claude Code
├── .gitignore
├── docker-compose.yml          # stack complet (backend + caddy)
├── Caddyfile                   # config Caddy avec DuckDNS
│
├── backend/
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── migrations/             # SQLx migrations
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs           # lecture du theme et env vars
│   │   ├── db.rs               # connexion SQLite
│   │   ├── error.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs         # création canard
│   │   │   ├── media.rs        # upload photos
│   │   │   ├── clips.rs        # upload vidéos
│   │   │   ├── voice.rs        # upload vocaux
│   │   │   ├── likes.rs        # likes
│   │   │   ├── phases.rs       # passage de palier
│   │   │   ├── orchestration.rs# admin témoins
│   │   │   ├── events.rs       # SSE
│   │   │   └── public.rs       # snapshot REST initial
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   ├── phase.rs
│   │   │   ├── media.rs
│   │   │   ├── clip.rs
│   │   │   ├── voice.rs
│   │   │   ├── snapshot.rs
│   │   │   └── theme.rs        # parsing RON
│   │   └── services/
│   │       ├── pseudo_generator.rs
│   │       ├── phase_logic.rs
│   │       ├── thumbnail.rs    # via image crate ou ffmpeg
│   │       ├── waveform.rs     # via symphonia
│   │       ├── hmac.rs
│   │       └── broadcast.rs    # tokio::sync::broadcast
│   └── tests/
│
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   ├── tsconfig.json
│   ├── Dockerfile
│   ├── public/
│   │   ├── manifest.json
│   │   └── icons/
│   └── src/
│       ├── main.ts
│       ├── App.vue
│       ├── router/
│       │   └── index.ts
│       ├── stores/
│       │   ├── snapshot.ts     # store SSE
│       │   ├── user.ts         # canard de l'invité courant
│       │   └── upload_queue.ts # queue locale d'uploads
│       ├── composables/
│       │   ├── useEventStream.ts
│       │   ├── useUploadQueue.ts
│       │   └── useTheme.ts
│       ├── components/
│       │   ├── Duck.vue        # SVG paramétrique du canard
│       │   ├── Pond.vue        # mare avec couple + invités
│       │   ├── PondCounter.vue
│       │   ├── PrimaryButton.vue
│       │   ├── SecondaryButton.vue
│       │   ├── MediaCard.vue
│       │   ├── ClipCard.vue
│       │   ├── VoiceCard.vue
│       │   ├── WaveformPlayer.vue
│       │   ├── PhasesList.vue  # pour orchestration
│       │   ├── ChallengeBanner.vue
│       │   └── ...
│       ├── views/
│       │   ├── WelcomeView.vue
│       │   ├── UploadView.vue
│       │   ├── ConfirmationView.vue
│       │   ├── GalleryView.vue
│       │   ├── ClipPlayerView.vue
│       │   ├── VoicePlayerView.vue
│       │   ├── OrchestrationView.vue
│       │   ├── MareTVView.vue
│       │   ├── MareTVInstructionsView.vue
│       │   └── ErrorView.vue
│       └── styles/
│           ├── tokens.css      # variables CSS du design system
│           └── global.css
│
├── themes/
│   └── duck-pond/
│       ├── theme.ron           # config principale
│       ├── pseudo.ron          # générateur de pseudos
│       ├── phases.ron          # paliers par défaut
│       ├── strings.ron         # toutes les microcopies
│       ├── palette.css         # variables CSS spécifiques au thème
│       └── assets/
│           └── duck.svg        # template du canard
│
└── design/
    ├── INDEX.md                      # inventaire commenté des HTML
    ├── The Pond - Design System.html # référence canonique du design system v0.2
    ├── Slideshow TV v2.html
    ├── Welcome v2.html
    ├── Upload v2.html
    ├── Confirmation.html
    ├── Mosaic 4 screens.html
    └── Mosaic 6 screens.html
```

## Phases de développement

### Phase 0 (V1) — Fonctionnalités du week-end

À livrer **avant le 5 juin 2026** :

**Backend**
- Migrations SQLx initiales
- Routes auth (création canard)
- Routes upload photo/clip/vocal avec génération thumbnail/waveform
- Routes orchestration (paliers, paramètres globaux, export zip)
- Endpoint SSE avec snapshot push
- Logique de calcul des paliers (à la demande, pas via job)
- HMAC pour auth témoins
- Parsing du thème depuis fichiers RON

**Frontend**
- Bootstrap Vite + Vue 3 + TypeScript + Tailwind
- Configuration PWA (manifest, service worker, install prompt)
- Composant Duck.vue paramétrique (SVG du canard avec contour)
- Composant Pond.vue (mare avec couple central + invités)
- Vue Router avec toutes les routes
- Store Pinia avec snapshot SSE
- Queue d'upload locale en localStorage
- Tous les écrans validés (slideshow, welcome, upload, confirmation,
  galerie 4 états, orchestration, Mare TV, vocal player, clip player,
  Mare TV instructions)
- Capture in-app pour clips et vocaux via MediaRecorder
- Compression vidéo côté client à 1080p
- Écrans d'erreur d'upload
- Sheet d'instructions iOS pour install PWA
- Likes (cœurs) sur tous les contenus

**Infrastructure**
- Docker Compose avec Caddy + backend + (frontend statique servi par Caddy)
- Caddyfile avec DuckDNS DNS-01
- Volumes persistants mappés sur `/volume1/docker/thepond/`
- Variables d'environnement (THEME_PATH, BIND_ADDRESS, DB_PATH, etc.)

### Phase 1 (P1) — À ajouter après le mariage si désiré

- Fonctionnalité défis (scavenger hunt) avec un défi actif à la fois
- Vue détaillée du défi en cours
- Commentaires textuels sur les contenus
- Réponses vocales en chaîne (vocal qui répond à un vocal)
- Sauvegarde personnelle de contenus en favoris
- État hors-ligne intelligent
- Variantes confirmation post-upload pour clip et vocal
- Compression vidéo plus avancée (option qualité)
- Notifications PWA push

### Phase 2 (P2) — Évolutions ultérieures

- Spotify queue (proposer des morceaux à la file d'attente DJ)
- Mur des paris (parier sur des événements de la soirée)
- Awards auto-générés (le canard le plus actif, le plus drôle, etc.)
- Supercut vidéo final automatique
- Impression thermique Niimbot

## Design system v0.2

Le design system complet est documenté dans `design/The Pond - Design System.html`.
Tokens CSS dans `frontend/src/styles/tokens.css` (variables CSS).

### Palette (9 couleurs)

```css
--pond-deep: #0E4F6B;      /* fonds, titres */
--pond-mid: #3E8AA8;       /* secondaire, focus */
--pond-light: #A9D8E5;     /* accents */
--pond-pale: #E6F3F8;      /* backgrounds clairs */
--cream: #FAF3E3;          /* fond app */
--cream-deep: #F2E8CF;     /* fond cards */
--duck-yellow: #FFC93C;    /* CTA primaire */
--duck-deep: #F2B400;      /* shadow CTA primaire */
--champagne: #E8C77A;      /* RÉSERVÉ aux mariées */
--coral: #E07A5F;          /* RÉSERVÉ destructif */
--coral-soft: #F8D9CF;     /* fond bouton vocal */
--ink: #1F2933;            /* texte courant */
```

### Typographie

- **Caprasimo** : display (titres, pseudos, "Splash !")
- **DM Sans** : humaniste (texte courant, labels, sub-text)
- **DM Mono** : annotations (timestamps, durées, sub-labels, meta)

### Composants visuels signatures

- **Bouton primaire physique** : `box-shadow: 0 4px 0 var(--duck-deep)`
- **Canards** : 4 variantes ratio 80/15/4/1, contour Pond Deep 1.5px
- **Mariées** : 2 canards 140px (jaune + blanc), couronnes champagne, côte à côte
- **Mode nuit** : déclenché après 23h pour Mare TV (galerie reste cream)

### Génération du SVG canard

Le composant Duck.vue prend en props :
- `color: 'yellow' | 'white' | 'blue' | 'rainbow'`
- `size: number` (en px, défaut 56)
- `crowned: boolean` (défaut false)
- `glow: boolean` (mode nuit, défaut false)

Le SVG est inline avec contour Pond Deep, et la couleur du body change selon
le prop. La couronne, si présente, mord le crâne de 2-3px (overlap).

## Conventions de code

### Backend Rust
- Format avec `cargo fmt` (rustfmt par défaut)
- Lint avec `cargo clippy -- -D warnings`
- Tests avec `cargo test`
- Logs structurés avec `tracing`
- Erreurs typées via `thiserror`

### Frontend Vue
- Format avec Prettier
- Lint avec ESLint (config Vue 3 + TypeScript recommandée)
- Composition API exclusivement, pas d'Options API
- `<script setup lang="ts">` partout
- Pas de mixins, pas de props drilling — utiliser Pinia ou provide/inject
- Naming : composants en PascalCase, composables avec préfixe `use`

### Commits
- Convention : Conventional Commits (`feat:`, `fix:`, `chore:`, etc.)
- Branche principale : `main`
- Pas de PR (dev solo), mais commits propres et atomiques

## Liens utiles

- Dépôt : https://github.com/Iluine/ThePond
- Maquettes Claude Design : voir `design/` (inventaire dans `design/INDEX.md`)
- Spec design system : voir `design/The Pond - Design System.html`
