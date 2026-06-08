import { defineConfig } from 'vitest/config';
import tailwindcss from '@tailwindcss/vite';
import solid from 'vite-plugin-solid';
import { VitePWA, type VitePWAOptions } from 'vite-plugin-pwa';

export const pwaOptions = {
  registerType: 'autoUpdate',
  injectRegister: 'auto',
  useCredentials: true,
  manifest: {
    name: 'Mail Shell',
    short_name: 'Mail Shell',
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
    globPatterns: ['**/*.{js,css,svg,png,ico}'],
    navigateFallback: undefined,
  },
  includeAssets: ['favicon.svg', 'apple-touch-icon.png', 'apple-touch-icon-180x180.png'],
} satisfies Partial<VitePWAOptions>;

const pwaPlugin = VitePWA(pwaOptions);

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
