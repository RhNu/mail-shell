import { describe, expect, it } from 'vitest';
import { messageDisplayDate } from './time';

describe('messageDisplayDate', () => {
  const receivedAt = '2026-09-13T08:00:00Z';

  it('prefers a valid Date header', () => {
    expect(messageDisplayDate('Sat, 12 Sep 2026 18:30:00 +0800', receivedAt)).toBe(
      'Sat, 12 Sep 2026 18:30:00 +0800',
    );
  });

  it('falls back to receipt time for absent or invalid headers', () => {
    expect(messageDisplayDate(undefined, receivedAt)).toBe(receivedAt);
    expect(messageDisplayDate('not a date', receivedAt)).toBe(receivedAt);
  });
});
