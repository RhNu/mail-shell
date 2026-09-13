import { For } from 'solid-js';
import type { JSX } from 'solid-js';
import { Paperclip, Star } from 'lucide-solid';
import type { Mailbox, MessageSummary } from '../features/messages/models';
import type { MessageLabel } from '../features/classification/api';
import { TagChip } from './ui/tag-chip';
import { messageDetailHref } from '../app/routes';
import { formatRelativeTime, messageDisplayDate } from '../lib/time';
import { MessageActionMenu } from './message-action-menu';

export type MessageListItemProps = {
  message: MessageSummary;
  tags: MessageLabel[];
  attachmentCount?: number;
  active?: boolean;
  returnTo: string;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox?: (_id: string, _mailbox: Mailbox) => void;
  // eslint-disable-next-line no-unused-vars
  onDelete?: (_id: string) => void;
  onUpdateState?: (
    // eslint-disable-next-line no-unused-vars
    _id: string,
    // eslint-disable-next-line no-unused-vars
    _state: { read?: boolean; starred?: boolean; trashed?: boolean },
  ) => void;
  trashView?: boolean;
  selected?: boolean;
  // eslint-disable-next-line no-unused-vars
  onSelectedChange?: (_id: string, _selected: boolean) => void;
  actionsDisabled?: boolean;
};

function MessageListItemLink(props: {
  message: MessageSummary;
  tags: MessageLabel[];
  attachmentCount?: number;
  returnTo: string;
}) {
  const sender = () => props.message.from_name?.trim() || props.message.from_address;

  return (
    <a
      href={`#${messageDetailHref(props.message.id, props.returnTo)}`}
      data-message-link
      class="flex min-w-0 flex-1 flex-col gap-1 sm:flex-row sm:items-center sm:gap-4"
    >
      <span
        class="w-48 shrink-0 truncate text-sm text-zinc-700 dark:text-zinc-300"
        title={
          props.message.from_name
            ? `${props.message.from_name} <${props.message.from_address}>`
            : props.message.from_address
        }
      >
        {sender()}
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
  tags: MessageLabel[];
  attachmentCount?: number;
}) {
  const displayDate = () => messageDisplayDate(props.message.date, props.message.created_at);

  return (
    <div class="flex shrink-0 items-center gap-3">
      {props.tags.length > 0 && (
        <div class="hidden items-center gap-1 md:flex">
          <For each={props.tags}>{(tag) => <TagChip label={tag.name} />}</For>
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
        datetime={displayDate()}
        title={new Date(displayDate()).toLocaleString()}
      >
        {formatRelativeTime(displayDate())}
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
      onTrash={
        !props.trashView && props.onUpdateState
          ? () => props.onUpdateState?.(props.message.id, { trashed: true })
          : undefined
      }
      onRestore={
        props.trashView && props.onUpdateState
          ? () => props.onUpdateState?.(props.message.id, { trashed: false })
          : undefined
      }
      onSetRead={
        props.onUpdateState
          ? (read) => props.onUpdateState?.(props.message.id, { read })
          : undefined
      }
      onSetStarred={
        props.onUpdateState
          ? (starred) => props.onUpdateState?.(props.message.id, { starred })
          : undefined
      }
      isRead={props.message.is_read}
      isStarred={props.message.is_starred}
      disabled={props.actionsDisabled}
    />
  );
}

export function MessageListItem(props: MessageListItemProps): JSX.Element {
  return (
    <div
      data-message-row
      data-message-id={props.message.id}
      data-message-starred={String(props.message.is_starred)}
      tabindex="0"
      class={[
        'group relative flex items-center gap-4 border-b border-zinc-100 px-4 py-3 transition-colors outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-zinc-500 dark:border-zinc-800/60',
        props.active
          ? 'bg-zinc-100 dark:bg-zinc-800'
          : 'hover:bg-zinc-50 dark:hover:bg-zinc-900/50',
      ].join(' ')}
    >
      {props.active && (
        <span class="absolute top-0 bottom-0 left-0 w-0.5 bg-zinc-900 dark:bg-zinc-100" />
      )}
      {props.onSelectedChange && (
        <input
          type="checkbox"
          checked={props.selected}
          onChange={(event) =>
            props.onSelectedChange?.(props.message.id, event.currentTarget.checked)
          }
          aria-label={`选择 ${props.message.subject || '无主题邮件'}`}
          class="h-4 w-4 shrink-0 accent-zinc-900"
        />
      )}
      {!props.message.is_read && (
        <span class="h-2 w-2 shrink-0 rounded-full bg-blue-600" aria-label="未读" />
      )}
      {props.message.is_starred && (
        <Star size={15} class="shrink-0 fill-current" aria-label="星标" />
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
