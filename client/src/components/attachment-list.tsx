import { For } from 'solid-js';
import type { JSX } from 'solid-js';
import { Download, File } from 'lucide-solid';
import type { AttachmentMeta } from '../features/messages/models';
import { attachmentDownloadUrl } from '../features/attachments/api';

export type AttachmentListProps = {
  attachments: AttachmentMeta[];
};

function formatBytes(bytes: number | null | undefined): string {
  if (bytes === null || bytes === undefined) return '';
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / k ** i).toFixed(1))} ${sizes[i]}`;
}

export function AttachmentList(props: AttachmentListProps): JSX.Element {
  return (
    <div class="border-t border-zinc-200 pt-6 dark:border-zinc-800">
      <h3 class="mb-3 text-sm font-medium text-zinc-900 dark:text-zinc-100">
        附件 ({props.attachments.length})
      </h3>
      <div class="flex flex-col gap-2">
        <For each={props.attachments}>
          {(att) => (
            <div class="flex items-center gap-3 rounded-sm border border-zinc-200 px-3 py-2.5 dark:border-zinc-800">
              <File
                size={18}
                class="shrink-0 text-zinc-400 dark:text-zinc-500"
                aria-hidden="true"
              />
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm text-zinc-900 dark:text-zinc-100">
                  {att.filename ?? '未命名附件'}
                </p>
                <p class="text-xs text-zinc-500 dark:text-zinc-500">
                  {att.content_type ?? '未知类型'}
                  {att.size !== null && att.size !== undefined ? ` · ${formatBytes(att.size)}` : ''}
                </p>
              </div>
              <a
                href={attachmentDownloadUrl(att.id)}
                download={att.filename ?? undefined}
                class="inline-flex shrink-0 items-center gap-1.5 rounded-sm border border-zinc-200 px-2.5 py-1.5 text-sm font-medium text-zinc-700 transition-colors hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-300 dark:hover:bg-zinc-800"
              >
                <Download size={14} />
                <span class="hidden sm:inline">下载</span>
              </a>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
