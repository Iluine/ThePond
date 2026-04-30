# BOOTSTRAP — Premier prompt pour Claude Code

> Ce fichier sert UNIQUEMENT pour la première session de dev avec Claude Code.
> Une fois que le projet est bootstrappé, ce fichier peut être supprimé.

## Comment utiliser ce fichier

1. Lis d'abord PROJECT.md en entier pour comprendre le projet
2. Copie le prompt ci-dessous dans Claude Code
3. Lance Claude Code à la racine du dépôt
4. Suis les étapes au fur et à mesure

---

## PROMPT À COLLER DANS CLAUDE CODE

```
Tu vas bootstrapper le projet The Pond, une PWA de partage de contenus
pour un mariage. Lis d'abord PROJECT.md à la racine pour comprendre le contexte
complet, le vocabulaire, la stack, et l'architecture.

PRÉREQUIS ABSOLU AVANT D'ÉCRIRE LA MOINDRE LIGNE DE CODE :

Tu dois OUVRIR ET LIRE INTÉGRALEMENT chacun des fichiers HTML du dossier
design/ avant de générer le moindre fichier de code. Utilise l'outil Read
sur chacun :

- design/INDEX.md (inventaire commenté, statut de chaque maquette)
- design/The Pond - Design System.html (SOURCE DE VÉRITÉ pour tokens.css
  et tailwind.config.js : palette exacte, typographies, box-shadows,
  composants signatures, SVG des canards)
- design/Welcome v2.html
- design/Upload v2.html
- design/Confirmation.html
- design/Slideshow TV v2.html
- design/Mosaic 4 screens.html
- design/Mosaic 6 screens.html

Les couleurs (codes hex), espacements, rayons, ombres, structures DOM,
et SVG des canards doivent être EXTRAITS DIRECTEMENT du HTML, pas
réinventés ni approximés. Si une valeur dans le HTML diffère de celle
listée dans PROJECT.md, le HTML fait foi (PROJECT.md est un résumé).

Cette lecture n'est pas optionnelle. Toute génération de tokens.css,
tailwind.config.js, ou composant Vue avant cette lecture est un échec
du bootstrap.

OBJECTIF DE CETTE SESSION : avoir un Hello World qui tourne en local avec :
- Backend Rust/Axum qui démarre, expose un endpoint de health check, lit
  la config depuis un fichier RON, se connecte à SQLite avec migrations
- Frontend Vue 3 + Vite + TypeScript + Tailwind qui démarre, affiche une
  page d'accueil basique, charge le design system (tokens.css)
- Docker Compose qui orchestre backend + Caddy + frontend statique
- Les fichiers de thème (themes/duck-pond/) avec des valeurs minimales

NE PAS implémenter les fonctionnalités métier dans cette session.
On veut juste que ça démarre proprement.

ÉTAPES À RÉALISER :

1. Crée le .gitignore racine avec les patterns nécessaires (target/, node_modules/,
   *.db, dist/, .env, .DS_Store, etc.)

2. Crée la structure de dossiers conforme à PROJECT.md :
   backend/, frontend/, themes/duck-pond/, docs/

3. Backend Rust :
   - Cargo.toml avec dépendances : axum 0.7, tokio (full features), sqlx
     (sqlite, runtime-tokio-rustls, macros), serde, serde_json, ron, tracing,
     tracing-subscriber, thiserror, anyhow, uuid (v4), chrono
   - src/main.rs : démarrage Axum sur 0.0.0.0:3000, endpoint GET /health
     qui retourne { "status": "ok", "version": env!("CARGO_PKG_VERSION") }
   - src/config.rs : lecture du fichier theme.ron via la variable d'env
     THEME_PATH (défaut: themes/duck-pond/theme.ron)
   - src/db.rs : connexion SQLite via SQLx avec migrations automatiques
   - src/error.rs : type AppError avec thiserror et conversion en réponse HTTP
   - migrations/0001_init.sql : tables event_config, users, phases, media,
     clips, voice_messages, likes (selon le schéma de PROJECT.md)
   - Dockerfile multi-stage : build avec rust:1.83-slim, runtime avec debian:bookworm-slim

4. Frontend Vue 3 :
   - Initialise avec : npm create vite@latest frontend -- --template vue-ts
   - Installe les deps : tailwindcss, postcss, autoprefixer, vue-router, pinia,
     vite-plugin-pwa
   - Configure tailwind.config.js avec les tokens du design system v0.2
     (couleurs, fonts, box-shadow custom)
   - src/styles/tokens.css avec les variables CSS du design system
   - src/App.vue minimal qui affiche "The Pond." en Caprasimo + un canard SVG
     basique (juste un rond jaune avec un bec) pour valider le rendu Tailwind
   - vite.config.ts avec PWA plugin configuré (manifest minimal)
   - Dockerfile multi-stage : build avec node:22-alpine, runtime statique servi
     par Caddy directement (pas besoin de container Node en prod)

5. Themes :
   - themes/duck-pond/theme.ron : config minimale avec event_name,
     expected_guests_count, witness_secret_env_var
   - themes/duck-pond/phases.ron : 5 paliers par défaut (Apéro, Dîner, Dessert,
     Café, Le réveil de la mare) avec target_time
   - themes/duck-pond/strings.ron : structure pour toutes les microcopies
   - themes/duck-pond/pseudo.ron : listes de mots pour générer les pseudos
     (Canard + adverbe + adjectif/verbe)

6. Caddy :
   - Caddyfile avec config DNS-01 DuckDNS, reverse proxy vers le backend Rust
     sur /api/* et /events, et serveur statique pour le reste (SPA fallback)
   - Note : pour le dev local, on utilise un Caddyfile.local sans TLS

7. Docker Compose :
   - docker-compose.yml avec services : backend, caddy
   - Volumes pour SQLite et uploads
   - Réseau interne
   - Variables d'environnement (DB_PATH, UPLOADS_PATH, THEME_PATH,
     WITNESS_SECRET, DUCKDNS_TOKEN)
   - Healthchecks
   - Restart policies

8. README.md à la racine :
   - Brève présentation du projet
   - Instructions de dev local (npm run dev pour le frontend,
     cargo run pour le backend)
   - Instructions de déploiement (docker compose up -d)
   - Lien vers PROJECT.md pour la doc complète

CONTRAINTES IMPORTANTES :

- Suis STRICTEMENT le vocabulaire du projet (la mare, Mare TV, barboter,
  cancaner, coin-coin) dans tous les textes UI et commentaires
- Utilise le design system v0.2 (couleurs, fonts, espacements) défini dans
  PROJECT.md
- Code en français pour les textes UI, en anglais pour le code et les
  commentaires
- Privilégie la simplicité : pas de patterns over-engineered, pas d'abstraction
  prématurée
- Code qui compile et qui tourne au premier coup. Si tu as un doute sur une
  dépendance ou une syntaxe, vérifie sur les docs officielles avant.
- Utilise SQLx avec compile-time query checking (active SQLX_OFFLINE et génère
  les .sqlx/ files si nécessaire)
- Pour le SVG du canard initial, fais quelque chose de très basique (un cercle
  jaune avec un bec triangulaire). On affinera plus tard avec le vrai design.

VALIDATION DE FIN DE SESSION :

À la fin, je dois pouvoir :
1. Aller dans backend/ et faire `cargo run`, le serveur démarre sur :3000
   et /health répond
2. Aller dans frontend/ et faire `npm run dev`, le serveur Vite démarre,
   j'ouvre le navigateur, je vois "The Pond." en Caprasimo et un canard
3. Faire `docker compose up --build` à la racine et avoir tout qui tourne
   ensemble (backend accessible via Caddy, frontend statique servi)

Si tu rencontres une décision non documentée dans PROJECT.md, demande-moi
avant de la prendre.
```

---

## Après cette session

Une fois le bootstrap réussi et committé, on enchaîne avec les prompts
thématiques dans cet ordre :

### Prompt 1 — Design System complet
Implémenter tokens.css complet avec toutes les variables, tailwind.config.js
avec tous les tokens custom, et créer les composants de base réutilisables :
PrimaryButton.vue, SecondaryButton.vue, PondCounter.vue, MareTVPill.vue.

### Prompt 2 — Composant Duck.vue paramétrique
Créer le composant SVG du canard avec props (color, size, crowned, glow).
Utiliser le design exact du design system v0.2 (forme du corps, contour Pond
Deep 1.5px, couronne champagne avec overlap 2-3px). Inclure les 4 variantes
de couleur. Tester avec une page de démo.

### Prompt 3 — Composant Pond.vue (la mare)
Créer le composant qui affiche la mare avec le couple central (mariées) et
les invités scattered en poisson-disk. Props : ducks (liste), couple_visible
(boolean), night_mode (boolean). Animation discrète des canards (bobbing).

### Prompt 4 — Routing et stores Pinia
Configurer Vue Router avec toutes les routes. Créer les stores Pinia :
useUserStore (canard de l'invité), useSnapshotStore (état SSE), useUploadQueueStore
(queue locale).

### Prompt 5 — Composable useEventStream et SSE backend
Implémenter le composable côté frontend pour consommer le SSE. Côté backend,
implémenter l'endpoint /api/events avec broadcast channel et calcul du snapshot
à la demande.

### Prompt 6 — Backend : routes auth et upload
Implémenter les routes POST /api/users (création canard avec génération de
pseudo), POST /api/media (upload photo avec génération thumbnail), POST /api/clips,
POST /api/voice. Génération de UUID, validation de taille, écriture dans le
système de fichiers, persistance en base.

### Prompt 7 — Écran Welcome (génération pseudo)
Implémenter WelcomeView.vue avec : header The Pond, mini-pond teaser, carte
de génération de canard avec rerouler, champ pseudo optionnel, CTA primaire,
zone install PWA. Connecté au backend via POST /api/users.

### Prompt 8 — Écran Upload + capture in-app
Implémenter UploadView.vue avec les 3 boutons (BARBOTER, CANCANER, FAIRE COIN-COIN).
Implémenter les vues de capture in-app pour clips (15s) et vocaux (60s) via
MediaRecorder. Compression vidéo à 1080p côté client.

### Prompt 9 — Confirmation post-upload
Implémenter ConfirmationView.vue avec l'animation 3-frames du canard qui plonge.
Optimistic UI : affiché dès que l'upload commence, basculé sur erreur si l'upload
échoue.

### Prompt 10 — Galerie 4 états
Implémenter GalleryView.vue qui détecte l'état actuel (endormie/réveillée/
nocturne/révélée) et affiche le bon contenu. Filter chips, masonry, cards
avec attribution.

### Prompt 11 — Lecteurs clip et vocal
Implémenter ClipPlayerView.vue (TikTok-style fullscreen vertical) et
VoicePlayerView.vue (waveform animé, gros bouton play, navigation prev/next).

### Prompt 12 — Page Orchestration témoins
Implémenter OrchestrationView.vue avec auth HMAC, gestion des paliers
(drag-drop, édition nom/heure, force trigger), paramètres globaux, export zip.

### Prompt 13 — Mare TV et instructions
Implémenter MareTVView.vue (page kiosque pour TV) avec rotation aléatoire
des contenus du palier en cours côté client. Implémenter
MareTVInstructionsView.vue avec QR de pairing.

### Prompt 14 — PWA installation et iOS sheet
Configurer le manifest PWA complet, le service worker pour l'offline-first,
les icônes. Implémenter la sheet d'instructions iOS pour install manuelle.

### Prompt 15 — Polish et états d'erreur
Implémenter ErrorView.vue, les toasts, les loaders, les états vides, la
gestion des reconnexions SSE. Tests sur device réel iOS et Android.

### Prompt 16 — Déploiement NAS
Build production, déploiement sur le NAS Synology, configuration Caddy
DuckDNS, tests grandeur nature avec quelques amis cobayes.

---

## Conseils pour l'utilisation de Claude Code avec Superpowers

- À chaque session, **rappelle à Claude Code de relire PROJECT.md** s'il
  semble dériver du vocabulaire ou des conventions
- Utilise Superpowers pour **appliquer les changements en multi-fichiers**
  (les écrans modifient souvent backend + frontend en parallèle)
- Quand tu valides un prompt, **vérifie immédiatement que ça compile et tourne**
  avant d'enchaîner
- **Commits atomiques après chaque prompt validé** pour pouvoir revenir en
  arrière si besoin
- Si Claude Code prend une décision non triviale, **demande-lui d'expliquer
  son raisonnement** avant d'accepter
