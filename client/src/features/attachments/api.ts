import { apiClient } from '../../api/core/client';
import { resolveApiBaseUrl } from '../../api/core/config';
import { executeBlob } from '../../api/core/response';

export function attachmentDownloadUrl(id: string): string {
  return `${resolveApiBaseUrl()}/api/attachments/${id}`;
}

export function downloadAttachment(id: string): Promise<Blob> {
  return executeBlob(
    apiClient.GET('/api/attachments/{id}', {
      params: { path: { id } },
      parseAs: 'stream',
    }),
  );
}
