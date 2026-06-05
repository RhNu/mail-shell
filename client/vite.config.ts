import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import solid from 'vite-plugin-solid';

export default defineConfig(() => {
  const isTest = process.env.VITEST === 'true';

  return {
    plugins: [
      solid({
        dev: !isTest,
        hot: !isTest,
      }),
      tailwindcss(),
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
