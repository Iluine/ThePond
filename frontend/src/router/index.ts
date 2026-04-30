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
    component: () => import('../views/UploadView.vue'),
    meta: { title: 'Upload' },
  },
  {
    path: '/upload/clip',
    name: 'upload-clip',
    component: () => import('../views/ClipCaptureView.vue'),
    meta: { title: 'Cancaner' },
  },
  {
    path: '/upload/voice',
    name: 'upload-voice',
    component: () => import('../views/VoiceCaptureView.vue'),
    meta: { title: 'Faire coin-coin' },
  },
  {
    path: '/confirmation',
    name: 'confirmation',
    component: () => import('../views/ConfirmationView.vue'),
    meta: { title: 'Splash !' },
  },

  // ─── Galerie & lecteurs ─────────────────────────────────────
  {
    path: '/gallery',
    name: 'gallery',
    component: () => import('../views/GalleryView.vue'),
    meta: { title: 'La mare' },
  },
  {
    path: '/clip/:id',
    name: 'clip-player',
    component: () => import('../views/ClipPlayerView.vue'),
    meta: { title: 'Cancan' },
  },
  {
    path: '/voice/:id',
    name: 'voice-player',
    component: () => import('../views/VoicePlayerView.vue'),
    meta: { title: 'Coin-coin' },
  },

  // ─── Témoins ────────────────────────────────────────────────
  {
    path: '/orchestration',
    name: 'orchestration',
    component: () => import('../views/OrchestrationView.vue'),
    meta: { title: 'Orchestration' },
  },

  // ─── Mare TV ────────────────────────────────────────────────
  {
    path: '/mare-tv',
    name: 'mare-tv',
    component: () => import('../views/MareTVView.vue'),
    meta: { title: 'Mare TV' },
  },
  {
    path: '/mare-tv/instructions',
    name: 'mare-tv-instructions',
    component: () => import('../views/MareTVInstructionsView.vue'),
    meta: { title: 'Mare TV · instructions' },
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
