import { describe, expect, it, vi } from 'vitest';

vi.mock('../../api/core/config', () => ({
  resolveApiBaseUrl: () => 'https://api.example.test/mail',
}));

describe('attachmentDownloadUrl', async () => {
  const { attachmentDownloadUrl } = await import('./api');

  it('builds download links from the configured API base URL', () => {
    expect(attachmentDownloadUrl('att-42')).toBe(
      'https://api.example.test/mail/api/attachments/att-42',
    );
  });
});
