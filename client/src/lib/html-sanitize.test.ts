import { describe, expect, it } from 'vitest';
import { sanitizeEmailHtml } from './html-sanitize';

describe('sanitizeEmailHtml', () => {
  it('strips remote resource URLs by default and reports that they were blocked', () => {
    const result = sanitizeEmailHtml(
      '<div style="background-image:url(https://tracker.test/bg.png)"><img src="https://tracker.test/pixel.png" alt="pixel" /></div>',
    );

    expect(result.hasBlockedRemoteResources).toBe(true);
    expect(result.html).not.toContain('https://tracker.test/pixel.png');
    expect(result.html).not.toContain('background-image');
  });

  it('preserves remote resource URLs when explicitly allowed', () => {
    const result = sanitizeEmailHtml('<img src="https://tracker.test/pixel.png" alt="pixel" />', {
      allowRemoteResources: true,
    });

    expect(result.hasBlockedRemoteResources).toBe(false);
    expect(result.html).toContain('https://tracker.test/pixel.png');
  });
});
