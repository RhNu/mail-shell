import { createQuery } from '@tanstack/solid-query';
import type { Accessor } from 'solid-js';
import { downloadAttachment } from './api';

const attachmentsKeys = {
  all: ['attachments'] as const,
  download: (id: string) => [...attachmentsKeys.all, 'download', id] as const,
};

export function useAttachmentDownload(id: Accessor<string>) {
  return createQuery(() => ({
    queryKey: attachmentsKeys.download(id()),
    queryFn: () => downloadAttachment(id()),
    enabled: Boolean(id()),
  }));
}
