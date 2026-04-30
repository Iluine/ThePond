import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import StubView from '../views/StubView.vue'

/**
 * Router de The Pond. Les vues non encore implémentées pointent toutes
 * vers StubView, qui s'auto-configure depuis route.meta. Quand une vraie
 * vue arrive à son prompt thématique, on remplace `component: StubView`
 * par `component: () => import('../views/XxxView.vue')`.
 *
 * Au prompt 7, WelcomeView a pris `/` (point d'entrée du parcours invité)
 * et le showcase design system a migré sur `/dev`.
 */
const routes: RouteRecordRaw[] = [
  // ─── Parcours invité ────────────────────────────────────────
  {
    path: '/',
    name: 'welcome',
    component: () => import('../views/WelcomeView.vue'),
    meta: { title: 'Bienvenue' },
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

  // ─── Showcase design system (dev seulement) ─────────────────
  {
    path: '/dev',
    name: 'dev-showcase',
    component: () => import('../views/HomeView.vue'),
    meta: { title: 'Dev showcase' },
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
