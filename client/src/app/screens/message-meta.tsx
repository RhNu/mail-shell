import { Show } from 'solid-js';
import { Calendar, User } from 'lucide-solid';
import { MessageLabelsEditor } from '../../components/classification/message-labels-editor';
import type { MessageDetailResponse } from '../../features/messages/models';
import { messageDisplayDate } from '../../lib/time';

type MessageMetaProps = {
  message: MessageDetailResponse;
};

function Address(props: { label: string; name?: string | null; address: string }) {
  return (
    <div class="text-zinc-900 dark:text-zinc-100">
      <span class="text-zinc-500 dark:text-zinc-400">{props.label}：</span>{' '}
      <Show when={props.name?.trim()}>{(name) => <span class="font-medium">{name()} </span>}</Show>
      <span class="break-all">{props.address}</span>
    </div>
  );
}

function formatDate(value: string) {
  return new Date(value).toLocaleString(undefined, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function MessageAddresses(props: MessageMetaProps) {
  const hasExtendedAddresses = () => Boolean(props.message.cc || props.message.reply_to);
  return (
    <div class="flex items-start gap-2">
      <User size={16} class="mt-0.5 shrink-0 text-zinc-400 dark:text-zinc-500" aria-hidden="true" />
      <div class="min-w-0 flex flex-col gap-0.5">
        <Address
          label="发件人"
          name={props.message.from_name}
          address={props.message.from_address}
        />
        <Address
          label="收件人"
          name={props.message.to_name}
          address={props.message.to_address ?? props.message.envelope_to}
        />
        <Show when={hasExtendedAddresses()}>
          <details class="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
            <summary class="w-fit cursor-pointer select-none">更多地址</summary>
            <div class="mt-1 flex flex-col gap-1 pl-3">
              <Show when={props.message.cc}>{(cc) => <Address label="抄送" address={cc()} />}</Show>
              <Show when={props.message.reply_to}>
                {(replyTo) => <Address label="回复至" address={replyTo()} />}
              </Show>
            </div>
          </details>
        </Show>
      </div>
    </div>
  );
}

function MessageDates(props: MessageMetaProps) {
  const sentAt = () => messageDisplayDate(props.message.date, props.message.created_at);
  return (
    <div class="flex items-start gap-2">
      <Calendar
        size={16}
        class="mt-0.5 shrink-0 text-zinc-400 dark:text-zinc-500"
        aria-hidden="true"
      />
      <div class="flex flex-col text-zinc-700 dark:text-zinc-300">
        <time datetime={sentAt()}>{formatDate(sentAt())}</time>
        <Show when={sentAt() !== props.message.created_at}>
          <time
            datetime={props.message.created_at}
            class="text-xs text-zinc-400 dark:text-zinc-500"
          >
            收到于 {formatDate(props.message.created_at)}
          </time>
        </Show>
      </div>
    </div>
  );
}

export function MessageMeta(props: MessageMetaProps) {
  return (
    <div class="flex flex-col gap-2 text-sm">
      <MessageAddresses message={props.message} />
      <MessageDates message={props.message} />
      <MessageLabelsEditor
        messageId={props.message.id}
        labelIds={props.message.labels.map((label) => label.id)}
      />
    </div>
  );
}
