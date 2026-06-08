// @vitest-environment node

import { describe, expect, it } from 'vitest';
import { pwaOptions } from './vite.config';

describe('PWA configuration', () => {
  it('does not precache the authenticated HTML entry or serve cached navigation fallback', () => {
    expect(pwaOptions.workbox?.globPatterns).not.toContain('**/*.{js,css,html,svg,png,ico}');
    expect(pwaOptions.workbox?.globPatterns).toContain('**/*.{js,css,svg,png,ico}');
    expect(pwaOptions.workbox?.navigateFallback).toBeUndefined();
  });
});
