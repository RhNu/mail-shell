import { apiFetch } from './client';

export async function downloadAttachment(id: string): Promise<Blob> {
  const response = await apiFetch(`/attachments/${id}`);
  return response.blob();
}
