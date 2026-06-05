import { apiClient } from '../../api/core/client';
import { executeBlob } from '../../api/core/response';

export function downloadAttachment(id: string): Promise<Blob> {
  return executeBlob(
    apiClient.GET('/api/attachments/{id}', {
      params: { path: { id } },
      parseAs: 'stream',
    }),
  );
}
