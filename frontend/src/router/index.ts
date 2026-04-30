import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import StubView from '../views/StubView.vue'

/**
 * Router de The Pond. Les vues non encore implémentées pointent toutes
 * vers StubView, qui s'auto-configure depuis route.meta. Quand une vraie
 * vue arrive à son prompt thématique, on remplace `component: StubView`
 * par `component: () => import('../views/XxxView.vue')`.
 *
 * Note : le `/` reste sur HomeView (le showcase design system) jusqu'au
 * prompt 7. À ce moment WelcomeView prendra `/`, et le showcase migrera
 * sur `/dev/showcase` (ou disparaîtra du build prod selon l'arbitrage).
 */
const routes: RouteRecordRaw[] = [
  // ─── Showcase (à déplacer ou retirer au prompt 7) ───────────
  {
    path: '/',
    name: 'home',
    component: () => import('../views/HomeView.vue'),
    meta: { title: 'Showcase' },
  },

  // ─── Parcours invité ────────────────────────────────────────
  {
    path: '/welcome',
    name: 'welcome',
    component: StubView,
    meta: {
      title: 'Bienvenue',
      screen: 'WelcomeView',
      implementedAt: 7,
    },
  },
  {
    path: '/upload',
    name: 'upload',
    component: StubView,
    meta: {
      title: 'Upload',
      screen: 'UploadView',
      implementedAt: 8,
    },
  },
  {
    path: '/upload/clip',
    name: 'upload-clip',
    component: StubView,
    meta: {
      title: 'Cancaner',
      screen: 'ClipCaptureView',
      implementedAt: 8,
    },
  },
  {
    path: '/upload/voice',
    name: 'upload-voice',
    component: StubView,
    meta: {
      title: 'Faire coin-coin',
      screen: 'VoiceCaptureView',
      implementedAt: 8,
    },
  },
  {
    path: '/confirmation',
    name: 'confirmation',
    component: StubView,
    meta: {
      title: 'Splash !',
      screen: 'ConfirmationView',
      implementedAt: 9,
    },
  },

  // ─── Galerie & lecteurs ─────────────────────────────────────
  {
    path: '/gallery',
    name: 'gallery',
    component: StubView,
    meta: {
      title: 'La mare',
      screen: 'GalleryView',
      implementedAt: 10,
    },
  },
  {
    path: '/clip/:id',
    name: 'clip-player',
    component: StubView,
    meta: {
      title: 'Cancan',
      screen: 'ClipPlayerView',
      implementedAt: 11,
    },
  },
  {
    path: '/voice/:id',
    name: 'voice-player',
    component: StubView,
    meta: {
      title: 'Coin-coin',
      screen: 'VoicePlayerView',
      implementedAt: 11,
    },
  },

  // ─── Témoins ────────────────────────────────────────────────
  {
    path: '/orchestration',
    name: 'orchestration',
    component: StubView,
    meta: {
      title: 'Orchestration',
      screen: 'OrchestrationView',
      implementedAt: 12,
    },
  },

  // ─── Mare TV ────────────────────────────────────────────────
  {
    path: '/mare-tv',
    name: 'mare-tv',
    component: StubView,
    meta: {
      title: 'Mare TV',
      screen: 'MareTVView',
      implementedAt: 13,
    },
  },
  {
    path: '/mare-tv/instructions',
    name: 'mare-tv-instructions',
    component: StubView,
    meta: {
      title: 'Mare TV · instructions',
      screen: 'MareTVInstructionsView',
      implementedAt: 13,
    },
  },

  // ─── Erreur catch-all ───────────────────────────────────────
  {
    path: '/:pathMatch(.*)*',
    name: 'error',
    component: StubView,
    meta: {
      title: 'Page introuvable',
      screen: 'ErrorView',
      implementedAt: 15,
    },
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(_to, _from, savedPosition) {
    return savedPosition ?? { top: 0 }
  },
})
