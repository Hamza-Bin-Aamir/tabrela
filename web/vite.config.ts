import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { VitePWA } from 'vite-plugin-pwa'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['logos/tabrela.png', 'logos/tabrela-192.png', 'logos/tabrela-512.png', 'og-image.png'],
      manifest: false, // We're using our own manifest.json in public/
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff,woff2}'],
        runtimeCaching: [
          {
            urlPattern: /^https:\/\/.*\.gikinetwork\.com\/api\/.*/i,
            handler: 'NetworkFirst',
            options: {
              cacheName: 'api-cache',
              expiration: {
                maxEntries: 50,
                maxAgeSeconds: 60 * 60, // 1 hour
              },
              cacheableResponse: {
                statuses: [0, 200],
              },
            },
          },
        ],
      },
    }),
  ],
  // Dev server proxy to mimic nginx routing in production
  // Routes /api/* requests to the appropriate backend services
  server: {
    proxy: {
      // Auth service: /api/auth/* → localhost:8081/*
      '/api/auth': {
        target: 'http://localhost:8081',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/auth/, ''),
      },
      // Attendance service: /api/attendance/* → localhost:8082/*
      '/api/attendance': {
        target: 'http://localhost:8082',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/attendance/, ''),
      },
      // Merit service: /api/merit/* → localhost:8083/*
      '/api/merit': {
        target: 'http://localhost:8083',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/merit/, ''),
      },
      // Tabulation service: /api/tabulation/* → localhost:8084/*
      '/api/tabulation': {
        target: 'http://localhost:8084',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/tabulation/, ''),
      },
    },
  },
})
