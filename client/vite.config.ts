import { defineConfig } from 'vitest/config';
import tailwindcss from '@tailwindcss/vite';
import solid from 'vite-plugin-solid';
import { VitePWA } from 'vite-plugin-pwa';

const pwaPlugin = VitePWA({
  registerType: 'autoUpdate',
  injectRegister: 'auto',
  useCredentials: true,
  manifest: {
    name: 'mail-shell',
    short_name: 'mail-shell',
    description: 'Small mail-ingest web UI',
    theme_color: '#0ea5e9',
    background_color: '#ffffff',
    display: 'standalone',
    start_url: '/',
    icons: [
      {
        src: 'pwa-64x64.png',
        sizes: '64x64',
        type: 'image/png',
      },
      {
        src: 'pwa-192x192.png',
        sizes: '192x192',
        type: 'image/png',
      },
      {
        src: 'pwa-512x512.png',
        sizes: '512x512',
        type: 'image/png',
      },
      {
        src: 'maskable-icon-512x512.png',
        sizes: '512x512',
        type: 'image/png',
        purpose: 'maskable',
      },
    ],
  },
  workbox: {
    globPatterns: ['**/*.{js,css,html,svg,png,ico}'],
  },
  includeAssets: ['favicon.svg', 'apple-touch-icon-180x180.png'],
});

export default defineConfig(({ mode }) => {
  const isTest = mode === 'test';

  return {
    plugins: [
      solid({
        dev: !isTest,
        hot: !isTest,
      }),
      tailwindcss(),
      ...(isTest ? [] : [pwaPlugin]),
    ],
    server: {
      port: 4173,
    },
    test: {
      environment: 'jsdom',
      setupFiles: './src/test/setup.ts',
    },
  };
});
