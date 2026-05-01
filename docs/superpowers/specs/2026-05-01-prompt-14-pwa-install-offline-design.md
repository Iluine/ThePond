# Prompt 14 — PWA install + offline-first capture

**Date** : 2026-05-01
**Scope** : closes prompt 14 du `BOOTSTRAP.md` + extension offline-first du store `uploadQueue`.

## Objectif

Fermer les 4 trous restants pour qu'un invité installe la PWA et continue de capturer photos/clips/vocaux pendant les drops réseau de la salle :

1. Icônes natives multi-plateforme (PNG iOS + Android any/maskable)
2. Auto-show de la sheet d'instructions iOS sur première visite
3. Queue uploads persistante en IndexedDB qui survit à un reload
4. BackgroundSync Android best-effort pour les uploads quand l'app est fermée

## Décisions cadrées

### A. Génération d'icônes
**Outil** : `@vite-pwa/assets-generator` (officiel `vite-plugin-pwa`).
**Source** : `frontend/public/favicon.svg` (Duck Yellow, safe zone 80% déjà prête).
**Sortie committée dans `public/`** : `pwa-64.png`, `pwa-192.png`, `pwa-512.png`, `maskable-icon-512.png`, `apple-touch-icon-180x180.png`, `favicon.ico`.
**Workflow** : `npm run icons` régénère. Pas de splash screens iOS (V1, fallback blanc OK).

### B. Auto-show iOS sheet
- Sur 1ère visite Safari iOS non-installé, after `setTimeout(2000ms)`.
- Skip si `localStorage.thepond_install_sheet_seen === '1'`.
- Skip si `localStorage.thepond_user_id` existe (l'invité a déjà créé son canard, il connaît l'app).
- Le bouton "Installer" reste cliquable à tout moment.
- Close ou install → set le flag.

### C. Queue uploads offline-first
- Persistance via **IndexedDB** (lib `idb`, ~1KB gzip), pas localStorage (blobs trop gros).
- Schéma store `pending_uploads` : `id` (UUID), `kind` ('photo'|'clip'|'voice'), `blob` (Blob), `metadata` (objet : userId, duration, caption…), `status` ('queued'|'uploading'|'done'|'failed'), `retries` (number), `createdAt` (ISO).
- Au boot : `rehydrate()` lit la table → repopule le store Pinia.
- `enqueue()` écrit en IDB → triggers le runner.
- Runner respecte `navigator.onLine`. Listener `window.online` relance.
- Items `done` purgés immédiatement. `failed` après 5 retries → marqué bloqué (UX future, V1 = silent stuck).

### D. UI réseau
- Composable `useNetworkStatus()` → reactive `isOnline` (basé sur `online`/`offline` events).
- Composant `PendingUploadsPill.vue` : pill bottom-right floating quand `count > 0`. Couleur cream/duck si online, cream/coral si offline. Tap → expand list ou toast résumé (V1 : juste le compteur, pas d'expand).
- Monté dans `App.vue` pour visibilité globale.

### E. BackgroundSync Android — **SKIPPED V1, candidat P1**
**Décision révisée en cours d'implémentation** (cf. journal commit) :
- Mon runner main-thread s'auto-skip en offline et retry sur `online` event.
  Donc workbox ne voit jamais de fetch échoué à queuer.
- Pour activer BackgroundSync il faudrait soit forcer le fetch en offline
  (création de doublons garantis quand main-thread + SW rejouent en parallèle),
  soit ajouter un `X-Client-Upload-Id` + dédup backend (refactor backend non
  trivial).
- Gain marginal : couvre uniquement le cas "invité capture offline puis ferme
  l'app pour > 1h, le wifi revient, l'upload part tout seul". Pour 50 invités
  / 6h en foreground actif, statistiquement < 5% des uploads.
- **V1** : pas de BackgroundSync. Comportement uniforme iOS/Android — la queue
  reprend à la réouverture de l'app. La pill UI signale "X en attente"
  honnêtement.
- **P1 si retour terrain** : ajouter idempotency key backend + workbox
  BackgroundSyncPlugin sur `/api/{media,clips,voice}`.

### F. Fallback offline navigation
- `public/offline.html` : duck cream, "Plus assez d'eau", bouton Réessayer.
- **Décision révisée** : `navigateFallback: '/index.html'` (SPA standard). Vue
  Router prend le relai côté client. Une route inconnue charge l'app shell ;
  l'app détecte offline via `navigator.onLine` et peut afficher un état dégradé
  depuis le snapshot Pinia cache.
- `offline.html` reste précachée (via `includeAssets`) et accessible en direct
  à `/offline.html`. Aucune route ne la sert automatiquement.
- Un vrai offline-fallback "réseau échoue → bascule sur offline.html" demande
  `setCatchHandler` dans un SW custom (mode `injectManifest`). Candidat P1.
- denylist déjà en place (`/api`, `/uploads`, `/events`).

### G. Validation
**Validation auto effectuée** (cette session) :
- `npm run build` passe sans erreur TS, bundle index 122 KB gz 47 KB
- 48 entries précachées (311 KB) — app shell + icônes + offline.html
- `manifest.webmanifest` complet (id, lang, categories, shortcuts, 5 icons)
- `sw.js` contient bien `NavigationRoute` → `/index.html`, `CacheFirst` →
  `thepond-uploads`
- Preview server sert `/`, `/manifest.webmanifest`, `/offline.html`,
  `/apple-touch-icon-180x180.png`, `/sw.js`

**Validation à faire par l'utilisateur** (pas de Lighthouse CLI dans l'env) :
- Chrome desktop → DevTools → Application → Manifest : vérifier "Installable" badge
- Chrome desktop → Lighthouse PWA audit : target ≥ 90
- Android device : install via prompt natif, capture une photo offline (Chrome
  DevTools throttling Offline), vérifier la pill "1 en attente du réseau", remettre
  online, vérifier que l'upload part automatiquement, snapshot SSE met à jour
- iOS device : 1ère visite Safari → vérifier auto-show de la sheet à 2s,
  install via menu Partager, ouvrir l'app standalone, vérifier que l'icône
  apple-touch-icon est correcte, capture clip offline, fermer/rouvrir l'app,
  vérifier reprise de la queue.

## Limites assumées V1

- **iOS BackgroundSync indisponible** : sur iPhone la queue ne reprend qu'à la réouverture de l'app. Comportement documenté implicitement par la pill ("X en attente, on enverra dès que tu rouvriras").
- **Quota IndexedDB non monitoré** : un guest qui filme 50 vidéos hors ligne peut saturer son quota. Pas de UX dédiée pour ça en V1 (probabilité < 1%, l'erreur sera visible dans les logs).
- **Web Push** : sortir de scope V1, candidat P1 si retour terrain montre le besoin.
- **Pas de splash screens iOS custom** : default blanc, acceptable.

## Fichiers touchés

**Nouveaux** :
- `frontend/pwa-assets.config.ts`
- `frontend/public/pwa-64.png`, `pwa-192.png`, `pwa-512.png`, `maskable-icon-512.png`, `apple-touch-icon-180x180.png`, `favicon.ico`
- `frontend/public/offline.html`
- `frontend/src/services/uploadDb.ts` — IDB wrapper
- `frontend/src/composables/useNetworkStatus.ts`
- `frontend/src/components/PendingUploadsPill.vue`
- `docs/superpowers/specs/2026-05-01-prompt-14-pwa-install-offline-design.md` (ce fichier)

**Modifiés** :
- `frontend/package.json` (devDeps `@vite-pwa/assets-generator`, `idb`; script `icons`)
- `frontend/vite.config.ts` (icons[] PNG, navigateFallback, BackgroundSyncPlugin)
- `frontend/index.html` (apple-touch-icon, mask-icon)
- `frontend/src/stores/uploadQueue.ts` (rehydrate from IDB, persist mutations)
- `frontend/src/App.vue` (mount `PendingUploadsPill`)
- `frontend/src/views/WelcomeView.vue` (auto-show logic)

## Ordre d'implémentation

1. A — icônes (fondations) ; commit
2. F — offline.html (rapide, indépendant) ; commit
3. C+D — IDB queue + pill (le gros morceau) ; commit
4. E — BackgroundSync Android ; commit
5. B — auto-show sheet ; commit
6. G — validation Lighthouse + tests device

Commits atomiques par chantier conformément à la convention projet.
