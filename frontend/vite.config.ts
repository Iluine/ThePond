import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  plugins: [
    vue(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.svg'],
      manifest: {
        name: 'The Pond',
        short_name: 'The Pond.',
        description: 'La mare des barbotages — partage de souvenirs de mariage',
        theme_color: '#0E4F6B',
        background_color: '#FAF3E3',
        display: 'standalone',
        orientation: 'portrait',
        start_url: '/',
        scope: '/',
        icons: [
          {
            src: '/favicon.svg',
            sizes: 'any',
            type: 'image/svg+xml',
            purpose: 'any maskable',
          },
        ],
      },
      // Pendant le dev local on veut que le SW ne cache pas tout
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
