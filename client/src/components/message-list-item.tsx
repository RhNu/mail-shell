import { For } from 'solid-js';
import type { JSX } from 'solid-js';
import { Paperclip } from 'lucide-solid';
import type { Mailbox, MessageSummary } from '../features/messages/models';
import type { Tag } from '../features/tags/api';
import { TagChip } from './ui/tag-chip';
import { messageDetailHref } from '../app/routes';
import { formatRelativeTime } from '../lib/time';
import { MessageActionMenu } from './message-action-menu';

export type MessageListItemProps = {
  message: MessageSummary;
  tags: Tag[];
  attachmentCount?: number;
  active?: boolean;
  returnTo: string;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox?: (_id: string, _mailbox: Mailbox) => void;
  // eslint-disable-next-line no-unused-vars
  onDelete?: (_id: string) => void;
};

function MessageListItemLink(props: {
  message: MessageSummary;
  tags: Tag[];
  attachmentCount?: number;
  returnTo: string;
}) {
  return (
    <a
      href={`#${messageDetailHref(props.message.id, props.returnTo)}`}
      class="flex min-w-0 flex-1 flex-col gap-1 sm:flex-row sm:items-center sm:gap-4"
    >
      <span class="w-48 shrink-0 truncate text-sm text-zinc-700 dark:text-zinc-300">
        {props.message.from_address}
      </span>
      <span class="min-w-0 flex-1 truncate text-sm text-zinc-900 dark:text-zinc-100">
        {props.message.subject ?? '（无主题）'}
      </span>
      <MessageListItemMeta
        message={props.message}
        tags={props.tags}
        attachmentCount={props.attachmentCount}
      />
    </a>
  );
}

function MessageListItemMeta(props: {
  message: MessageSummary;
  tags: Tag[];
  attachmentCount?: number;
}) {
  return (
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
            <span class="sr-only">附件：</span>
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
  );
}

function MessageListItemActions(props: MessageListItemProps) {
  return (
    <MessageActionMenu
      messageId={props.message.id}
      mailbox={props.message.mailbox}
      onMoveToMailbox={
        props.onMoveToMailbox
          ? (mailbox) => props.onMoveToMailbox?.(props.message.id, mailbox)
          : undefined
      }
      onDelete={props.onDelete ? () => props.onDelete?.(props.message.id) : undefined}
    />
  );
}

export function MessageListItem(props: MessageListItemProps): JSX.Element {
  return (
    <div
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
      <MessageListItemLink
        message={props.message}
        tags={props.tags}
        attachmentCount={props.attachmentCount}
        returnTo={props.returnTo}
      />
      <MessageListItemActions {...props} />
    </div>
  );
}
