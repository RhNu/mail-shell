import { describe, expect, it } from 'vitest';
import { isPreviewableAttachment } from './presentation';

describe('isPreviewableAttachment', () => {
  it('accepts browser-friendly media and documents', () => {
    expect(isPreviewableAttachment('image/png')).toBe(true);
    expect(isPreviewableAttachment('application/pdf')).toBe(true);
    expect(isPreviewableAttachment('text/plain')).toBe(true);
  });

  it('rejects absent and binary content types', () => {
    expect(isPreviewableAttachment(null)).toBe(false);
    expect(isPreviewableAttachment('application/zip')).toBe(false);
  });
});
