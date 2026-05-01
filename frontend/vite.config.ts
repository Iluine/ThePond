import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  plugins: [
    vue(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: [
        'favicon.svg',
        'favicon.ico',
        'apple-touch-icon-180x180.png',
        'offline.html',
      ],
      manifest: {
        id: '/?source=pwa',
        name: 'The Pond.',
        short_name: 'The Pond.',
        description:
          'La mare des barbotages — partage de souvenirs de mariage en temps réel.',
        lang: 'fr',
        dir: 'ltr',
        theme_color: '#0E4F6B',
        background_color: '#FAF3E3',
        display: 'standalone',
        orientation: 'portrait',
        start_url: '/',
        scope: '/',
        categories: ['photo', 'social', 'lifestyle'],
        prefer_related_applications: false,
        icons: [
          // PNG multi-tailles générés via @vite-pwa/assets-generator depuis
          // public/favicon.svg. Lancer `npm run icons` pour régénérer.
          // Le SVG reste référencé en parallèle pour les browsers récents.
          { src: '/pwa-64x64.png', sizes: '64x64', type: 'image/png' },
          { src: '/pwa-192x192.png', sizes: '192x192', type: 'image/png' },
          { src: '/pwa-512x512.png', sizes: '512x512', type: 'image/png' },
          {
            src: '/maskable-icon-512x512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'maskable',
          },
          {
            src: '/favicon.svg',
            sizes: 'any',
            type: 'image/svg+xml',
            purpose: 'any',
          },
        ],
        // Raccourcis ajoutables sur l'écran d'accueil (Android long-press)
        shortcuts: [
          {
            name: 'Barboter',
            short_name: 'Upload',
            description: 'Poster une photo, un clip ou un coin-coin',
            url: '/upload',
          },
          {
            name: 'La mare',
            short_name: 'Galerie',
            description: 'Voir les souvenirs déjà révélés',
            url: '/gallery',
          },
        ],
      },
      workbox: {
        // SPA fallback : Vue Router prend le relai côté client.
        // /offline.html reste accessible en direct (précachée via
        // includeAssets) mais n'intercepte aucune navigation : un vrai
        // offline-fallback sur erreur réseau demanderait setCatchHandler
        // dans un SW custom (injectManifest), candidat P1.
        navigateFallback: '/index.html',
        // On ajoute du runtime caching pour /uploads/* (cache-first) car
        // les thumbnails sont immutables : on les sert depuis le cache si
        // dispo, sinon network puis on cache.
        runtimeCaching: [
          {
            urlPattern: /^.*\/uploads\/.*\.(jpg|jpeg|png|webp|mp4|webm|m4a|ogg|mp3)$/,
            handler: 'CacheFirst',
            options: {
              cacheName: 'thepond-uploads',
              expiration: {
                maxEntries: 200,
                maxAgeSeconds: 60 * 60 * 24 * 30, // 30 jours
              },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
        ],
        // /api/events est un flux SSE qu'il NE FAUT PAS cacher
        navigateFallbackDenylist: [/^\/api/, /^\/uploads/, /^\/events/],
      },
      devOptions: { enabled: false },
    }),
  ],
  server: {
    port: 5173,
    proxy: {
      // Permet d'appeler /api/*, /events et /uploads depuis le front en
      // dev sans CORS, le backend tourne sur :3000 en parallèle.
      '/api':     { target: 'http://localhost:3000', changeOrigin: true },
      '/events':  { target: 'http://localhost:3000', changeOrigin: true },
      '/uploads': { target: 'http://localhost:3000', changeOrigin: true },
    },
  },
})
