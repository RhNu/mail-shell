import { For } from 'solid-js';
import type { JSX } from 'solid-js';
import { Paperclip } from 'lucide-solid';
import type { MessageSummary } from '../features/messages/models';
import type { Tag } from '../features/tags/api';
import { TagChip } from './ui/tag-chip';
import { messageDetailHref } from '../app/routes';
import { formatRelativeTime } from '../lib/time';

export type MessageListItemProps = {
  message: MessageSummary;
  tags: Tag[];
  attachmentCount?: number;
  active?: boolean;
};

export function MessageListItem(props: MessageListItemProps): JSX.Element {
  return (
    <a
      href={`#${messageDetailHref(props.message.id)}`}
      class={[
        'group relative flex items-center gap-4 border-b border-zinc-100 px-4 py-3 transition-colors dark:border-zinc-800/60',
        props.active
          ? 'bg-zinc-100 dark:bg-zinc-800'
          : 'hover:bg-zinc-50 dark:hover:bg-zinc-900/50',
      ].join(' ')}
    >
      {props.active && (
        <span class="absolute top-0 bottom-0 left-0 w-0.5 bg-zinc-900 dark:bg-zinc-100" />
      )}
      <div class="flex min-w-0 flex-1 flex-col gap-1 sm:flex-row sm:items-center sm:gap-4">
        <span class="w-48 shrink-0 truncate text-sm text-zinc-700 dark:text-zinc-300">
          {props.message.from_address}
        </span>
        <span class="min-w-0 flex-1 truncate text-sm text-zinc-900 dark:text-zinc-100">
          {props.message.subject ?? '(no subject)'}
        </span>
        <div class="flex shrink-0 items-center gap-3">
          {props.tags.length > 0 && (
            <div class="hidden items-center gap-1 md:flex">
              <For each={props.tags}>{(tag) => <TagChip label={tag.label} />}</For>
            </div>
          )}
          <span class="flex items-center gap-1.5 text-xs text-zinc-400 dark:text-zinc-500">
            {props.attachmentCount ? (
              <>
                <Paperclip size={12} aria-hidden="true" />
                <span class="sr-only">Attachments:</span>
                <span>{props.attachmentCount}</span>
              </>
            ) : null}
          </span>
          <time
            class="w-16 shrink-0 text-right text-xs text-zinc-400 tabular-nums dark:text-zinc-500"
            datetime={props.message.created_at}
          >
            {formatRelativeTime(props.message.created_at)}
          </time>
        </div>
      </div>
    </a>
  );
}
